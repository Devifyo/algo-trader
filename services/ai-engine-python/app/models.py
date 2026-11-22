"""Inference logic for the AI layer.

Ships with transparent, defensible *heuristic* models so the service returns
sensible scores out of the box with no training step. Each function is a drop-in
seam: replace the body with a trained XGBoost/LightGBM booster or a PyTorch net
(see requirements-ml.txt) once you have labelled data. Keeping the interface
stable means the Rust signals service never changes.
"""

from __future__ import annotations

import numpy as np


def _returns(closes: list[float]) -> np.ndarray:
    arr = np.asarray(closes, dtype=float)
    if arr.size < 2:
        return np.zeros(0)
    return np.diff(arr) / np.clip(arr[:-1], 1e-9, None)


def score_signal(side: str, strength: float, closes: list[float]) -> float:
    """Probability-like conviction in [0,1] that this signal is worth taking.

    Blends the rule strength with (a) trend alignment of recent returns with the
    proposed side and (b) a volatility penalty — we fade signals fired into chaos.
    """
    rets = _returns(closes)
    if rets.size == 0:
        return float(np.clip(strength, 0.0, 1.0))

    momentum = float(np.tanh(np.mean(rets) * 50.0))          # [-1,1]
    vol = float(np.std(rets))
    vol_penalty = float(np.clip(vol * 25.0, 0.0, 0.6))       # higher vol -> lower score

    direction = 1.0 if side == "buy" else -1.0
    alignment = (direction * momentum + 1.0) / 2.0           # [0,1], 1 = fully aligned

    base = 0.5 * float(np.clip(strength, 0.0, 1.0)) + 0.5 * alignment
    return float(np.clip(base - vol_penalty, 0.0, 1.0))


def detect_regime(closes: list[float]) -> dict:
    """Classify market regime as trending / ranging / volatile."""
    rets = _returns(closes)
    if rets.size < 10:
        return {"regime": "unknown", "confidence": 0.0}

    arr = np.asarray(closes, dtype=float)
    # Trend strength: |net move| relative to path length (efficiency ratio).
    net = abs(arr[-1] - arr[-min(len(arr), 50)])
    path = float(np.sum(np.abs(np.diff(arr[-min(len(arr), 50):])))) + 1e-9
    efficiency = net / path                                  # ~1 trend, ~0 chop
    vol = float(np.std(rets))
    vol_z = float(np.clip(vol / (np.mean(np.abs(rets)) + 1e-9), 0.0, 3.0))

    if vol_z > 2.0:
        regime, conf = "volatile", min(1.0, vol_z / 3.0)
    elif efficiency > 0.4:
        regime, conf = "trending", min(1.0, efficiency)
    else:
        regime, conf = "ranging", min(1.0, 1.0 - efficiency)
    return {"regime": regime, "confidence": round(float(conf), 3)}


def forecast_volatility(closes: list[float], halflife: int = 20) -> dict:
    """EWMA volatility estimate (per-bar), plus a simple 1-step forecast."""
    rets = _returns(closes)
    if rets.size == 0:
        return {"volatility": 0.0, "forecast": 0.0}
    weights = 0.5 ** (np.arange(rets.size)[::-1] / max(halflife, 1))
    weights /= weights.sum()
    ewma_var = float(np.sum(weights * (rets**2)))
    vol = float(np.sqrt(ewma_var))
    # naive persistence forecast nudged by the latest shock
    forecast = float(np.sqrt(0.94 * ewma_var + 0.06 * rets[-1] ** 2))
    return {"volatility": round(vol, 6), "forecast": round(forecast, 6)}
