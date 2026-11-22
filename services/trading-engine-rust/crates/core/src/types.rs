use crate::tenant::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn sign(self) -> f64 {
        match self {
            Side::Buy => 1.0,
            Side::Sell => -1.0,
        }
    }
    pub fn opposite(self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
}

/// A single trade print.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tick {
    pub exchange: String,
    pub symbol: String,
    pub price: f64,
    pub qty: f64,
    pub side: Option<Side>,
    pub ts: DateTime<Utc>,
}

/// An OHLCV candle for a given timeframe.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candle {
    pub exchange: String,
    pub symbol: String,
    pub timeframe: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub closed: bool,
    pub ts: DateTime<Utc>,
}

/// A strategy's directional opinion, *before* risk sizing. Carries the stop so
/// risk can size against defined risk — a signal with no stop is rejected.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal {
    pub id: Uuid,
    pub tenant: TenantId,
    pub strategy: String,
    pub exchange: String,
    pub symbol: String,
    pub side: Side,
    /// Reference entry price (last close).
    pub price: f64,
    /// Protective stop price. Required.
    pub stop: f64,
    pub take_profit: Option<f64>,
    /// Rule-based conviction in [0,1].
    pub strength: f64,
    /// Optional AI score in [0,1], filled by the signals service if available.
    pub ai_score: Option<f64>,
    pub ts: DateTime<Utc>,
}

/// A risk-approved order intent. The *only* thing execution will act on.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderIntent {
    pub client_id: String,
    pub tenant: TenantId,
    pub strategy: String,
    /// Broker/exchange the order is routed to (whitelist-checked under SEBI).
    pub broker: String,
    pub exchange: String,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub qty: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub take_profit: Option<f64>,
    pub leverage: f64,
    /// True when this order CLOSES an existing position (an exit/stop). Exits must
    /// always be allowed through — never blocked by the min-notional gate, since a
    /// position can dip below the venue minimum and still needs to be closable
    /// (and stop-losses must always fire). Defaults false (a normal entry).
    #[serde(default)]
    pub reduce_only: bool,
    pub ts: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecStatus {
    Filled,
    Partial,
    Rejected,
    Canceled,
}

/// Result of attempting to execute an `OrderIntent`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Execution {
    pub client_id: String,
    pub tenant: TenantId,
    pub exchange: String,
    pub symbol: String,
    pub side: Side,
    pub status: ExecStatus,
    pub filled_qty: f64,
    pub avg_price: f64,
    pub fee: f64,
    pub exchange_oid: Option<String>,
    pub reason: Option<String>,
    pub ts: DateTime<Utc>,
}

/// A position update broadcast after a fill changes net exposure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub tenant: TenantId,
    pub exchange: String,
    pub symbol: String,
    pub strategy: Option<String>,
    pub side: Side,
    pub qty: f64,
    pub entry_price: f64,
    pub stop_price: Option<f64>,
    pub take_profit: Option<f64>,
    pub leverage: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub closed: bool,
    pub ts: DateTime<Utc>,
}

/// Operator control message on `control.*`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Control {
    KillSwitch { on: bool, actor: String },
    Strategy { name: String, enabled: bool, actor: String },
}
