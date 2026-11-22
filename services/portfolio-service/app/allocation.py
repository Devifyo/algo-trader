"""Smart Capital Allocation — distributes capital across strategies/bots/assets/
exchanges by risk-adjusted performance. Sharpe-led, penalized by drawdown and
volatility, capped per item to enforce diversification."""

from __future__ import annotations

MAX_WEIGHT = 0.40   # no single target gets more than 40% (diversification floor)


def allocate(total_capital: float, candidates: list[dict], rebalance: str = "weekly") -> dict:
    """candidates: [{id, kind, historical_return, sharpe, drawdown, win_rate,
    volatility, regime_compat}] → normalized weights + dollar amounts."""
    if not candidates:
        return {"allocations": [], "rebalance": rebalance}

    scored = []
    for c in candidates:
        sharpe = max(0.0, float(c.get("sharpe", 0)))
        ret = float(c.get("historical_return", 0))           # e.g. 0.2 = +20%
        dd = abs(float(c.get("drawdown", 0)))                # 0.1 = -10%
        vol = max(0.0, float(c.get("volatility", 0)))
        wr = float(c.get("win_rate", 0.5))
        compat = float(c.get("regime_compat", 1.0))          # 0..1
        score = (sharpe * (1.0 + max(-0.9, ret)) * (0.5 + wr) * compat) / (1.0 + dd + vol)
        scored.append((c, max(0.0, score)))

    total = sum(s for _, s in scored)
    if total <= 0:
        # Degrade to equal weight when nothing scores.
        w = 1.0 / len(scored)
        weights = [(c, w) for c, _ in scored]
    else:
        weights = [(c, s / total) for c, s in scored]

    weights = _cap_and_renormalize(weights, MAX_WEIGHT)

    return {
        "rebalance": rebalance,
        "allocations": [
            {
                "id": c.get("id"),
                "kind": c.get("kind"),
                "weight": round(w, 4),
                "amount_usd": round(total_capital * w, 2),
            }
            for c, w in weights
        ],
    }


def _cap_and_renormalize(weights: list[tuple[dict, float]], cap: float) -> list[tuple[dict, float]]:
    """Iteratively cap weights at `cap`, redistributing excess to the rest."""
    if len(weights) <= 1:
        return weights
    cap = max(cap, 1.0 / len(weights))   # cap can't be below equal-weight
    for _ in range(10):
        excess = sum(max(0.0, w - cap) for _, w in weights)
        if excess <= 1e-9:
            break
        uncapped = [(c, w) for c, w in weights if w < cap]
        room = sum(cap - w for _, w in uncapped) or 1.0
        weights = [
            (c, cap if w >= cap else w + excess * ((cap - w) / room))
            for c, w in weights
        ]
    s = sum(w for _, w in weights) or 1.0
    return [(c, w / s) for c, w in weights]
