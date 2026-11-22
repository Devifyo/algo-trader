"""News Influence Engine. Converts advisory inputs (technical conviction, news
sentiment, on-chain bias) into a *combined confidence* and a non-binding
trade-allowed hint. This is ADVISORY ONLY — the Rust risk engine is the final
authority and performs the real gate. We expose the same math here so the
dashboard can show users why confidence rose or fell."""

from __future__ import annotations

# Blend weights (sum to 1.0). Technical leads; news/on-chain modulate.
W_TECH, W_NEWS, W_ONCHAIN = 0.55, 0.25, 0.20
TRADE_ALLOW_THRESHOLD = 55.0   # combined confidence floor (advisory)


def combined_confidence(
    technical_pct: float,          # 0..100 strategy conviction
    news_score: float,             # -100..100 (signed sentiment)
    news_conf: float,              # 0..100
    side_is_long: bool,
    onchain_score: float = 0.0,    # -100..100
) -> dict:
    """Returns {confidence, trade_allowed, alignment}. News that contradicts the
    trade direction reduces confidence; aligned news increases it."""
    direction = 1.0 if side_is_long else -1.0

    # Align signed sentiment with the trade direction → [0..100] support.
    news_support = max(0.0, min(100.0, 50.0 + direction * news_score * 0.5))
    onchain_support = max(0.0, min(100.0, 50.0 + direction * onchain_score * 0.5))

    # Weight news by its own confidence; unknown news stays neutral (50).
    news_component = (news_support * (news_conf / 100.0)) + 50.0 * (1 - news_conf / 100.0)

    confidence = (
        W_TECH * technical_pct
        + W_NEWS * news_component
        + W_ONCHAIN * onchain_support
    )
    alignment = direction * news_score  # >0 aligned, <0 contradicts
    return {
        "confidence": round(confidence, 1),
        "trade_allowed": confidence >= TRADE_ALLOW_THRESHOLD,
        "alignment": round(alignment, 1),
    }
