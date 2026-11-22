"""Opportunity Engine — momentum/trend/volume screen of the LIVE tradable universe.

Honest by design: it ranks liquid markets by observable momentum, trend strength
and volume surge, and returns each candidate WITH its reasons AND its risk flags.
It does NOT predict a % gain, does NOT promise a win rate, and does NOT place
orders. Research signal only. Crypto via the Binance public API (dynamic universe,
no key). Stocks (Alpaca) are a TODO — they need per-tenant keys + market hours.
"""
from __future__ import annotations

import asyncio
import datetime as dt
import httpx

BINANCE = "https://api.binance.com/api/v3"
MIN_QUOTE_VOL = 5_000_000.0  # liquidity floor (24h quote volume, USD)
DEEP_SCAN = 45               # how many top-momentum names to pull klines for


def _clamp(x: float, lo: float, hi: float) -> float:
    return max(lo, min(hi, x))


async def scan(limit: int = 12) -> dict:
    try:
        async with httpx.AsyncClient(timeout=20) as cli:
            tickers = (await cli.get(f"{BINANCE}/ticker/24hr")).json()
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "error": f"market data unavailable: {e}", "candidates": []}

    # Dynamic universe: liquid spot USDT pairs, excluding leveraged tokens & stablecoins.
    cand = []
    for t in tickers:
        s = t.get("symbol", "")
        if not s.endswith("USDT"):
            continue
        if any(x in s for x in ("UPUSDT", "DOWNUSDT", "BULLUSDT", "BEARUSDT")):
            continue
        base = s[:-4]
        if base in ("USDC", "FDUSD", "TUSD", "BUSD", "DAI", "EUR", "GBP"):
            continue
        try:
            qv = float(t.get("quoteVolume", 0))
            if qv < MIN_QUOTE_VOL:
                continue
            cand.append({"symbol": s, "base": base, "chg": float(t.get("priceChangePercent", 0)),
                         "qv": qv, "last": float(t.get("lastPrice", 0))})
        except (TypeError, ValueError):
            continue

    cand.sort(key=lambda x: x["chg"], reverse=True)
    top = cand[:DEEP_SCAN]

    async def deep(c: dict) -> dict | None:
        try:
            async with httpx.AsyncClient(timeout=15) as cli:
                k = (await cli.get(f"{BINANCE}/klines", params={"symbol": c["symbol"], "interval": "1d", "limit": 30})).json()
            closes = [float(x[4]) for x in k]
            vols = [float(x[5]) for x in k]
            highs = [float(x[2]) for x in k]
            lows = [float(x[3]) for x in k]
            if len(closes) < 25:
                return None
            ma20 = sum(closes[-20:]) / 20
            trend = (closes[-1] - ma20) / ma20 * 100 if ma20 else 0.0
            ret7 = (closes[-1] / closes[-8] - 1) * 100 if closes[-8] else 0.0
            avg_vol = sum(vols[-20:-1]) / 19 if sum(vols[-20:-1]) > 0 else 0.0
            vol_surge = vols[-1] / avg_vol if avg_vol > 0 else 1.0
            atrpct = (sum(highs[i] - lows[i] for i in range(-14, 0)) / 14) / closes[-1] * 100 if closes[-1] else 0.0
            return {**c, "trend": trend, "ret7": ret7, "vol_surge": vol_surge, "atrpct": atrpct}
        except Exception:  # noqa: BLE001
            return None

    deep_res = [d for d in await asyncio.gather(*[deep(c) for c in top]) if d]

    out = []
    for d in deep_res:
        # Composite 0–100: momentum (40) + above-MA trend (30) + volume surge (30).
        m = _clamp(d["chg"], -20, 20) / 20 * 40
        tr = _clamp(d["trend"], -15, 15) / 15 * 30
        vs = _clamp(d["vol_surge"] - 1, 0, 3) / 3 * 30
        score = max(0.0, m + tr + vs)
        reasons, risks = [], []
        if d["chg"] > 3: reasons.append(f"+{d['chg']:.1f}% 24h")
        if d["trend"] > 2: reasons.append(f"{d['trend']:.0f}% above 20-day MA")
        if d["vol_surge"] > 1.5: reasons.append(f"{d['vol_surge']:.1f}× avg volume")
        if d["ret7"] > 5: reasons.append(f"+{d['ret7']:.0f}% 7-day")
        if d["atrpct"] > 8: risks.append(f"high volatility (~{d['atrpct']:.0f}% daily range)")
        if d["chg"] > 25: risks.append("parabolic 24h move — reversal/pump risk")
        if d["qv"] < 15_000_000: risks.append("thinner liquidity")
        if d["trend"] < 0: risks.append("below its 20-day MA (counter-trend)")
        out.append({
            "symbol": d["base"] + "/USDT", "score": round(score, 1), "price": d["last"],
            "chg24": round(d["chg"], 2), "trend_pct": round(d["trend"], 1),
            "vol_surge": round(d["vol_surge"], 1), "ret7": round(d["ret7"], 1),
            "reasons": reasons or ["mild momentum"], "risks": risks or ["standard market risk"],
        })

    out.sort(key=lambda x: x["score"], reverse=True)
    return {
        "ok": True,
        "asof": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "universe_scanned": len(cand),
        "candidates": out[:limit],
        "note": "Momentum/trend/volume screen of liquid Binance USD pairs. Research signal only — "
                "NOT investment advice and NOT a return forecast. You decide; trade in paper first.",
    }
