//! Daily backtester for the 18MA Qualified Breakout strategy. Reuses the exact
//! `signals::ma18` pipeline (no logic drift vs. future live code).
//!
//! Cost model is a per-broker PROFILE (not Alpaca-specific):
//!   BrokerProfile { fee_per_side, maker_fee, taker_fee, shortable, symbols, min_order, entry_slip, exit_slip }
//! Market orders pay the per-side fee (taker) on both legs + adverse slippage.
//!
//!   cargo run -q -p signals --bin ma18_backtest -- [--symbols ...] [--days 1000] [--profile FILE]

use std::collections::{BTreeMap, HashMap};
use tcore::types::{Candle, Side};
use signals::ma18::{self, Gate, Ma18Config, Outcome};

const GATES: [Gate; 8] = [Gate::Data, Gate::Fundamental, Gate::Day1, Gate::Day2, Gate::Consolidation, Gate::Obv, Gate::Break, Gate::RiskReward];

#[derive(Clone, serde::Deserialize)]
struct BrokerProfile {
    name: String,
    fee_per_side: f64,      // effective per-side fee for our MARKET orders (= taker)
    maker_fee: f64,
    taker_fee: f64,
    shortable: bool,        // may we open shorts on this venue?
    symbols: Vec<String>,   // tradeable bases, e.g. "BTC" (without USDT)
    min_order: f64,         // min notional USD
    entry_slip: f64,        // adverse breakout-fill slippage
    exit_slip: f64,         // adverse exit slippage
}

impl BrokerProfile {
    /// Tier-1 retail spot (e.g. Alpaca crypto): 0.25%/side, long-only, $1 min.
    fn tier1_retail() -> Self {
        Self {
            name: "tier1_retail".into(),
            fee_per_side: 0.0025, maker_fee: 0.0015, taker_fee: 0.0025,
            shortable: false,
            symbols: ["BTC", "ETH", "AVAX", "DOGE", "DOT", "LINK", "LTC"].iter().map(|s| s.to_string()).collect(),
            min_order: 1.0, entry_slip: 0.0015, exit_slip: 0.0010,
        }
    }
    fn with_fee(name: &str, fee: f64) -> Self {
        let mut p = Self::tier1_retail();
        p.name = name.into();
        p.fee_per_side = fee;
        p.taker_fee = fee;
        p.maker_fee = (fee - 0.001).max(0.0);
        p
    }
    fn tradeable(&self, symbol: &str) -> bool {
        self.symbols.iter().any(|s| s.eq_ignore_ascii_case(symbol.trim_end_matches("USDT")))
    }
}

struct Trade { side: Side, gross_r: f64, net_r: f64, date: String, entry_date: String, symbol: String }

#[derive(Default)]
struct Funnel { stop: BTreeMap<&'static str, u64>, enters: u64 }
impl Funnel {
    fn record(&mut self, o: &Outcome) {
        match o {
            Outcome::Enter(_) => self.enters += 1,
            Outcome::NoTrade { gate, .. } => *self.stop.entry(gate.label()).or_default() += 1,
        }
    }
    fn print(&self, title: &str) {
        println!("  -- {title} --");
        for g in GATES { println!("     {:<14} {}", g.label(), self.stop.get(g.label()).copied().unwrap_or(0)); }
        println!("     {:<14} {}", "ENTER", self.enters);
    }
}

async fn fetch_daily(symbol: &str, days: usize) -> Vec<Candle> {
    let url = format!("https://api.binance.com/api/v3/klines?symbol={symbol}&interval=1d&limit={days}");
    let rows: Vec<Vec<serde_json::Value>> = match reqwest::Client::new().get(&url).send().await {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => return Vec::new(),
    };
    rows.iter().map(|r| {
        let g = |i: usize| r.get(i).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        let close_ms = r.get(6).and_then(|v| v.as_i64()).unwrap_or(0);
        Candle { exchange: "binance".into(), symbol: symbol.into(), timeframe: "1d".into(),
            open: g(1), high: g(2), low: g(3), close: g(4), volume: g(5), closed: true,
            ts: chrono::DateTime::from_timestamp_millis(close_ms).unwrap_or_else(chrono::Utc::now) }
    }).collect()
}

fn date_of(c: &Candle) -> String { c.ts.format("%Y-%m-%d").to_string() }

fn net_r(side: Side, entry: f64, raw_exit: f64, r_planned: f64, p: &BrokerProfile) -> f64 {
    if r_planned <= 0.0 { return 0.0; }
    let (entry_fill, exit_fill) = match side {
        Side::Buy => (entry * (1.0 + p.entry_slip), raw_exit * (1.0 - p.exit_slip)),
        Side::Sell => (entry * (1.0 - p.entry_slip), raw_exit * (1.0 + p.exit_slip)),
    };
    let fees = p.fee_per_side * (entry_fill + exit_fill);
    let pl = match side { Side::Buy => exit_fill - entry_fill, Side::Sell => entry_fill - exit_fill };
    (pl - fees) / r_planned
}

/// Simulate over pre-fetched data for one config + profile. Shorts are ALWAYS
/// simulated for measurement, but flagged paper-only/gated by `shortable` in the
/// report (no live wiring here). Returns trades (gross+net R) + funnels.
fn run_sim(data: &[(String, Vec<Candle>)], cfg: &Ma18Config, p: &BrokerProfile, assume_pass: bool,
           bias: Option<&ma18::Fundamentals>) -> (Vec<Trade>, Funnel, Funnel) {
    let mut trades = Vec::new();
    let (mut lf, mut sf) = (Funnel::default(), Funnel::default());
    let start = cfg.ma_length + cfg.consolidation_window + cfg.entry_validity_days + 2;
    for (sym, c) in data {
        if c.len() < start + 5 { continue; }
        let mut pos: Option<(Side, f64, f64, f64, f64, String)> = None;
        for i in start..c.len() {
            let sub = &c[..=i];
            let day = &c[i];
            if let Some((side, entry, init_stop, target, hard, entry_date)) = pos.clone() {
                let r_planned = match side { Side::Buy => entry - init_stop, Side::Sell => init_stop - entry };
                let raw_exit = match side {
                    Side::Buy => {
                        if cfg.use_hard_stop && day.low <= hard { Some(hard) }
                        else if day.high >= target { Some(target) }
                        else if ma18::structural_exit(Side::Buy, sub, cfg) { Some(day.close) } else { None }
                    }
                    Side::Sell => {
                        if cfg.use_hard_stop && day.high >= hard { Some(hard) }
                        else if day.low <= target { Some(target) }
                        else if ma18::structural_exit(Side::Sell, sub, cfg) { Some(day.close) } else { None }
                    }
                };
                if let Some(ex) = raw_exit {
                    let gross = match side { Side::Buy => (ex - entry) / r_planned, Side::Sell => (entry - ex) / r_planned };
                    trades.push(Trade { side, gross_r: gross, net_r: net_r(side, entry, ex, r_planned, p),
                        date: date_of(day), entry_date, symbol: sym.clone() });
                    pos = None;
                }
                continue;
            }
            let d = date_of(day);
            let f_long = assume_pass || bias.map_or(false, |b| b.bias_on(sym, &d) == "bullish");
            let f_short = assume_pass || bias.map_or(false, |b| b.bias_on(sym, &d) == "bearish");
            let ol = ma18::evaluate_long(sub, cfg, f_long);
            lf.record(&ol);
            let os = ma18::evaluate_short(sub, cfg, f_short);
            sf.record(&os);
            if let Outcome::Enter(s) = ol {
                pos = Some((Side::Buy, s.price, s.stop, s.take_profit.unwrap_or(s.price), s.price * (1.0 - cfg.hard_stop_pct), d.clone()));
            } else if let Outcome::Enter(s) = os {
                pos = Some((Side::Sell, s.price, s.stop, s.take_profit.unwrap_or(s.price), s.price * (1.0 + cfg.hard_stop_pct), d.clone()));
            }
        }
    }
    (trades, lf, sf)
}

fn summarize(rs: &[f64]) -> (usize, f64, f64, f64, f64) {
    let n = rs.len();
    if n == 0 { return (0, 0.0, 0.0, 0.0, 0.0); }
    let wins = rs.iter().filter(|r| **r > 0.0).count();
    let total: f64 = rs.iter().sum();
    let (mut cum, mut peak, mut dd) = (0.0f64, 0.0f64, 0.0f64);
    for r in rs { cum += r; peak = peak.max(cum); dd = dd.max(peak - cum); }
    (n, wins as f64 / n as f64 * 100.0, total / n as f64, total, dd)
}

fn side_rs(trades: &mut Vec<Trade>, side: Side, net: bool) -> Vec<f64> {
    trades.sort_by(|a, b| a.date.cmp(&b.date));
    trades.iter().filter(|t| t.side == side).map(|t| if net { t.net_r } else { t.gross_r }).collect()
}

/// date → true if BTC's daily close was above its own 50-day SMA that day.
fn btc_regime_map(data: &[(String, Vec<Candle>)]) -> HashMap<String, bool> {
    let mut m = HashMap::new();
    if let Some((_, c)) = data.iter().find(|(s, _)| s.eq_ignore_ascii_case("BTCUSDT")) {
        let closes: Vec<f64> = c.iter().map(|x| x.close).collect();
        for i in 49..c.len() {
            let ma = closes[i - 49..=i].iter().sum::<f64>() / 50.0;
            m.insert(date_of(&c[i]), c[i].close > ma);
        }
    }
    m
}

fn stability(trades: &mut [Trade], regime: &HashMap<String, bool>) {
    let mut longs: Vec<&Trade> = trades.iter().filter(|t| t.side == Side::Buy).collect();
    longs.sort_by(|a, b| a.entry_date.cmp(&b.entry_date));

    let (n, wr, ar, tr, dd) = summarize(&longs.iter().map(|t| t.net_r).collect::<Vec<_>>());
    println!("  Overall (long-only net): n={n} win%={wr:.1} avgR={ar:.2} totalR={tr:.2} maxDD={dd:.2}R\n");

    println!("  -- By calendar year (entry date) --");
    println!("     {:<12} n    win%   avgR   totalR   maxDD(R)", "year");
    for y in ["2023", "2024", "2025", "2026"] {
        let rs: Vec<f64> = longs.iter().filter(|t| t.entry_date.starts_with(y)).map(|t| t.net_r).collect();
        let (n, wr, ar, tr, dd) = summarize(&rs);
        let label = match y { "2023" => "2023(part)", "2026" => "2026 YTD", _ => y };
        println!("     {:<12} {:>3}  {:>5.1}  {:>5.2}  {:>6.2}  {:>7.2}", label, n, wr, ar, tr, dd);
    }

    println!("\n  -- By BTC regime at entry (BTC close vs its 50-day SMA on the entry day) --");
    for (lab, want) in [("BTC > 50MA (risk-on)", true), ("BTC < 50MA (risk-off)", false)] {
        let rs: Vec<f64> = longs.iter().filter(|t| regime.get(&t.entry_date).copied() == Some(want)).map(|t| t.net_r).collect();
        let (n, wr, ar, tr, dd) = summarize(&rs);
        println!("     {:<22} n={:>3} win%={:>5.1} avgR={:>5.2} totalR={:>6.2} maxDD={:>5.2}R", lab, n, wr, ar, tr, dd);
    }
    let unknown = longs.iter().filter(|t| regime.get(&t.entry_date).is_none()).count();
    if unknown > 0 { println!("     ({unknown} trades pre-50MA history — excluded from regime split)"); }

    println!("\n  -- Symbol concentration (long-only total net R) --");
    let mut by: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for t in &longs { let e = by.entry(t.symbol.clone()).or_insert((0.0, 0)); e.0 += t.net_r; e.1 += 1; }
    let mut v: Vec<(String, f64, usize)> = by.into_iter().map(|(s, (r, n))| (s, r, n)).collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (s, r, n) in &v { println!("     {:<9} totalR={:>6.2}  ({n} trades)", s, r); }
    let total: f64 = v.iter().map(|x| x.1).sum();
    println!("     total net R:            {:>6.2}", total);
    if !v.is_empty() { println!("     − top1 ({:<5}):        {:>6.2}", v[0].0, total - v[0].1); }
    if v.len() > 1 { println!("     − top2 ({}+{}): {:>6.2}", v[0].0, v[1].0, total - v[0].1 - v[1].1); }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |f: &str| args.windows(2).find(|w| w[0] == f).map(|w| w[1].clone());
    let symbols: Vec<String> = arg("--symbols").unwrap_or_else(||
        "BTCUSDT,ETHUSDT,SOLUSDT,BNBUSDT,XRPUSDT,DOGEUSDT,AVAXUSDT,LINKUSDT,ADAUSDT,DOTUSDT,LTCUSDT,ATOMUSDT".into())
        .split(',').map(|s| s.trim().to_string()).collect();
    let days: usize = arg("--days").and_then(|d| d.parse().ok()).unwrap_or(1000);
    let bias = arg("--bias").and_then(|p| std::fs::read_to_string(&p).ok()).map(|s| ma18::Fundamentals::from_json(&s));
    let assume_pass = bias.is_none();
    let loaded = arg("--profile").and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str::<BrokerProfile>(&s).ok());

    // Cost bracket (slippage held constant; only the per-side fee varies).
    let low = BrokerProfile::with_fee("low_cost(0.10%)", 0.0010);
    let mid = loaded.clone().unwrap_or_else(BrokerProfile::tier1_retail); // default Tier-1 retail (0.25%)
    let high = BrokerProfile::with_fee("high_cost(0.40%)", 0.0040);

    let mut data: Vec<(String, Vec<Candle>)> = Vec::new();
    for s in &symbols { let c = fetch_daily(s, days).await; if c.len() >= 60 { data.push((s.clone(), c)); } }
    let period = data.first().map(|(_, c)| (date_of(&c[0]), date_of(c.last().unwrap()))).unwrap_or_default();

    // Temporal-stability breakdown only (default config + mid profile, long-only net).
    if args.iter().any(|a| a == "--stability") {
        let cfg = Ma18Config::default();
        let (mut t, _, _) = run_sim(&data, &cfg, &mid, assume_pass, bias.as_ref());
        let reg = btc_regime_map(&data);
        println!("\n=== Temporal-stability — long-only NET @ {} ({:.2}%/side) ===", mid.name, mid.fee_per_side * 100.0);
        println!("Period {} .. {} | regime rule: BTC daily close vs its own 50-day SMA on the trade's ENTRY day.\n", period.0, period.1);
        stability(&mut t, &reg);
        return;
    }

    println!("\n=== 18MA Qualified Breakout — Daily Backtest (per-broker cost profiles) ===");
    println!("Symbols: {} | Period: {} .. {}", symbols.join(","), period.0, period.1);
    println!("Slippage (all profiles): entry {:.2}% / exit {:.2}% adverse | Fundamental gate: {}\n",
        mid.entry_slip * 100.0, mid.exit_slip * 100.0, if assume_pass { "ASSUME-PASS" } else { "bias file" });

    // Funnel from the default/mid run.
    let cfg = Ma18Config::default();
    let (_t0, lf, sf) = run_sim(&data, &cfg, &mid, assume_pass, bias.as_ref());
    lf.print("LONG entry funnel"); sf.print("SHORT entry funnel");

    // ── Cost bracket: long-only NET per profile ──
    println!("\n  -- LONG-ONLY net of costs, across broker profiles --");
    println!("     {:<20} fee/side   n   win%   avgR   totalR   maxDD(R)", "profile");
    let brackets = [&low, &mid, &high];
    let mut pts: Vec<(f64, f64)> = Vec::new(); // (fee, avgR) for breakeven fit
    for p in brackets {
        let (mut t, _, _) = run_sim(&data, &cfg, p, assume_pass, bias.as_ref());
        let (n, wr, ar, tr, dd) = summarize(&side_rs(&mut t, Side::Buy, true));
        pts.push((p.fee_per_side, ar));
        println!("     {:<20} {:>6.2}%  {:>3}  {:>5.1}  {:>5.2}  {:>6.2}  {:>7.2}", p.name, p.fee_per_side * 100.0, n, wr, ar, tr, dd);
    }
    // Breakeven fee/side where long-only avg R → 0 (linear fit on the bracket).
    let (f0, a0) = pts[0]; let (f2, a2) = pts[2];
    let slope = (a2 - a0) / (f2 - f0);
    let breakeven = if slope.abs() > 1e-9 { f0 - a0 / slope } else { f64::NAN };
    println!("     → approx breakeven fee/side (avgR=0): {:.2}%/side", breakeven * 100.0);

    // ── Short side: separate, paper-only, flagged ──
    let (mut ts, _, _) = run_sim(&data, &cfg, &mid, assume_pass, bias.as_ref());
    let (sn, swr, sar, str_, sdd) = summarize(&side_rs(&mut ts, Side::Sell, true));
    println!("\n  -- SHORT-ONLY net (mid profile) — PAPER-ONLY --");
    println!("     n={} win%={:.1} avgR={:.2} totalR={:.2} maxDD(R)={:.2}", sn, swr, sar, str_, sdd);
    println!("     ⚠ shorts gated by profile.shortable (mid='{}'.shortable={}); kept PAPER-ONLY regardless.", mid.name, mid.shortable);
    if sn < 30 { println!("     ⚠ SMALL SAMPLE (n={sn} < 30) — not statistically reliable; do not act on these."); }

    // ── Robustness sweep at MID profile (long-only avg net R / n) ──
    println!("\n  -- Robustness sweep @ MID profile '{}' ({:.2}%/side) — long-only avg net R / n --", mid.name, mid.fee_per_side * 100.0);
    let (ranges, mas, windows) = ([0.06_f64, 0.08, 0.10], [16usize, 18, 20], [8usize, 10, 12]);
    for ma in mas {
        println!("\n   ma_length = {ma}");
        print!("     {:<10}", ""); for r in ranges { print!("  range={:.2}   ", r); } println!();
        for w in windows {
            print!("     window={:<3}", w);
            for r in ranges {
                let mut cc = Ma18Config::default();
                cc.ma_length = ma; cc.consolidation_window = w; cc.range_pct_threshold = r;
                let (mut t, _, _) = run_sim(&data, &cc, &mid, assume_pass, bias.as_ref());
                let (n, _w, ar, _tr, _dd) = summarize(&side_rs(&mut t, Side::Buy, true));
                print!("  {:>6.2}/{:<4} ", ar, n);
            }
            println!();
        }
    }
    println!("\n  Tradeable (long) under mid profile '{}': {:?}", mid.name, mid.symbols);
    println!("  (cell = avg net R / trade count. No live wiring; no fundamentals filter.)\n");
}
