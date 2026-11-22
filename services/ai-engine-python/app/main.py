"""AI engine HTTP surface.

Two classes of endpoint:
  * Hot-path heuristics (`/score`, `/regime`, `/volatility`) — pure-math, sub-ms,
    called by the Rust signals service with a 50ms timeout. NEVER call an LLM here.
  * LLM-backed advisory (`/ai/*`) — routed through the multi-provider AIRouter
    with failover + consensus. Used by the news service and the dashboard.
"""

from __future__ import annotations

import os

from fastapi import FastAPI
from fastapi.responses import PlainTextResponse
from prometheus_client import CONTENT_TYPE_LATEST, Counter, Histogram, generate_latest
from pydantic import BaseModel

from . import models as heuristics
from .config_store import ConfigStore
from .router import AIRouter
from .tenancy import DEFAULT_TENANT

app = FastAPI(title="algo-trader AI engine", version="0.1.0")

REQ = Counter("ai_engine_requests_total", "AI engine requests", ["endpoint"])
LAT = Histogram("ai_engine_latency_seconds", "AI engine latency", ["endpoint"])

_store: ConfigStore | None = None
_router: AIRouter | None = None


@app.on_event("startup")
async def _startup() -> None:
    global _store, _router
    dsn = _pg_dsn()
    _store = ConfigStore(dsn)
    try:
        await _store.connect()
        _router = AIRouter(_store)
        await _router.refresh()
    except Exception as e:  # DB not ready yet — endpoints degrade gracefully
        import logging
        logging.getLogger("ai-engine").warning("router init deferred: %s", e)


# ── Hot-path heuristics (called by Rust) ─────────────────────────────────
class ScoreRequest(BaseModel):
    strategy: str
    symbol: str
    side: str
    price: float
    strength: float
    closes: list[float] = []


@app.post("/score")
def score(req: ScoreRequest) -> dict:
    REQ.labels("score").inc()
    with LAT.labels("score").time():
        s = heuristics.score_signal(req.side, req.strength, req.closes)
    return {"score": s}


class SeriesRequest(BaseModel):
    symbol: str
    closes: list[float]


@app.post("/regime")
def regime(req: SeriesRequest) -> dict:
    REQ.labels("regime").inc()
    return heuristics.detect_regime(req.closes)


@app.post("/volatility")
def volatility(req: SeriesRequest) -> dict:
    REQ.labels("volatility").inc()
    return heuristics.forecast_volatility(req.closes)


# ── LLM-backed advisory (routed) ─────────────────────────────────────────
class SentimentRequest(BaseModel):
    text: str
    symbol: str | None = None
    tenant_id: str | None = None
    consensus: bool = True


SENTIMENT_SYSTEM = (
    "You are a financial markets sentiment analyst. Read the text and reply with "
    "ONLY a JSON object: {\"bias\":\"bullish|bearish|neutral\",\"score\":-100..100,"
    "\"confidence\":0..100,\"market_impact\":\"low|medium|high\",\"duration\":\"hours|days|weeks\","
    "\"assets\":[\"BTC\"],\"risk\":\"low|medium|high\"}. No prose."
)


@app.post("/ai/sentiment")
async def ai_sentiment(req: SentimentRequest) -> dict:
    REQ.labels("ai_sentiment").inc()
    tenant = req.tenant_id or str(DEFAULT_TENANT)
    if _router is None:
        return {"bias": "neutral", "score": 0.0, "confidence": 0.0, "label": "unavailable"}
    with LAT.labels("ai_sentiment").time():
        if req.consensus:
            return await _router.consensus(req.text, kind="sentiment", tenant_id=tenant,
                                           system=SENTIMENT_SYSTEM, symbol=req.symbol)
        return await _router.single(req.text, kind="sentiment", tenant_id=tenant,
                                    system=SENTIMENT_SYSTEM)


class StrategyResearchRequest(BaseModel):
    context: str | None = None      # live performance stats / notes to ground the pick
    tenant_id: str | None = None


STRATEGY_RESEARCH_SYSTEM = (
    "You are a quantitative strategy advisor for a crypto trading platform. From the "
    "EXACT list [trend_following, momentum, mean_reversion, breakout, vol_expansion], "
    "recommend ONE strategy to set as the platform default for high-liquidity crypto "
    "(BTC/ETH/SOL/BNB). Weigh robustness, risk-adjusted returns and overfitting "
    "resistance over raw win rate; institutional evidence favors trend-following / "
    "time-series momentum. Consider the provided live performance stats if any. "
    "Respond ONLY as compact JSON: {\"recommended\":\"<one item from the list>\","
    "\"summary\":\"<=300 char reasoning\"}. No markdown, no extra text."
)


@app.post("/ai/strategy-research")
async def strategy_research(req: StrategyResearchRequest) -> dict:
    REQ.labels("strategy_research").inc()
    if _router is None:
        return {"available": False, "error": "router_uninitialized"}
    tenant = req.tenant_id or str(DEFAULT_TENANT)
    prompt = req.context or "No live performance stats yet — recommend the most robust default for crypto."
    with LAT.labels("strategy_research").time():
        return await _router.complete_text(prompt, kind="strategy_research", tenant_id=tenant,
                                           system=STRATEGY_RESEARCH_SYSTEM, max_tokens=400)


class BacktestRequest(BaseModel):
    strategy: str = "trend_following"
    symbol: str = "BTCUSDT"
    timeframe: str = "1h"


@app.post("/ai/backtest")
async def run_backtest(req: BacktestRequest) -> dict:
    REQ.labels("backtest").inc()
    from . import backtest as bt
    with LAT.labels("backtest").time():
        return await bt.backtest(req.strategy, req.symbol.upper(), req.timeframe)


@app.get("/ai/opportunities")
async def opportunities(limit: int = 12) -> dict:
    REQ.labels("opportunities").inc()
    from . import opportunities as opp
    with LAT.labels("opportunities").time():
        return await opp.scan(limit=max(1, min(limit, 30)))


@app.get("/ai/providers/status")
async def providers_status() -> dict:
    if _router is None:
        return {"providers": [], "mode": "uninitialized"}
    await _router.refresh()
    return _router.status()


@app.get("/health")
def health() -> dict:
    return {"status": "ok", "router": _router is not None}


@app.get("/metrics")
def metrics() -> PlainTextResponse:
    return PlainTextResponse(generate_latest(), media_type=CONTENT_TYPE_LATEST)


def _pg_dsn() -> str:
    url = os.environ.get("DATABASE_URL", "postgres://algotrader:algotrader@postgres:5432/algotrader")
    # asyncpg wants postgresql:// and no extra query params
    return url.replace("postgres://", "postgresql://", 1).split("?")[0]
