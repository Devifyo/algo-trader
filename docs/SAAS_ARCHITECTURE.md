# Multi-tenant SaaS architecture

The platform is a **hub-and-spoke** multi-tenant SaaS.

```
                ┌──────────────────────── HUB (control plane) ───────────────────────┐
                │  Laravel 12 + Vue — onboarding, billing, dashboards, admin,        │
                │  AI-provider config, global risk analytics. RLS-restricted DB role. │
                └───────────────┬────────────────────────────────┬───────────────────┘
                                │                                 │
              ┌─────────────────▼────────┐         ┌─────────────▼────────────────┐
              │ SPOKE: us-east-1 (FinRA)  │         │ SPOKE: ap-south-1 (SEBI)      │
              │ Rust engine (per region)  │         │ Rust engine (per region)      │
              │ market-data│signals│risk  │         │ + SEBI: 10 ord/s cap, broker  │
              │ │execution  + AI/news/pf  │         │   whitelist, static egress IP │
              └───────────────────────────┘         └───────────────────────────────┘
```

## Tenant isolation — three enforcing layers

1. **Postgres Row-Level Security.** Every private table carries `tenant_id` and a
   `tenant_isolation` policy keyed on `current_setting('app.tenant_id')`. The hub
   connects as the restricted `algotrader_app` role (RLS enforced) and runs
   `SET app.tenant_id` per request; the engine connects as the table owner
   (trusted data-plane, writes `tenant_id` explicitly). See
   `infrastructure/docker/postgres/init/03_multitenancy.sql`.
2. **NATS subjects.** Per-tenant subjects (`tenant.{id}.signals`,
   `tenant.{id}.orders`, …) so one tenant's runaway, high-frequency strategy can
   never starve another's execution queue. The regional engine subscribes to the
   `tenant.*.{signals,orders}` wildcards.
3. **Vault partitions.** Each tenant's broker keys live at
   `secret/tenants/{id}/brokers/{broker}`. The engine pulls keys into memory only
   at strategy start and drops them on stop / kill-switch.

## What is and isn't tenant-scoped

- **Shared (per region):** market data (candles/ticks/order-books) — it's public,
  so duplicating per tenant would waste storage; the AI provider catalog; the
  news corpus + sentiment (advisory, public).
- **Tenant-private (RLS):** strategies, orders, fills, positions, pnl, audit,
  bots, portfolios, allocations, AI decisions, exchange-key references,
  marketplace subscriptions, copy-trading follows, onboarding progress.

## Regional spokes

A spoke is one regional deployment of the Rust engine (`REGION` env). It multiplexes
all tenants in that region and applies the region's compliance regime
(`docs/COMPLIANCE.md`). Run market-data/signals/risk/execution as one `engine`
process on a single VPS, or split them into separate binaries per region.

## Kubernetes — deferred

The platform currently targets a **single VPS with Docker Compose**, tuned for
latency and resource efficiency (`docs/PERFORMANCE.md`). The
`infrastructure/kubernetes/` directory is a placeholder for the future
multi-region, multi-cluster rollout (NATS supercluster across regions, per-region
node pools, Vault auto-unseal). It is intentionally **not** wired yet.
