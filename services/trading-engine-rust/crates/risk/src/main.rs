use tcore::{config::Config, metrics, Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tcore::init_tracing();
    let config = Config::from_env();
    let _ = metrics::install(&config.metrics_addr);
    let ctx = Context::bootstrap(config).await?;
    risk::run(ctx).await
}
