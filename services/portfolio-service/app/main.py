"""Intelligence / Portfolio / Safety service — advisory analytics HTTP API.

Every endpoint is ADVISORY: it returns scores, recommendations, and a full
explanation. It never places or blocks trades; the Rust risk engine validates
and is the final authority. The dashboard and the trading engine consume these."""

from __future__ import annotations

from fastapi import FastAPI
from fastapi.responses import PlainTextResponse
from prometheus_client import CONTENT_TYPE_LATEST, Counter, generate_latest
from pydantic import BaseModel, Field

from . import allocation, decision, health, optimizer, portfolio, recovery, regime, safety
from .risk_manager import RiskInputs, compute as risk_compute

app = FastAPI(title="algo-trader intelligence/portfolio service", version="0.1.0")
REQ = Counter("portfolio_requests_total", "Requests", ["endpoint"])


@app.get("/health")
def healthz() -> dict:
    return {"status": "ok"}


@app.get("/metrics")
def metrics() -> PlainTextResponse:
    return PlainTextResponse(generate_latest(), media_type=CONTENT_TYPE_LATEST)


# ── AI Risk Manager ──────────────────────────────────────────────────────
class RiskSizeRequest(BaseModel):
    equity_usd: float
    symbol: str
    side: str
    entry: float
    stop: float | None = None
    atr: float | None = None
    profile: str = "balanced"
    profile_overrides: dict | None = None
    volatility_pct: float = 1.0
    market_regime: str = "unknown"
    news_score: float = 0.0
    news_confidence: float = 0.0
    portfolio_exposure_pct: float = 0.0
    win_rate: float = 0.5
    sharpe: float = 1.0
    funding_rate: float = 0.0
    open_interest_z: float = 0.0
    conditions: list[str] = Field(default_factory=list)


@app.post("/risk/size")
def risk_size(req: RiskSizeRequest) -> dict:
    REQ.labels("risk_size").inc()
    return risk_compute(RiskInputs(**req.model_dump()))


# ── Smart Capital Allocation ─────────────────────────────────────────────
class AllocationRequest(BaseModel):
    total_capital: float
    candidates: list[dict]
    rebalance: str = "weekly"


@app.post("/allocation")
def allocate(req: AllocationRequest) -> dict:
    REQ.labels("allocation").inc()
    return allocation.allocate(req.total_capital, req.candidates, req.rebalance)


# ── AI Trade Recovery ────────────────────────────────────────────────────
class RecoveryRequest(BaseModel):
    position: dict
    context: dict


@app.post("/recovery")
def recovery_recommend(req: RecoveryRequest) -> dict:
    REQ.labels("recovery").inc()
    return recovery.recommend(req.position, req.context)


# ── Strategy Health + Validation ─────────────────────────────────────────
@app.post("/health/evaluate")
def health_eval(s: dict) -> dict:
    REQ.labels("health_evaluate").inc()
    return health.evaluate(s)


@app.post("/health/validate")
def health_validate(s: dict) -> dict:
    REQ.labels("health_validate").inc()
    return health.validate_for_live(s)


# ── Multi-Bot Portfolio ──────────────────────────────────────────────────
class PortfolioRequest(BaseModel):
    bots: list[dict]
    equity_usd: float


@app.post("/portfolio/analyze")
def portfolio_analyze(req: PortfolioRequest) -> dict:
    REQ.labels("portfolio_analyze").inc()
    return portfolio.analyze(req.bots, req.equity_usd)


# ── Strategy Optimizer ───────────────────────────────────────────────────
class OptimizeRequest(BaseModel):
    space: dict[str, list[float]]
    method: str = "grid"
    max_evals: int = 200


@app.post("/optimize")
def optimize(req: OptimizeRequest) -> dict:
    REQ.labels("optimize").inc()
    return optimizer.search(req.space, method=req.method, max_evals=req.max_evals)


# ── Market Regime ────────────────────────────────────────────────────────
class RegimeRequest(BaseModel):
    closes: list[float]


@app.post("/regime")
def regime_classify(req: RegimeRequest) -> dict:
    REQ.labels("regime").inc()
    return regime.classify(req.closes)


# ── Unified AI Decision Layer ────────────────────────────────────────────
@app.post("/decision")
def decision_synthesize(inp: dict) -> dict:
    REQ.labels("decision").inc()
    return decision.synthesize(inp)


# ── Exchange API Safety ──────────────────────────────────────────────────
class SafetyRequest(BaseModel):
    permissions: dict
    ip_whitelisted: bool = False
    reachable: bool = True
    valid_key: bool = True


@app.post("/safety/exchange-key")
def safety_check(req: SafetyRequest) -> dict:
    REQ.labels("safety").inc()
    return safety.evaluate(req.permissions, ip_whitelisted=req.ip_whitelisted,
                           reachable=req.reachable, valid_key=req.valid_key)
