"""AI Request Router — the single entry point the rest of the platform uses to
reach any LLM. Selects provider + key by the configured load-balancing mode,
fails over key→key→provider→provider→queue, trips a circuit breaker on
repeated failures, retries transient errors, and tracks token cost + budgets.

A single provider outage can never break trading: the router degrades to the
next provider, and callers that need a result fall back to a neutral verdict.
"""

from __future__ import annotations

import asyncio
import itertools
import os
import random
import time

from .config_store import ConfigStore, KeyCfg, ProviderCfg
from .consensus import Verdict, aggregate, parse_verdict
from .providers.base import CompletionResult, Message, ProviderError
from .providers.registry import build_provider

CIRCUIT_OPEN_SECONDS = int(os.environ.get("AI_CIRCUIT_OPEN_SECONDS", "60"))
MAX_FAILURES_BEFORE_TRIP = int(os.environ.get("AI_MAX_FAILURES", "3"))
RETRIES_PER_KEY = int(os.environ.get("AI_RETRIES_PER_KEY", "1"))


class _Candidate:
    __slots__ = ("provider_cfg", "key", "model", "input_price", "output_price")

    def __init__(self, p: ProviderCfg, key: KeyCfg, model: str, ip: float, op: float):
        self.provider_cfg = p
        self.key = key
        self.model = model
        self.input_price = ip
        self.output_price = op


class AIRouter:
    def __init__(self, store: ConfigStore, default_mode: str = "priority"):
        self.store = store
        # "priority" = strict failover order: highest-priority provider first,
        # then its keys in order, then the next provider (the configured failover
        # spec). Other modes (cost/performance/health/weighted/round_robin)
        # load-balance instead. Override with AI_LB_MODE.
        self.default_mode = os.environ.get("AI_LB_MODE", default_mode)
        self._providers: list[ProviderCfg] = []
        self._rr = itertools.count()                     # round-robin counter
        self._latency_ewma: dict[str, float] = {}        # provider -> ms
        self._open_until: dict[int, float] = {}          # key_id -> epoch
        self._spend_today: dict[int, float] = {}         # key_id -> usd
        # Admin-tunable failover settings (loaded from DB on refresh; env = fallback).
        self._retries_per_key = RETRIES_PER_KEY
        self._max_failures = MAX_FAILURES_BEFORE_TRIP
        self._circuit_seconds = CIRCUIT_OPEN_SECONDS
        # Live config: reload providers + settings every TTL so admin changes
        # (priority, enable/disable, retry/cooldown) take effect WITHOUT a restart.
        self._last_refresh = 0.0
        self._refresh_ttl = float(os.environ.get("AI_CONFIG_TTL_SECONDS", "15"))

    async def refresh(self) -> None:
        self._providers = await self.store.load_providers()
        s = await self.store.load_settings()
        self._retries_per_key = int(s.get("retries_per_key", RETRIES_PER_KEY))
        self._max_failures = int(s.get("max_failures", MAX_FAILURES_BEFORE_TRIP))
        self._circuit_seconds = int(s.get("circuit_open_seconds", CIRCUIT_OPEN_SECONDS))
        self._last_refresh = time.time()

    async def _maybe_refresh(self) -> None:
        """Reload config if it's gone stale (TTL) — makes admin changes live."""
        if time.time() - self._last_refresh >= self._refresh_ttl:
            try:
                await self.refresh()
            except Exception:
                pass

    @staticmethod
    def _unavailable() -> dict:
        """Consistent shape the frontend can rely on when no provider could serve."""
        return {"available": False, "error": "no_provider_available", "bias": "neutral",
                "score": 0.0, "confidence": 0.0, "label": "unavailable", "providers": []}

    # ── public API ───────────────────────────────────────────────────────
    async def single(self, prompt: str, *, kind: str, tenant_id, system: str | None = None,
                     model_tier: str = "default", max_tokens: int = 512) -> dict:
        """Route to ONE provider with full failover. Returns parsed verdict."""
        await self._maybe_refresh()
        candidates = self._candidates(self.default_mode, model_tier)
        result = await self._call_with_failover(candidates, prompt, kind, tenant_id, system, max_tokens)
        if result is None:
            return self._unavailable()
        v = parse_verdict(result.provider, result.text)
        return {**aggregate([v], mode="weighted"), "available": True}

    async def complete_text(self, prompt: str, *, kind: str, tenant_id, system: str | None = None,
                            max_tokens: int = 600) -> dict:
        """Raw text completion with full provider/key failover (for non-verdict
        tasks like strategy research). Returns {available, text, provider, model}."""
        await self._maybe_refresh()
        candidates = self._candidates(self.default_mode, "default")
        result = await self._call_with_failover(candidates, prompt, kind, tenant_id, system, max_tokens)
        if result is None:
            return {"available": False, "text": "", "error": "no_provider_available"}
        return {"available": True, "text": result.text, "provider": result.provider, "model": result.model}

    async def consensus(self, prompt: str, *, kind: str, tenant_id, system: str | None = None,
                        symbol: str | None = None, model_tier: str = "default",
                        max_tokens: int = 512, mode: str = "weighted") -> dict:
        """Query every healthy provider once (one key each) in parallel and
        combine. Distinct providers => distinct viewpoints."""
        await self._maybe_refresh()
        tasks = []
        for p in self._providers:
            cand = self._first_candidate_for(p, model_tier)
            if cand is not None:
                tasks.append(self._one_call(cand, prompt, kind, tenant_id, system, max_tokens))
        results = await asyncio.gather(*tasks, return_exceptions=True)

        verdicts: list[Verdict] = []
        for p, r in zip([pp for pp in self._providers if self._first_candidate_for(pp, model_tier)], results):
            if isinstance(r, CompletionResult):
                verdicts.append(parse_verdict(r.provider, r.text, weight=float(p.weight or 1)))
        if not verdicts:
            return self._unavailable()
        consensus = aggregate(verdicts, mode=mode)
        if symbol:
            await self.store.record_consensus(
                tenant_id=tenant_id, symbol=symbol, signal=kind,
                bias=consensus["bias"], confidence=consensus["confidence"],
                providers=consensus["providers"])
        return {**consensus, "available": True}

    # ── candidate selection ──────────────────────────────────────────────
    def _candidates(self, mode: str, model_tier: str) -> list[_Candidate]:
        cands: list[_Candidate] = []
        for p in self._providers:
            for key in p.keys:
                if not self._key_available(key):
                    continue
                model, ip, op = self._model_for(p, model_tier)
                if model:
                    cands.append(_Candidate(p, key, model, ip, op))
        return self._order(cands, mode)

    def _first_candidate_for(self, p: ProviderCfg, model_tier: str) -> _Candidate | None:
        for key in p.keys:
            if self._key_available(key):
                model, ip, op = self._model_for(p, model_tier)
                if model:
                    return _Candidate(p, key, model, ip, op)
        return None

    def _order(self, cands: list[_Candidate], mode: str) -> list[_Candidate]:
        if not cands:
            return cands
        if mode == "priority":
            # Strict failover: provider priority (lower = first), then key order.
            # So all of provider A's keys are tried before provider B's.
            return sorted(cands, key=lambda c: (c.provider_cfg.priority, c.key.id))
        if mode == "cost":
            return sorted(cands, key=lambda c: c.input_price + c.output_price)
        if mode == "performance":
            return sorted(cands, key=lambda c: self._latency_ewma.get(c.provider_cfg.name, 1e9))
        if mode == "health":
            return sorted(cands, key=lambda c: c.key.consecutive_failures)
        if mode == "weighted":
            # weighted random order without replacement
            pool = cands[:]
            ordered = []
            while pool:
                weights = [max(1, c.provider_cfg.weight) for c in pool]
                pick = random.choices(pool, weights=weights, k=1)[0]
                ordered.append(pick)
                pool.remove(pick)
            return ordered
        # round_robin (default): rotate the list by a global counter
        n = next(self._rr) % len(cands)
        return cands[n:] + cands[:n]

    def _key_available(self, key: KeyCfg) -> bool:
        if not key.enabled or key.status == "disabled":
            return False
        until = self._open_until.get(key.id, 0.0)
        if time.time() < until:
            return False
        if key.daily_limit_usd is not None and self._spend_today.get(key.id, 0.0) >= key.daily_limit_usd:
            return False
        return True

    def _model_for(self, p: ProviderCfg, tier: str) -> tuple[str | None, float, float]:
        """Resolve the model for a tier from the provider's `models` map. The
        map keys are model ids; an optional `_tiers` entry maps tier->model."""
        tiers = p.models.get("_tiers", {}) if isinstance(p.models, dict) else {}
        model = tiers.get(tier)
        if not model:
            # first concrete model in the map
            model = next((k for k in p.models if not k.startswith("_")), None)
        if not model:
            return None, 0.0, 0.0
        price = p.models.get(model, {})
        return model, float(price.get("input_per_mtok", 0)), float(price.get("output_per_mtok", 0))

    # ── execution ────────────────────────────────────────────────────────
    async def _call_with_failover(self, candidates, prompt, kind, tenant_id, system, max_tokens):
        for cand in candidates:
            try:
                return await self._one_call(cand, prompt, kind, tenant_id, system, max_tokens)
            except ProviderError as e:
                if not e.retryable:
                    continue          # bad request/auth — skip this candidate
                # retryable handled inside _one_call; if it bubbled, key tripped
                continue
            except Exception:
                continue
        return None                    # all providers exhausted — caller degrades

    async def _one_call(self, cand: _Candidate, prompt, kind, tenant_id, system, max_tokens) -> CompletionResult:
        api_key = await self.store.resolve_secret(cand.key.secret_ref)
        if not api_key:
            raise ProviderError("secret unresolved", retryable=False)
        provider = build_provider(
            cand.provider_cfg.name, api_key, cand.provider_cfg.base_url,
            timeout=cand.provider_cfg.timeout_ms / 1000.0)

        attempt = 0
        while True:
            attempt += 1
            t0 = time.time()
            try:
                res = await provider.complete([Message("user", prompt)], cand.model,
                                              max_tokens=max_tokens, system=system)
                latency_ms = int((time.time() - t0) * 1000)
                cost = self._cost(cand, res.input_tokens, res.output_tokens)
                self._spend_today[cand.key.id] = self._spend_today.get(cand.key.id, 0.0) + cost
                self._ewma(cand.provider_cfg.name, latency_ms)
                await self.store.record_request(
                    tenant_id=tenant_id, provider=res.provider, model=res.model,
                    key_id=cand.key.id, kind=kind, latency_ms=latency_ms,
                    input_tokens=res.input_tokens, output_tokens=res.output_tokens,
                    cost_usd=cost, success=True)
                return res
            except ProviderError as e:
                latency_ms = int((time.time() - t0) * 1000)
                await self.store.record_request(
                    tenant_id=tenant_id, provider=cand.provider_cfg.name, model=cand.model,
                    key_id=cand.key.id, kind=kind, latency_ms=latency_ms,
                    input_tokens=0, output_tokens=0, cost_usd=0.0,
                    success=False, error=str(e))
                if e.retryable and attempt <= self._retries_per_key:
                    await asyncio.sleep(0.2 * attempt)
                    continue
                self._maybe_trip(cand.key)
                raise

    def _maybe_trip(self, key: KeyCfg) -> None:
        key.consecutive_failures += 1
        if key.consecutive_failures >= self._max_failures:
            self._open_until[key.id] = time.time() + self._circuit_seconds
            asyncio.create_task(self.store.trip_key(key.id, self._circuit_seconds))

    def _cost(self, cand: _Candidate, in_tok: int, out_tok: int) -> float:
        return in_tok / 1e6 * cand.input_price + out_tok / 1e6 * cand.output_price

    def _ewma(self, provider: str, latency_ms: int) -> None:
        prev = self._latency_ewma.get(provider, latency_ms)
        self._latency_ewma[provider] = 0.7 * prev + 0.3 * latency_ms

    # ── monitoring ───────────────────────────────────────────────────────
    def status(self) -> dict:
        return {
            "mode": self.default_mode,
            "providers": [
                {
                    "name": p.name, "enabled": p.enabled, "priority": p.priority,
                    "weight": p.weight, "avg_latency_ms": round(self._latency_ewma.get(p.name, 0), 1),
                    "keys": [
                        {"id": k.id, "label": k.label, "status": k.status,
                         "failures": k.consecutive_failures,
                         "circuit_open": time.time() < self._open_until.get(k.id, 0),
                         "spend_today_usd": round(self._spend_today.get(k.id, 0.0), 4)}
                        for k in p.keys
                    ],
                }
                for p in self._providers
            ],
        }
