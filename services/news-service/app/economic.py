"""Economic calendar. In `rss`/live mode this would call Trading Economics /
Investing; offline it seeds a recurring synthetic calendar (CPI, FOMC, NFP…).
When a high-importance event is near, it raises a Redis 'macro event window'
flag the risk engine reads to lower confidence during uncertainty."""

from __future__ import annotations

import asyncio
import os
from datetime import datetime, timedelta, timezone

EVENT_WINDOW_MIN = int(os.environ.get("MACRO_EVENT_WINDOW_MIN", "60"))

# Synthetic high-impact recurring events (offset hours from now on each pass).
_SYNTHETIC = [
    ("CPI (YoY)", "US", "high", 6),
    ("FOMC Rate Decision", "US", "high", 30),
    ("Non-Farm Payrolls", "US", "high", 54),
    ("GDP (QoQ)", "US", "medium", 78),
    ("PPI (MoM)", "US", "medium", 12),
    ("Unemployment Rate", "US", "medium", 54),
]


async def run(store, redis, metrics) -> None:
    while True:
        now = datetime.now(timezone.utc)
        for name, country, importance, offset_h in _SYNTHETIC:
            when = now + timedelta(hours=offset_h)
            await store.insert_economic_event(
                name=name, country=country, importance=importance, scheduled_at=when)
            # Raise the risk-awareness flag if a high-impact event is imminent.
            if importance == "high" and timedelta(0) <= (when - now) <= timedelta(minutes=EVENT_WINDOW_MIN):
                await redis.set("macro:event_soon", name, ex=EVENT_WINDOW_MIN * 60)
                metrics["macro_alerts"].inc()
        await asyncio.sleep(900)  # refresh calendar every 15 min
