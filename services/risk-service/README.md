# risk-service (deployment seam)

Deployment seam for the `risk` binary in `services/trading-engine-rust`. The
risk service is the **only** path that produces an order: it gates every signal
through kill-switch, tenant-halt, stop-defined, stale-data, max-open,
daily-drawdown, AI floor, and **regional compliance** (FinRA PDT, SEBI order-rate
cap + broker whitelist) before sizing the position.

Run standalone with `command: ["risk"]`. On a single VPS it runs inside `engine`.
