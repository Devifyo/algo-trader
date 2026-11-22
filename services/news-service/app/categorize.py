"""Keyword classification: categories, affected assets, and critical-event
detection. Cheap and deterministic — runs before any AI call so we only pay for
LLM analysis on relevant articles."""

from __future__ import annotations

from .sources import ASSET_KEYWORDS, CATEGORY_KEYWORDS, EVENT_KEYWORDS


def classify(text: str) -> list[str]:
    t = text.lower()
    return [cat for cat, kws in CATEGORY_KEYWORDS.items() if any(k in t for k in kws)]


def affected_assets(text: str) -> list[str]:
    t = text.lower()
    return [asset for asset, kws in ASSET_KEYWORDS.items() if any(k in t for k in kws)]


def detect_event(text: str) -> str | None:
    t = text.lower()
    for event, kws in EVENT_KEYWORDS.items():
        if any(k in t for k in kws):
            return event
    return None
