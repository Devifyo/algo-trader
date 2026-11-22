//! Venue adapters. Each adapter loads the tenant's keys from Vault *into a
//! local* `ExchangeCreds` immediately before signing and lets it drop right
//! after, so secrets never outlive a single order's lifetime.
//!
//! Implemented: Alpaca (REST v2, paper + live). Other brokers return a clear
//! "adapter not implemented" rejection so the engine never silently no-ops.

use tcore::types::{OrderIntent, Side};
use tcore::vault::ExchangeCreds;

/// Result of a live submission attempt.
pub struct VenueAck {
    pub accepted: bool,
    pub exchange_oid: Option<String>,
    pub reason: Option<String>,
}

fn reject(reason: impl Into<String>) -> VenueAck {
    VenueAck { accepted: false, exchange_oid: None, reason: Some(reason.into()) }
}

/// Route an order to the tenant's connected venue.
pub async fn submit(broker: &str, creds: &ExchangeCreds, order: &OrderIntent) -> VenueAck {
    match broker {
        "alpaca" => alpaca_submit(creds, order).await,
        other => {
            tracing::warn!(broker = %other, "no live adapter for broker yet");
            reject(format!("{other}_ADAPTER_NOT_IMPLEMENTED"))
        }
    }
}

/// The USD a broker will actually let us spend on crypto right now. For Alpaca
/// that's NON-marginable buying power (crypto can't use margin), falling back to
/// settled cash. `None` if it can't be determined (caller then sizes normally).
pub async fn available_usd(broker: &str, creds: &ExchangeCreds) -> Option<f64> {
    match broker {
        "alpaca" => alpaca_available_usd(creds).await,
        _ => None,
    }
}

async fn alpaca_available_usd(creds: &ExchangeCreds) -> Option<f64> {
    let live = creds.environment.as_deref() == Some("live");
    let base = if live { "https://api.alpaca.markets" } else { "https://paper-api.alpaca.markets" };
    let resp = reqwest::Client::new()
        .get(format!("{base}/v2/account"))
        .header("APCA-API-KEY-ID", &creds.api_key)
        .header("APCA-API-SECRET-KEY", &creds.api_secret)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let j: serde_json::Value = resp.json().await.ok()?;
    let field = |k: &str| j.get(k).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok());
    // Crypto spends non-marginable buying power; fall back to settled cash.
    field("non_marginable_buying_power").or_else(|| field("cash"))
}

/// Place a market order on Alpaca. Uses the paper host unless the stored
/// credential environment is explicitly "live".
async fn alpaca_submit(creds: &ExchangeCreds, order: &OrderIntent) -> VenueAck {
    let live = creds.environment.as_deref() == Some("live");
    let base = if live { "https://api.alpaca.markets" } else { "https://paper-api.alpaca.markets" };

    let qty = (order.qty * 1e6).round() / 1e6;
    if qty <= 0.0 {
        return reject("ALPACA_BAD_QTY");
    }
    // Only Alpaca-supported crypto pairs; reject others cleanly instead of
    // letting Alpaca treat a bare ticker as an equity (the fractional/DAY error).
    let symbol = match alpaca_symbol(&order.symbol) {
        Some(s) => s,
        None => return reject(format!("ALPACA_UNSUPPORTED_ASSET: {}", order.symbol)),
    };
    let side = match order.side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    };

    let (status, json) = match alpaca_place(base, creds, &symbol, side, qty, &order.client_id).await {
        Ok(r) => r,
        Err(e) => return reject(format!("ALPACA_HTTP: {e}")),
    };
    if (200..300).contains(&status) {
        tracing::info!(env = if live { "live" } else { "paper" }, symbol = %order.symbol, "alpaca order accepted");
        return VenueAck {
            accepted: true,
            exchange_oid: json.get("id").and_then(|v| v.as_str()).map(String::from),
            reason: None,
        };
    }

    let raw = json.get("message").and_then(|v| v.as_str()).unwrap_or("rejected").to_string();
    let lower = raw.to_ascii_lowercase();

    // Dust-tolerant close: a reduce-only SELL can be rejected because the held
    // qty is fractionally less than ordered (the buy fee shaves the base asset).
    // Retry ONCE selling exactly what Alpaca says is available — closes real
    // positions cleanly. `available: 0` → the venue holds none (a phantom), so
    // there's nothing to close. Anything unexpected falls through to a reject.
    if order.reduce_only && side == "sell" && lower.contains("insufficient balance") {
        match parse_alpaca_available(&raw) {
            Some(avail) if avail <= 0.0 => {
                return reject(format!("POSITION_NOT_AT_VENUE (http {status}): {raw}"));
            }
            Some(avail) => {
                let q2 = (avail * 1e9).floor() / 1e9; // round DOWN to stay within holdings
                if q2 > 0.0 {
                    let retry_id = format!("{}-c", order.client_id);
                    if let Ok((s2, j2)) = alpaca_place(base, creds, &symbol, side, q2, &retry_id).await {
                        if (200..300).contains(&s2) {
                            tracing::info!(symbol = %order.symbol, qty = q2, "alpaca close filled at available qty");
                            return VenueAck {
                                accepted: true,
                                exchange_oid: j2.get("id").and_then(|v| v.as_str()).map(String::from),
                                reason: None,
                            };
                        }
                    }
                }
            }
            None => {}
        }
    }

    // Translate Alpaca's opaque text into an accurate, actionable tag (raw venue
    // message appended for debugging). A "$10 minimal amount" rejection means the
    // cost basis is below the venue minimum — NOT that funds are missing.
    let tag = if lower.contains("minimal amount") || lower.contains("cost basis") {
        "ALPACA_BELOW_MIN_NOTIONAL"
    } else if lower.contains("insufficient") || lower.contains("buying power") {
        "ALPACA_INSUFFICIENT_FUNDS"
    } else if lower.contains("not shortable") || lower.contains("short") {
        "ALPACA_NO_SHORT"
    } else {
        "ALPACA_REJECTED"
    };
    tracing::warn!(code = status, tag, %raw, "alpaca rejected order");
    reject(format!("{tag} (http {status}): {raw}"))
}

/// POST a market order to Alpaca; returns (http status, json body). Crypto uses
/// GTC; the symbol must already be the `BASE/USD` crypto form.
async fn alpaca_place(
    base: &str,
    creds: &ExchangeCreds,
    symbol: &str,
    side: &str,
    qty: f64,
    client_id: &str,
) -> Result<(u16, serde_json::Value), reqwest::Error> {
    let body = serde_json::json!({
        "symbol": symbol,
        "qty": format!("{qty}"),
        "side": side,
        "type": "market",
        "time_in_force": "gtc",
        "client_order_id": client_id,
    });
    let resp = reqwest::Client::new()
        .post(format!("{base}/v2/orders"))
        .header("APCA-API-KEY-ID", &creds.api_key)
        .header("APCA-API-SECRET-KEY", &creds.api_secret)
        .timeout(std::time::Duration::from_secs(8))
        .json(&body)
        .send()
        .await?;
    let status = resp.status().as_u16();
    let json: serde_json::Value = resp.json().await.unwrap_or_default();
    Ok((status, json))
}

/// Parse the `available: N` figure out of an Alpaca insufficient-balance message
/// like `insufficient balance for DOT (requested: 1.94, available: 1.9404)`.
fn parse_alpaca_available(msg: &str) -> Option<f64> {
    let idx = msg.find("available:")?;
    let rest = msg[idx + "available:".len()..].trim_start();
    let num: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num.parse::<f64>().ok()
}

/// Map an internal symbol (e.g. `BTCUSDT`) to Alpaca's crypto notation
/// (`BTC/USD`). Returns None for assets Alpaca does not list as crypto, so we
/// never accidentally submit a bare ticker as an equity order.
fn alpaca_symbol(symbol: &str) -> Option<String> {
    const SUPPORTED: &[&str] = &[
        "BTC", "ETH", "SOL", "DOGE", "AVAX", "LINK", "DOT", "LTC", "BCH", "AAVE",
        "UNI", "SHIB", "XTZ", "GRT", "MKR", "CRV", "SUSHI", "YFI", "BAT", "USDC", "USDT",
    ];
    let base = symbol.trim_end_matches("USDT").trim_end_matches("USD");
    if SUPPORTED.contains(&base) {
        Some(format!("{base}/USD"))
    } else {
        None
    }
}
