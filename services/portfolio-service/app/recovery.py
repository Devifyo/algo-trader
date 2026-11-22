"""AI Trade Recovery — analyzes a losing position and recommends ONE action.
Hard safety rules: never average down blindly, never exceed risk limits, always
require risk validation. Advisory only; the Rust risk engine executes/validates."""

from __future__ import annotations

ACTIONS = ["hold", "exit", "reduce_position", "scale_in", "move_stop_loss", "partial_profit_take"]


def recommend(pos: dict, ctx: dict) -> dict:
    """pos: {side, entry, current, stop, qty, unrealized_pct}
    ctx: {atr, news_score, news_confidence, trend, max_risk_pct, current_risk_pct}"""
    side = pos.get("side", "long")
    upl = float(pos.get("unrealized_pct", 0))          # e.g. -3.2
    direction = 1.0 if side == "long" else -1.0
    news_aligned = direction * float(ctx.get("news_score", 0))
    trend = ctx.get("trend", "unknown")
    reasons: list[str] = []

    # Safety: if already at/over risk limits, only de-risking actions allowed.
    over_risk = float(ctx.get("current_risk_pct", 0)) >= float(ctx.get("max_risk_pct", 100))

    if upl <= -8 or (news_aligned < -40 and float(ctx.get("news_confidence", 0)) >= 60):
        action, reasons = "exit", ["loss>=8% or strong contradicting news"]
    elif upl <= -4:
        action, reasons = "reduce_position", ["moderate loss; cut exposure, keep core"]
    elif upl >= 4 and trend in ("ranging", "volatile"):
        action, reasons = "partial_profit_take", ["in profit but choppy regime"]
    elif -2 <= upl < 2 and trend == "trending" and news_aligned > 20 and not over_risk:
        # Disciplined add only with confirming trend + aligned news + risk room.
        action, reasons = "scale_in", ["thesis intact, trend+news aligned, risk room available"]
    elif upl < 0 and trend == "trending" and news_aligned >= 0:
        action, reasons = "move_stop_loss", ["trail stop to reduce risk while thesis holds"]
    else:
        action, reasons = "hold", ["no decisive signal"]

    # Enforce safety rules.
    if action == "scale_in" and over_risk:
        action, reasons = "hold", ["scale-in blocked: at/over risk limit (never_exceed_risk_limits)"]

    return {
        "action": action,
        "requires_risk_validation": True,
        "safety_rules": ["never_average_down_blindly", "never_exceed_risk_limits"],
        "explain": {"unrealized_pct": upl, "trend": trend, "news_aligned": round(news_aligned, 1),
                    "reasons": reasons},
    }
