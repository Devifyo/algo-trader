"""NautilusTrader backtest — LONG-ONLY EMA-cross on Binance daily bars.
Long-only matches spot reality (Alpaca/Binance spot can't short). Realistic
engine: instrument maker/taker fees, multi-currency cash account.

Usage: python backtest.py BTCUSDT 0.10 [fast] [slow]   (one symbol per process)
"""
import sys
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


class LongEmaConfig(StrategyConfig, frozen=True):
    instrument_id: InstrumentId
    bar_type: BarType
    trade_size: Decimal
    fast: int = 10
    slow: int = 20


class LongEma(Strategy):
    """Long when fast EMA > slow EMA; flat otherwise. Never shorts."""

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


def run(symbol, instrument, size, fast, slow):
    engine = BacktestEngine(config=BacktestEngineConfig(trader_id="BT-001", logging=LoggingConfig(log_level="ERROR")))
    engine.add_venue(venue=Venue("BINANCE"), oms_type=OmsType.NETTING, account_type=AccountType.CASH,
                     base_currency=None, starting_balances=[Money(START, USDT)])
    engine.add_instrument(instrument)
    df = pd.read_csv(f"{symbol}_1d.csv")
    df["timestamp"] = pd.to_datetime(df["timestamp"], unit="ms")
    df = df.set_index("timestamp")[["open", "high", "low", "close", "volume"]].astype(float)
    bar_type = BarType.from_str(f"{instrument.id}-1-DAY-LAST-EXTERNAL")
    engine.add_data(BarDataWrangler(bar_type=bar_type, instrument=instrument).process(df))
    engine.add_strategy(LongEma(LongEmaConfig(instrument_id=instrument.id, bar_type=bar_type,
                                              trade_size=Decimal(size), fast=fast, slow=slow)))
    engine.run()
    pos = engine.trader.generate_positions_report()
    engine.dispose()

    vals = []
    if pos is not None and len(pos) and "realized_pnl" in pos:
        for v in pos["realized_pnl"].tolist():
            try: vals.append(float(str(v).split()[0]))
            except (ValueError, IndexError): pass
    n = len(vals); total = sum(vals)
    win = (sum(1 for x in vals if x > 0) / n * 100) if n else 0.0
    ret = total / START * 100
    print(f"{symbol:<9} bars={len(df):<5} closed_trades={n:<4} win%={win:<5.1f} net_pnl=${total:<10.2f} return={ret:+.2f}%")


if __name__ == "__main__":
    sym = sys.argv[1] if len(sys.argv) > 1 else "BTCUSDT"
    size = sys.argv[2] if len(sys.argv) > 2 else "0.10"
    fast = int(sys.argv[3]) if len(sys.argv) > 3 else 10
    slow = int(sys.argv[4]) if len(sys.argv) > 4 else 20
    inst = TestInstrumentProvider.btcusdt_binance() if sym == "BTCUSDT" else TestInstrumentProvider.ethusdt_binance()
    run(sym, inst, size, fast, slow)
