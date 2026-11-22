# Operations runbook

## Daily
- Check Grafana "Overview" (PnL, open positions, error rate, latency p99).
- Confirm all services `healthy` (`make health`).
- Review `audit_log` for any config/control changes.

## Bring-up order (small box / cold start)
```bash
make up-infra        # postgres, redis, nats, vault — wait for healthy
make up-core         # engine + ai
make up-app          # dashboard + monitoring
make health
```

## Incidents

### "PnL dropping fast" / strategy misbehaving
1. `./scripts/kill-switch.sh on`  ← flatten + halt immediately.
2. Confirm in Grafana that open positions → 0 and no new orders.
3. Investigate via Loki logs (`{service="execution"}`) and `executions.reject`.
4. Disable the offending strategy in the dashboard before clearing the switch.

### Exchange WebSocket flapping
- Check `reconnects_total` metric. The stale-price guard should have halted that
  symbol automatically. Verify exchange status page before forcing reconnect.

### NATS down
- New signals stop (fail-closed). `execution` keeps managing existing orders.
- Restart NATS; engines reconnect automatically with backoff.

### Database full / slow
- `make backup` then prune old hypertable chunks per retention policy.
- Timescale compression should be on for chunks older than 7d (see init SQL).

## Recovery after crash
Engine binaries reconcile on boot: they pull open orders from each exchange and
compare to local state in Timescale before resuming. Never start `execution`
with a stale local state and a live exchange without reconciliation — it is
automatic but verify `reconciliation_complete` log line appears.

## Backups
- `scripts/backup.sh` → `pg_dump` (logical) nightly + WAL archiving for PITR in
  production. Test a restore monthly.
