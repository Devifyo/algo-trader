# Architecture

## Design principles

1. **The hot path is sacred.** Everything on the order-critical path
   (market-data → signal → risk → execution) lives in the Rust workspace, shares
   no GC, and never makes a synchronous call to Python/PHP. AI and the dashboard
   are *advisory* and *observational* — they can never block or slow a fill.
2. **Fail closed.** Any ambiguity in risk state, connectivity, or config halts
   *new* entries and (optionally) flattens. Capital preservation > opportunity.
3. **Everything is an event.** Services communicate over NATS subjects, never by
   direct RPC on the hot path. This decouples scaling and lets us replay.
4. **Stateless compute, stateful stores.** Engine instances hold only ephemeral
   working state (mirrored to Redis); the source of truth is Timescale + Redis.

## Service responsibilities

### trading-engine-rust (hot path)
A Cargo workspace. Binaries map 1:1 to deployable services so they scale
independently in Kubernetes:

| Crate | Binary | Role |
|-------|--------|------|
| `core` | (lib) | Domain types (`Symbol`, `Order`, `Position`, `Signal`), NATS subjects, Redis keys, config, metrics. Shared by all. |
| `market-data` | `market-data` | Connects to exchange WebSockets, normalizes ticks/candles/order-books, publishes `market.*`, writes Timescale, caches latest to Redis. |
| `signals` | `signals` | Subscribes `market.*`, runs strategies (trend, momentum, mean-reversion, breakout, stat-arb, vol-expansion), optionally asks the AI layer for a score, publishes `signals.*`. |
| `risk` | `risk` | Subscribes `signals.*`, validates against limits (per-trade %, drawdown, leverage, max positions, kill-switch), sizes the position, publishes approved `orders.*`. The only path to an order. |
| `execution` | `execution` | Subscribes `orders.*`, routes to the exchange (or the paper simulator), manages order lifecycle, publishes `executions.*`, updates positions. |
| `engine` | `engine` | Single-process supervisor that runs all of the above in one binary (for dev / small deployments). |

### ai-engine-python
FastAPI service. Exposes `/score`, `/regime`, `/volatility` and also runs a NATS
subscriber that answers `signals.score.request` with `signals.score.response`.
Models: gradient-boosted scorers (XGBoost/LightGBM) + a small PyTorch regime
classifier. **Advisory only** — a timeout means the signal proceeds on its rule
score alone (fail-open for *advice*, never for *risk*).

### dashboard/laravel
Admin + operator surface. Reads Timescale/Redis for analytics, writes strategy
config and API-key references (keys themselves live in Vault). Issues control
commands (pause strategy, kill-switch) by publishing to NATS `control.*`.

## Data flow & subjects

```
market.{exchange}.{symbol}.tick
market.{exchange}.{symbol}.candle.{tf}
market.{exchange}.{symbol}.book
signals.{strategy}.{symbol}            # raw signal
signals.score.request / .response      # AI scoring RPC over NATS
orders.new / orders.cancel             # risk-approved intents
executions.fill / executions.reject    # exchange results
positions.update
control.kill_switch / control.strategy # operator commands
alerts.{severity}
```

## Storage model

- **Redis** — latest price per symbol (`px:{ex}:{sym}`), active positions
  (`pos:{ex}:{sym}`), order queues (lists), session/snapshot. TTLs everywhere.
- **TimescaleDB** — hypertables: `candles`, `ticks`, `order_books`,
  `funding_rates`, `indicators`, plus relational `orders`, `fills`, `positions`,
  `pnl_snapshots`, `strategies`, `audit_log`. See `infrastructure/docker/postgres/init`.
- **Vault** — `secret/exchanges/{exchange}` holds api_key/secret/passphrase.

## Latency budget (p99)

| Hop | Budget | Where measured |
|-----|--------|----------------|
| WS frame → normalized tick on NATS | 10 ms | `market-data` histogram |
| tick → signal emitted | 20 ms | `signals` histogram |
| signal → risk verdict | 5 ms | `risk` histogram |
| approved order → exchange ack | 50 ms | `execution` histogram |

Each is a Prometheus histogram (`*_latency_seconds`) with alerting on p99 breach.

## Failure & recovery

- Engine crash → k8s/compose `restart: unless-stopped` + on boot it rebuilds
  working state from Redis/Timescale and **reconciles open orders** with the
  exchange before resuming.
- NATS partition → execution keeps managing existing orders; signals stop being
  produced (fail-closed on new entries).
- Exchange WS drop → exponential-backoff reconnect; stale-price guard halts
  trading on a symbol whose data is older than `STALE_MS`.
- Kill switch (`control.kill_switch` / Redis `kill_switch`) → flatten + halt,
  honored by `risk` (no new orders) and `execution` (cancel + market-close).
