"""NautilusTrader backtest microservice (FastAPI).

Wraps the institutional NautilusTrader BacktestEngine as an in-stack HTTP service
so the SaaS can validate strategies on a professional engine — BACKTEST ONLY.
This service places NO live orders and holds NO broker keys.

Each backtest runs in a fresh subprocess: NautilusTrader initialises a global Rust
logger once per process, so one-engine-per-process avoids a re-init panic.
"""
import json
import subprocess
import sys

from fastapi import FastAPI, Query

app = FastAPI(title="NautilusTrader Backtest Service", version="1.0")


@app.get("/health")
def health():
    import nautilus_trader
    return {"ok": True, "engine": "nautilus_trader", "version": nautilus_trader.__version__,
            "mode": "backtest-only", "places_live_orders": False}


@app.get("/backtest")
def backtest(
    symbol: str = Query("BTCUSDT"),
    fast: int = Query(10, ge=2, le=200),
    slow: int = Query(20, ge=3, le=400),
    size: str = Query("0.10"),
    interval: str = Query("1d"),
    limit: int = Query(1000, ge=50, le=1500),
):
    cmd = [sys.executable, "backtest.py", symbol, size, str(fast), str(slow), interval, str(limit)]
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=150)
    except subprocess.TimeoutExpired:
        return {"ok": False, "error": "backtest timed out"}
    for line in reversed(p.stdout.strip().splitlines()):
        line = line.strip()
        if line.startswith("{"):
            return json.loads(line)
    return {"ok": False, "error": "no result", "stderr": (p.stderr or "")[-600:]}
