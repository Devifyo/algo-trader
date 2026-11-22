"""Unified AI Decision Layer. The central orchestrator that folds every module's
output into one explainable set of scores. ADVISORY: it produces a
recommendation + a full explanation, but the Rust risk engine remains the final
authority and trade execution is never blocked by this layer."""

from __future__ import annotations


def synthesize(inp: dict) -> dict:
    """inp keys (all optional, sensible defaults):
       side, technical_pct, news_score, news_confidence, regime, regime_confidence,
       health_score, portfolio_warnings, exposure_pct, macro_event_soon,
       risk_manager_confidence."""
    side = inp.get("side", "long")
    direction = 1.0 if side == "long" else -1.0
    tech = float(inp.get("technical_pct", 50))
    news = float(inp.get("news_score", 0))
    news_conf = float(inp.get("news_confidence", 0))
    regime = inp.get("regime", "unknown")
    regime_conf = float(inp.get("regime_confidence", 0.5))
    health = float(inp.get("health_score", 70))
    warnings = inp.get("portfolio_warnings", []) or []
    exposure = float(inp.get("exposure_pct", 0))
    macro = bool(inp.get("macro_event_soon", False))
    rm_conf = float(inp.get("risk_manager_confidence", tech))

    # Directional support from news (weighted by its confidence).
    news_support = direction * news * (news_conf / 100.0)   # -100..100
    regime_support = _regime_support(side, regime) * regime_conf * 100.0

    bullish = _clamp(50 + 0.4 * (tech - 50) + 0.3 * max(0, news_support) + 0.3 * max(0, regime_support))
    bearish = _clamp(100 - bullish)

    risk_factors: list[str] = []
    risk = 20.0
    if macro:
        risk += 25; risk_factors.append("macro_event_window")
    if news_support < -30:
        risk += 20; risk_factors.append("news_contradicts")
    if exposure > 70:
        risk += 15; risk_factors.append("high_exposure")
    risk += 0.5 * len(warnings) * 10
    for w in warnings:
        risk_factors.append(w)
    risk = _clamp(risk)

    market_score = _clamp(regime_support + 50)
    portfolio_score = _clamp(100 - 12 * len(warnings) - max(0, exposure - 60))
    confidence = _clamp(0.35 * rm_conf + 0.25 * tech + 0.20 * health
                        + 0.20 * (50 + news_support / 2) - 0.3 * (risk - 20))

    if risk >= 70 or confidence < 45:
        decision = "reject"
    elif risk >= 50 or confidence < 60:
        decision = "hold"
    else:
        decision = "allow"

    reason = _reason(decision, side, regime, news_support, risk_factors, confidence)
    return {
        "decision": decision,
        "scores": {
            "bullish_score": round(bullish, 1),
            "bearish_score": round(bearish, 1),
            "risk_score": round(risk, 1),
            "confidence_score": round(confidence, 1),
            "market_score": round(market_score, 1),
            "portfolio_score": round(portfolio_score, 1),
        },
        # Mandatory explainability — never a black box.
        "explainability": {
            "reason": reason,
            "data_used": ["technical", "news_sentiment", "market_regime",
                          "strategy_health", "portfolio_exposure",
                          "macro_calendar" if macro else "no_macro_event"],
            "confidence": round(confidence, 1),
            "risk_factors": risk_factors or ["none"],
            "expected_outcome": _expected(decision, confidence),
        },
    }


def _regime_support(side: str, regime: str) -> float:
    bull = regime in ("bull_market", "trend_market")
    bear = regime in ("bear_market",)
    if side == "long":
        return 0.6 if bull else -0.6 if bear else 0.0
    return 0.6 if bear else -0.6 if bull else 0.0


def _reason(decision, side, regime, news_support, risk_factors, conf) -> str:
    bias = "supports" if news_support >= 0 else "opposes"
    rf = ", ".join(risk_factors) if risk_factors else "no notable risks"
    verb = {"allow": "Cleared", "hold": "Holding", "reject": "Blocked"}[decision]
    return (f"{verb} a {side} idea in a {regime.replace('_', ' ')}. News {bias} the "
            f"direction; confidence {conf:.0f}/100. Risk factors: {rf}.")


def _expected(decision, conf) -> str:
    if decision == "allow":
        return f"Positive expectancy if thesis holds (confidence {conf:.0f}%)."
    if decision == "hold":
        return "Wait for confirmation; signal not strong enough yet."
    return "Avoid: risk outweighs expected reward right now."


def _clamp(v: float) -> float:
    return max(0.0, min(100.0, v))
