//! Shared runtime context handed (as `Arc<Context>`) to every service task.

use crate::bus::Bus;
use crate::config::Config;
use crate::store::Store;
use anyhow::Result;
use std::sync::Arc;

pub struct Context {
    pub config: Config,
    pub bus: Bus,
    pub store: Store,
}

impl Context {
    /// Connect to NATS + Redis + Postgres and assemble the shared context.
    pub async fn bootstrap(config: Config) -> Result<Arc<Self>> {
        let bus = Bus::connect(&config.nats_url).await?;
        let store = Store::connect(&config.redis_url, &config.database_url).await?;
        Ok(Arc::new(Self { config, bus, store }))
    }
}
