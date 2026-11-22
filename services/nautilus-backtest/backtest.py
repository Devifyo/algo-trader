"""NautilusTrader backtest runner (one symbol per process → emits one JSON line).
LONG-ONLY EMA cross on Binance klines. Realistic engine: instrument maker/taker
fees, multi-currency cash account. Backtest/validation only — never live wiring.

Usage: python backtest.py SYMBOL SIZE FAST SLOW INTERVAL LIMIT
"""
import sys
import json
import urllib.request
from decimal import Decimal

import pandas as pd
from nautilus_trader.backtest.engine import BacktestEngine, BacktestEngineConfig
from nautilus_trader.config import LoggingConfig, StrategyConfig
from nautilus_trader.model.currencies import USDT
from nautilus_trader.model.enums import AccountType, OmsType, OrderSide
from nautilus_trader.model.identifiers import InstrumentId, Venue
from nautilus_trader.model.objects import Money
from nautilus_trader.model.data import Bar, BarType
from nautilus_trader.persistence.wranglers import BarDataWrangler
from nautilus_trader.test_kit.providers import TestInstrumentProvider
from nautilus_trader.trading.strategy import Strategy
from nautilus_trader.indicators.averages import ExponentialMovingAverage

START = 100_000

# Symbols we can model with a provider-backed instrument (correct precision/fees).
PROVIDERS = {
    "BTCUSDT": TestInstrumentProvider.btcusdt_binance,
    "ETHUSDT": TestInstrumentProvider.ethusdt_binance,
}
# Bar aggregation token by Binance interval.
AGG = {"1d": "1-DAY", "4h": "4-HOUR", "1h": "1-HOUR", "15m": "15-MINUTE", "5m": "5-MINUTE"}


class LongEmaConfig(StrategyConfig, frozen=True):
    instrument_id: InstrumentId
    bar_type: BarType
    trade_size: Decimal
    fast: int = 10
    slow: int = 20


class LongEma(Strategy):
    """Long when fast EMA > slow EMA; flat otherwise. Never shorts (spot reality)."""

    def __init__(self, config: LongEmaConfig):
        super().__init__(config)
        self.fast = ExponentialMovingAverage(config.fast)
        self.slow = ExponentialMovingAverage(config.slow)
        self.instrument = None

    def on_start(self):
        self.instrument = self.cache.instrument(self.config.instrument_id)
        self.register_indicator_for_bars(self.config.bar_type, self.fast)
        self.register_indicator_for_bars(self.config.bar_type, self.slow)
        self.subscribe_bars(self.config.bar_type)

    def on_bar(self, bar: Bar):
        if not (self.fast.initialized and self.slow.initialized):
            return
        long = self.portfolio.is_net_long(self.config.instrument_id)
        if self.fast.value > self.slow.value and not long:
            self.submit_order(self.order_factory.market(
                instrument_id=self.config.instrument_id, order_side=OrderSide.BUY,
                quantity=self.instrument.make_qty(self.config.trade_size)))
        elif self.fast.value < self.slow.value and long:
            self.close_all_positions(self.config.instrument_id)


def fetch_bars(symbol, interval, limit):
    url = f"https://api.binance.com/api/v3/klines?symbol={symbol}&interval={interval}&limit={limit}"
    rows = json.load(urllib.request.urlopen(url, timeout=30))
    df = pd.DataFrame(rows, columns=["ts", "open", "high", "low", "close", "volume",
                                     "ct", "qv", "n", "tb", "tq", "ig"])
    df["timestamp"] = pd.to_datetime(df["ts"].astype("int64"), unit="ms")
    return df.set_index("timestamp")[["open", "high", "low", "close", "volume"]].astype(float)


def run(symbol, size, fast, slow, interval, limit):
    instrument = PROVIDERS[symbol]()
    df = fetch_bars(symbol, interval, limit)
    engine = BacktestEngine(config=BacktestEngineConfig(trader_id="BT-001", logging=LoggingConfig(log_level="ERROR")))
    engine.add_venue(venue=Venue("BINANCE"), oms_type=OmsType.NETTING, account_type=AccountType.CASH,
                     base_currency=None, starting_balances=[Money(START, USDT)])
    engine.add_instrument(instrument)
    bar_type = BarType.from_str(f"{instrument.id}-{AGG[interval]}-LAST-EXTERNAL")
    engine.add_data(BarDataWrangler(bar_type=bar_type, instrument=instrument).process(df))
    engine.add_strategy(LongEma(LongEmaConfig(instrument_id=instrument.id, bar_type=bar_type,
                                              trade_size=Decimal(size), fast=fast, slow=slow)))
    engine.run()
    pos = engine.trader.generate_positions_report()
    engine.dispose()

    vals = []
    if pos is not None and len(pos) and "realized_pnl" in pos:
        for v in pos["realized_pnl"].tolist():
            try:
                vals.append(float(str(v).split()[0]))
            except (ValueError, IndexError):
                pass
    n = len(vals)
    total = sum(vals)
    return {
        "ok": True, "engine": "nautilus_trader", "symbol": symbol, "interval": interval,
        "strategy": f"long_ema_{fast}_{slow}", "bars": int(len(df)),
        "closed_trades": n, "win_rate_pct": round((sum(1 for x in vals if x > 0) / n * 100), 1) if n else 0.0,
        "net_pnl_usd": round(total, 2), "return_pct": round(total / START * 100, 2),
        "start_equity_usd": START, "fees_modeled": True, "validated": False,
        "note": "Single backtest — NOT validated (no OOS/robustness/multiple-testing pass).",
    }


if __name__ == "__main__":
    sym = sys.argv[1] if len(sys.argv) > 1 else "BTCUSDT"
    size = sys.argv[2] if len(sys.argv) > 2 else "0.10"
    fast = int(sys.argv[3]) if len(sys.argv) > 3 else 10
    slow = int(sys.argv[4]) if len(sys.argv) > 4 else 20
    interval = sys.argv[5] if len(sys.argv) > 5 else "1d"
    limit = int(sys.argv[6]) if len(sys.argv) > 6 else 1000
    try:
        if sym not in PROVIDERS:
            print(json.dumps({"ok": False, "error": f"unsupported symbol {sym}; supported: {list(PROVIDERS)}"}))
        elif interval not in AGG:
            print(json.dumps({"ok": False, "error": f"unsupported interval {interval}; supported: {list(AGG)}"}))
        else:
            print(json.dumps(run(sym, size, fast, slow, interval, limit)))
    except Exception as e:  # emit machine-readable failure for the API layer
        print(json.dumps({"ok": False, "error": f"{type(e).__name__}: {e}"}))
