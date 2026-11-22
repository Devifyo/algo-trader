# 18-Day MA Qualified Breakout — strategy module

**Status: BACKTEST-FIRST / PAPER-ONLY.** The module (`src/ma18.rs`) is implemented,
unit-tested, and exercised by the backtester (`src/bin/ma18_backtest.rs`). It is
**NOT registered in the live strategy registry** (`strategies::build`) — no live
execution is wired until the backtest is reviewed. Existing strategies/behaviour
are untouched.

## Decision pipeline (short-circuits at the first failure; the failing gate is logged)

```
1 Fundamental → 2 Day1 → 3 Day2 → 4 Consolidation → 5 OBV → 6 Break → 7 R:R → ENTER
```

The **fundamental gate is the absolute first hard block** — a technically perfect
setup with no fundamental backing is rejected. (Implementation note: consolidation
& OBV are measured over the window ending just before **Day 1**, so Day 1/Day 2 are
located first; that's the only deviation from the doc's listed order.)

- **18MA** = SMA of the last 18 daily closes.
- **OBV** = running cumulative volume (+ on up-close, − on down-close). "Rising" =
  positive least-squares slope across the consolidation window (falling for shorts).
- **Day 1 / Day 2** (long) = two days whose **LOW > 18MA** within `qualifying_day_max_gap`.
  **True High** = highest high of the two. **Entry** = a later day **breaks above
  True High** (buy-stop at True High; *never anticipated*). Short = exact mirror
  (HIGH < 18MA → True Low → break below).
- **Stop (structural, re-evaluated each bar):** `mirror` = exit when a daily candle
  forms fully on the opposite side of the 18MA; `strict` = exit only on a full
  opposite two-qualifying-day setup. Optional `use_hard_stop` gap safety-net.
- **R:R:** R = |entry − 18MA|; target = entry ± `target_r`·R; skip if `target_r < min_rr`.

## Configurable parameters (all in `Ma18Config`, no hardcoding)

| Param | Default | Meaning |
|---|---|---|
| `ma_length` | 18 | SMA length (daily closes) |
| `consolidation_window` | 10 | days ending just before Day 1 |
| `range_pct_threshold` | 0.08 | "tight" = (max high − min low)/mean close ≤ 8% |
| `qualifying_day_max_gap` | 1 | ≤1 non-qualifying day allowed between Day 1 and Day 2 |
| `entry_validity_days` | 3 | how long the True-High/Low break stays armed |
| `target_r` | 2.0 | profit target as a multiple of R |
| `min_rr` | 2.0 | reject if computed R:R below this |
| `stop_mode` | `mirror` | `mirror` \| `strict` |
| `use_hard_stop` | false | gap safety-net protective stop |
| `hard_stop_pct` | 0.20 | hard-stop distance when enabled |
| `near_structure_check` | false | require consolidation near resistance/support |
| `structure_lookback` | 50 | lookback for that structure check |
| `proximity_pct` | 0.05 | how near to structure counts |

(`risk_per_trade` is intentionally **not** here — live sizing reuses the existing
risk crate's allocation sizing.)

## Fundamental bias — date-ranged file (defaults to BLOCKED)

Discretionary, never invented. Provide a JSON file of date-ranged entries
(`ma18_fundamentals.example.json` is a template):

```json
[
  { "symbol": "BTCUSDT", "bias": "bullish", "start_date": "2024-01-01", "end_date": "2024-03-31" },
  { "symbol": "ETHUSDT", "bias": "bearish", "start_date": "2024-04-01", "end_date": "2024-06-30" }
]
```

- Longs need `bullish`, shorts need `bearish`; any symbol/date **not covered → `none` → blocked**.
- Dates are inclusive `YYYY-MM-DD` (UTC). Never apply a single static "current" bias
  across all history — use real date ranges.
- Live phase (deferred): this file is replaced by a DB table + admin setter, plus a
  `TODO` hook for an automated news/fundamentals feed.

## Backtester

Runs over **Binance 1-day klines** (UTC midnight), reusing the exact `ma18` pipeline.

```bash
# from the workspace root, in a Rust container (network on):
docker run --rm -v "$PWD/services/trading-engine-rust:/app" -w /app rust:1-slim-bookworm \
  sh -c "apt-get update -qq && apt-get install -y -qq pkg-config >/dev/null; \
         cargo run -q -p signals --bin ma18_backtest -- --days 1000"

# second, fundamentally-filtered run:
  ... cargo run -q -p signals --bin ma18_backtest -- --bias crates/signals/ma18_fundamentals.example.json
```

Flags: `--symbols A,B,C` (default the 12-symbol universe), `--days N` (kline limit,
≤1000), `--bias <file>` (date-ranged; omit → **assume-pass**, showing the raw
technical edge).

Output: per-gate **funnel** (where setups die), **combined vs long-only** stats
(n / win% / avg R / total R / max DD in R), per-symbol breakdown, and Alpaca
long-eligibility (curated list — confirmed against `/v2/assets` at the live phase).

## Live wiring — deferred (needs your review of backtest first)

Before this can trade live, the live phase adds: (1) a **daily-candle feed**
(the engine runs 1-minute candles today); (2) **armed stop-entry orders** +
structural-exit hooks in risk/execution; (3) the **fundamental DB store**;
(4) register in `strategies::build`. Shorts remain **paper-only** until a
short-capable broker is connected (Alpaca spot is long-only).
