"""Analysis worker. Pops articles off the Redis queue, asks the AI engine for a
multi-provider consensus sentiment, derives the full AI output schema, detects
critical events, updates the rolling per-asset sentiment, and publishes the
advisory sentiment on NATS for the risk engine to read. Never places a trade."""

from __future__ import annotations

import json
import os

import httpx

from . import categorize
from .store import Store

AI_URL = os.environ.get("AI_ENGINE_URL", "http://ai-engine:8000")


async def worker(store: Store, nats, metrics) -> None:
    async with httpx.AsyncClient(timeout=30.0) as http:
        while True:
            article_id = await store.dequeue(timeout=5)
            if not article_id:
                continue
            try:
                await _process(store, http, nats, article_id, metrics)
            except Exception as e:
                metrics["analysis_errors"].inc()
                _log(f"analysis failed for {article_id}: {e}")


async def _process(store: Store, http, nats, article_id, metrics) -> None:
    article = await store.article(article_id)
    if not article:
        return
    text = f"{article['title']}. {article.get('summary') or ''}"
    assets = list(article.get("assets") or []) or ["BTC"]

    consensus = await _ask_ai(http, text, assets[0])
    schema = _derive_schema(consensus, assets)
    await store.record_analysis(article_id, consensus, schema)
    metrics["analyzed"].inc()

    # Critical-event detection.
    event = categorize.detect_event(text)
    if event:
        severity = "critical" if schema["market_impact"] == "high" else "warning"
        await store.record_event(article_id, event, severity, assets[0], article["title"])
        metrics["events"].labels(type=event).inc()

    # Roll up per-asset sentiment and publish the advisory snapshot.
    for asset in assets:
        score, conf, n = await store.recent_asset_sentiment(asset)
        await store.update_sentiment(asset, score, conf, n)
        if nats is not None:
            payload = json.dumps({"asset": asset, "score": score, "confidence": conf,
                                  "count": n, "label": consensus.get("label")})
            await nats.publish(f"sentiment.{asset}", payload.encode())


# Lexicon for the keyword fallback when no AI provider keys are configured.
_BULL = ("surge", "surges", "soar", "soars", "rally", "rallies", "jump", "jumps", "gain",
         "gains", "record", "all-time high", "ath", "approve", "approval", "adopt",
         "adoption", "partnership", "upgrade", "bullish", "breakout", "inflow", "inflows",
         "accumulate", "institutional", "etf approval", "rebound", "recover", "boost")
_BEAR = ("crash", "crashes", "plunge", "plunges", "plummet", "tumble", "slump", "sink",
         "drop", "drops", "fall", "falls", "hack", "hacked", "exploit", "ban", "banned",
         "lawsuit", "sue", "sued", "depeg", "selloff", "sell-off", "dump", "bearish",
         "outflow", "outflows", "liquidation", "liquidations", "fraud", "warning", "halt",
         "delist", "bankrupt", "fear", "downgrade", "scam")


def _heuristic_sentiment(text: str) -> dict:
    """Deterministic keyword sentiment — used when no AI keys are configured so
    the news pipeline still produces real (non-zero) per-asset sentiment. Upgrades
    automatically to multi-provider AI consensus once keys are added."""
    t = text.lower()
    bull = sum(t.count(w) for w in _BULL)
    bear = sum(t.count(w) for w in _BEAR)
    total = bull + bear
    if total == 0:
        return {"bias": "neutral", "score": 0.0, "confidence": 0.0, "label": "heuristic", "providers": []}
    score = max(-100.0, min(100.0, (bull - bear) / total * 100.0))
    confidence = min(70.0, 25.0 + total * 12.0)  # heuristic is never high-confidence
    bias = "bullish" if score > 15 else "bearish" if score < -15 else "neutral"
    return {"bias": bias, "score": round(score, 1), "confidence": round(confidence, 1),
            "label": "heuristic", "providers": []}


async def _ask_ai(http, text: str, symbol: str) -> dict:
    """Multi-provider consensus sentiment from the AI router when keys exist;
    otherwise a deterministic keyword heuristic. Fail-open so a model outage or
    missing keys never stalls the news pipeline or zeroes out sentiment."""
    try:
        r = await http.post(f"{AI_URL}/ai/sentiment",
                            json={"text": text, "symbol": symbol, "consensus": True})
        if r.status_code == 200:
            j = r.json()
            # Real AI consensus only if at least one provider actually voted.
            if j.get("providers") and float(j.get("confidence", 0) or 0) > 0:
                return j
    except httpx.HTTPError:
        pass
    return _heuristic_sentiment(text)


def _derive_schema(consensus: dict, assets: list[str]) -> dict:
    score = float(consensus.get("score", 0.0))
    conf = float(consensus.get("confidence", 0.0))
    impact = "high" if abs(score) >= 70 else "medium" if abs(score) >= 40 else "low"
    duration = {"high": "2-5 days", "medium": "1-2 days", "low": "hours"}[impact]
    if (score < -50) or conf < 40:
        risk = "high"
    elif abs(score) >= 40:
        risk = "medium"
    else:
        risk = "low"
    return {
        "sentiment": score, "confidence": conf, "market_impact": impact,
        "duration": duration, "risk": risk, "assets": assets,
    }


def _log(msg: str) -> None:
    import logging
    logging.getLogger("news.analyzer").warning(msg)
