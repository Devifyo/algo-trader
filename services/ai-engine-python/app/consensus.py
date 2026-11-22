"""AI Consensus engine. Parses each provider's structured verdict and combines
them by majority vote, weighted average, and confidence averaging. Used for
both trade-signal scoring and news-article sentiment."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass


@dataclass
class Verdict:
    provider: str
    bias: str            # bullish | bearish | neutral
    score: float         # -100..+100
    confidence: float    # 0..100
    weight: float = 1.0


def parse_verdict(provider: str, text: str, weight: float = 1.0) -> Verdict:
    """Best-effort parse of a model's JSON reply; falls back to neutral."""
    obj = _extract_json(text)
    bias = str(obj.get("bias", "neutral")).lower()
    if bias not in ("bullish", "bearish", "neutral"):
        bias = "neutral"
    return Verdict(
        provider=provider,
        bias=bias,
        score=_clamp(_num(obj.get("score"), 0.0), -100, 100),
        confidence=_clamp(_num(obj.get("confidence"), 0.0), 0, 100),
        weight=weight,
    )


def aggregate(verdicts: list[Verdict], mode: str = "weighted") -> dict:
    """mode: majority | weighted | confidence_avg. Returns the consensus dict."""
    if not verdicts:
        return {"bias": "neutral", "score": 0.0, "confidence": 0.0, "label": "no_data"}

    total_w = sum(v.weight for v in verdicts) or 1.0
    weighted_score = sum(v.score * v.weight for v in verdicts) / total_w
    avg_conf = sum(v.confidence * v.weight for v in verdicts) / total_w

    if mode == "majority":
        bias = _majority_bias(verdicts)
    else:
        bias = "bullish" if weighted_score > 10 else "bearish" if weighted_score < -10 else "neutral"

    # Agreement sharpens confidence; disagreement dampens it.
    agree = sum(1 for v in verdicts if v.bias == bias) / len(verdicts)
    confidence = avg_conf * (0.5 + 0.5 * agree)

    return {
        "bias": bias,
        "score": round(weighted_score, 1),
        "confidence": round(confidence, 1),
        "label": _confidence_label(confidence),
        "agreement": round(agree, 2),
        "providers": [{"provider": v.provider, "bias": v.bias, "score": v.score,
                       "confidence": v.confidence, "weight": v.weight} for v in verdicts],
    }


def _majority_bias(verdicts: list[Verdict]) -> str:
    tally: dict[str, float] = {}
    for v in verdicts:
        tally[v.bias] = tally.get(v.bias, 0.0) + v.weight
    return max(tally, key=tally.get)


def _confidence_label(c: float) -> str:
    if c >= 85:
        return "very_high"
    if c >= 70:
        return "high"
    if c >= 50:
        return "medium"
    if c >= 30:
        return "low"
    return "very_low"


def _extract_json(text: str) -> dict:
    try:
        return json.loads(text)
    except Exception:
        m = re.search(r"\{.*\}", text, re.DOTALL)
        if m:
            try:
                return json.loads(m.group(0))
            except Exception:
                return {}
    return {}


def _num(v, default: float) -> float:
    try:
        return float(v)
    except (TypeError, ValueError):
        return default


def _clamp(v: float, lo: float, hi: float) -> float:
    return max(lo, min(hi, v))
