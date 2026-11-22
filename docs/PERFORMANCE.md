# Performance & single-VPS tuning

The platform targets a **single VPS** today (Kubernetes is deferred). It is tuned
for low latency and efficient resource use.

## Latency budget (p99)

| Stage | Target | Where |
|-------|--------|-------|
| Market-data ingest → normalized tick on NATS | < 10 ms | `market-data` histogram |
| Signal generation | < 20 ms | `signals` histogram |
| Risk validation | < 5 ms | `risk` histogram |
| Order submission | < 50 ms | `execution` histogram |
| Dashboard response | < 100 ms | hub |

The hot path (market-data → signal → risk → execution) is entirely in Rust, with
no GC and no synchronous calls to Python/PHP. AI/news/portfolio are advisory and
read asynchronously from Redis/NATS — they can never slow a fill.

## Where the speed comes from

- **Rust workspace**, release profile with `lto=thin`, `codegen-units=1`,
  `panic=abort`. In-process NATS fan-out for the single-binary `engine`.
- **Redis** holds hot state (latest price, positions, sentiment snapshot,
  kill-switch) so the risk path makes no DB round-trip on the critical decision.
- **TimescaleDB** hypertables + compression + retention for history; the hot path
  writes are fire-and-forget.
- **Advisory decoupling:** sentiment/AI/portfolio results are precomputed and
  read as a single Redis GET in the risk gate.

## Resource sizing (4 vCPU / 16 GB reference)

`docker-compose.yml` sets `deploy.resources.limits` per heavy service
(engine 512m, ai-engine 1g, news/portfolio/dashboard 512m). On a small box bring
the stack up in stages:

```bash
make up-infra     # postgres, redis, nats, vault
make up-core      # engine + ai + news + portfolio
make up-app       # dashboard + monitoring
```

## Scaling out (when you outgrow one box)

Split the Rust binaries (`market-data`/`signals`/`risk`/`execution`) across nodes,
run one regional engine per spoke, and move to the deferred Kubernetes overlay
with a NATS supercluster. Postgres/Redis/NATS each scale independently.
