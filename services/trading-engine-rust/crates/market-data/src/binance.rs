//! Binance (spot/testnet) WebSocket market data via combined trade streams.
//! Trades are normalised to ticks and aggregated into 1m candles. Reconnects
//! with exponential backoff; a stale-data guard lives in the risk service.

use super::{emit_candle, emit_tick, CandleBuilder};
use tcore::types::{Side, Tick};
use tcore::{metrics, Context, Result};
use futures::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const EXCHANGE: &str = "binance";

#[derive(Deserialize)]
struct Combined {
    data: TradeEvent,
}

#[derive(Deserialize)]
struct TradeEvent {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    qty: String,
    #[serde(rename = "T")]
    trade_time_ms: i64,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

fn ws_url(symbols: &[String]) -> String {
    let testnet = std::env::var("BINANCE_TESTNET").map(|v| v != "false").unwrap_or(true);
    let host = if testnet {
        "wss://stream.testnet.binance.vision/stream"
    } else {
        "wss://stream.binance.com:9443/stream"
    };
    let streams = symbols
        .iter()
        .map(|s| format!("{}@trade", s.to_lowercase()))
        .collect::<Vec<_>>()
        .join("/");
    format!("{host}?streams={streams}")
}

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    let url = ws_url(&ctx.config.symbols);
    let mut backoff = 1u64;
    let mut builders: HashMap<String, CandleBuilder> = HashMap::new();

    loop {
        tracing::info!(%url, "connecting to binance ws");
        match connect_async(&url).await {
            Ok((mut ws, _)) => {
                backoff = 1;
                while let Some(msg) = ws.next().await {
                    match msg {
                        Ok(Message::Text(txt)) => {
                            if let Err(e) = handle(&ctx, &mut builders, txt.as_str()).await {
                                tracing::warn!(error = %e, "handle trade failed");
                            }
                        }
                        Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                        Ok(Message::Close(_)) | Err(_) => break,
                        _ => {}
                    }
                }
            }
            Err(e) => tracing::error!(error = %e, "binance ws connect failed"),
        }

        metrics::inc_labeled("ws_reconnects_total", "exchange", EXCHANGE.to_string());
        tracing::warn!(backoff, "binance ws disconnected; backing off");
        tokio::time::sleep(Duration::from_secs(backoff)).await;
        backoff = (backoff * 2).min(30);
    }
}

async fn handle(
    ctx: &Context,
    builders: &mut HashMap<String, CandleBuilder>,
    txt: &str,
) -> Result<()> {
    let ev: Combined = serde_json::from_str(txt)?;
    let t = ev.data;
    let price: f64 = t.price.parse().unwrap_or(0.0);
    let qty: f64 = t.qty.parse().unwrap_or(0.0);
    if price <= 0.0 {
        return Ok(());
    }
    let tick = Tick {
        exchange: EXCHANGE.to_string(),
        symbol: t.symbol.clone(),
        price,
        qty,
        side: Some(if t.is_buyer_maker { Side::Sell } else { Side::Buy }),
        ts: chrono::DateTime::from_timestamp_millis(t.trade_time_ms).unwrap_or_else(chrono::Utc::now),
    };
    emit_tick(ctx, &tick).await?;

    let builder = builders
        .entry(t.symbol.clone())
        .or_insert_with(|| CandleBuilder::new(EXCHANGE, &t.symbol, ctx.config.candle_seconds));
    if let Some(candle) = builder.on_tick(&tick) {
        emit_candle(ctx, &candle).await?;
    }
    Ok(())
}
