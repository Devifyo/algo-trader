"""Multi-Bot Portfolio Manager — treats many bots as one book. Computes
exposure, concentration (HHI), correlation risk, leverage risk, and profit/risk
contributions; flags conflicting positions, overexposure, and bot competition."""

from __future__ import annotations

import numpy as np

OVEREXPOSURE_PCT = 80.0


def analyze(bots: list[dict], equity_usd: float) -> dict:
    if not bots or equity_usd <= 0:
        return {"bots": 0, "warnings": ["no_bots_or_equity"]}

    exposures = np.array([float(b.get("exposure_usd", 0)) for b in bots], dtype=float)
    total_exposure = float(exposures.sum())
    exposure_pct = total_exposure / equity_usd * 100.0
    shares = exposures / total_exposure if total_exposure > 0 else np.zeros_like(exposures)

    hhi = float((shares ** 2).sum())                 # 1 = one bot owns everything
    pnls = np.array([float(b.get("pnl", 0)) for b in bots], dtype=float)
    total_pnl = float(pnls.sum())
    lev = np.array([float(b.get("leverage", 1)) for b in bots], dtype=float)
    leverage_risk = float((shares * lev).sum())

    correlation_risk = _correlation_risk(bots)
    conflicts, competition = _position_conflicts(bots)

    warnings: list[str] = []
    if exposure_pct > OVEREXPOSURE_PCT:
        warnings.append("overexposure")
    if hhi > 0.4:
        warnings.append("concentration_risk")
    if correlation_risk > 0.6:
        warnings.append("correlation_risk")
    if conflicts:
        warnings.append("conflicting_positions")
    if competition:
        warnings.append("bot_competition")
    if leverage_risk > 5:
        warnings.append("leverage_risk")

    return {
        "bots": len(bots),
        "total_exposure_usd": round(total_exposure, 2),
        "exposure_pct": round(exposure_pct, 2),
        "concentration_hhi": round(hhi, 3),
        "correlation_risk": round(correlation_risk, 3),
        "leverage_risk": round(leverage_risk, 2),
        "total_pnl_usd": round(total_pnl, 2),
        "allocations": [
            {"id": b.get("id"), "type": b.get("type"),
             "weight": round(float(shares[i]), 4),
             "profit_contribution": round(float(pnls[i]), 2),
             "risk_contribution": round(float(shares[i] * lev[i]), 3)}
            for i, b in enumerate(bots)
        ],
        "conflicts": conflicts,
        "competition": competition,
        "warnings": warnings,
    }


def _correlation_risk(bots: list[dict]) -> float:
    series = [np.asarray(b.get("returns", []), dtype=float) for b in bots]
    series = [s for s in series if s.size >= 5]
    if len(series) < 2:
        return 0.0
    n = min(s.size for s in series)
    mat = np.vstack([s[-n:] for s in series])
    if np.allclose(mat.std(axis=1), 0):
        return 0.0
    corr = np.corrcoef(mat)
    iu = np.triu_indices_from(corr, k=1)
    return float(np.nanmean(np.abs(corr[iu]))) if iu[0].size else 0.0


def _position_conflicts(bots: list[dict]) -> tuple[list, list]:
    by_symbol: dict[str, list[tuple]] = {}
    for b in bots:
        for p in b.get("positions", []):
            by_symbol.setdefault(p["symbol"], []).append((b.get("id"), p.get("side")))
    conflicts, competition = [], []
    for sym, holders in by_symbol.items():
        sides = {s for _, s in holders}
        if len(sides) > 1:                              # long and short same symbol
            conflicts.append({"symbol": sym, "holders": holders})
        elif len(holders) >= 3:                         # many bots crowd one trade
            competition.append({"symbol": sym, "bots": [h[0] for h in holders]})
    return conflicts, competition
