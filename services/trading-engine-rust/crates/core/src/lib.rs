//! Shared kernel for the trading engine: domain types, config, the NATS bus,
//! Redis/Postgres stores, Vault secret loading, and metrics. Everything on the
//! hot path depends on this crate and nothing else shared.

pub mod brokers;
pub mod bus;
pub mod compliance;
pub mod config;
pub mod context;
pub mod metrics;
pub mod store;
pub mod subjects;
pub mod tenant;
pub mod types;
pub mod vault;

pub use anyhow::{Context as _, Result};
pub use context::Context;

/// Initialise tracing once, reading `RUST_LOG` (defaults to `info`).
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    // JSON logs ship cleanly into Loki via the docker json-file driver.
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}
