use crate::tenant::Region;
use serde::Deserialize;
use std::env;

/// Whole-engine configuration, sourced from environment variables. Risk limits
/// here are *defaults*; per-strategy overrides come from the `strategies` table.
#[derive(Clone, Debug)]
pub struct Config {
    pub trading_mode: TradingMode,
    pub database_url: String,
    pub redis_url: String,
    pub nats_url: String,
    pub vault_addr: String,
    pub vault_token: String,
    pub ai_engine_url: String,
    pub metrics_addr: String,
    /// Region this spoke serves; drives the compliance regime.
    pub region: Region,
    /// Default per-tenant order-rate cap (orders/sec). SEBI region uses 10.
    pub default_max_orders_per_sec: usize,
    pub market_data_source: MarketDataSource,
    pub symbols: Vec<String>,
    /// Seconds per candle bucket. 60 for real markets; lower (e.g. 3) in
    /// synthetic/paper mode so strategies + scanner warm up in minutes, not hours.
    pub candle_seconds: i64,
    pub risk: RiskConfig,
    pub starting_equity_usd: f64,
    pub stale_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradingMode {
    /// No order ever reaches a real exchange.
    Paper,
    /// Real signed orders. Requires Vault keys + per-venue go-live.
    Live,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketDataSource {
    /// Self-contained random-walk generator — runs with no internet.
    Synthetic,
    /// Real exchange WebSockets (testnet by default).
    Exchange,
}

#[derive(Clone, Debug)]
pub struct RiskConfig {
    pub max_per_trade_pct: f64,
    pub daily_drawdown_pct: f64,
    pub weekly_drawdown_pct: f64,
    pub max_leverage: f64,
    pub max_open_positions: u32,
    pub kill_switch: bool,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn env_u32(key: &str, default: u32) -> u32 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Self {
        let trading_mode = match env_or("TRADING_MODE", "paper").as_str() {
            "live" => TradingMode::Live,
            _ => TradingMode::Paper,
        };
        let market_data_source = match env_or("MARKET_DATA_SOURCE", "synthetic").as_str() {
            "exchange" => MarketDataSource::Exchange,
            _ => MarketDataSource::Synthetic,
        };
        let symbols = env_or("SYMBOLS", "BTCUSDT,ETHUSDT,SOLUSDT")
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect();
        let kill_switch = matches!(env_or("KILL_SWITCH", "off").as_str(), "on" | "1" | "true");
        let region = Region::from_env();
        // India region defaults to the SEBI 10 orders/sec ceiling.
        let default_max_orders_per_sec = match region {
            Region::ApSouth1 => 10,
            Region::UsEast1 => env_u32("MAX_ORDERS_PER_SEC", 25) as usize,
        };

        Config {
            trading_mode,
            database_url: env_or("DATABASE_URL", "postgres://algotrader:algotrader@postgres:5432/algotrader"),
            redis_url: env_or("REDIS_URL", "redis://:algotrader@redis:6379/0"),
            nats_url: env_or("NATS_URL", "nats://nats:4222"),
            vault_addr: env_or("VAULT_ADDR", "http://vault:8200"),
            vault_token: env_or("VAULT_TOKEN", "dev-only-root-token"),
            ai_engine_url: env_or("AI_ENGINE_URL", "http://ai-engine:8000"),
            metrics_addr: env_or("ENGINE_METRICS_ADDR", "0.0.0.0:9100"),
            region,
            default_max_orders_per_sec,
            market_data_source,
            symbols,
            candle_seconds: env_or("CANDLE_SECONDS", "60").parse().unwrap_or(60).max(1),
            starting_equity_usd: env_f64("STARTING_EQUITY_USD", 100_000.0),
            stale_ms: env_or("STALE_MS", "5000").parse().unwrap_or(5000),
            risk: RiskConfig {
                max_per_trade_pct: env_f64("RISK_MAX_PER_TRADE_PCT", 0.5),
                daily_drawdown_pct: env_f64("RISK_DAILY_DRAWDOWN_PCT", 3.0),
                weekly_drawdown_pct: env_f64("RISK_WEEKLY_DRAWDOWN_PCT", 8.0),
                max_leverage: env_f64("RISK_MAX_LEVERAGE", 5.0),
                max_open_positions: env_u32("RISK_MAX_OPEN_POSITIONS", 10),
                kill_switch,
            },
        }
    }
}
