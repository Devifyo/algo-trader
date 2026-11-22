//! Market-data service. Two sources:
//!   * `synthetic` — a self-contained random walk so the whole stack runs with
//!     no internet (default in paper mode).
//!   * `exchange`  — real exchange WebSockets (Binance implemented; others stub).
//! Both normalise into `Tick` + 1-minute `Candle`, publish to NATS, cache the
//! latest price to Redis, and persist closed candles to TimescaleDB.

mod binance;
mod synthetic;

use tcore::config::MarketDataSource;
use tcore::types::{Candle, Tick};
use tcore::{metrics, subjects, Context, Result};
use std::sync::Arc;

const TF: &str = "1m";

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    tracing::info!(source = ?ctx.config.market_data_source, symbols = ?ctx.config.symbols, "market-data starting");
    match ctx.config.market_data_source {
        MarketDataSource::Synthetic => synthetic::run(ctx).await,
        MarketDataSource::Exchange => binance::run(ctx).await,
    }
}

/// Publish a tick everywhere it needs to go. Shared by both sources.
pub(crate) async fn emit_tick(ctx: &Context, tick: &Tick) -> Result<()> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let latency = (now_ms - tick.ts.timestamp_millis()).max(0) as f64 / 1000.0;
    metrics::observe_latency("market_data_latency_seconds", latency);
    metrics::gauge("market_data_last_tick_timestamp_seconds", tick.ts.timestamp() as f64);

    ctx.store
        .set_price(&tick.exchange, &tick.symbol, tick.price, tick.ts.timestamp_millis())
        .await?;
    ctx.bus
        .publish(subjects::market_tick(&tick.exchange, &tick.symbol), tick)
        .await?;
    Ok(())
}

/// Publish a candle; persist when closed.
pub(crate) async fn emit_candle(ctx: &Context, candle: &Candle) -> Result<()> {
    ctx.bus
        .publish(
            subjects::market_candle(&candle.exchange, &candle.symbol, &candle.timeframe),
            candle,
        )
        .await?;
    if candle.closed {
        if let Err(e) = ctx.store.insert_candle(candle).await {
            tracing::warn!(error = %e, "failed to persist candle");
        }
    }
    Ok(())
}

/// Accumulates ticks into the current 1-minute bucket and yields a closed
/// candle when the minute rolls over.
pub(crate) struct CandleBuilder {
    exchange: String,
    symbol: String,
    secs: i64,
    bucket: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    started: bool,
}

impl CandleBuilder {
    pub fn new(exchange: &str, symbol: &str, secs: i64) -> Self {
        Self {
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            secs: secs.max(1),
            bucket: 0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            started: false,
        }
    }

    /// Feed a tick. Returns `Some(closed_candle)` when this tick belongs to a
    /// new minute (and thus closes the previous one), plus always-current open
    /// candle handling internally.
    pub fn on_tick(&mut self, tick: &Tick) -> Option<Candle> {
        let bucket = tick.ts.timestamp() / self.secs;
        let mut closed = None;

        if !self.started {
            self.start(bucket, tick);
            return None;
        }
        if bucket != self.bucket {
            closed = Some(self.snapshot(true));
            self.start(bucket, tick);
        } else {
            self.high = self.high.max(tick.price);
            self.low = self.low.min(tick.price);
            self.close = tick.price;
            self.volume += tick.qty;
        }
        closed
    }

    fn start(&mut self, bucket: i64, tick: &Tick) {
        self.bucket = bucket;
        self.open = tick.price;
        self.high = tick.price;
        self.low = tick.price;
        self.close = tick.price;
        self.volume = tick.qty;
        self.started = true;
    }

    fn snapshot(&self, closed: bool) -> Candle {
        Candle {
            exchange: self.exchange.clone(),
            symbol: self.symbol.clone(),
            timeframe: TF.to_string(),
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            closed,
            ts: chrono::DateTime::from_timestamp(self.bucket * self.secs, 0).unwrap_or_else(chrono::Utc::now),
        }
    }

    /// Current (open) candle — reserved for intra-bar publishing.
    #[allow(dead_code)]
    pub fn current(&self) -> Option<Candle> {
        self.started.then(|| self.snapshot(false))
    }
}
