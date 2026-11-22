"""Durable + hot storage for the news pipeline: Postgres for history, Redis for
the collection queue and the live per-asset sentiment snapshot the risk engine
reads. Deduplication is enforced by a unique hash of the article URL."""

from __future__ import annotations

import hashlib
import json

import asyncpg
import redis.asyncio as aioredis

QUEUE_KEY = "news:queue"


def url_hash(url: str) -> str:
    return hashlib.sha256(url.encode()).hexdigest()


class Store:
    def __init__(self, pg_dsn: str, redis_url: str):
        self._dsn = pg_dsn
        self._redis_url = redis_url
        self._pg: asyncpg.Pool | None = None
        self._redis: aioredis.Redis | None = None

    async def connect(self) -> None:
        self._pg = await asyncpg.create_pool(self._dsn, min_size=1, max_size=8)
        self._redis = aioredis.from_url(self._redis_url, decode_responses=True)

    # ── collection ───────────────────────────────────────────────────────
    async def insert_article(self, *, source, url, title, summary, published_at,
                             categories, assets) -> str | None:
        """Insert if new; returns article id, or None if a duplicate."""
        assert self._pg
        h = url_hash(url)
        async with self._pg.acquire() as con:
            row = await con.fetchrow(
                "INSERT INTO news_articles (source,url,url_hash,title,summary,published_at,categories,assets) "
                "VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (url_hash) DO NOTHING RETURNING id",
                source, url, h, title, summary, published_at, categories, assets)
        return str(row["id"]) if row else None

    async def enqueue(self, article_id: str) -> None:
        assert self._redis
        await self._redis.lpush(QUEUE_KEY, article_id)

    async def dequeue(self, timeout: int = 5) -> str | None:
        assert self._redis
        res = await self._redis.brpop(QUEUE_KEY, timeout=timeout)
        return res[1] if res else None

    async def article(self, article_id: str) -> dict | None:
        assert self._pg
        async with self._pg.acquire() as con:
            row = await con.fetchrow(
                "SELECT id,source,title,summary,categories,assets FROM news_articles WHERE id=$1",
                article_id)
        return dict(row) if row else None

    # ── analysis output ──────────────────────────────────────────────────
    async def record_analysis(self, article_id: str, consensus: dict, schema: dict) -> None:
        assert self._pg
        async with self._pg.acquire() as con:
            await con.execute(
                "INSERT INTO news_ai_analysis (article_id,bias,score,confidence,label,providers) "
                "VALUES ($1,$2,$3,$4,$5,$6)",
                article_id, consensus["bias"], consensus["score"], consensus["confidence"],
                consensus.get("label"), json.dumps(consensus.get("providers", [])))
            await con.execute(
                "INSERT INTO news_sentiments (article_id,sentiment_score,confidence,market_impact,"
                "duration,risk_level,assets) VALUES ($1,$2,$3,$4,$5,$6,$7)",
                article_id, schema["sentiment"], schema["confidence"], schema["market_impact"],
                schema["duration"], schema["risk"], schema["assets"])

    async def record_event(self, article_id, event_type, severity, asset, headline) -> None:
        assert self._pg
        async with self._pg.acquire() as con:
            await con.execute(
                "INSERT INTO market_events (article_id,event_type,severity,asset,headline) "
                "VALUES ($1,$2,$3,$4,$5)", article_id, event_type, severity, asset, headline)

    async def update_sentiment(self, asset: str, score: float, confidence: float, news_count: int) -> None:
        assert self._pg and self._redis
        async with self._pg.acquire() as con:
            await con.execute(
                "INSERT INTO sentiment_history (asset,sentiment_score,confidence,news_count) "
                "VALUES ($1,$2,$3,$4)", asset, score, confidence, news_count)
        # Hot snapshot for the risk engine (TTL guards against stale sentiment).
        await self._redis.set(
            f"sentiment:{asset}",
            json.dumps({"score": score, "confidence": confidence}),
            ex=3600)

    async def insert_economic_event(self, *, name, country, importance, scheduled_at,
                                    forecast=None, previous=None) -> None:
        assert self._pg
        async with self._pg.acquire() as con:
            await con.execute(
                "INSERT INTO economic_events (name,country,importance,scheduled_at,forecast,previous) "
                "VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (name,scheduled_at) DO NOTHING",
                name, country, importance, scheduled_at, forecast, previous)

    # ── event-risk (defensive layer) ─────────────────────────────────────
    async def recent_events(self, hours: int = 6) -> list[dict]:
        """Critical events detected by the analyzer within the last `hours`."""
        assert self._pg
        async with self._pg.acquire() as con:
            rows = await con.fetch(
                "SELECT event_type, severity, asset, headline, detected_at AS created_at "
                "FROM market_events WHERE detected_at > now() - make_interval(hours => $1::int) "
                "ORDER BY detected_at DESC", hours)
        return [dict(r) for r in rows]

    async def upcoming_economic(self, minutes: int = 60) -> list[dict]:
        """High-impact economic-calendar events scheduled within `minutes`."""
        assert self._pg
        async with self._pg.acquire() as con:
            rows = await con.fetch(
                "SELECT name, country, importance, scheduled_at FROM economic_events "
                "WHERE scheduled_at BETWEEN now() AND now() + make_interval(mins => $1::int) "
                "ORDER BY scheduled_at ASC", minutes)
        return [dict(r) for r in rows]

    async def set_event_risk(self, payload: dict, ttl: int = 900) -> None:
        """Publish the defensive event-risk snapshot the risk engine consults."""
        assert self._redis
        await self._redis.set("news:event_risk", json.dumps(payload), ex=ttl)
        for asset in payload.get("assets", []):
            await self._redis.set(
                f"event_risk:{asset}",
                json.dumps({"level": payload["level"], "score": payload["score"], "asof": payload["asof"]}),
                ex=ttl)

    async def recent_asset_sentiment(self, asset: str, window: int = 50) -> tuple[float, float, int]:
        """Rolling mean sentiment/confidence over the last `window` analyses."""
        assert self._pg
        async with self._pg.acquire() as con:
            row = await con.fetchrow(
                "SELECT avg(sentiment_score) s, avg(confidence) c, count(*) n FROM ("
                "  SELECT sentiment_score,confidence FROM news_sentiments "
                "  WHERE $1 = ANY(assets) ORDER BY created_at DESC LIMIT $2) q",
                asset, window)
        if not row or row["n"] == 0:
            return 0.0, 0.0, 0
        return float(row["s"] or 0), float(row["c"] or 0), int(row["n"])
