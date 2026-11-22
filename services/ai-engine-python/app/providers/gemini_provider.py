"""Google Gemini provider via the generativelanguage REST API (httpx).

Resilient model fallback: Gemini free-tier quota is PER-MODEL, so when the
preferred model is exhausted (429) we fall through a chain of models until one
succeeds (ending at gemini-2.5-flash-lite, which tends to have headroom). To
avoid hammering an exhausted model on every request (which just spams 429s), a
model that returns 429/404 is skipped for a short cooldown, then retried later
(its daily quota may have reset). The chain is tunable via GEMINI_MODEL_CHAIN.

The 1.5 models were retired by Google (404 on v1beta) — never use them."""

from __future__ import annotations

import os
import time

import httpx

from .base import AIProvider, CompletionResult, Message, ProviderError

DEFAULT_MODEL = "gemini-2.5-flash"
API_BASE = "https://generativelanguage.googleapis.com/v1beta/models"

# Best → most-available. Top models first; flash-lite last as the reliable
# fallback. Override with GEMINI_MODEL_CHAIN="m1,m2,...".
DEFAULT_CHAIN = ["gemini-2.5-flash", "gemini-2.0-flash", "gemini-2.5-pro", "gemini-2.5-flash-lite"]

# Per-model skip window after a quota/availability error (seconds). Daily quota
# resets, so we retry the model again after this.
COOLDOWN_SECS = int(os.environ.get("GEMINI_MODEL_COOLDOWN_SECS", "1800"))

# Module-level (per worker): model -> epoch until which to skip it.
_cooldown: dict[str, float] = {}


def _chain(primary: str) -> list[str]:
    env = os.environ.get("GEMINI_MODEL_CHAIN")
    order = [m.strip() for m in env.split(",")] if env else list(DEFAULT_CHAIN)
    order = [m for m in order if m]
    merged = ([primary] if primary else []) + order
    seen: set[str] = set()
    out: list[str] = []
    for m in merged:
        if m and m not in seen:
            seen.add(m)
            out.append(m)
    return out


class GeminiProvider(AIProvider):
    name = "gemini"

    async def complete(
        self,
        messages: list[Message],
        model: str,
        *,
        max_tokens: int = 1024,
        system: str | None = None,
    ) -> CompletionResult:
        contents = [
            {"role": "user" if m.role == "user" else "model", "parts": [{"text": m.content}]}
            for m in messages
        ]
        body: dict = {"contents": contents, "generationConfig": {"maxOutputTokens": max_tokens}}
        if system:
            body["systemInstruction"] = {"parts": [{"text": system}]}

        base = self.base_url or API_BASE
        chain = _chain(model or DEFAULT_MODEL)
        now = time.time()
        # Prefer models not in cooldown; if ALL are cooling, try the full chain
        # anyway (quota may have reset since we last marked them).
        candidates = [m for m in chain if _cooldown.get(m, 0.0) < now] or chain

        last_err: ProviderError | None = None
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            for m in candidates:
                url = f"{base}/{m}:generateContent?key={self.api_key}"
                try:
                    r = await client.post(url, json=body)
                except httpx.HTTPError as e:
                    last_err = ProviderError(str(e), retryable=True)
                    continue
                if r.status_code < 400:
                    data = r.json()
                    cand = (data.get("candidates") or [{}])[0]
                    parts = cand.get("content", {}).get("parts", [{}])
                    text = "".join(p.get("text", "") for p in parts)
                    usage = data.get("usageMetadata", {})
                    return CompletionResult(
                        text=text,
                        input_tokens=usage.get("promptTokenCount", 0),
                        output_tokens=usage.get("candidatesTokenCount", 0),
                        model=m,
                        provider=self.name,
                    )
                status = r.status_code
                # Model-specific: quota exhausted / model gone / transient → cool
                # this model down and fall through to the next one.
                if status in (429, 404, 500, 502, 503):
                    _cooldown[m] = time.time() + COOLDOWN_SECS
                    last_err = ProviderError(r.text, retryable=True, status=status)
                    continue
                # Key/request problem (401/403/400): other models won't help — stop.
                raise ProviderError(r.text, retryable=False, status=status)

        raise last_err or ProviderError("all gemini models unavailable", retryable=True)
