"""Disciplined strategy validation on the NautilusTrader engine.

ANTI-OVERFITTING CONTRACT (do not weaken):
  * Configs are PRE-REGISTERED below, before seeing results.
  * In-sample (IS) is used for discovery; out-of-sample (OOS) is touched ONCE,
    only on the single best IS survivor. No retry-until-pass.
  * Multiple-testing correction (Bonferroni) across all configs tested.
  * Robustness gates: broad-based across symbols + family stability + min trades.
  * Conservative costs: 0.25%/side fee baked into every instrument.
  * If nothing survives, that is the result. Report it. Halt at the ceiling.

Long-only (spot reality). One symbol per engine; engines reuse one process
(bypass_logging avoids the global-logger re-init panic).
"""
import json
import math
import urllib.request
from collections import defaultdict
from decimal import Decimal

import pandas as pd
from nautilus_trader.backtest.engine import BacktestEngine, BacktestEngineConfig
from nautilus_trader.config import LoggingConfig, StrategyConfig
from nautilus_trader.model.currencies import USDT
from nautilus_trader.model.enums import AccountType, OmsType, OrderSide
from nautilus_trader.model.identifiers import InstrumentId, Symbol, Venue
from nautilus_trader.model.instruments import CurrencyPair
from nautilus_trader.model.objects import Money, Price, Quantity
from nautilus_trader.model.data import Bar, BarType
from nautilus_trader.persistence.wranglers import BarDataWrangler
from nautilus_trader.trading.strategy import Strategy
import nautilus_trader.model.currencies as CUR

# ── Pre-registered universe, split, costs ──────────────────────────────────
UNIVERSE = ["BTCUSDT", "ETHUSDT", "SOLUSDT", "BNBUSDT", "XRPUSDT", "ADAUSDT", "LTCUSDT", "DOGEUSDT"]
INTERVAL = "1d"
OOS_START = "2025-09-01"           # IS = before; OOS = on/after. OOS touched ONCE.
FEE = 0.0025                        # 0.25% per side (conservative, Alpaca-like)
NOTIONAL = 10_000.0                # fixed $ per entry → per-trade return = pnl/NOTIONAL
START = 1_000_000
MIN_TRADES = 50                    # IS minimum to consider a config at all
AGG = "1-DAY"

# ── Pre-registered strategy configs (family, params) ───────────────────────
CONFIGS = [
    ("ema", {"fast": 10, "slow": 20}),
    ("ema", {"fast": 20, "slow": 50}),
    ("ema", {"fast": 8, "slow": 21}),
    ("ema", {"fast": 12, "slow": 26}),
    ("ema", {"fast": 5, "slow": 20}),
    ("ema", {"fast": 50, "slow": 100}),
    ("mom", {"n": 30}),
    ("mom", {"n": 60}),
    ("mom", {"n": 90}),
    ("don", {"n": 20}),
    ("don", {"n": 55}),
    ("rsi", {"period": 14, "lo": 30, "hi": 55}),
]
N_CONFIGS = len(CONFIGS)


def cfg_label(family, p):
    return f"{family}_" + "_".join(f"{k}{v}" for k, v in p.items())


# ── Data ───────────────────────────────────────────────────────────────────
def fetch(symbol, interval="1d", limit=1000):
    url = f"https://api.binance.com/api/v3/klines?symbol={symbol}&interval={interval}&limit={limit}"
    rows = json.load(urllib.request.urlopen(url, timeout=30))
    df = pd.DataFrame(rows, columns=["ts", "o", "h", "l", "c", "v", "ct", "qv", "n", "tb", "tq", "ig"])
    df["timestamp"] = pd.to_datetime(df["ts"].astype("int64"), unit="ms")
    df = df.set_index("timestamp")
    df = df.rename(columns={"o": "open", "h": "high", "l": "low", "c": "close", "v": "volume"})
    return df[["open", "high", "low", "close", "volume"]].astype(float)


def make_instrument(symbol):
    base = getattr(CUR, symbol[:-4])
    pp, sp = 6, 6
    return CurrencyPair(
        instrument_id=InstrumentId(Symbol(symbol), Venue("BINANCE")),
        raw_symbol=Symbol(symbol),
        base_currency=base, quote_currency=USDT,
        price_precision=pp, size_precision=sp,
        price_increment=Price(10 ** -pp, pp), size_increment=Quantity(10 ** -sp, sp),
        lot_size=None,
        max_quantity=Quantity(1_000_000_000, sp), min_quantity=Quantity(10 ** -sp, sp),
        max_notional=None, min_notional=Money(1, USDT),
        max_price=Price(10 ** 9, pp), min_price=Price(10 ** -pp, pp),
        margin_init=Decimal(0), margin_maint=Decimal(0),
        maker_fee=Decimal(str(FEE)), taker_fee=Decimal(str(FEE)),
        ts_event=0, ts_init=0,
    )


# ── Strategy (long-only, multi-family) ──────────────────────────────────────
class SigConfig(StrategyConfig, frozen=True):
    instrument_id: InstrumentId
    bar_type: BarType
    family: str
    params: str            # JSON-encoded params (StrategyConfig wants simple types)


class SigStrategy(Strategy):
    def __init__(self, config: SigConfig):
        super().__init__(config)
        self.p = json.loads(config.params)
        self.family = config.family
        self.closes, self.highs, self.lows = [], [], []
        self.ema_f = self.ema_s = None
        self.instrument = None
        if self.family == "ema":
            self.warmup = self.p["slow"] + 1
        elif self.family == "mom":
            self.warmup = self.p["n"] + 1
        elif self.family == "don":
            self.warmup = self.p["n"] + 1
        else:  # rsi
            self.warmup = self.p["period"] + 1

    def on_start(self):
        self.instrument = self.cache.instrument(self.config.instrument_id)
        self.subscribe_bars(self.config.bar_type)

    def _ema(self, price):
        kf = 2 / (self.p.get("fast", 1) + 1)
        ks = 2 / (self.p.get("slow", 1) + 1)
        self.ema_f = price if self.ema_f is None else price * kf + self.ema_f * (1 - kf)
        self.ema_s = price if self.ema_s is None else price * ks + self.ema_s * (1 - ks)

    def _rsi(self, period):
        if len(self.closes) <= period:
            return 50.0
        gains = losses = 0.0
        for i in range(len(self.closes) - period, len(self.closes)):
            d = self.closes[i] - self.closes[i - 1]
            gains += max(d, 0.0)
            losses += max(-d, 0.0)
        if losses == 0:
            return 100.0
        rs = (gains / period) / (losses / period)
        return 100 - 100 / (1 + rs)

    def _buy(self, price):
        qty = self.instrument.make_qty(Decimal(str(NOTIONAL / price)))
        self.submit_order(self.order_factory.market(
            instrument_id=self.config.instrument_id, order_side=OrderSide.BUY, quantity=qty))

    def on_bar(self, bar: Bar):
        c = float(bar.close)
        self.closes.append(c)
        self.highs.append(float(bar.high))
        self.lows.append(float(bar.low))
        if self.family == "ema":
            self._ema(c)
        if len(self.closes) < self.warmup:
            return
        long = self.portfolio.is_net_long(self.config.instrument_id)

        if self.family == "ema":
            want = self.ema_f > self.ema_s
            if want and not long:
                self._buy(c)
            elif not want and long:
                self.close_all_positions(self.config.instrument_id)
        elif self.family == "mom":
            n = self.p["n"]
            want = c > self.closes[-1 - n]
            if want and not long:
                self._buy(c)
            elif not want and long:
                self.close_all_positions(self.config.instrument_id)
        elif self.family == "don":
            n = self.p["n"]
            hh = max(self.highs[-n - 1:-1])
            ll = min(self.lows[-n - 1:-1])
            if not long and c > hh:
                self._buy(c)
            elif long and c < ll:
                self.close_all_positions(self.config.instrument_id)
        else:  # rsi mean-reversion
            r = self._rsi(self.p["period"])
            if not long and r < self.p["lo"]:
                self._buy(c)
            elif long and r > self.p["hi"]:
                self.close_all_positions(self.config.instrument_id)


# ── One backtest → list of per-trade returns ────────────────────────────────
def backtest(symbol, df, family, params):
    inst = make_instrument(symbol)
    engine = BacktestEngine(config=BacktestEngineConfig(
        trader_id="VAL-001", logging=LoggingConfig(bypass_logging=True)))
    engine.add_venue(venue=Venue("BINANCE"), oms_type=OmsType.NETTING, account_type=AccountType.CASH,
                     base_currency=None, starting_balances=[Money(START, USDT)])
    engine.add_instrument(inst)
    bar_type = BarType.from_str(f"{inst.id}-{AGG}-LAST-EXTERNAL")
    engine.add_data(BarDataWrangler(bar_type=bar_type, instrument=inst).process(df))
    engine.add_strategy(SigStrategy(SigConfig(
        instrument_id=inst.id, bar_type=bar_type, family=family, params=json.dumps(params))))
    engine.run()
    pos = engine.trader.generate_positions_report()
    engine.dispose()
    rets = []
    if pos is not None and len(pos) and "realized_pnl" in pos:
        for v in pos["realized_pnl"].tolist():
            try:
                rets.append(float(str(v).split()[0]) / NOTIONAL)
            except (ValueError, IndexError):
                pass
    return rets


# ── Stats ────────────────────────────────────────────────────────────────────
def phi(x):
    return 0.5 * (1 + math.erf(x / math.sqrt(2)))


def stats(returns):
    n = len(returns)
    if n < 2:
        return {"n": n, "mean": 0.0, "t": 0.0, "p": 1.0, "net": sum(returns)}
    mean = sum(returns) / n
    var = sum((r - mean) ** 2 for r in returns) / (n - 1)
    sd = math.sqrt(var)
    if sd == 0:
        t = float("inf") if mean > 0 else 0.0
        p = 0.0 if mean > 0 else 1.0
    else:
        t = mean / (sd / math.sqrt(n))
        p = 1 - phi(t)        # one-sided H1: mean per-trade return > 0
    return {"n": n, "mean": mean, "t": t, "p": p, "net": sum(returns)}


# ── Run ──────────────────────────────────────────────────────────────────────
def main():
    print(f"Loading {len(UNIVERSE)} symbols, daily, split at {OOS_START} (OOS touched once)...")
    data_is, data_oos = {}, {}
    for s in UNIVERSE:
        try:
            df = fetch(s, INTERVAL, 1000)
        except Exception as e:
            print(f"  {s}: fetch failed ({e}) — skipped")
            continue
        cut = pd.Timestamp(OOS_START)
        data_is[s] = df[df.index < cut]
        data_oos[s] = df[df.index >= cut]
    syms = list(data_is.keys())
    span = f"{min(d.index.min() for d in data_is.values()).date()} … {max(d.index.max() for d in data_oos.values()).date()}"
    print(f"  symbols={syms}\n  history span: {span}\n")

    # ── PHASE 0 — IS discovery (OOS NOT touched) ──
    print("=" * 78)
    print("PHASE 0 — IN-SAMPLE DISCOVERY  (OOS untouched)")
    print("=" * 78)
    per_cfg = {}
    fam_pos = defaultdict(lambda: [0, 0])   # family -> [positive_configs, total_configs]
    for family, params in CONFIGS:
        label = cfg_label(family, params)
        all_rets, sym_net = [], {}
        for s in syms:
            r = backtest(s, data_is[s], family, params)
            all_rets += r
            sym_net[s] = sum(r)
        st = stats(all_rets)
        st["adj_p"] = min(1.0, st["p"] * N_CONFIGS)
        pos_syms = sum(1 for v in sym_net.values() if v > 0)
        total = st["net"]
        top_share = (max(sym_net.values()) / total) if total > 0 and sym_net else 1.0
        st["broad"] = (pos_syms >= math.ceil(len(syms) / 2)) and (total > 0) and (top_share <= 0.5)
        st["pos_syms"] = pos_syms
        per_cfg[label] = (family, params, st)
        fam_pos[family][1] += 1
        if total > 0:
            fam_pos[family][0] += 1
        print(f"  {label:<16} n={st['n']:<4} net={st['net']*100:+7.1f}%R  mean={st['mean']*100:+5.2f}%  "
              f"t={st['t']:+5.2f}  p={st['p']:.3f}  adjP={st['adj_p']:.3f}  "
              f"posSym={pos_syms}/{len(syms)}  broad={'Y' if st['broad'] else 'n'}")

    # survivor criteria (all must hold)
    survivors = []
    for label, (family, params, st) in per_cfg.items():
        stable = fam_pos[family][0] / fam_pos[family][1] >= 0.5
        if st["n"] >= MIN_TRADES and st["adj_p"] < 0.05 and st["broad"] and stable:
            survivors.append((label, family, params, st))

    print("\nGates: n>=%d AND Bonferroni adjP<0.05 AND broad-based AND family-stable" % MIN_TRADES)
    if not survivors:
        print("\nRESULT: NONE SURVIVED at Phase 0. OOS was never touched.")
        print("        (Truthful ceiling — no validated edge on this engine/universe.)")
        return
    survivors.sort(key=lambda x: x[3]["t"], reverse=True)
    best_label, bf, bp, bst = survivors[0]
    print(f"\n{len(survivors)} survivor(s). Best by t-stat: {best_label} "
          f"(t={bst['t']:.2f}, adjP={bst['adj_p']:.3f}). Advancing ONE to OOS.")

    # ── PHASE 1 — OOS confirmation (touched ONCE, best survivor only) ──
    print("\n" + "=" * 78)
    print(f"PHASE 1 — OUT-OF-SAMPLE CONFIRMATION  ({best_label}, used once)")
    print("=" * 78)
    oos_rets = []
    for s in syms:
        oos_rets += backtest(s, data_oos[s], bf, bp)
    o = stats(oos_rets)
    print(f"  OOS: n={o['n']}  net={o['net']*100:+.1f}%R  mean={o['mean']*100:+.2f}%  t={o['t']:+.2f}  p={o['p']:.3f}")
    passed = o["n"] >= 20 and o["net"] > 0 and o["p"] < 0.10
    print("\nRESULT: " + (
        f"PASS — {best_label} held out-of-sample. Candidate for FORWARD PAPER (not live)."
        if passed else
        f"FAIL — {best_label} did not confirm out-of-sample. No validated edge. Halt."))


if __name__ == "__main__":
    main()
