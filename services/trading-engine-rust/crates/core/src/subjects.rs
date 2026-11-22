//! NATS subject naming. Centralised so producers and consumers never drift.

pub fn market_tick(exchange: &str, symbol: &str) -> String {
    format!("market.{exchange}.{symbol}.tick")
}
pub fn market_candle(exchange: &str, symbol: &str, tf: &str) -> String {
    format!("market.{exchange}.{symbol}.candle.{tf}")
}
/// Wildcard for all candle streams.
pub const MARKET_CANDLE_ALL: &str = "market.*.*.candle.*";
pub const MARKET_TICK_ALL: &str = "market.*.*.tick";

pub fn signal(strategy: &str, symbol: &str) -> String {
    format!("signals.{strategy}.{symbol}")
}
pub const SIGNAL_ALL: &str = "signals.*.*";

pub const ORDERS_NEW: &str = "orders.new";
pub const ORDERS_CANCEL: &str = "orders.cancel";

pub const EXEC_FILL: &str = "executions.fill";
pub const EXEC_REJECT: &str = "executions.reject";

pub const POSITIONS_UPDATE: &str = "positions.update";

pub const CONTROL_KILL: &str = "control.kill_switch";
pub const CONTROL_STRATEGY: &str = "control.strategy";
pub const CONTROL_ALL: &str = "control.*";

/// AI scoring RPC.
pub const AI_SCORE_REQUEST: &str = "signals.score.request";

pub fn alert(severity: &str) -> String {
    format!("alerts.{severity}")
}
