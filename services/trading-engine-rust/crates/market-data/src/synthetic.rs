//! Self-contained random-walk market data. Lets the entire platform run end to
//! end (signals → risk → paper execution) with zero external connectivity.

use super::{emit_candle, emit_tick, CandleBuilder};
use tcore::types::{Side, Tick};
use tcore::{Context, Result};
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;

const EXCHANGE: &str = "paper";

/// Realistic seed prices so synthetic data roughly matches real markets — this
/// keeps order sizes sane when routed to a real broker (e.g. clearing Alpaca's
/// $10 minimum) and makes the dashboard prices believable.
fn seed_price(symbol: &str) -> f64 {
    match symbol.trim_end_matches("USDT") {
        "BTC" => 65_000.0,
        "ETH" => 3_200.0,
        "SOL" => 150.0,
        "BNB" => 580.0,
        "LTC" => 80.0,
        "AVAX" => 30.0,
        "LINK" => 15.0,
        "ATOM" => 6.5,
        "DOT" => 5.0,
        "XRP" => 0.55,
        "ADA" => 0.45,
        "DOGE" => 0.15,
        _ => 100.0,
    }
}

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    let mut handles = Vec::new();
    for symbol in ctx.config.symbols.clone() {
        let ctx = ctx.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = walk(ctx, symbol.clone()).await {
                tracing::error!(symbol, error = %e, "synthetic walk failed");
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

async fn walk(ctx: Arc<Context>, symbol: String) -> Result<()> {
    let seed = seed_price(&symbol);
    let mut price = seed;
    // Per-tick noise (ticks land ~4/sec).
    let vol = seed * 0.0004;
    // A slowly-drifting anchor (sine) sets sustained ~10-min trend legs; price
    // tracks it with a short lag + noise. Trends last far longer than a strategy's
    // indicator lag, so trend-following can actually ride them (and still reverse,
    // so positions cycle). Phase is seeded per symbol so markets aren't in lock-step.
    let phase = (symbol.bytes().map(|b| b as u32).sum::<u32>() % 1000) as f64 / 1000.0 * std::f64::consts::TAU;
    let mut t: f64 = 0.0;
    let mut builder = CandleBuilder::new(EXCHANGE, &symbol, ctx.config.candle_seconds);
    let mut ticker = tokio::time::interval(Duration::from_millis(250));

    loop {
        ticker.tick().await;
        t += 1.0;
        // ~10 min trend half-cycles at 4 ticks/sec (half-period ≈ 2400 ticks), 8% swings.
        let anchor = seed * (1.0 + 0.08 * (t / 760.0 + phase).sin());
        let (delta, qty, side) = {
            let mut rng = rand::thread_rng();
            let noise: f64 = rng.gen_range(-1.0..1.0) * vol;
            let revert = 0.012 * (anchor - price); // track the trend with a short lag
            let delta = revert + noise;
            let qty: f64 = rng.gen_range(0.001..0.5);
            let side = if delta >= 0.0 { Side::Buy } else { Side::Sell };
            (delta, qty, side)
        };
        price = (price + delta).max(0.01);

        let tick = Tick {
            exchange: EXCHANGE.to_string(),
            symbol: symbol.clone(),
            price,
            qty,
            side: Some(side),
            ts: chrono::Utc::now(),
        };
        emit_tick(&ctx, &tick).await?;
        if let Some(candle) = builder.on_tick(&tick) {
            emit_candle(&ctx, &candle).await?;
        }
    }
}
