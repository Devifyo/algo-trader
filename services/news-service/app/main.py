"""News Intelligence orchestrator. Runs the collector, N analysis workers, and
the economic-calendar loop concurrently. Exposes Prometheus metrics on :8000.
Designed to never block trade execution — it only produces advisory sentiment."""

from __future__ import annotations

import asyncio
import logging
import os

import nats
import redis.asyncio as aioredis
from prometheus_client import Counter, start_http_server

from . import analyzer, collector, economic, event_risk
from .store import Store

logging.basicConfig(level=logging.INFO)
log = logging.getLogger("news")

WORKERS = int(os.environ.get("NEWS_WORKERS", "3"))
METRICS_PORT = int(os.environ.get("METRICS_PORT", "8000"))

METRICS = {
    "collected": Counter("news_collected_total", "Articles collected", ["source"]),
    "analyzed": Counter("news_analyzed_total", "Articles analyzed"),
    "events": Counter("news_events_total", "Critical events detected", ["type"]),
    "errors": Counter("news_collect_errors_total", "Collection errors", ["source"]),
    "analysis_errors": Counter("news_analysis_errors_total", "Analysis errors"),
    "macro_alerts": Counter("news_macro_alerts_total", "Macro event-window alerts"),
}


def _pg_dsn() -> str:
    url = os.environ.get("DATABASE_URL", "postgres://algotrader:algotrader@postgres:5432/algotrader")
    return url.replace("postgres://", "postgresql://", 1).split("?")[0]


async def main() -> None:
    start_http_server(METRICS_PORT)
    store = Store(_pg_dsn(), os.environ.get("REDIS_URL", "redis://:algotrader@redis:6379/0"))
    await store.connect()

    redis = aioredis.from_url(os.environ.get("REDIS_URL", "redis://:algotrader@redis:6379/0"),
                             decode_responses=True)
    try:
        nc = await nats.connect(os.environ.get("NATS_URL", "nats://nats:4222"))
    except Exception as e:
        log.warning("NATS unavailable, sentiment publishing disabled: %s", e)
        nc = None

    log.info("news intelligence starting: %d workers, mode=%s", WORKERS, os.environ.get("NEWS_SOURCE", "synthetic"))
    tasks = [
        asyncio.create_task(collector.run(store, METRICS)),
        asyncio.create_task(economic.run(store, redis, METRICS)),
        asyncio.create_task(event_risk.run(store, redis, METRICS)),
    ]
    tasks += [asyncio.create_task(analyzer.worker(store, nc, METRICS)) for _ in range(WORKERS)]
    await asyncio.gather(*tasks)


if __name__ == "__main__":
    asyncio.run(main())
