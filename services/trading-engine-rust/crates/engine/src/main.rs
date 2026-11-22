//! Single-process supervisor. Runs market-data + signals + risk + execution in
//! one binary sharing one `Context` — ideal for a single VPS where cross-service
//! traffic stays in-process over NATS without extra network hops. In a regional
//! split deployment each service runs from its own binary instead.

use tcore::{config::Config, metrics, Context, Result};
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    tcore::init_tracing();

    // Install a process-wide rustls crypto provider BEFORE any TLS. The Binance
    // wss:// market-data feed (tokio-tungstenite/rustls 0.23) panics on connect
    // without one when multiple backends are present. Idempotent; ignore if set.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let config = Config::from_env();
    tracing::info!(
        mode = ?config.trading_mode,
        region = config.region.as_str(),
        source = ?config.market_data_source,
        "engine booting"
    );

    // One Prometheus exporter for the whole process.
    let _ = metrics::install(&config.metrics_addr);
    metrics::gauge("kill_switch_engaged", 0.0);

    let ctx = Context::bootstrap(config).await?;

    // Each service owns a long-lived loop. Supervise them: if any exits, log and
    // keep the others running. compose/systemd restarts the process on full exit.
    let mut tasks = Vec::new();
    tasks.push(spawn("market-data", ctx.clone(), |c| Box::pin(market_data::run(c))));
    tasks.push(spawn("signals", ctx.clone(), |c| Box::pin(signals::run(c))));
    tasks.push(spawn("scanner", ctx.clone(), |c| Box::pin(signals::scanner::run(c))));
    tasks.push(spawn("risk", ctx.clone(), |c| Box::pin(risk::run(c))));
    tasks.push(spawn("monitor", ctx.clone(), |c| Box::pin(risk::monitor::run(c))));
    tasks.push(spawn("execution", ctx.clone(), |c| Box::pin(execution::run(c))));

    // Graceful shutdown on SIGINT/SIGTERM.
    tokio::select! {
        _ = signal::ctrl_c() => tracing::warn!("shutdown signal received"),
        _ = futures_all(tasks) => tracing::error!("all services exited"),
    }
    Ok(())
}

type RunFut = std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;

fn spawn(
    name: &'static str,
    ctx: Arc<Context>,
    f: impl FnOnce(Arc<Context>) -> RunFut + Send + 'static,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!(service = name, "starting");
        if let Err(e) = f(ctx).await {
            tracing::error!(service = name, error = %e, "service exited with error");
        } else {
            tracing::warn!(service = name, "service exited");
        }
    })
}

async fn futures_all(tasks: Vec<tokio::task::JoinHandle<()>>) {
    for t in tasks {
        let _ = t.await;
    }
}
