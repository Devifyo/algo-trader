"""Event-risk aggregator — the defensive decision layer.

Reads the critical events the analyzer already detects (market_events) plus the
economic calendar, and condenses them into a single DEFENSIVE signal:

    Redis  news:event_risk        -> {level, score, reasons, assets, asof}
    Redis  event_risk:<ASSET>     -> {level, score, asof}

`level` is one of: calm | elevated | high.

The risk engine consults this to VETO or SHRINK new entries during high-impact
windows (FOMC/CPI prints, regulatory actions, exchange hacks, depegs/black-swans).
It is FAIL-SAFE by construction: its only effect is to *withhold* entries — it can
never open or force a position. If this loop dies or Redis expires the key, the
engine simply sees no event-risk and trades normally (fail-open).
"""
from __future__ import annotations

import asyncio
import json
import logging
import os
from datetime import datetime, timezone

log = logging.getLogger("news.event_risk")

POLL_SECONDS = int(os.environ.get("EVENT_RISK_POLL_SECONDS", "60"))
TTL_SECONDS = int(os.environ.get("EVENT_RISK_TTL_SECONDS", "900"))

# Severe events that can move markets violently — count even at "warning".
HIGH_TYPES = {"black_swan", "exchange_hack", "exchange_bankruptcy", "etf_rejection"}
# Macro events — count ONLY when the analyzer rated them critical (high impact),
# so routine FOMC/CPI mentions don't constantly raise risk.
MACRO_TYPES = {"rate_decision", "cpi_data", "fed_statement", "etf_approval"}
# Regulatory press is overwhelmingly routine (registrations, rule tweaks). It only
# counts when critical — otherwise the regulator's daily press wire pins risk high.


def _recency_factor(created_at: datetime, now: datetime) -> float:
    age_h = (now - created_at).total_seconds() / 3600.0
    if age_h <= 2:
        return 1.0
    if age_h <= 6:
        return 0.5
    return 0.0


def _assess(events: list[dict], upcoming: list[dict], now: datetime) -> dict:
    score = 0.0
    reasons: list[str] = []
    assets: set[str] = set()
    severe_now = False   # a genuinely severe event in the last 2h

    for e in events:
        created = e.get("created_at")
        if created is None:
            continue
        if created.tzinfo is None:
            created = created.replace(tzinfo=timezone.utc)
        rf = _recency_factor(created, now)
        if rf == 0.0:
            continue
        etype = e.get("event_type", "")
        crit = e.get("severity") == "critical"

        # Weighting — routine (non-critical) macro/regulatory items score ZERO so
        # the regulator press wire can't pin risk high on a quiet day.
        if etype in HIGH_TYPES:
            # Critical only drives "high" — keyword hits on rhetorical "collapse"
            # land as low-impact warnings and barely move the score.
            w = (2.0 if crit else 0.5) * rf
            if crit and rf == 1.0:
                severe_now = True
        elif etype in MACRO_TYPES and crit:
            w = 1.0 * rf
        elif etype == "regulation" and crit:
            w = 0.6 * rf
        else:
            w = 0.0
        if w <= 0.0:
            continue

        score += w
        if e.get("asset"):
            assets.add(e["asset"])
        if w >= 1.0 and len(reasons) < 6:
            reasons.append(f"{etype}: {(e.get('headline') or '')[:90]}")

    # Imminent high-importance macro prints → elevate even before the headline.
    imminent_30 = imminent_60 = False
    for ev in upcoming:
        mins = (ev["scheduled_at"] - now).total_seconds() / 60.0
        imp = str(ev.get("importance", "")).lower()
        high_imp = imp in {"high", "3", "2"} or imp.startswith("hig")
        if not high_imp:
            continue
        if 0 <= mins <= 30:
            imminent_30 = True
            if len(reasons) < 6:
                reasons.append(f"imminent macro: {ev.get('name')} in {int(mins)}m")
        elif 0 <= mins <= 60:
            imminent_60 = True

    # "high" requires a genuinely severe, critical event (or a pile of them).
    # Imminent macro only elevates — the calendar may be synthetic until a real
    # economic-data provider key is configured, so it must not force a veto.
    if severe_now or score >= 4.0:
        level = "high"
    elif score >= 1.5 or imminent_30 or imminent_60:
        level = "elevated"
    else:
        level = "calm"

    return {
        "level": level,
        "score": round(score, 2),
        "reasons": reasons,
        "assets": sorted(assets),
        "asof": now.isoformat(),
    }


async def run(store, redis, metrics) -> None:
    log.info("event-risk aggregator starting (poll=%ds, ttl=%ds)", POLL_SECONDS, TTL_SECONDS)
    while True:
        try:
            now = datetime.now(timezone.utc)
            events = await store.recent_events(hours=6)
            upcoming = await store.upcoming_economic(minutes=60)
            payload = _assess(events, upcoming, now)
            await store.set_event_risk(payload, ttl=TTL_SECONDS)
            if payload["level"] != "calm":
                metrics["macro_alerts"].inc()
                log.info("event-risk=%s score=%.2f assets=%s",
                         payload["level"], payload["score"], payload["assets"])
        except Exception as e:  # never let the defensive loop crash the service
            log.warning("event-risk assessment failed: %s", e)
        await asyncio.sleep(POLL_SECONDS)
