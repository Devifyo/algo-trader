# Multi-AI Provider Management

A single **AI Request Router** (in the Python AI engine) is the only path the
platform uses to reach any LLM. Nothing else calls a provider directly, so new
providers are added without touching trading logic.

## Provider pattern

`AIProvider` interface → `AnthropicProvider` (official `anthropic` SDK, default
**Claude Opus 4.8** `claude-opus-4-8`), `OpenAIProvider`, `GeminiProvider` (REST).
Register a new provider (DeepSeek, Grok, OpenRouter, Ollama, custom) in
`providers/registry.py` — OpenAI-compatible ones just set a `base_url`.

## Keys & secrets

- Config lives in `ai_providers` / `ai_provider_keys`. **Secrets never live in
  Postgres** — `secret_ref` points to an env var (dev) or `vault:<path>#<field>`
  (prod). Unlimited keys per provider; each can be enabled/disabled with daily /
  monthly budget caps.

## Routing & reliability

- **Load-balancing modes** (`AI_LB_MODE`): `round_robin`, `weighted`, `cost`
  (cheapest first), `performance` (lowest latency), `health` (fewest failures).
- **Failover:** key → key → provider → provider → (caller degrades to neutral).
  A single provider outage never blocks trading.
- **Circuit breaker:** after N consecutive failures a key's circuit opens for a
  cool-off; **retries** with backoff on transient errors; per-request timeouts.
- **Cost tracking:** per-model `$/1M` pricing in `ai_providers.models`; every
  call logs tokens + cost to `ai_requests`; budgets in `ai_cost_tracking`.

## Consensus engine

`/ai/sentiment?consensus=true` queries every healthy provider once in parallel
and combines via majority / weighted / confidence-averaging. Admin sets provider
weights. Result carries per-provider verdicts for transparency.

## Usage in trading (advisory only)

AI never executes trades. It contributes sentiment, risk, confidence, and regime
scores. The **risk engine is final authority** and trade execution is never
blocked by an AI call.

## Admin

The hub's **AI Providers** page: enable/disable providers + keys, view 24h
requests/cost/latency/failures, live router status, and routing mode.
