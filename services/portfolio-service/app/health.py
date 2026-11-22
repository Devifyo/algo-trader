"""Strategy Health Monitor + paper→live validation gate.

`evaluate` scores a strategy's live health and flags decay/degradation/
overfitting/market-mismatch. `validate_for_live` is the promotion gate: a
strategy may go live only after enough paper history at acceptable quality."""

from __future__ import annotations

# Promotion criteria (from the paper_trading_and_forward_testing spec).
MIN_PAPER_DAYS = 30
MIN_TRADES = 100
MIN_WIN_RATE = 0.55
MIN_PROFIT_FACTOR = 1.3


def evaluate(s: dict) -> dict:
    wr = float(s.get("win_rate", 0))
    pf = float(s.get("profit_factor", 0))
    sharpe = float(s.get("sharpe", 0))
    dd = abs(float(s.get("max_drawdown", 0)))
    recent_wr = float(s.get("recent_win_rate", wr))
    in_sh = float(s.get("in_sample_sharpe", sharpe))
    out_sh = float(s.get("out_sample_sharpe", sharpe))
    regime_fit = float(s.get("strategy_regime_fit", 1.0))   # 0..1

    flags: list[str] = []
    if recent_wr < wr - 0.10:
        flags.append("performance_decay")
    if pf < 1.0 or sharpe < 0.5:
        flags.append("strategy_degradation")
    if in_sh > 0 and out_sh < in_sh * 0.5:
        flags.append("overfitting")
    if regime_fit < 0.4:
        flags.append("market_mismatch")

    # Composite health 0..100.
    health = 100.0 * (
        0.30 * min(1.0, wr / 0.6)
        + 0.25 * min(1.0, pf / 2.0)
        + 0.20 * min(1.0, max(0.0, sharpe) / 2.0)
        + 0.15 * max(0.0, 1.0 - dd / 0.3)
        + 0.10 * regime_fit
    )
    health *= (0.8 ** len(flags))   # each flag discounts health

    return {
        "health_score": round(max(0.0, min(100.0, health)), 1),
        "risk_score": round(min(100.0, dd * 200 + (1 - regime_fit) * 50), 1),
        "ai_confidence": round(min(100.0, max(0.0, sharpe) * 30 + wr * 40), 1),
        "market_compatibility": round(regime_fit, 2),
        "flags": flags,
        "warning": bool(flags),
    }


def validate_for_live(s: dict) -> dict:
    """Returns whether a paper/forward-tested strategy may be promoted to live."""
    checks = {
        "paper_days": (float(s.get("paper_days", 0)) >= MIN_PAPER_DAYS, MIN_PAPER_DAYS),
        "trades": (int(s.get("trades", 0)) >= MIN_TRADES, MIN_TRADES),
        "win_rate": (float(s.get("win_rate", 0)) >= MIN_WIN_RATE, MIN_WIN_RATE),
        "profit_factor": (float(s.get("profit_factor", 0)) >= MIN_PROFIT_FACTOR, MIN_PROFIT_FACTOR),
    }
    failed = [k for k, (ok, _) in checks.items() if not ok]
    return {
        "eligible_for_live": not failed,
        "stage": "live_ready" if not failed else "needs_more_validation",
        "criteria": {k: {"pass": ok, "required": req} for k, (ok, req) in checks.items()},
        "failed": failed,
    }
