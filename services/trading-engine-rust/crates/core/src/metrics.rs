//! Prometheus metrics. We install one HTTP exporter per process (engine or a
//! single split service) on `ENGINE_METRICS_ADDR`. Metric names match the
//! Prometheus alert rules and the Grafana "Overview" dashboard.

use anyhow::Result;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;

/// Install the Prometheus recorder + HTTP `/metrics` listener. Idempotent-ish:
/// a second call in the same process returns the install error, which callers
/// may ignore.
pub fn install(addr: &str) -> Result<()> {
    let socket: SocketAddr = addr.parse()?;
    PrometheusBuilder::new()
        .with_http_listener(socket)
        .install()?;
    Ok(())
}

/// Record a stage latency histogram in seconds. `stage` becomes the metric
/// family, e.g. `market_data_latency_seconds`.
pub fn observe_latency(metric: &'static str, seconds: f64) {
    metrics::histogram!(metric).record(seconds);
}

pub fn inc(metric: &'static str) {
    metrics::counter!(metric).increment(1);
}

pub fn inc_labeled(metric: &'static str, label: &'static str, value: String) {
    metrics::counter!(metric, label => value).increment(1);
}

pub fn gauge(metric: &'static str, value: f64) {
    metrics::gauge!(metric).set(value);
}
