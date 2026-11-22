//! Opportunity Scanner. Market-wide (tenant-agnostic): maintains a rolling
//! window per symbol and, on each closed candle, scores that market from live
//! price action (trend, momentum, volatility, RSI) blended with news sentiment.
//! It then derives the market regime and a dynamic capital allocation toward the
//! best risk-adjusted opportunities. All output is real, computed from live data
//! — never static. Consumers: the Markets dashboard + portfolio allocator.

use crate::indicators as ind;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tcore::bus::decode;
use tcore::store::MarketScore;
use tcore::types::Candle;
use tcore::{subjects, Context, Result};

const WIN: usize = 120;
const MIN_BARS: usize = 30;

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    tracing::info!("opportunity scanner starting");
    let mut windows: HashMap<String, VecDeque<Candle>> = HashMap::new();
    let mut latest: HashMap<String, MarketScore> = HashMap::new();

    let mut sub = ctx.bus.subscribe(subjects::MARKET_CANDLE_ALL).await?;
    while let Some(msg) = sub.next().await {
        let candle: Candle = match decode::<Candle>(&msg.payload) {
            Some(c) if c.closed => c,
            _ => continue,
        };
        let win = windows.entry(candle.symbol.clone()).or_default();
        win.push_back(candle.clone());
        while win.len() > WIN {
            win.pop_front();
        }
        if win.len() < MIN_BARS {
            continue;
        }
        if let Some(score) = compute(ctx.as_ref(), &candle.symbol, win).await {
            if let Err(e) = ctx.store.upsert_market_score(&score).await {
                tracing::warn!(error = %e, "scanner: upsert score failed");
            }
            latest.insert(score.symbol.clone(), score);
            recompute(ctx.as_ref(), &latest).await;
        }
    }
    Ok(())
}

/// Score a single market from its window + news sentiment.
async fn compute(ctx: &Context, symbol: &str, win: &VecDeque<Candle>) -> Option<MarketScore> {
    let candles: Vec<Candle> = win.iter().cloned().collect();
    let closes = ind::closes(&candles);
    let price = *closes.last()?;
    if price <= 0.0 {
        return None;
    }

    let ema_fast = ind::ema(&closes, 12)?;
    let ema_slow = ind::ema(&closes, 26)?;
    let trend = (((ema_fast - ema_slow) / ema_slow) * 100.0 * 15.0).clamp(-100.0, 100.0);

    let lb = 14usize;
    let momentum = if closes.len() > lb {
        let past = closes[closes.len() - 1 - lb];
        if past > 0.0 { ((price - past) / past * 100.0).clamp(-50.0, 50.0) } else { 0.0 }
    } else {
        0.0
    };

    let atr = ind::atr(&candles, 14).unwrap_or(0.0);
    let volatility = atr / price * 100.0;
    let rsi = ind::rsi(&closes, 14).unwrap_or(50.0);

    // News sentiment (score -100..100, confidence 0..100) → normalize to -1..1 / 0..1.
    let coin = symbol.trim_end_matches("USDT");
    let (sent, conf) = ctx
        .store
        .sentiment(coin)
        .await
        .ok()
        .flatten()
        .map(|(s, c)| (s / 100.0, c / 100.0))
        .unwrap_or((0.0, 0.0));

    // Directional conviction in [-1, 1] from blended factors.
    let dir = 0.45 * (trend / 40.0).tanh()
        + 0.35 * (momentum / 15.0).tanh()
        + 0.20 * ((rsi - 50.0) / 50.0)
        + 0.20 * sent;
    let dir = dir.clamp(-1.0, 1.0);

    let bullish = (50.0 + dir * 50.0).clamp(0.0, 100.0);
    let bearish = 100.0 - bullish;
    let confidence = (dir.abs() * 70.0 + conf * 30.0).clamp(0.0, 100.0);
    let risk = (volatility * 8.0 + if !(30.0..=70.0).contains(&rsi) { 20.0 } else { 0.0 }).clamp(0.0, 100.0);
    let market_score = (0.5 * bullish.max(bearish) + 0.5 * confidence - 0.25 * risk + 12.0).clamp(0.0, 100.0);
    let bias = if bullish >= 58.0 { "bullish" } else if bullish <= 42.0 { "bearish" } else { "neutral" };

    Some(MarketScore {
        symbol: symbol.to_string(),
        market_score,
        bullish,
        bearish,
        confidence,
        risk,
        trend,
        momentum,
        volatility,
        sentiment: sent,
        price,
        bias: bias.to_string(),
    })
}

/// Derive market regime + dynamic capital allocation from all current scores.
async fn recompute(ctx: &Context, latest: &HashMap<String, MarketScore>) {
    if latest.is_empty() {
        return;
    }
    let n = latest.len() as f64;
    let bullish = latest.values().filter(|s| s.bias == "bullish").count() as f64;
    let breadth = bullish / n * 100.0;
    let avg_vol = latest.values().map(|s| s.volatility).sum::<f64>() / n;

    let regime = if breadth >= 60.0 {
        "bull_market"
    } else if breadth <= 40.0 {
        "bear_market"
    } else {
        "sideways_market"
    };
    let vol_regime = if avg_vol >= 1.5 { "high_volatility" } else { "low_volatility" };
    let favored: Vec<String> = match regime {
        "bull_market" => vec!["trend_following", "breakout", "momentum"],
        "bear_market" => vec!["mean_reversion", "vol_expansion"],
        _ => vec!["mean_reversion", "breakout"],
    }
    .into_iter()
    .map(String::from)
    .collect();
    let _ = ctx.store.set_market_regime(regime, vol_regime, breadth, avg_vol, &favored).await;

    // Allocate toward the top non-bearish opportunities, risk-adjusted, + cash.
    let mut ranked: Vec<&MarketScore> = latest.values().filter(|s| s.bias != "bearish").collect();
    ranked.sort_by(|a, b| b.market_score.partial_cmp(&a.market_score).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(5);

    let cash = if regime == "bear_market" { 0.30 } else { 0.10 };
    let weighted: Vec<(String, f64, f64, f64)> = ranked
        .iter()
        .map(|s| (s.symbol.clone(), s.market_score * (1.0 - s.risk / 200.0), s.market_score, s.risk))
        .collect();
    let total: f64 = weighted.iter().map(|(_, w, _, _)| *w).sum();

    let mut rows: Vec<(String, f64, i32, String)> = Vec::new();
    if total > 0.0 {
        for (i, (sym, w, score, risk)) in weighted.iter().enumerate() {
            rows.push((
                sym.clone(),
                (w / total) * (1.0 - cash),
                (i + 1) as i32,
                format!("score {score:.0}, risk {risk:.0}"),
            ));
        }
    }
    rows.push(("cash_reserve".to_string(), cash, 99, format!("{regime} reserve")));
    let _ = ctx.store.set_capital_allocation(&rows).await;
}
