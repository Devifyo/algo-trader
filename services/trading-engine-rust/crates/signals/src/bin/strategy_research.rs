//! Strategy research harness — BACKTEST-ONLY, anti-overfitting by construction.
//!
//! This does NOT "search until positive". It runs a PRE-REGISTERED, bounded grid
//! of strategy families on IN-SAMPLE data, kills anything that fails strict gates
//! (net-of-cost expectancy, ≥50 trades, parameter stability, broad-based across
//! year & symbol), applies a multiple-testing correction (Bonferroni on N tested),
//! and evaluates the SINGLE best survivor exactly ONCE on a LOCKED out-of-sample
//! slice. If nothing survives it says so — that's the honest base rate. No live wiring.
//!
//!   cargo run -q -p signals --bin strategy_research -- [--days 1000]
//!
//! Universe: Alpaca long-tradeable crypto (live-relevant). Long-only. Uniform exit
//! (2*ATR stop / 4*ATR target / 40-bar time stop) so families are compared on entry
//! edge. Costs: tier1 retail 0.25%/side + adverse slippage.

use std::collections::BTreeMap;
use tcore::types::{Candle, Side};
use signals::indicators as ind;

const UNIVERSE: [&str; 7] = ["BTCUSDT", "ETHUSDT", "AVAXUSDT", "DOGEUSDT", "DOTUSDT", "LINKUSDT", "LTCUSDT"];
const IS_END: &str = "2025-06-30";
const OOS_START: &str = "2025-07-01";
const FEE: f64 = 0.0025;
const ESLIP: f64 = 0.0015;
const XSLIP: f64 = 0.0010;
const MIN_TRADES: usize = 50;
const MAX_HOLD: usize = 40;

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

/// Pre-registered families as (name, parameter axes). Configs = grid points.
fn families() -> Vec<(&'static str, Vec<Vec<f64>>)> {
    vec![
        ("ma_cross",  vec![vec![10., 20., 50.], vec![50., 100., 150.]]), // fast, slow (fast<slow)
        ("donchian",  vec![vec![20., 40., 55.]]),                        // breakout N
        ("rsi_mr",    vec![vec![2., 14.], vec![10., 30.]]),              // period, oversold thresh
        ("momentum",  vec![vec![20., 60., 120.]]),                       // lookback L
        ("vol_break", vec![vec![1.0, 2.0], vec![14., 20.]]),             // k, atr period
    ]
}

#[derive(Clone)]
struct Cfg { fam: usize, idx: Vec<usize>, p: Vec<f64> }
impl Cfg {
    fn label(&self, fams: &[(&str, Vec<Vec<f64>>)]) -> String {
        let pv: Vec<String> = self.p.iter().map(|v| format!("{}", *v as i64)).collect();
        format!("{}({})", fams[self.fam].0, pv.join(","))
    }
}

fn all_configs(fams: &[(&str, Vec<Vec<f64>>)]) -> Vec<Cfg> {
    let mut out = Vec::new();
    for (fi, (name, axes)) in fams.iter().enumerate() {
        // cartesian product of axis indices
        let mut combos: Vec<Vec<usize>> = vec![vec![]];
        for axis in axes {
            let mut next = Vec::new();
            for c in &combos { for j in 0..axis.len() { let mut v = c.clone(); v.push(j); next.push(v); } }
            combos = next;
        }
        for idx in combos {
            let p: Vec<f64> = idx.iter().enumerate().map(|(a, &j)| axes[a][j]).collect();
            if *name == "ma_cross" && p[0] >= p[1] { continue; } // need fast < slow
            out.push(Cfg { fam: fi, idx, p });
        }
    }
    out
}

/// Long entry signal for a family at bar i (uses history c[..=i]).
fn entry_long(name: &str, p: &[f64], c: &[Candle], i: usize) -> bool {
    let closes: Vec<f64> = c[..=i].iter().map(|x| x.close).collect();
    match name {
        "ma_cross" => {
            let (f, s) = (p[0] as usize, p[1] as usize);
            if i < s + 1 { return false; }
            let ff = ind::ema(&closes, f); let ss = ind::ema(&closes, s);
            let pf = ind::ema(&closes[..closes.len() - 1], f); let ps = ind::ema(&closes[..closes.len() - 1], s);
            matches!((ff, ss, pf, ps), (Some(a), Some(b), Some(c2), Some(d)) if a > b && c2 <= d)
        }
        "donchian" => {
            let n = p[0] as usize;
            if i < n + 1 { return false; }
            let hh = c[i - n..i].iter().map(|x| x.high).fold(f64::MIN, f64::max);
            c[i].close > hh
        }
        "rsi_mr" => {
            let (per, th) = (p[0] as usize, p[1]);
            ind::rsi(&closes, per).map_or(false, |r| r < th)
        }
        "momentum" => {
            let l = p[0] as usize;
            i >= l && c[i].close > c[i - l].close
        }
        "vol_break" => {
            let (k, per) = (p[0], p[1] as usize);
            if i < per + 1 { return false; }
            ind::atr(&c[..=i], per).map_or(false, |a| c[i].close > c[i - 1].close + k * a)
        }
        _ => false,
    }
}

struct TradeR { r: f64, year: String, symbol: String }

/// Backtest one config over the universe, entering only on bars whose date is in
/// [lo, hi]. Long-only, uniform 2*ATR stop / 4*ATR target / time stop, net of costs.
fn backtest(cfg: &Cfg, fams: &[(&str, Vec<Vec<f64>>)], data: &[(String, Vec<Candle>)], lo: &str, hi: &str) -> Vec<TradeR> {
    let name = fams[cfg.fam].0;
    let mut trades = Vec::new();
    for (sym, c) in data {
        let mut i = 30usize;
        while i < c.len() {
            let d = date_of(&c[i]);
            if d.as_str() < lo || d.as_str() > hi { i += 1; continue; }
            if !entry_long(name, &cfg.p, c, i) { i += 1; continue; }
            let atr = match ind::atr(&c[..=i], 14) { Some(a) if a > 0.0 => a, _ => { i += 1; continue; } };
            let entry = c[i].close;
            let stop = entry - 2.0 * atr;
            let target = entry + 4.0 * atr;
            let stop_dist = 2.0 * atr;
            // walk forward to exit
            let mut j = i + 1;
            let raw_exit = loop {
                if j >= c.len() { break c[c.len() - 1].close; }
                if c[j].low <= stop { break stop; }
                if c[j].high >= target { break target; }
                if j - i >= MAX_HOLD { break c[j].close; }
                j += 1;
            };
            let entry_fill = entry * (1.0 + ESLIP);
            let exit_fill = raw_exit * (1.0 - XSLIP);
            let fees = FEE * (entry_fill + exit_fill);
            let r = (exit_fill - entry_fill - fees) / stop_dist;
            trades.push(TradeR { r, year: d[..4].to_string(), symbol: sym.clone() });
            i = j + 1; // flat until the trade closes
        }
    }
    trades
}

fn phi(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989423 * (-x * x / 2.0).exp();
    let p = d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));
    if x > 0.0 { 1.0 - p } else { p }
}

struct Metrics { n: usize, total: f64, avg: f64, win: f64, sharpe: f64, t: f64, raw_p: f64,
    pos_years: usize, top_share: f64, by_year: BTreeMap<String, f64>, by_sym: BTreeMap<String, f64> }

fn metrics(tr: &[TradeR]) -> Metrics {
    let n = tr.len();
    let total: f64 = tr.iter().map(|t| t.r).sum();
    if n == 0 { return Metrics { n, total: 0.0, avg: 0.0, win: 0.0, sharpe: 0.0, t: 0.0, raw_p: 1.0, pos_years: 0, top_share: 1.0, by_year: BTreeMap::new(), by_sym: BTreeMap::new() }; }
    let avg = total / n as f64;
    let win = tr.iter().filter(|t| t.r > 0.0).count() as f64 / n as f64 * 100.0;
    let var = tr.iter().map(|t| (t.r - avg).powi(2)).sum::<f64>() / (n.max(2) - 1) as f64;
    let sd = var.sqrt();
    let sharpe = if sd > 0.0 { avg / sd } else { 0.0 };
    let t = if sd > 0.0 { avg / sd * (n as f64).sqrt() } else { 0.0 };
    let raw_p = 1.0 - phi(t); // one-sided H0: no edge
    let mut by_year: BTreeMap<String, f64> = BTreeMap::new();
    let mut by_sym: BTreeMap<String, f64> = BTreeMap::new();
    for x in tr { *by_year.entry(x.year.clone()).or_default() += x.r; *by_sym.entry(x.symbol.clone()).or_default() += x.r; }
    let pos_years = by_year.values().filter(|v| **v > 0.0).count();
    let top_share = if total > 0.0 { by_sym.values().cloned().fold(f64::MIN, f64::max) / total } else { 1.0 };
    Metrics { n, total, avg, win, sharpe, t, raw_p, pos_years, top_share, by_year, by_sym }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: usize = args.windows(2).find(|w| w[0] == "--days").and_then(|w| w[1].parse().ok()).unwrap_or(1000);

    let mut data = Vec::new();
    for s in UNIVERSE { let c = fetch_daily(s, days).await; if c.len() >= 200 { data.push((s.to_string(), c)); } }
    let dstart = data.first().map(|(_, c)| date_of(&c[0])).unwrap_or_default();
    let dend = data.first().map(|(_, c)| date_of(c.last().unwrap())).unwrap_or_default();

    let fams = families();
    let configs = all_configs(&fams);
    let n_tested = configs.len();
    let bonf = 0.05 / n_tested as f64;

    println!("\n=== Strategy Research Harness (BACKTEST-ONLY · anti-overfitting) ===");
    println!("Universe (Alpaca long-tradeable): {}", UNIVERSE.join(","));
    println!("Data: {dstart} .. {dend}  |  IN-SAMPLE: {dstart}..{IS_END}  OUT-OF-SAMPLE (LOCKED): {OOS_START}..{dend}");
    println!("Costs: {:.2}%/side + slip {:.2}/{:.2}% | Exit: 2*ATR stop / 4*ATR target / {}-bar | Long-only", FEE * 100.0, ESLIP * 100.0, XSLIP * 100.0, MAX_HOLD);
    println!("Pre-registered configs tested (N): {n_tested}  →  Bonferroni significance threshold: p < {bonf:.5}");
    println!("\n  HARD RULES: success = honesty of the conclusion (not a positive). Each held-out slice used");
    println!("  at most ONCE. No phase skipped. No live-money wiring. If a gate would have to be loosened or");
    println!("  spent data re-touched to advance → STOP at the evidence ceiling instead.\n");

    // ── In-sample metrics for every config ──
    let is_metrics: Vec<Metrics> = configs.iter().map(|c| metrics(&backtest(c, &fams, &data, &dstart, IS_END))).collect();

    // family positivity (for the stability gate proxy: ≥half of same-family grid net-positive)
    let mut fam_pos: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
    for (c, m) in configs.iter().zip(&is_metrics) {
        let e = fam_pos.entry(c.fam).or_insert((0, 0)); e.1 += 1; if m.total > 0.0 { e.0 += 1; }
    }
    let fam_stable = |fam: usize| { let (p, t) = fam_pos[&fam]; t > 0 && p * 2 >= t };

    // ── Gates ──
    println!("  -- In-sample configs (gates: P=net-positive N=≥{MIN_TRADES} S=param-stable B=broad-based) --");
    println!("     {:<22} {:>4} {:>7} {:>5} {:>6} {:>7} {:>8}  gates  survive", "config", "n", "totR", "avgR", "win%", "rawP", "adjP");
    let mut survivors: Vec<(usize, f64)> = Vec::new(); // (config idx, adj_p)
    let mut funnel = BTreeMap::new();
    for (i, (c, m)) in configs.iter().zip(&is_metrics).enumerate() {
        let g_p = m.total > 0.0;
        let g_n = m.n >= MIN_TRADES;
        let g_s = fam_stable(c.fam) && m.total > 0.0;
        let g_b = m.pos_years >= 2 && m.top_share <= 0.5;
        let adj_p = (m.raw_p * n_tested as f64).min(1.0);
        let pass = g_p && g_n && g_s && g_b && adj_p < 0.05;
        let gates = format!("{}{}{}{}", if g_p {"P"} else {"·"}, if g_n {"N"} else {"·"}, if g_s {"S"} else {"·"}, if g_b {"B"} else {"·"});
        if !g_p { *funnel.entry("positive").or_insert(0) += 1; }
        else if !g_n { *funnel.entry("sample≥50").or_insert(0) += 1; }
        else if !g_s { *funnel.entry("param-stable").or_insert(0) += 1; }
        else if !g_b { *funnel.entry("broad-based").or_insert(0) += 1; }
        else if adj_p >= 0.05 { *funnel.entry("significance").or_insert(0) += 1; }
        else { *funnel.entry("SURVIVED").or_insert(0) += 1; survivors.push((i, adj_p)); }
        println!("     {:<22} {:>4} {:>7.2} {:>5.2} {:>6.1} {:>7.4} {:>8.4}  {:<5}  {}",
            c.label(&fams), m.n, m.total, m.avg, m.win, m.raw_p, adj_p, gates, if pass {"✓"} else {""});
    }

    println!("\n  -- Gate funnel (where configs died) --");
    for g in ["positive", "sample≥50", "param-stable", "broad-based", "significance", "SURVIVED"] {
        println!("     {:<14} {}", g, funnel.get(g).copied().unwrap_or(0));
    }

    // ═════════════ STAGED PROMOTION PIPELINE ═════════════
    let mut phase_reached = 0;
    let mut terminal: String;

    println!("\n  ===== PHASE 0 — Discovery (in-sample {dstart}..{IS_END}) =====");
    if survivors.is_empty() {
        println!("     Result: NONE SURVIVED ({n_tested} configs tested; all gates applied, none loosened).");
        println!("     → STOP. OOS slice NOT accessed (nothing earned the right). No gate weakened, no data re-touched.");
        terminal = "No in-sample candidate. Best honest phase: PHASE 0 — keep researching; nothing to promote.".into();
    } else {
        println!("     Result: {} survivor(s) cleared all gates + multiple-testing correction.", survivors.len());
        phase_reached = 1;
        survivors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // best = lowest adjusted p
        let (best_i, best_adjp) = survivors[0];
        let best = &configs[best_i];

        println!("\n  ===== PHASE 1 — Out-of-sample (LOCKED {OOS_START}..{dend}, ONE shot) =====");
        println!("     >>> OOS ACCESSED ONCE — single best survivor only: {} (adjP={:.4})", best.label(&fams), best_adjp);
        let om = metrics(&backtest(best, &fams, &data, OOS_START, &dend));
        let holds = om.total > 0.0 && om.avg > 0.0;
        println!("     OOS: n={} totalR={:.2} avgR={:.2} win%={:.1} | by year {:?}", om.n, om.total, om.avg, om.win,
            om.by_year.iter().map(|(y, r)| format!("{y}:{r:.1}")).collect::<Vec<_>>());
        if !holds {
            println!("     → FAIL. STOP. OOS slice is now SPENT for this candidate — no variant may be retested against it.");
            terminal = "Failed out-of-sample — candidate discarded. Best honest phase: PHASE 1.".into();
        } else {
            phase_reached = 2;
            println!("     → PASS (positive out-of-sample).");
            println!("\n  ===== PHASE 2 — Forward paper-trading (handoff; NOT wired here) =====");
            println!("     Candidate {} earned forward validation. HANDOFF (manual, paper-only):", best.label(&fams));
            println!("       • run this config on the LIVE feed in PAPER mode (simulated fills + the cost model),");
            println!("       • accumulate ≥3 months of forward record before ANY live-money discussion.");
            println!("     This loop does NOT auto-wire it (no phase skipped; no live-money execution, ever).");
            terminal = format!("Reached PHASE 2 — forward paper validation queued for {}. Needs months of \
                live-but-simulated data before any live-money discussion.", best.label(&fams));
        }
    }

    println!("\n  ===== PIPELINE CONCLUSION =====");
    println!("     Phase reached: {phase_reached} of 2.");
    println!("     Best honest phase achieved: {terminal}");
    if phase_reached == 0 {
        println!("     'NONE SURVIVED' is a valid, successful terminal output — the guardrails protected you from");
        println!("     promoting noise. Real edges are rare; an honest first pass usually returns nothing.");
    }
    println!();
}
