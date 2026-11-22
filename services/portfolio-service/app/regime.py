"""Market Regime Engine. Classifies the market into the seven regimes the spec
lists and maps each to preferred strategies, so the platform can adapt which
strategies it favors. Detection is from trend efficiency + volatility."""

from __future__ import annotations

import numpy as np

PREFERRED = {
    "bull_market": ["trend_following", "breakout", "momentum"],
    "bear_market": ["short_selling", "mean_reversion"],
    "sideways_market": ["mean_reversion", "grid"],
    "high_volatility": ["breakout", "vol_expansion"],
    "low_volatility": ["mean_reversion", "grid"],
    "trend_market": ["trend_following", "momentum"],
    "range_market": ["mean_reversion", "grid"],
}


def classify(closes: list[float]) -> dict:
    arr = np.asarray(closes, dtype=float)
    if arr.size < 20:
        return {"regime": "unknown", "confidence": 0.0, "preferred_strategies": []}

    rets = np.diff(arr) / np.clip(arr[:-1], 1e-9, None)
    vol = float(np.std(rets))
    look = min(arr.size, 50)
    net = arr[-1] - arr[-look]
    path = float(np.sum(np.abs(np.diff(arr[-look:])))) + 1e-9
    efficiency = abs(net) / path                      # ~1 trend, ~0 chop
    drift = net / arr[-look]                           # signed trend

    # Volatility regime overlay.
    median_abs = float(np.median(np.abs(rets))) + 1e-9
    vol_ratio = vol / median_abs

    if vol_ratio > 2.0:
        regime, conf = "high_volatility", min(1.0, vol_ratio / 3.0)
    elif vol_ratio < 0.6:
        regime, conf = "low_volatility", min(1.0, 1.0 - vol_ratio)
    elif efficiency > 0.45:
        if drift > 0.02:
            regime, conf = "bull_market", min(1.0, efficiency)
        elif drift < -0.02:
            regime, conf = "bear_market", min(1.0, efficiency)
        else:
            regime, conf = "trend_market", min(1.0, efficiency)
    else:
        regime, conf = ("sideways_market" if abs(drift) < 0.01 else "range_market",
                        min(1.0, 1.0 - efficiency))

    return {
        "regime": regime,
        "confidence": round(float(conf), 3),
        "efficiency": round(float(efficiency), 3),
        "volatility_ratio": round(float(vol_ratio), 2),
        "preferred_strategies": PREFERRED.get(regime, []),
    }
