# Security

## Secrets
- Exchange API keys **never** live in `.env`, git, or the database. They live in
  **Vault** at `secret/exchanges/{exchange}` (`api_key`, `api_secret`,
  `passphrase`). The engine reads them at boot via the Vault API using a scoped
  AppRole token; the dashboard only stores a *reference*, never the secret.
- Dev runs Vault in `-dev` mode (in-memory, root token). **Production** must run
  sealed Vault with auto-unseal (KMS) and audit logging enabled.
- At rest, any cached key material is encrypted; keys are redacted from all logs.

## Exchange key hygiene
- Create **trade-only** keys: withdrawals **disabled**.
- Bind keys to the server's egress IP (exchange-side allow-list).
- Use sub-accounts with hard balance caps.

## Network
- Only the dashboard (and ingress) are reachable externally. Data plane
  (postgres/redis/nats/vault) binds to `127.0.0.1` / the internal docker network.
- Mutual TLS between services in the k8s overlay; NATS auth tokens per service.

## Audit
- Every config change, control command (kill-switch, strategy pause), and login
  is written to `audit_log` (immutable, append-only) and shipped to Loki.

## Fail-safe shutdown
- `KILL_SWITCH=on` / Redis `kill_switch=1` / `control.kill_switch` NATS message →
  risk rejects all new orders, execution cancels working orders and market-closes
  open positions. Honored even if the dashboard is down.

## AuthN/Z (dashboard)
- Laravel auth + roles (admin / operator / read-only). Operator actions
  (kill-switch, go-live) require re-auth and are rate-limited and audited.
