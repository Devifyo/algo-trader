//! 18-Day MA Qualified Breakout — DAILY timeframe, "fundamentals-first".
//!
//! Self-contained, BACKTEST-FIRST / PAPER-ONLY. This module is NOT registered in
//! the live strategy registry (`strategies::build`) yet — live wiring (daily-candle
//! feed + armed stop orders + a real fundamental store) is deferred until the
//! backtest is reviewed. The backtester (`bin/ma18_backtest.rs`) reuses the exact
//! functions below, so the tested logic == the future live logic (no drift).
//!
//! Pipeline (short-circuits at the first failure; the failing gate is reported):
//!   1 Fundamental → 2 Day1 → 3 Day2 → 4 Consolidation → 5 OBV → 6 Break → 7 R:R
//! NOTE: consolidation/OBV are evaluated over the window ending just before Day 1,
//! so Day 1/Day 2 must be located first — the only deviation from the doc's listed
//! order. The fundamental gate remains the absolute first hard block.

use crate::indicators as ind;
use crate::strategies::RawSignal;
use tcore::types::{Candle, Side};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopMode {
    Mirror,
    Strict,
}

#[derive(Clone, Debug)]
pub struct Ma18Config {
    pub ma_length: usize,
    pub consolidation_window: usize,
    pub range_pct_threshold: f64,
    pub qualifying_day_max_gap: usize,
    pub entry_validity_days: usize,
    pub target_r: f64,
    pub min_rr: f64,
    pub stop_mode: StopMode,
    pub use_hard_stop: bool,
    pub hard_stop_pct: f64,
    pub near_structure_check: bool,
    pub structure_lookback: usize,
    pub proximity_pct: f64,
}

impl Default for Ma18Config {
    fn default() -> Self {
        Self {
            ma_length: 18,
            consolidation_window: 10,
            range_pct_threshold: 0.08,
            qualifying_day_max_gap: 1,
            entry_validity_days: 3,
            target_r: 2.0,
            min_rr: 2.0,
            stop_mode: StopMode::Mirror,
            use_hard_stop: false,
            hard_stop_pct: 0.20,
            near_structure_check: false,
            structure_lookback: 50,
            proximity_pct: 0.05,
        }
    }
}

/// Pipeline gates in evaluation order (+ Data for insufficient history).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Data,
    Fundamental,
    Day1,
    Day2,
    Consolidation,
    Obv,
    Break,
    RiskReward,
}

impl Gate {
    pub fn label(&self) -> &'static str {
        match self {
            Gate::Data => "data",
            Gate::Fundamental => "fundamental",
            Gate::Day1 => "day1",
            Gate::Day2 => "day2",
            Gate::Consolidation => "consolidation",
            Gate::Obv => "obv",
            Gate::Break => "break",
            Gate::RiskReward => "risk_reward",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Outcome {
    Enter(RawSignal),
    NoTrade { gate: Gate, reason: String },
}

/// SMA of `closes` over `len` values ending at index `end` (inclusive).
fn sma_ending_at(closes: &[f64], end: usize, len: usize) -> Option<f64> {
    if len == 0 || end + 1 < len {
        return None;
    }
    let s: f64 = closes[end + 1 - len..=end].iter().sum();
    Some(s / len as f64)
}

/// LONG pipeline. `fundamental_ok` is resolved by the caller (live = real bias;
/// backtest = toggle / date-ranged file). `c` is ascending daily candles; the
/// last element is the potential break day.
pub fn evaluate_long(c: &[Candle], cfg: &Ma18Config, fundamental_ok: bool) -> Outcome {
    let n = c.len();
    let need = cfg.ma_length + cfg.consolidation_window + cfg.entry_validity_days + 2;
    if n < need {
        return Outcome::NoTrade { gate: Gate::Data, reason: format!("need {need} bars, have {n}") };
    }
    if !fundamental_ok {
        return Outcome::NoTrade { gate: Gate::Fundamental, reason: "no fundamental backing → invalid setup".into() };
    }

    let closes: Vec<f64> = c.iter().map(|x| x.close).collect();
    let today = n - 1;
    let ma = |i: usize| sma_ending_at(&closes, i, cfg.ma_length);
    let qualifies = |i: usize| ma(i).map_or(false, |m| c[i].low > m); // entire range above MA

    // Most-recent Day1/Day2 pair within the armed window (Day2 in [today-validity, today-1]).
    let mut found: Option<(usize, usize)> = None;
    let win_lo = today.saturating_sub(cfg.entry_validity_days);
    'outer: for d2 in (win_lo..today).rev() {
        if !qualifies(d2) {
            continue;
        }
        let d1_lo = d2.saturating_sub(cfg.qualifying_day_max_gap + 1);
        for d1 in (d1_lo..d2).rev() {
            if qualifies(d1) {
                found = Some((d1, d2));
                break 'outer;
            }
        }
    }
    let (d1, d2) = match found {
        Some(p) => p,
        None => {
            let any_q = (win_lo..today).any(qualifies);
            let gate = if any_q { Gate::Day2 } else { Gate::Day1 };
            return Outcome::NoTrade { gate, reason: "no qualifying 2-day setup (low > 18MA)".into() };
        }
    };
    let true_high = c[d1].high.max(c[d2].high);

    // Consolidation window ends just before Day 1.
    if d1 < cfg.consolidation_window {
        return Outcome::NoTrade { gate: Gate::Data, reason: "insufficient pre-Day1 history".into() };
    }
    let cs = d1 - cfg.consolidation_window;
    let win = &c[cs..d1];
    let hi = win.iter().map(|x| x.high).fold(f64::MIN, f64::max);
    let lo = win.iter().map(|x| x.low).fold(f64::MAX, f64::min);
    let meanc = win.iter().map(|x| x.close).sum::<f64>() / win.len() as f64;
    let range_pct = if meanc > 0.0 { (hi - lo) / meanc } else { f64::MAX };
    if range_pct > cfg.range_pct_threshold {
        return Outcome::NoTrade { gate: Gate::Consolidation, reason: format!("range {range_pct:.3} > {:.3}", cfg.range_pct_threshold) };
    }
    if cfg.near_structure_check {
        let sl = d1.saturating_sub(cfg.structure_lookback);
        let hh = c[sl..d1].iter().map(|x| x.high).fold(f64::MIN, f64::max);
        if hh > 0.0 && (hh - hi) / hh > cfg.proximity_pct {
            return Outcome::NoTrade { gate: Gate::Consolidation, reason: "not near resistance".into() };
        }
    }

    // OBV slope must be rising across the window through Day 2.
    let obv_all = ind::obv(&c[..=d2]);
    let slope = ind::linreg_slope(&obv_all[cs..=d2]);
    if slope <= 0.0 {
        return Outcome::NoTrade { gate: Gate::Obv, reason: format!("OBV slope {slope:.2} not rising") };
    }

    // Break of True High today (no anticipation).
    if c[today].high <= true_high {
        return Outcome::NoTrade { gate: Gate::Break, reason: "no break above True High".into() };
    }

    let entry = true_high;
    let stop = ma(today).unwrap_or(closes[today]); // structural ref = 18MA
    let r = entry - stop;
    if r <= 0.0 || cfg.target_r < cfg.min_rr {
        return Outcome::NoTrade { gate: Gate::RiskReward, reason: format!("RR {:.2} < {:.2} or non-positive risk", cfg.target_r, cfg.min_rr) };
    }
    let take_profit = entry + cfg.target_r * r;
    let strength = (1.0 - range_pct / cfg.range_pct_threshold).clamp(0.1, 1.0);
    Outcome::Enter(RawSignal { side: Side::Buy, price: entry, stop, take_profit: Some(take_profit), strength })
}

/// SHORT pipeline — exact mirror of `evaluate_long`.
pub fn evaluate_short(c: &[Candle], cfg: &Ma18Config, fundamental_ok: bool) -> Outcome {
    let n = c.len();
    let need = cfg.ma_length + cfg.consolidation_window + cfg.entry_validity_days + 2;
    if n < need {
        return Outcome::NoTrade { gate: Gate::Data, reason: format!("need {need} bars, have {n}") };
    }
    if !fundamental_ok {
        return Outcome::NoTrade { gate: Gate::Fundamental, reason: "no fundamental backing → invalid setup".into() };
    }

    let closes: Vec<f64> = c.iter().map(|x| x.close).collect();
    let today = n - 1;
    let ma = |i: usize| sma_ending_at(&closes, i, cfg.ma_length);
    let qualifies = |i: usize| ma(i).map_or(false, |m| c[i].high < m); // entire range below MA

    let mut found: Option<(usize, usize)> = None;
    let win_lo = today.saturating_sub(cfg.entry_validity_days);
    'outer: for d2 in (win_lo..today).rev() {
        if !qualifies(d2) {
            continue;
        }
        let d1_lo = d2.saturating_sub(cfg.qualifying_day_max_gap + 1);
        for d1 in (d1_lo..d2).rev() {
            if qualifies(d1) {
                found = Some((d1, d2));
                break 'outer;
            }
        }
    }
    let (d1, d2) = match found {
        Some(p) => p,
        None => {
            let any_q = (win_lo..today).any(qualifies);
            let gate = if any_q { Gate::Day2 } else { Gate::Day1 };
            return Outcome::NoTrade { gate, reason: "no qualifying 2-day setup (high < 18MA)".into() };
        }
    };
    let true_low = c[d1].low.min(c[d2].low);

    if d1 < cfg.consolidation_window {
        return Outcome::NoTrade { gate: Gate::Data, reason: "insufficient pre-Day1 history".into() };
    }
    let cs = d1 - cfg.consolidation_window;
    let win = &c[cs..d1];
    let hi = win.iter().map(|x| x.high).fold(f64::MIN, f64::max);
    let lo = win.iter().map(|x| x.low).fold(f64::MAX, f64::min);
    let meanc = win.iter().map(|x| x.close).sum::<f64>() / win.len() as f64;
    let range_pct = if meanc > 0.0 { (hi - lo) / meanc } else { f64::MAX };
    if range_pct > cfg.range_pct_threshold {
        return Outcome::NoTrade { gate: Gate::Consolidation, reason: format!("range {range_pct:.3} > {:.3}", cfg.range_pct_threshold) };
    }
    if cfg.near_structure_check {
        let sl = d1.saturating_sub(cfg.structure_lookback);
        let ll = c[sl..d1].iter().map(|x| x.low).fold(f64::MAX, f64::min);
        if ll > 0.0 && (lo - ll) / ll > cfg.proximity_pct {
            return Outcome::NoTrade { gate: Gate::Consolidation, reason: "not near support".into() };
        }
    }

    let obv_all = ind::obv(&c[..=d2]);
    let slope = ind::linreg_slope(&obv_all[cs..=d2]);
    if slope >= 0.0 {
        return Outcome::NoTrade { gate: Gate::Obv, reason: format!("OBV slope {slope:.2} not falling") };
    }

    if c[today].low >= true_low {
        return Outcome::NoTrade { gate: Gate::Break, reason: "no break below True Low".into() };
    }

    let entry = true_low;
    let stop = ma(today).unwrap_or(closes[today]);
    let r = stop - entry;
    if r <= 0.0 || cfg.target_r < cfg.min_rr {
        return Outcome::NoTrade { gate: Gate::RiskReward, reason: format!("RR {:.2} < {:.2} or non-positive risk", cfg.target_r, cfg.min_rr) };
    }
    let take_profit = entry - cfg.target_r * r;
    let strength = (1.0 - range_pct / cfg.range_pct_threshold).clamp(0.1, 1.0);
    Outcome::Enter(RawSignal { side: Side::Sell, price: entry, stop, take_profit: Some(take_profit), strength })
}

/// Structural stop — re-evaluated every daily bar. Returns true to EXIT.
/// Mirror: a daily candle fully on the opposite side of the 18MA.
/// Strict: a full opposite 2-qualifying-day setup has formed ending at the last bar.
pub fn structural_exit(side: Side, c: &[Candle], cfg: &Ma18Config) -> bool {
    let closes: Vec<f64> = c.iter().map(|x| x.close).collect();
    let i = c.len().saturating_sub(1);
    let ma = match sma_ending_at(&closes, i, cfg.ma_length) {
        Some(m) => m,
        None => return false,
    };
    match cfg.stop_mode {
        StopMode::Mirror => match side {
            Side::Buy => c[i].high < ma,  // long exits when a bar forms fully below the MA
            Side::Sell => c[i].low > ma,  // short exits when a bar forms fully above the MA
        },
        StopMode::Strict => {
            // two qualifying opposite days (consecutive) ending at i
            let opp = |j: usize| {
                sma_ending_at(&closes, j, cfg.ma_length).map_or(false, |m| match side {
                    Side::Buy => c[j].high < m,
                    Side::Sell => c[j].low > m,
                })
            };
            i >= 1 && opp(i) && opp(i - 1)
        }
    }
}

// ── Fundamental bias resolver (date-ranged file; defaults to blocked) ──────────

#[derive(Clone, Debug, serde::Deserialize, Default)]
pub struct BiasEntry {
    pub symbol: String,
    pub bias: String,       // "bullish" | "bearish" | "none"
    pub start_date: String, // inclusive, "YYYY-MM-DD"
    pub end_date: String,   // inclusive, "YYYY-MM-DD"
}

#[derive(Clone, Debug, Default)]
pub struct Fundamentals {
    entries: Vec<BiasEntry>,
}

impl Fundamentals {
    pub fn from_json(s: &str) -> Self {
        let entries: Vec<BiasEntry> = serde_json::from_str(s).unwrap_or_default();
        Self { entries }
    }
    /// Bias for `symbol` active on `date` (YYYY-MM-DD). Defaults to "none" (blocked).
    /// YYYY-MM-DD compares correctly as a lexicographic string.
    pub fn bias_on(&self, symbol: &str, date: &str) -> &'static str {
        for e in &self.entries {
            if e.symbol.eq_ignore_ascii_case(symbol)
                && date >= e.start_date.as_str()
                && date <= e.end_date.as_str()
            {
                return match e.bias.to_ascii_lowercase().as_str() {
                    "bullish" => "bullish",
                    "bearish" => "bearish",
                    _ => "none",
                };
            }
        }
        "none"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn candle(o: f64, h: f64, l: f64, cl: f64, v: f64) -> Candle {
        Candle { exchange: "test".into(), symbol: "BTCUSDT".into(), timeframe: "1d".into(),
            open: o, high: h, low: l, close: cl, volume: v, closed: true, ts: Utc::now() }
    }

    #[test]
    fn obv_accumulates_by_direction() {
        let cs = vec![candle(0.0, 0.0, 0.0, 10.0, 5.0), candle(0.0, 0.0, 0.0, 11.0, 7.0), candle(0.0, 0.0, 0.0, 10.0, 3.0), candle(0.0, 0.0, 0.0, 10.0, 9.0)];
        let o = ind::obv(&cs);
        assert_eq!(o, vec![0.0, 7.0, 4.0, 4.0]); // +7 up, -3 down, flat
    }

    #[test]
    fn linreg_slope_sign() {
        assert!(ind::linreg_slope(&[1.0, 2.0, 3.0, 4.0]) > 0.0);
        assert!(ind::linreg_slope(&[4.0, 3.0, 2.0, 1.0]) < 0.0);
        assert_eq!(ind::linreg_slope(&[5.0, 5.0, 5.0]), 0.0);
    }

    /// Build a series: a tight, rising-OBV consolidation, two qualifying days
    /// (low > 18MA), then a break above the True High → should ENTER long.
    fn long_setup() -> Vec<Candle> {
        let mut c = Vec::new();
        // 20 base days ~100 to seed the MA + satisfy the data guard (tight)
        for k in 0..20 {
            let p = 100.0 + (k as f64) * 0.05;
            c.push(candle(p, p + 0.5, p - 0.5, p, 100.0 + k as f64)); // rising volume → OBV up
        }
        // consolidation 10 days, tight, rising closes (OBV up), still ~101
        for k in 0..10 {
            let p = 101.0 + (k as f64) * 0.05;
            c.push(candle(p, p + 0.3, p - 0.3, p, 200.0 + k as f64));
        }
        // Day1 & Day2: low clearly above the 18MA (push price up so low > MA)
        c.push(candle(106.0, 107.0, 105.5, 106.5, 260.0)); // Day1
        c.push(candle(106.5, 108.0, 106.0, 107.5, 270.0)); // Day2  (True High = 108)
        // Break day: high > 108
        c.push(candle(108.0, 109.5, 107.8, 109.0, 280.0));
        c
    }

    #[test]
    fn long_enters_on_valid_break() {
        let c = long_setup();
        match evaluate_long(&c, &Ma18Config::default(), true) {
            Outcome::Enter(s) => {
                assert_eq!(s.side, Side::Buy);
                assert!((s.price - 108.0).abs() < 1e-6, "entry at True High");
                assert!(s.stop < s.price && s.take_profit.unwrap() > s.price);
            }
            Outcome::NoTrade { gate, reason } => panic!("expected entry, got {:?}: {}", gate, reason),
        }
    }

    #[test]
    fn fundamental_gate_blocks_first() {
        let c = long_setup();
        match evaluate_long(&c, &Ma18Config::default(), false) {
            Outcome::NoTrade { gate, .. } => assert_eq!(gate, Gate::Fundamental),
            _ => panic!("fundamental gate must block"),
        }
    }

    #[test]
    fn no_break_no_trade() {
        let mut c = long_setup();
        let last = c.len() - 1;
        // squash the break day below True High (108)
        c[last] = candle(107.0, 107.5, 106.5, 107.0, 280.0);
        match evaluate_long(&c, &Ma18Config::default(), true) {
            Outcome::NoTrade { gate, .. } => assert_eq!(gate, Gate::Break),
            _ => panic!("must not enter without a break"),
        }
    }

    #[test]
    fn loose_consolidation_rejected() {
        let mut c = long_setup();
        // widen one consolidation bar massively → range_pct over threshold
        c[20] = candle(101.0, 140.0, 70.0, 101.0, 205.0);
        match evaluate_long(&c, &Ma18Config::default(), true) {
            Outcome::NoTrade { gate, .. } => assert_eq!(gate, Gate::Consolidation),
            _ => panic!("loose consolidation must be rejected"),
        }
    }

    #[test]
    fn structural_mirror_exit_long() {
        let cfg = Ma18Config::default();
        let mut c = long_setup();
        // a bar fully below the MA should trigger exit for a long
        c.push(candle(90.0, 95.0, 88.0, 90.0, 100.0));
        assert!(structural_exit(Side::Buy, &c, &cfg));
    }

    #[test]
    fn fundamentals_date_ranged() {
        let f = Fundamentals::from_json(r#"[{"symbol":"BTCUSDT","bias":"bullish","start_date":"2024-01-01","end_date":"2024-06-30"}]"#);
        assert_eq!(f.bias_on("BTCUSDT", "2024-03-15"), "bullish");
        assert_eq!(f.bias_on("BTCUSDT", "2024-09-01"), "none"); // outside range → blocked
        assert_eq!(f.bias_on("ETHUSDT", "2024-03-15"), "none"); // unset → blocked
    }
}
