"""AI Risk Manager — dynamic, explainable risk sizing.

Turns market + portfolio + sentiment context into position size, stop, take
profit, leverage, exposure limit, and a confidence score, scaled by the user's
risk profile. ADVISORY: the Rust risk engine still validates and is final.
Auto-reduces (or halts) under high volatility, major news, exchange outage, or
black-swan conditions."""

from __future__ import annotations

from dataclasses import dataclass, field

from .profiles import RiskProfile, resolve

# Multiplicative risk factor per market regime.
REGIME_FACTOR = {"trending": 1.0, "ranging": 0.8, "volatile": 0.5, "unknown": 0.7}
# Auto-reduce factors; outage and black swan effectively halt new risk.
AUTO_REDUCE = {
    "high_volatility": 0.4,
    "major_news_event": 0.5,
    "exchange_outage": 0.0,
    "black_swan_event": 0.0,
}


@dataclass
class RiskInputs:
    equity_usd: float
    symbol: str
    side: str                      # long|short
    entry: float
    stop: float | None = None
    atr: float | None = None
    profile: str = "balanced"
    profile_overrides: dict | None = None
    volatility_pct: float = 1.0    # ATR/price * 100
    market_regime: str = "unknown"
    news_score: float = 0.0        # -100..100
    news_confidence: float = 0.0   # 0..100
    portfolio_exposure_pct: float = 0.0
    win_rate: float = 0.5          # 0..1
    sharpe: float = 1.0
    funding_rate: float = 0.0
    open_interest_z: float = 0.0   # standardized OI
    conditions: list[str] = field(default_factory=list)


def compute(inp: RiskInputs) -> dict:
    p: RiskProfile = resolve(inp.profile, inp.profile_overrides)
    reasons: list[str] = []
    factor = 1.0

    # 1. Regime.
    rf = REGIME_FACTOR.get(inp.market_regime, 0.7)
    factor *= rf
    reasons.append(f"regime={inp.market_regime} x{rf}")

    # 2. Volatility taper (higher vol => smaller size).
    vol_factor = max(0.3, min(1.0, 1.5 / (1.0 + inp.volatility_pct)))
    factor *= vol_factor
    reasons.append(f"volatility={inp.volatility_pct:.2f}% x{vol_factor:.2f}")

    # 3. News alignment (advisory): contradicting, confident news shrinks risk.
    direction = 1.0 if inp.side == "long" else -1.0
    aligned = direction * inp.news_score
    if inp.news_confidence >= 50 and aligned < -30:
        factor *= 0.5
        reasons.append("news_contradicts x0.5")
    elif inp.news_confidence >= 60 and aligned > 40:
        factor *= 1.1
        reasons.append("news_aligned x1.1")

    # 4. Auto-reduce conditions (can halt).
    applied = []
    for c in inp.conditions:
        if c in AUTO_REDUCE:
            factor *= AUTO_REDUCE[c]
            applied.append(c)
            reasons.append(f"auto_reduce:{c} x{AUTO_REDUCE[c]}")

    # 5. Effective risk %, capped by exposure headroom.
    eff_risk_pct = p.risk_per_trade_pct * max(0.0, factor)
    headroom_pct = max(0.0, p.max_exposure_pct - inp.portfolio_exposure_pct)

    # 6. Stop / risk-per-unit.
    if inp.stop is not None and abs(inp.entry - inp.stop) > 0:
        risk_per_unit = abs(inp.entry - inp.stop)
        stop = inp.stop
    elif inp.atr:
        risk_per_unit = p.stop_atr_mult * inp.atr
        stop = inp.entry - direction * risk_per_unit
    else:
        return {"error": "no_stop_or_atr", "position_size": 0.0, "confidence_score": 0.0}

    take_profit = inp.entry + direction * risk_per_unit * p.tp_rr

    # 7. Size and clamp by exposure + leverage notional.
    risk_budget = inp.equity_usd * (eff_risk_pct / 100.0)
    qty = risk_budget / risk_per_unit if risk_per_unit > 0 else 0.0
    notional = qty * inp.entry
    max_notional = inp.equity_usd * min(p.max_leverage, max(0.0, headroom_pct / 100.0 * p.max_leverage) or p.max_leverage)
    exposure_cap = inp.equity_usd * (headroom_pct / 100.0)
    if notional > exposure_cap > 0:
        qty *= exposure_cap / notional
        notional = exposure_cap
        reasons.append("clamped_by_exposure")
    if notional > max_notional > 0:
        qty *= max_notional / notional
        notional = max_notional
        reasons.append("clamped_by_leverage")

    leverage = round(min(p.max_leverage, max(1.0, notional / max(inp.equity_usd, 1e-9))), 2)

    # 8. Confidence: history + news + regime, dampened by auto-reduce.
    base_conf = 100.0 * (0.5 * inp.win_rate + 0.5 * min(1.0, max(0.0, inp.sharpe / 3.0)))
    news_adj = (inp.news_confidence / 100.0) * (1.0 if aligned >= 0 else -1.0) * 10.0
    confidence = max(0.0, min(100.0, base_conf + news_adj)) * (factor if applied else 1.0)

    return {
        "symbol": inp.symbol,
        "side": inp.side,
        "position_size": round(qty, 8),
        "notional_usd": round(notional, 2),
        "stop_loss": round(stop, 8),
        "take_profit": round(take_profit, 8),
        "leverage": leverage,
        "exposure_limit_pct": round(p.max_exposure_pct, 2),
        "effective_risk_pct": round(eff_risk_pct, 4),
        "confidence_score": round(confidence, 1),
        "auto_reductions": applied,
        "explain": {"profile": p.name, "factor": round(factor, 3), "reasons": reasons},
    }
