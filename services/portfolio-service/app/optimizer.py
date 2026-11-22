"""AI Strategy Optimizer. Implements grid + random search over a parameter
space against a pluggable objective. Bayesian / genetic / ML methods are
declared seams (scikit-optimize / DEAP) — wire them in `search()` when the
heavier deps are added. Returns the optimized params + expected metrics."""

from __future__ import annotations

import itertools
import math
from collections.abc import Callable

Params = dict[str, float]
Objective = Callable[[Params], float]   # higher is better (e.g. Sharpe)


def search(space: dict[str, list[float]], objective: Objective | None = None,
           method: str = "grid", max_evals: int = 200) -> dict:
    obj = objective or _default_objective
    if method == "random":
        candidates = _random_candidates(space, max_evals)
    elif method in ("bayesian", "genetic", "ml"):
        # Seam: fall back to grid for now; plug scikit-optimize / DEAP here.
        candidates = _grid_candidates(space, max_evals)
        method = f"{method}->grid(fallback)"
    else:
        candidates = _grid_candidates(space, max_evals)

    best, best_score = None, -math.inf
    evaluated = 0
    for cand in candidates:
        score = obj(cand)
        evaluated += 1
        if score > best_score:
            best, best_score = cand, score

    return {
        "method": method,
        "evaluated": evaluated,
        "optimized_strategy": best or {},
        "expected_return": round(_default_return(best or {}), 4),
        "risk_reduction": round(max(0.0, best_score) * 0.1, 4),
        "confidence_score": round(min(100.0, max(0.0, best_score) * 25), 1),
    }


def _grid_candidates(space: dict[str, list[float]], cap: int):
    keys = list(space)
    combos = itertools.product(*(space[k] for k in keys))
    for i, vals in enumerate(combos):
        if i >= cap:
            break
        yield dict(zip(keys, vals))


def _random_candidates(space: dict[str, list[float]], n: int):
    # Deterministic pseudo-random sampling (LCG) — no Math.random dependency.
    keys = list(space)
    seed = 1
    for _ in range(n):
        cand = {}
        for k in keys:
            seed = (1103515245 * seed + 12345) & 0x7FFFFFFF
            choices = space[k]
            cand[k] = choices[seed % len(choices)]
        yield cand


def _default_objective(p: Params) -> float:
    """Concave default: rewards balanced EMA spread + mid RSI + sane R:R.
    Stands in for a real backtest until trade-history simulation is wired."""
    fast = p.get("fast_ema", 21)
    slow = p.get("slow_ema", 55)
    rsi = p.get("rsi", 14)
    rr = p.get("tp_rr", 2.0)
    spread = abs(slow - fast)
    spread_score = math.exp(-((spread - 34) ** 2) / (2 * 18 ** 2))   # peak near 34
    rsi_score = math.exp(-((rsi - 14) ** 2) / (2 * 6 ** 2))
    rr_score = math.exp(-((rr - 2.5) ** 2) / (2 * 1.0 ** 2))
    return 3.0 * spread_score + 1.5 * rsi_score + 1.5 * rr_score


def _default_return(p: Params) -> float:
    return 0.05 + 0.1 * _default_objective(p) / 6.0
