# execution-service (deployment seam)

Deployment seam for the `execution` binary in `services/trading-engine-rust`.
Consumes `tenant.*.orders`, routes to the paper simulator (default) or live
venue adapters, and persists order/fill/position with the owning `tenant_id`.
Live routing pulls each tenant's keys from its Vault partition into memory only
for the duration of one order, then drops them.

Run standalone with `command: ["execution"]`. On a single VPS it runs inside
`engine`.
