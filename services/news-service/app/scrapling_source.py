"""Scrapling-based collection for high-impact sources that lack a clean RSS feed
or sit behind anti-bot protection (e.g. regulator press pages).

Design rules:
  * HTTP fetcher only (no browser engine) — keeps the image light.
  * FAIL-OPEN: any error returns [] (or falls back to httpx) so the news pipeline
    never stalls because one page changed or blocked us.
  * Relevance-filtered: only headlines that classify to a known category/asset are
    kept, so generic nav/footer links don't pollute the stream.
  * Be a good citizen: one request per poll, short timeout, identifies itself.

We deliberately target US-government regulator pages (SEC/CFTC/Fed/Treasury) for
the event-risk signal — they are high-impact AND public-domain (no copyright risk
for a commercial product), unlike scraping paywalled news articles.
"""
from __future__ import annotations

import logging
import re
from urllib.parse import urljoin

from . import categorize

log = logging.getLogger("news.scrapling")


# Only crypto/markets/regulatory/macro categories matter for the risk signal;
# generic matches (e.g. "Institutional Adoption" off the word BlackRock) are noise.
_RISK_CATEGORIES = {
    "Bitcoin", "Ethereum", "Solana", "Altcoins", "ETF News", "SEC News",
    "Regulations", "Exchange News", "Security Breaches", "Stablecoins",
    "Federal Reserve", "Interest Rates", "Inflation", "Macro Economics",
}


def _relevant(title: str) -> bool:
    if categorize.affected_assets(title):
        return True
    return bool(set(categorize.classify(title)) & _RISK_CATEGORIES)


def _clean(items: list[tuple[str, str]], base: str, limit: int) -> list[tuple[str, str]]:
    seen, out = set(), []
    for title, href in items:
        title = re.sub(r"\s+", " ", (title or "")).strip()
        if len(title) < 28 or not href:
            continue
        url = urljoin(base, href)
        if url in seen or not _relevant(title):
            continue
        seen.add(url)
        out.append((title, url))
        if len(out) >= limit:
            break
    return out


def fetch_headlines(url: str, limit: int = 30) -> list[tuple[str, str]]:
    """Return [(title, absolute_url)] of relevant headlines. Never raises."""
    # Primary: Scrapling (adaptive parsing + stealthy headers, gets past anti-bot).
    try:
        from scrapling.fetchers import Fetcher

        page = Fetcher.get(url, timeout=20, stealthy_headers=True)
        anchors = page.css("a")
        items = []
        for a in anchors:
            try:
                title = a.text
                href = a.attrib.get("href", "")
            except Exception:
                continue
            items.append((str(title), str(href)))
        result = _clean(items, url, limit)
        if result:
            return result
        log.info("scrapling returned 0 relevant for %s; trying fallback", url)
    except Exception as e:
        log.warning("scrapling fetch failed %s: %s", url, e)

    # Fallback: plain HTTP + regex anchor extraction (no parser dependency).
    return _fallback(url, limit)


def _fallback(url: str, limit: int) -> list[tuple[str, str]]:
    try:
        import httpx

        # A descriptive contact UA satisfies SEC/Fed/Treasury fair-access policy
        # (they 403 anonymous generic agents). Looks like a polite bot, not a faker.
        r = httpx.get(url, timeout=15.0, follow_redirects=True, headers={
            "User-Agent": "tradepilot-news/1.0 (defensive risk monitor; admin@tradepilot.devifyo.cloud)",
            "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            "Accept-Language": "en-US,en;q=0.9",
        })
        anchors = re.findall(r'<a[^>]+href="([^"]+)"[^>]*>(.*?)</a>', r.text, re.I | re.S)
        items = [(re.sub(r"<[^>]+>", "", t), h) for h, t in anchors]
        return _clean(items, url, limit)
    except Exception as e:
        log.warning("fallback fetch failed %s: %s", url, e)
        return []
