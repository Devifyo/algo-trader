"""Minimal, HONEST backtester.

Single-pass over REAL Binance klines, long-only (matches spot execution), with
per-side trading fees and an ATR trailing stop. This is an INDICATIVE sanity
check — NOT walk-forward / CPCV / Deflated-Sharpe validation, and it does not
model slippage, funding, partial fills or latency. Treat the numbers as a smoke
test of a strategy's shape, not proof of a live edge.
"""
from __future__ import annotations

import math
import httpx

FEE = 0.0004  # 4 bps per side
BINANCE = "https://api.binance.com/api/v3/klines"
VALID_TF = {"5m", "15m", "1h", "4h", "1d"}


async def fetch_klines(symbol: str, interval: str, limit: int = 1000):
    async with httpx.AsyncClient(timeout=30) as cli:
        r = await cli.get(BINANCE, params={"symbol": symbol, "interval": interval, "limit": limit})
        r.raise_for_status()
        rows = r.json()
    o = [float(k[1]) for k in rows]
    h = [float(k[2]) for k in rows]
    l = [float(k[3]) for k in rows]
    c = [float(k[4]) for k in rows]
    return o, h, l, c


def _ema_series(vals, period):
    k = 2.0 / (period + 1)
    e = None
    out = []
    for v in vals:
        e = v if e is None else v * k + e * (1 - k)
        out.append(e)
    return out


def _atr_series(h, l, c, period):
    n = len(c)
    out = [None] * n
    if n <= period:
        return out
    tr = [0.0] * n
    for i in range(1, n):
        tr[i] = max(h[i] - l[i], abs(h[i] - c[i - 1]), abs(l[i] - c[i - 1]))
    a = sum(tr[1:period + 1]) / period
    out[period] = a
    for i in range(period + 1, n):
        a = (a * (period - 1) + tr[i]) / period
        out[i] = a
    return out


def _rsi_series(c, period):
    n = len(c)
    out = [None] * n
    if n <= period:
        return out
    gains = [0.0] * n
    losses = [0.0] * n
    for i in range(1, n):
        d = c[i] - c[i - 1]
        gains[i] = max(d, 0.0)
        losses[i] = max(-d, 0.0)
    ag = sum(gains[1:period + 1]) / period
    al = sum(losses[1:period + 1]) / period
    out[period] = 100.0 if al == 0 else 100 - 100 / (1 + ag / al)
    for i in range(period + 1, n):
        ag = (ag * (period - 1) + gains[i]) / period
        al = (al * (period - 1) + losses[i]) / period
        out[i] = 100.0 if al == 0 else 100 - 100 / (1 + ag / al)
    return out


def _adx_series(h, l, c, period):
    n = len(c)
    out = [None] * n
    if n < 2 * period + 1:
        return out
    pdm = [0.0] * n
    ndm = [0.0] * n
    tr = [0.0] * n
    for i in range(1, n):
        up = h[i] - h[i - 1]
        dn = l[i - 1] - l[i]
        pdm[i] = up if (up > dn and up > 0) else 0.0
        ndm[i] = dn if (dn > up and dn > 0) else 0.0
        tr[i] = max(h[i] - l[i], abs(h[i] - c[i - 1]), abs(l[i] - c[i - 1]))

    def wilder(arr):
        sm = [None] * n
        s = sum(arr[1:period + 1])
        sm[period] = s
        for i in range(period + 1, n):
            s = s - (s / period) + arr[i]
            sm[i] = s
        return sm

    str_, spdm, sndm = wilder(tr), wilder(pdm), wilder(ndm)
    dx = [None] * n
    for i in range(period, n):
        if str_[i] and str_[i] > 0:
            pdi = 100 * spdm[i] / str_[i]
            ndi = 100 * sndm[i] / str_[i]
            den = pdi + ndi
            dx[i] = 100 * abs(pdi - ndi) / den if den > 0 else 0.0
    start = 2 * period
    seed = [d for d in dx[period:start] if d is not None]
    if not seed or start >= n:
        return out
    a = sum(seed) / len(seed)
    out[start] = a
    for i in range(start + 1, n):
        if dx[i] is not None:
            a = (a * (period - 1) + dx[i]) / period
            out[i] = a
    return out


def _signals(strategy, o, h, l, c):
    """Return a per-bar boolean 'want to be long' array (long-only)."""
    n = len(c)
    sig = [False] * n
    if strategy == "trend_following":
        fast = _ema_series(c, 21)
        slow = _ema_series(c, 55)
        adx = _adx_series(h, l, c, 14)
        for i in range(n):
            sig[i] = adx[i] is not None and adx[i] >= 20 and fast[i] > slow[i] and c[i] > fast[i]
    elif strategy == "momentum":
        look = 10
        for i in range(look, n):
            sig[i] = c[i] > c[i - look]
    elif strategy == "mean_reversion":
        rsi = _rsi_series(c, 14)
        ema = _ema_series(c, 50)
        # enter when oversold, stay long until back above the mean
        holding = False
        for i in range(n):
            if rsi[i] is None:
                continue
            if not holding and rsi[i] < 30:
                holding = True
            elif holding and c[i] >= ema[i]:
                holding = False
            sig[i] = holding
    elif strategy == "breakout":
        win = 20
        for i in range(win, n):
            sig[i] = c[i] >= max(h[i - win:i])
    elif strategy == "vol_expansion":
        atr = _atr_series(h, l, c, 14)
        ema = _ema_series(c, 20)
        for i in range(15, n):
            sig[i] = atr[i] is not None and atr[i - 1] is not None and atr[i] > atr[i - 1] and c[i] > ema[i]
    return sig


def run_backtest(strategy, o, h, l, c):
    n = len(c)
    atr = _atr_series(h, l, c, 14)
    sig = _signals(strategy, o, h, l, c)

    equity = 1.0
    peak = 1.0
    maxdd = 0.0
    in_pos = False
    entry = 0.0
    stop = 0.0
    dist = 0.0
    rets = []

    for i in range(n):
        price = c[i]
        if in_pos:
            if atr[i]:
                dist = 2.0 * atr[i]
            stop = max(stop, price - dist)
            if price <= stop or not sig[i]:
                net = (price / entry) * (1 - FEE) * (1 - FEE) - 1.0
                equity *= (1 + net)
                rets.append(net)
                in_pos = False
        elif sig[i] and atr[i]:
            in_pos = True
            entry = price
            dist = 2.0 * atr[i]
            stop = price - dist
        peak = max(peak, equity)
        if peak > 0:
            maxdd = max(maxdd, (peak - equity) / peak)

    t = len(rets)
    wins = [r for r in rets if r > 0]
    losses = [r for r in rets if r < 0]
    gp = sum(wins)
    gl = abs(sum(losses))
    mean = sum(rets) / t if t else 0.0
    sd = math.sqrt(sum((r - mean) ** 2 for r in rets) / (t - 1)) if t > 1 else 0.0
    dn = [r for r in rets if r < 0]
    dsd = math.sqrt(sum(r * r for r in dn) / len(dn)) if dn else 0.0

    return {
        "trades": t,
        "win_rate": round(len(wins) / t * 100, 1) if t else 0.0,
        "net_return_pct": round((equity - 1) * 100, 2),
        "profit_factor": round(gp / gl, 2) if gl > 0 else (99.0 if gp > 0 else 0.0),
        "sharpe": round(mean / sd, 2) if sd > 0 else None,
        "sortino": round(mean / dsd, 2) if dsd > 0 else (99.0 if mean > 0 else None),
        "max_dd_pct": round(maxdd * 100, 2),
        "avg_trade_pct": round(mean * 100, 3),
    }


async def backtest(strategy: str, symbol: str, timeframe: str) -> dict:
    if timeframe not in VALID_TF:
        return {"ok": False, "error": f"unsupported timeframe {timeframe}"}
    try:
        o, h, l, c = await fetch_klines(symbol, timeframe, 1000)
    except Exception as e:  # noqa: BLE001
        return {"ok": False, "error": f"could not fetch Binance data: {e}"}
    if len(c) < 120:
        return {"ok": False, "error": "not enough historical bars"}
    metrics = run_backtest(strategy, o, h, l, c)
    return {
        "ok": True,
        "bars": len(c),
        "note": "Indicative single-pass long-only backtest with 4bps/side fees. Not walk-forward/CPCV validation.",
        "metrics": metrics,
    }
