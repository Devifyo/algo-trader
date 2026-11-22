# Risk policy & go-live checklist

> The risk engine is the **only** code path that can produce an order. It is
> deliberately conservative and fails closed.

## Hard limits (enforced in `risk` crate)

| Rule | Default | Env |
|------|---------|-----|
| Max risk per trade | 0.5% of equity | `RISK_MAX_PER_TRADE_PCT` |
| Daily drawdown halt | 3.0% | `RISK_DAILY_DRAWDOWN_PCT` |
| Weekly drawdown halt | 8.0% | `RISK_WEEKLY_DRAWDOWN_PCT` |
| Max leverage | 5× | `RISK_MAX_LEVERAGE` |
| Max concurrent open positions | 10 | `RISK_MAX_OPEN_POSITIONS` |
| Kill switch | off | `KILL_SWITCH` / Redis `kill_switch` |

### Position sizing
`qty = (equity * risk_per_trade_pct) / (entry - stop)` per unit, then clamped by
leverage, notional caps, and exchange lot/step rules. A signal with no stop is
**rejected** — we never size against undefined risk.

### Circuit breakers
- Per-symbol stale-data guard (`STALE_MS`).
- Order-reject-rate breaker: > N rejects/min on an exchange → pause that venue.
- PnL velocity breaker: loss faster than X%/min → kill switch.
- Reconnect-storm breaker: > N reconnects/min → pause venue.

## Go-live checklist (do NOT skip)

- [ ] Backtest each strategy on ≥ 2 years of Timescale candles; document Sharpe,
      max drawdown, win rate, expectancy.
- [ ] Paper-trade the full stack for ≥ 2 weeks; reconcile paper PnL vs. what the
      exchange *would* have filled (slippage model validated).
- [ ] Run on each exchange **testnet** with real API plumbing for ≥ 1 week.
- [ ] Load real keys into Vault (never .env); verify IP allow-list on the
      exchange side; verify withdrawal permission is **disabled** on the keys.
- [ ] Set hard sub-account balance caps at the exchange.
- [ ] Verify kill-switch flattens within target time under load (drill it).
- [ ] Start live with the smallest viable size; ramp only after a clean week.
- [ ] Confirm alerting (Grafana → Alertmanager → pager) actually pages a human.

## Things this scaffold does NOT yet do (must build before live)
- Real signed order placement for each venue (see `execution/src/exchanges/` —
  currently the paper simulator is wired; venue adapters are stubs with the
  request/sign shape laid out).
- Funding-rate-aware and fee-aware sizing.
- Cross-exchange netting for stat-arb.
- Tax-lot / accounting export.
