# Dashboard (Laravel 12 + Inertia + Vue 3)

The **control plane / hub**: tenant onboarding, portfolio + bot management,
AI-provider admin, News Intelligence, Marketplace, Copy Trading, the exchange
API-key safety flow, the emergency kill switch, the help center, and the
onboarding guide.

## How it's built

The `Dockerfile` is multi-stage:
1. **php-base** — `composer create-project laravel/laravel:^12` for a correct
   framework base, then adds Inertia + Ziggy + Predis, then overlays our
   authored code from `overlay/` (controllers, middleware, routes, Inertia
   views, bootstrap wiring).
2. **frontend** — installs Vue 3 / Inertia / Tailwind and runs `vite build`.
3. **runtime** — `php:8.4-apache` serving `public/`.

Everything we author lives in [`overlay/`](overlay/). The generated Laravel base
is intentionally not committed — it's produced at build time.

## Security & tenancy

- Connects as the **RLS-restricted `algotrader_app`** role (not the engine
  owner). `SetTenantContext` runs `SET app.tenant_id` each request, so Postgres
  Row-Level Security filters every query to the current tenant.
- Exchange API secrets are **never** stored in Postgres — only a Vault reference.
  The connect flow calls the safety service and refuses withdrawal-enabled keys.
- The kill switch sets the Redis `kill_switch` the Rust engine honors directly,
  so it works even if the dashboard later goes down.

## Design system

Dark-first, calm fintech aesthetic (display *Sora* / body *Manrope*), built for
TradingView-grade clarity and Stripe/Linear/Notion-level simplicity. Shared
components in `overlay/resources/js/Components` cover every state (empty /
loading / error / success), tooltips, contextual help, risk badges, and health
scores. Each page passes a "confusion audit": what it is, why it matters, what
enabling it does, the risks, and the recommended setting.

## Local notes

Served at `http://127.0.0.1:8130`. Bring it up with `make up-app` after the data
plane (`make up-infra`) is healthy.
