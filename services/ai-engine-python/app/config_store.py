"""Loads AI provider + key configuration from Postgres and writes usage,
failures, and consensus results back. API key *secrets* are never stored in
Postgres — the DB holds a `secret_ref`; this resolver reads it from the
environment (dev) or Vault (prod, when `VAULT_ADDR` is set). See docs/AI_PROVIDERS.md."""

from __future__ import annotations

import os
from dataclasses import dataclass, field

import asyncpg
import httpx


@dataclass
class KeyCfg:
    id: int
    label: str
    secret_ref: str
    enabled: bool
    daily_limit_usd: float | None
    monthly_limit_usd: float | None
    status: str
    consecutive_failures: int


@dataclass
class ProviderCfg:
    id: int
    name: str
    enabled: bool
    priority: int
    weight: int
    base_url: str | None
    timeout_ms: int
    models: dict            # model -> {"input_per_mtok": x, "output_per_mtok": y}
    keys: list[KeyCfg] = field(default_factory=list)


class ConfigStore:
    def __init__(self, dsn: str):
        self._dsn = dsn
        self._pool: asyncpg.Pool | None = None

    async def connect(self) -> None:
        self._pool = await asyncpg.create_pool(self._dsn, min_size=1, max_size=5)

    async def load_providers(self) -> list[ProviderCfg]:
        assert self._pool
        async with self._pool.acquire() as con:
            prov_rows = await con.fetch(
                "SELECT id,name,enabled,priority,weight,base_url,timeout_ms,models "
                "FROM ai_providers WHERE enabled = true ORDER BY priority ASC"
            )
            key_rows = await con.fetch(
                "SELECT id,provider_id,label,secret_ref,enabled,daily_limit_usd,"
                "monthly_limit_usd,status,consecutive_failures FROM ai_provider_keys "
                "WHERE enabled = true"
            )
        by_provider: dict[int, list[KeyCfg]] = {}
        for k in key_rows:
            by_provider.setdefault(k["provider_id"], []).append(
                KeyCfg(k["id"], k["label"], k["secret_ref"], k["enabled"],
                       _f(k["daily_limit_usd"]), _f(k["monthly_limit_usd"]),
                       k["status"], k["consecutive_failures"])
            )
        providers = []
        for p in prov_rows:
            import json
            models = p["models"] if isinstance(p["models"], dict) else json.loads(p["models"] or "{}")
            providers.append(ProviderCfg(
                p["id"], p["name"], p["enabled"], p["priority"], p["weight"],
                p["base_url"], p["timeout_ms"], models, by_provider.get(p["id"], []),
            ))
        return providers

    async def load_settings(self) -> dict:
        """Router failover tuning (admin-editable). Falls back to defaults if the
        table/row is missing."""
        defaults = {"retries_per_key": 1, "max_failures": 3, "circuit_open_seconds": 60}
        if not self._pool:
            return defaults
        try:
            async with self._pool.acquire() as con:
                row = await con.fetchrow(
                    "SELECT retries_per_key, max_failures, circuit_open_seconds "
                    "FROM ai_router_settings WHERE id = 1")
            return dict(row) if row else defaults
        except Exception:
            return defaults

    async def resolve_secret(self, secret_ref: str) -> str | None:
        """vault:<path>#<field> → Vault KV-v2; otherwise an env var name."""
        if secret_ref.startswith("vault:"):
            return await self._vault_secret(secret_ref[len("vault:"):])
        return os.environ.get(secret_ref)

    async def _vault_secret(self, ref: str) -> str | None:
        addr = os.environ.get("VAULT_ADDR")
        token = os.environ.get("VAULT_TOKEN")
        if not addr or not token:
            return None
        path, _, field_name = ref.partition("#")
        field_name = field_name or "api_key"
        url = f"{addr}/v1/secret/data/{path}"
        async with httpx.AsyncClient(timeout=5.0) as client:
            r = await client.get(url, headers={"X-Vault-Token": token})
        if r.status_code != 200:
            return None
        return r.json().get("data", {}).get("data", {}).get(field_name)

    # ── write paths ──────────────────────────────────────────────────────
    async def record_request(self, *, tenant_id, provider, model, key_id, kind,
                             latency_ms, input_tokens, output_tokens, cost_usd,
                             success, error=None) -> None:
        assert self._pool
        async with self._pool.acquire() as con:
            await con.execute(
                "INSERT INTO ai_requests (tenant_id,provider,model,key_id,kind,latency_ms,"
                "input_tokens,output_tokens,cost_usd,success,error) "
                "VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
                tenant_id, provider, model, key_id, kind, latency_ms,
                input_tokens, output_tokens, cost_usd, success, error,
            )
            if success:
                await con.execute(
                    "UPDATE ai_provider_keys SET consecutive_failures=0, status='healthy' WHERE id=$1",
                    key_id)
            else:
                await con.execute(
                    "INSERT INTO ai_failures (provider,key_id,error) VALUES ($1,$2,$3)",
                    provider, key_id, (error or "")[:500])

    async def trip_key(self, key_id: int, open_seconds: int) -> None:
        """Circuit breaker: mark a key unhealthy and open its circuit."""
        assert self._pool
        async with self._pool.acquire() as con:
            await con.execute(
                "UPDATE ai_provider_keys SET consecutive_failures=consecutive_failures+1, "
                "status='circuit_open', circuit_open_until = now() + ($2 || ' seconds')::interval "
                "WHERE id=$1", key_id, str(open_seconds))

    async def record_consensus(self, *, tenant_id, symbol, signal, bias, confidence, providers) -> None:
        assert self._pool
        import json
        async with self._pool.acquire() as con:
            await con.execute(
                "INSERT INTO ai_consensus_results (tenant_id,symbol,signal,bias,confidence,providers) "
                "VALUES ($1,$2,$3,$4,$5,$6)",
                tenant_id, symbol, signal, bias, confidence, json.dumps(providers),
            )


def _f(v):
    return float(v) if v is not None else None
