"""Probe: (1) can we run multiple BacktestEngines in one process (logging)?
(2) generic CurrencyPair construction + which base currencies are built in."""
import inspect
from nautilus_trader.config import LoggingConfig

print("=== LoggingConfig params ===")
print([p for p in inspect.signature(LoggingConfig).parameters])

# Try bypass to avoid the global-logger re-init panic across engines.
from nautilus_trader.backtest.engine import BacktestEngine, BacktestEngineConfig
from nautilus_trader.model.enums import AccountType, OmsType
from nautilus_trader.model.identifiers import Venue
from nautilus_trader.model.objects import Money
from nautilus_trader.model.currencies import USDT

def mk():
    try:
        lc = LoggingConfig(bypass_logging=True)
    except TypeError:
        try:
            lc = LoggingConfig(bypass=True)
        except TypeError:
            lc = LoggingConfig(log_level="ERROR")
    e = BacktestEngine(config=BacktestEngineConfig(trader_id="BT-001", logging=lc))
    e.add_venue(venue=Venue("BINANCE"), oms_type=OmsType.NETTING, account_type=AccountType.CASH,
                base_currency=None, starting_balances=[Money(100_000, USDT)])
    return e

try:
    a = mk(); a.dispose()
    b = mk(); b.dispose()
    print("MULTI-ENGINE-OK: two engines in one process succeeded")
except Exception as e:
    print("MULTI-ENGINE-FAIL:", type(e).__name__, e)

print("\n=== CurrencyPair signature ===")
from nautilus_trader.model.instruments import CurrencyPair
print([p for p in inspect.signature(CurrencyPair.__init__).parameters])

print("\n=== built-in currencies ===")
import nautilus_trader.model.currencies as C
for sym in ["BTC", "ETH", "SOL", "BNB", "XRP", "ADA", "DOGE", "LTC", "AVAX", "LINK", "USDT"]:
    print(sym, hasattr(C, sym))
