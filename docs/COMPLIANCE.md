# Regional compliance

Enforced inside the `risk` crate (`core::compliance`) before any order is
approved. Hard gates — a violation rejects the order.

## USA — FinRA Pattern Day Trading (us-east-1)

- Accounts **under $25,000 equity** may not place a **4th day-trade within any
  rolling 5 business days**. `PdtGuard` tracks day-trades per tenant and blocks
  the 4th. Above the threshold, PDT does not apply.

## India — SEBI 2026 algo constraints (ap-south-1)

- **Order-rate cap:** per-tenant **10 orders/second** (sliding window,
  `OrderRateLimiter`) to stay below the HFT-classification threshold. Excess
  orders are rejected and metered (`orders_rate_limited_total`).
- **Approved/empanelled brokers only:** orders route only to whitelisted brokers
  (`compliance::broker_whitelisted`); the venue is resolved by `SEBI_BROKER`.
- **Static egress Elastic IP:** the spoke's broker calls must originate from a
  fixed allow-listed IP. Enforced at the infrastructure layer (the regional VM's
  egress IP, recorded per tenant in `tenants.egress_ip`); the exchange-side
  allow-list is configured during onboarding.

## Exchange-key safety (all regions)

- **Withdrawals must be disabled.** The safety layer rejects withdrawal-enabled
  keys; the `exchange_keys` table even enforces `withdraw_enabled = false` as a
  DB constraint. Keys should be IP-restricted and scoped to trading only.

## Go-live gate

A strategy may be promoted paper → forward → live only after meeting the
validation bar (≥30 paper days, ≥100 trades, ≥55% win rate, ≥1.3 profit factor) —
see `health.validate_for_live` and `docs/RISK.md`.
