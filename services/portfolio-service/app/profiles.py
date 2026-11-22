"""Risk-profile presets. The AI risk manager scales its outputs by these.
`custom` profiles override any field."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class RiskProfile:
    name: str
    risk_per_trade_pct: float    # % of equity risked per trade
    max_leverage: float
    max_exposure_pct: float      # % of equity deployable at once
    diversification: str         # high|medium|low
    stop_atr_mult: float
    tp_rr: float                 # take-profit reward:risk multiple


PRESETS: dict[str, RiskProfile] = {
    "conservative": RiskProfile("conservative", 0.25, 2.0, 30.0, "high", 2.5, 2.0),
    "balanced":     RiskProfile("balanced", 0.5, 5.0, 60.0, "medium", 2.0, 2.5),
    "aggressive":   RiskProfile("aggressive", 1.0, 10.0, 90.0, "low", 1.5, 3.0),
}


def resolve(name: str, overrides: dict | None = None) -> RiskProfile:
    base = PRESETS.get(name, PRESETS["balanced"])
    if not overrides:
        return base
    return RiskProfile(
        name="custom",
        risk_per_trade_pct=float(overrides.get("risk_per_trade_pct", base.risk_per_trade_pct)),
        max_leverage=float(overrides.get("max_leverage", base.max_leverage)),
        max_exposure_pct=float(overrides.get("max_exposure_pct", base.max_exposure_pct)),
        diversification=overrides.get("diversification", base.diversification),
        stop_atr_mult=float(overrides.get("stop_atr_mult", base.stop_atr_mult)),
        tp_rr=float(overrides.get("tp_rr", base.tp_rr)),
    )
