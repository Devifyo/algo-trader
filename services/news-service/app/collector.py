"""News collection. Polls RSS feeds (real, no key needed) or generates a
synthetic stream so the pipeline runs fully offline (default in paper mode).
Dedupes, categorizes, and enqueues each new article for AI analysis."""

from __future__ import annotations

import asyncio
import os
from datetime import datetime, timezone

import feedparser
import httpx

from . import categorize
from .sources import SOURCES, Source
from .store import Store

MODE = os.environ.get("NEWS_SOURCE", "synthetic")     # synthetic | rss
POLL_SECONDS = int(os.environ.get("NEWS_POLL_SECONDS", "60"))


async def run(store: Store, metrics) -> None:
    if MODE == "rss":
        await _rss_loop(store, metrics)
    else:
        await _synthetic_loop(store, metrics)


async def _rss_loop(store: Store, metrics) -> None:
    # Free/open feeds: RSS + no-key JSON APIs (CryptoCompare, GDELT) + scraped
    # regulator press pages (Scrapling) for the defensive event-risk signal.
    feeds = [s for s in SOURCES if s.kind in ("rss", "json_api", "scrape")]
    async with httpx.AsyncClient(timeout=15.0, follow_redirects=True,
                                 headers={"User-Agent": "algo-trader-news/0.1"}) as http:
        while True:
            for src in feeds:
                try:
                    if src.kind == "rss":
                        await _poll_rss(store, http, src, metrics)
                    elif src.kind == "scrape":
                        await _poll_scrape(store, src, metrics)
                    else:
                        await _poll_json_api(store, http, src, metrics)
                except Exception as e:  # one bad feed must not stall the rest
                    metrics["errors"].labels(source=src.name).inc()
                    _log(f"feed {src.name} failed: {e}")
            await asyncio.sleep(POLL_SECONDS)


async def _poll_scrape(store: Store, src: Source, metrics) -> None:
    """Scrapling-based collection (sync fetch run off the event loop)."""
    from . import scrapling_source
    headlines = await asyncio.to_thread(scrapling_source.fetch_headlines, src.url)
    for title, url in headlines:
        await _ingest(store, src.name, url, title, "", datetime.now(timezone.utc), metrics)


async def _poll_json_api(store: Store, http: httpx.AsyncClient, src: Source, metrics) -> None:
    """Open JSON news APIs that need no key: CryptoCompare and GDELT."""
    r = await http.get(src.url)
    data = r.json()
    if src.name == "CryptoCompare":
        for a in (data.get("Data") or [])[:50]:
            url, title = a.get("url", ""), a.get("title", "")
            if not url or not title:
                continue
            ts = a.get("published_on")
            published = datetime.fromtimestamp(ts, timezone.utc) if ts else datetime.now(timezone.utc)
            await _ingest(store, "CryptoCompare", url, title, (a.get("body") or "")[:2000], published, metrics)
    elif src.name == "GDELT":
        for a in (data.get("articles") or [])[:50]:
            url, title = a.get("url", ""), a.get("title", "")
            if not url or not title:
                continue
            published = _gdelt_date(a.get("seendate")) or datetime.now(timezone.utc)
            await _ingest(store, f"GDELT·{a.get('domain', 'web')}", url, title, "", published, metrics)


def _gdelt_date(s: str | None) -> datetime | None:
    if not s:
        return None
    try:
        return datetime.strptime(s, "%Y%m%dT%H%M%SZ").replace(tzinfo=timezone.utc)
    except (ValueError, TypeError):
        return None


async def _poll_rss(store: Store, http: httpx.AsyncClient, src: Source, metrics) -> None:
    r = await http.get(src.url)
    feed = feedparser.parse(r.content)
    for entry in feed.entries[:50]:
        title = getattr(entry, "title", "")
        summary = getattr(entry, "summary", "")[:2000]
        url = getattr(entry, "link", "")
        if not url or not title:
            continue
        published = _published(entry)
        await _ingest(store, src.name, url, title, summary, published, metrics)


async def _synthetic_loop(store: Store, metrics) -> None:
    """Deterministic rotating sample stream (no internet, no randomness needed)."""
    samples = [
        ("SEC Approves New Spot Bitcoin ETF", "Regulators approved a spot bitcoin ETF, opening institutional access."),
        ("Major Exchange Reports Security Breach", "An exchange disclosed an exploit; user funds were drained."),
        ("Federal Reserve Holds Interest Rates Steady", "The FOMC kept rates unchanged amid cooling inflation."),
        ("Ethereum Network Upgrade Goes Live", "The latest Ethereum upgrade reduced fees and improved throughput."),
        ("Solana Sees Record Institutional Inflows", "Institutions increased Solana exposure this quarter."),
        ("CPI Comes In Below Expectations", "Consumer prices rose less than forecast, easing rate-hike fears."),
        ("Stablecoin Briefly Depegs Amid Volatility", "A major stablecoin briefly lost its peg before recovering."),
    ]
    i = 0
    while True:
        title, summary = samples[i % len(samples)]
        # synthetic stable URL so dedupe lets each cycle through once per hour
        bucket = i // len(samples)
        url = f"synthetic://news/{i % len(samples)}/{bucket}"
        from datetime import datetime, timezone
        await _ingest(store, "synthetic", url, title, summary,
                      datetime.now(timezone.utc), metrics)
        i += 1
        await asyncio.sleep(POLL_SECONDS)


async def _ingest(store: Store, source, url, title, summary, published, metrics) -> None:
    text = f"{title}. {summary}"
    categories = categorize.classify(text)
    assets = categorize.affected_assets(text)
    article_id = await store.insert_article(
        source=source, url=url, title=title, summary=summary,
        published_at=published, categories=categories, assets=assets)
    if article_id:
        await store.enqueue(article_id)
        metrics["collected"].labels(source=source).inc()


def _published(entry):
    from datetime import datetime, timezone
    if getattr(entry, "published_parsed", None):
        import calendar
        return datetime.fromtimestamp(calendar.timegm(entry.published_parsed), tz=timezone.utc)
    return datetime.now(timezone.utc)


def _log(msg: str) -> None:
    import logging
    logging.getLogger("news.collector").warning(msg)
