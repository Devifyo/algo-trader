# algo-trader

A production-grade, container-native algorithmic trading platform. Polyglot by
design: a low-latency **Rust** trading core, a **Python** AI/ML inference layer,
and a **Laravel 12** management dashboard, glued together by **NATS**, backed by
**PostgreSQL/TimescaleDB** + **Redis**, and observed by **Prometheus / Grafana /
Loki**. Secrets in **Vault**.

> ⚠️ **Safety first.** The platform ships in `TRADING_MODE=paper`. No order ever
> reaches a real exchange until you (a) load real keys into Vault, (b) flip the
> per-exchange `*_TESTNET` flags, and (c) set `TRADING_MODE=live`. Do **not** run
> live capital until you have completed the staged validation in
> [`docs/RISK.md`](docs/RISK.md). Trading is risky; you can lose everything.

## Architecture at a glance

```
                 ┌──────────────────────────────────────────────┐
   Exchanges ───►│  trading-engine-rust  (the hot path)         │
  (WS + REST)    │  market-data │ signals │ risk │ execution    │◄── Vault (keys)
                 └──────┬───────────────┬───────────────┬───────┘
                        │ NATS subjects │               │
        market.* ◄──────┘   signals.*   │   orders.* / executions.*
                        ▼               ▼               ▼
                 ┌──────────────┐  ┌───────────┐  ┌──────────────┐
                 │ ai-engine    │  │ Timescale │  │ Redis (hot   │
                 │ (Python ML)  │  │ (history) │  │ state/queues)│
                 └──────────────┘  └─────┬─────┘  └──────────────┘
                                         │
                                 ┌───────▼────────┐   ┌───────────────────┐
                                 │ Laravel 12     │   │ Prometheus/Grafana│
                                 │ admin + API    │   │ + Loki (logs)     │
                                 └────────────────┘   └───────────────────┘
```

Full design: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## Layout

| Path | What |
|------|------|
| `services/trading-engine-rust` | Cargo workspace — the hot path. Member crates: `engine`, `market-data`, `signals`, `risk`, `execution`, `core` (shared domain types + NATS). |
| `services/market-data-service` `execution-service` `risk-service` | Deployment seams — each runs a binary from the Rust workspace so they can scale independently in k8s. |
| `services/ai-engine-python` | FastAPI ML inference: signal scoring, regime detection, volatility forecasting. |
| `dashboard/laravel` | Laravel 12 admin: users, strategy config, analytics, trade history, API mgmt. |
| `infrastructure/docker` | docker-compose stack + per-service container assets. |
| `infrastructure/monitoring` | Prometheus, Grafana provisioning + dashboards, Loki, alert rules. |
| `infrastructure/kubernetes` | Kustomize base + production overlay (skeleton). |
| `scripts` | bootstrap, health, **kill-switch**, backup. |
| `docs` | Architecture, runbook, risk policy, security. |

## Quick start (paper mode)

```bash
cd /var/www/algo-trader
cp .env.example .env                 # then edit secrets
./scripts/bootstrap.sh               # generate keys, init Vault, build images
make up                              # bring the stack up
make health                          # check every service
```

### Access over IP now, add a domain later

The **dashboard is published on all interfaces** so you can reach it immediately
at your server's IP:

- Dashboard:   `http://<server-ip>:8130`   (set `APP_URL` to match in `.env`)

Everything else binds to `127.0.0.1` (data plane + monitoring) for safety —
reach them via an SSH tunnel while developing:

- Grafana `127.0.0.1:8136` · Prometheus `8135` · NATS mon `8134` · AI `8140` ·
  News `8141` · Portfolio `8142`

**Adding a domain later:** put a reverse proxy (nginx/Caddy/Traefik) in front,
point the domain at it, and proxy to the dashboard. Then set
`DASHBOARD_BIND=127.0.0.1` in `.env` (so the app is only reachable through the
proxy) and update `APP_URL` to `https://your-domain`. `trustProxies` is already
enabled for this.

Bring it up in stages on a small box (see `make up-infra` / `make up-core`).

## Emergency stop

```bash
./scripts/kill-switch.sh on          # flatten all positions + halt entries
```

See [`docs/RUNBOOK.md`](docs/RUNBOOK.md) for incident procedures.

## Performance targets

| Stage | Target (p99) |
|-------|------|
| Market-data ingest→normalized | < 10 ms |
| Signal generation | < 20 ms |
| Risk validation | < 5 ms |
| Order submission | < 50 ms |
| Dashboard response | < 100 ms |

Measured continuously; see the "Latency" Grafana dashboard.
