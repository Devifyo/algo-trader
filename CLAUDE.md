# CLAUDE.md — algo-trader

Project context for Claude. **Polyglot, container-native algorithmic trading platform.**
A low-latency **Rust** trading core + a **Python** AI/ML & intelligence layer + a
**Laravel 12** admin dashboard, glued by **NATS**, backed by **PostgreSQL/TimescaleDB**
+ **Redis**, observed by **Prometheus / Grafana / Loki**, secrets in **Vault**.

- **Working dir:** `/root/projects/laravel/disputer/algo-trader`
  (bind-mounted / same inode as `/var/www/algo-trader` — editing either edits both).
- **Operator entrypoints:** the [`Makefile`](Makefile). Everything is `docker compose`
  under the hood:
  `docker compose -f infrastructure/docker/docker-compose.yml --env-file .env <cmd>`
- **Containers are prefixed** `algotrader-<service>-1`.

---

## Container / service map

| Container | Language / Runtime | Build context | Host port → in-container | Role |
|-----------|-------------------|---------------|--------------------------|------|
| `trading-engine` | **Rust** (`rust:1-slim` build → `debian:bookworm-slim` runtime) | `services/trading-engine-rust` | `8139` → `9100` (`/metrics`) | The hot path. Runs the `engine` supervisor binary. |
| `ai-engine` | **Python 3.12** (FastAPI + uvicorn, 2 workers) | `services/ai-engine-python` | `8140` → `8000` | Multi-provider AI router: signal scoring, regime, consensus. |
| `news-service` | **Python 3.12** (`python -m app.main`) | `services/news-service` | `8141` → `8000` | News/economic intelligence, sentiment, influence scoring. |
| `portfolio-service` | **Python 3.12** (FastAPI + uvicorn, 2 workers) | `services/portfolio-service` | `8142` → `8000` | Portfolio optimization, allocation, risk-manager, safety/recovery, feature flags. |
| `dashboard` | **PHP 8.4 / Laravel 12** (`php:8.4-apache`, Vue + Inertia, Vite) | `dashboard/laravel` | `8130` → `80` (**public, all interfaces**) | Admin + tenant SaaS UI & API. |
| `postgres` | `timescale/timescaledb:2.17.2-pg16` | image | `8131` → `5432` | TimescaleDB: history + relational state. **Row-Level Security (RLS) per tenant.** |
| `redis` | `redis:7.4-alpine` | image | `8132` → `6379` | Hot state / queues. |
| `nats` | `nats:2.10-alpine` | image | client `8133`→`4222`, mon `8134`→`8222` | Message bus between all services. |
| `vault` | `hashicorp/vault:1.18` | image | `8138` → `8200` | Secrets (exchange/broker keys, per-tenant partitions). Dev-mode token in `.env`. |
| `prometheus` | `prom/prometheus:v3.0.1` | image | `8135` → `9090` | Metrics scrape. |
| `grafana` | `grafana/grafana:11.4.0` | image | `8136` → `3000` | Dashboards. Admin user `admin`. |
| `loki` | `grafana/loki:3.3.2` | image | `8137` → `3100` | Log aggregation. |

> Only `dashboard` binds `0.0.0.0` (public at `http://<server-ip>:8130`). Everything
> else binds `127.0.0.1` — reach via SSH tunnel. Network: `algonet`.
> Named volumes: `pgdata redisdata natsdata vaultdata promdata grafanadata lokidata`.

---

## Where Rust is used — `services/trading-engine-rust`

Cargo **workspace** (edition 2021). The single `engine` container runs all crates
in-process on one VPS; each crate is also a standalone binary so it can be split out
in k8s (the `services/{market-data,execution,risk}-service` dirs are **deployment
seams** — README-only, no separate code; run e.g. `command: ["risk"]`).

| Crate | Purpose |
|-------|---------|
| `crates/core` | Shared domain types + NATS plumbing. |
| `crates/market-data` | Ingests exchange WS/REST → normalized ticks/candles. Source via `MARKET_DATA_SOURCE` (`synthetic`\|`exchange`). Shared per region (public, **not** tenant-partitioned). |
| `crates/signals` | Strategy signal generation (trend_following, momentum, mean_reversion, breakout, vol_expansion). |
| `crates/risk` | **Only path that produces an order.** Gates every signal: kill-switch, tenant-halt, stop-defined, stale-data, max-open, daily-drawdown, AI floor, regional compliance (FinRA PDT / SEBI rate-cap + broker whitelist), then position sizing. |
| `crates/execution` | Consumes `tenant.*.orders` → paper simulator (default) or live venue adapters. Pulls tenant keys from Vault into memory only for one order, then drops them. Persists order/fill/position with `tenant_id`. |
| `crates/engine` | Supervisor binary (the default `CMD`); boots the above in-process. |

Key deps: `tokio`, `async-nats`, `sqlx` (postgres), `redis`, `reqwest`/`rustls`,
`metrics-exporter-prometheus`. Quick check: `make check-rust`.

## Where AI is used — `services/ai-engine-python`

FastAPI ML inference & **multi-provider AI router** with consensus voting.

- `app/router.py`, `app/consensus.py` — route a scoring request to N providers, combine.
- `app/providers/` — `openai_provider`, `anthropic_provider`, `gemini_provider`,
  `deepseek_provider` (+ `base`, `registry`). Provider keys live per-tenant
  (`ai_provider_keys` table, surfaced via the dashboard `AiProviderController`).
- `app/tenancy.py`, `app/config_store.py`, `app/models.py`.
- Consumes `signals.score.request`, returns AI confidence used by the Rust `risk`
  crate's "AI floor" gate. Heavy ML deps split into `requirements-ml.txt`.

## Where Python is used (besides AI)

- **`news-service`** (`app/`): `collector`, `sources`, `analyzer`, `categorize`,
  `influence`, `economic`, `store`. Sentiment + economic-calendar intelligence feeding
  the dashboard `News` page and signal context.
- **`portfolio-service`** (`app/`): `portfolio`, `optimizer`, `allocation`, `regime`,
  `risk_manager`, `safety`, `recovery`, `decision`, `profiles`, `feature_flags`,
  `health`. Intelligence/portfolio/safety API.

## Where Laravel/PHP is used — `dashboard/laravel`

Laravel 12 + **Inertia + Vue 3 + Vite**, multi-tenant SaaS (Postgres RLS).
Overlay code at `dashboard/laravel/overlay/`.

- Routes: `overlay/routes/web.php` (web + Inertia), `overlay/routes/console.php`
  (artisan, incl. `admin:ensure`).
- Controllers (`overlay/app/Http/Controllers/`): `Dashboard`, `Bots`, `Markets`,
  `Portfolio`, `News`, `AiDecisions`, `AiProvider`, `Broker`, `ExchangeKey`,
  `ExecutionLogs`, `KillSwitch`, `Marketplace`, `CopyTrading`, `Notifications`,
  `Onboarding`, `Help`, plus `Auth/AuthController`.
- Vue pages: `overlay/resources/js/Pages/*.vue`.
- Container entrypoint: `overlay/entrypoint.sh` (runs migrations + `admin:ensure`).

---

## Message bus (NATS subjects)

```
market.*.*.tick / market.*.*.candle.*   market data fan-out (per region)
signals.*.* / signals.score.request     strategy signals + AI scoring requests
tenant.*.signals / tenant.*.orders      per-tenant signal & order streams
orders.new / orders.cancel / orders.status
executions.fill / executions.reject
```

Flow: **market-data → signals → (ai-engine score) → risk (gate+size) → execution →
fills**; Timescale stores history, Redis holds hot state, Laravel reads for the UI.

---

## Common commands

```bash
make up            # start everything        make down       # stop (keep volumes)
make up-infra      # postgres redis nats vault
make up-core       # trading-engine ai-engine news portfolio
make up-app        # dashboard prometheus grafana loki
make ps / make logs S=trading-engine / make health
make db-shell      # psql -U algotrader -d algotrader
make redis-cli
make kill-switch   # EMERGENCY: flatten positions + halt entries
make unkill        # clear kill switch
make check-config  # validate compose      make check-rust  # cargo check engine
make clean         # down -v  (DESTROYS DATA)
```

---

## Config & safety notes

- **Admin login:** `admin@gmail.com` / `<redacted, see .env>` (env `ADMIN_EMAIL`/`ADMIN_PASSWORD`
  in `.env`; DB row in `users`, `role=admin`, `tenant_id = app.default_tenant`).
  `admin:ensure` is **idempotent on email** — it never updates an existing user's
  password; change the DB row directly to rotate.
- ⚠️ **`TRADING_MODE=live` is currently set in `.env`** (README ships `paper`). Live
  orders also require real keys in Vault + per-broker `*_TESTNET` flags. Confirm intent
  before bringing `up-core` with real keys.
- `.env` holds plaintext placeholder secrets (`change_me_in_prod_use_vault`); production
  pulls from Vault.
- `REGION` selects compliance regime: `us-east-1`→FinRA, `ap-south-1`→SEBI.
- Internal DSNs: `DATABASE_URL=postgres://…@postgres:5432/algotrader`,
  `REDIS_URL=redis://…@redis:6379/0`, `NATS_URL=nats://nats:4222`.

## Known issues / gotchas

- **`/ai-decisions` HTTP 500 — RESOLVED 2026-06-23 by `docker compose ... restart dashboard`.**
  Root cause was **stale runtime state in the dashboard container** (OPcache / cached
  `bootstrap/cache/routes-v7.php` serving broken state), NOT the code: the controller
  rendered **200** via `artisan tinker` / kernel sub-request for both the admin tenant
  (0 rows) and the owner tenant (23k rows), and the 500 hit even the 0-row admin — i.e.
  it was data-independent and SAPI/cache-specific. **Fix / if it recurs:**
  `make restart` (or `docker compose -f infrastructure/docker/docker-compose.yml --env-file .env restart dashboard`),
  optionally `php artisan optimize:clear` inside the container.
- **Debugging dashboard 500s:** `APP_DEBUG` is hardcoded `"false"` in
  [docker-compose.yml](infrastructure/docker/docker-compose.yml) (overrides the container
  `.env`), so errors show a generic "Server Error" page. Do **not** flip it to `true` on
  the public (`0.0.0.0:8130`) dashboard. Instead reproduce in-process:
  `docker compose ... exec -u www-data -e HOME=/tmp dashboard php artisan tinker` →
  `Auth::loginUsingId(<id>)`, build a `Request`, and run it through the HTTP kernel to
  catch the real exception. Tenant RLS is set by `SetTenantContext` middleware via
  `set_config('app.tenant_id', …)`; the hub connects as the RLS-enforced `algotrader_app`
  role. (Tip: set `HOME=/tmp` so PsySH doesn't fail writing `/var/www/.config/psysh`.)
- Logs: app emits to `storage/logs/laravel.log` (mail/debug only seen so far); apache
  access logs go to container stdout (`docker logs algotrader-dashboard-1`).
- Inertia full-page version mismatch shows as **HTTP 409** (a reload signal), not an
  error — don't confuse it with the 500 above.

## Docs index (`docs/`)

`ARCHITECTURE.md` · `SAAS_ARCHITECTURE.md` · `AI_PROVIDERS.md` · `NEWS.md` ·
`PORTFOLIO.md` · `RISK.md` · `PERFORMANCE.md` · `SECURITY.md` · `COMPLIANCE.md` ·
`RUNBOOK.md` · `UX.md` · `LAUNCH_AUDIT.md`
