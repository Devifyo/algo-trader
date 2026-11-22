"""Fetch Binance daily klines → CSV for the NautilusTrader backtest.
Columns: timestamp(ms open time), open, high, low, close, volume.
Usage: python fetch_bars.py BTCUSDT 1000
"""
import sys
import json
import urllib.request

symbol = sys.argv[1] if len(sys.argv) > 1 else "BTCUSDT"
limit = int(sys.argv[2]) if len(sys.argv) > 2 else 1000
url = f"https://api.binance.com/api/v3/klines?symbol={symbol}&interval=1d&limit={limit}"
rows = json.load(urllib.request.urlopen(url, timeout=30))
with open(f"{symbol}_1d.csv", "w") as f:
    f.write("timestamp,open,high,low,close,volume\n")
    for r in rows:
        f.write(f"{r[0]},{r[1]},{r[2]},{r[3]},{r[4]},{r[5]}\n")
print(f"wrote {symbol}_1d.csv  ({len(rows)} daily bars)")
