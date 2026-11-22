//! Signals service. Market data is shared per region; strategies are per
//! tenant. We maintain one rolling window per symbol and, for each closed
//! candle, evaluate every tenant's enabled strategies, stamping the resulting
//! `Signal` with its tenant and publishing on the tenant-isolated subject
//! `tenant.{id}.signals` so one tenant can never crowd another's queue.

mod ai;
pub mod indicators;
pub mod ma18; // 18MA Qualified Breakout — backtest-first, NOT yet registered for live
pub mod scanner;
pub mod strategies;

use tcore::bus::decode;
use tcore::tenant::{self, TenantId};
use tcore::types::{Candle, Side, Signal};
use tcore::{metrics, subjects, Context, Result};
use futures::StreamExt;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const WINDOW: usize = 300;

/// Best-effort fetch of recent CLOSED klines from Binance to pre-fill a strategy
/// window on startup. Without this, the in-memory window rebuilds from scratch
/// after every restart and trading silently halts for ~slow-EMA minutes. Never
/// errors — returns empty on any failure so we fall back to live rebuild.
async fn warmup_klines(http: &reqwest::Client, symbol: &str, interval: &str, limit: usize) -> Vec<Candle> {
    let url = format!("https://api.binance.com/api/v3/klines?symbol={symbol}&interval={interval}&limit={limit}");
    let rows: Vec<Vec<serde_json::Value>> = match http
        .get(&url)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
    {
        Ok(resp) => match resp.json().await {
            Ok(j) => j,
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };
    rows.iter()
        .map(|r| {
            let g = |i: usize| {
                r.get(i)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
            };
            let close_ms = r.get(6).and_then(|v| v.as_i64()).unwrap_or(0);
            Candle {
                exchange: "binance".to_string(),
                symbol: symbol.to_string(),
                timeframe: interval.to_string(),
                open: g(1),
                high: g(2),
                low: g(3),
                close: g(4),
                volume: g(5),
                closed: true,
                ts: chrono::DateTime::from_timestamp_millis(close_ms).unwrap_or_else(chrono::Utc::now),
            }
        })
        .collect()
}

/// Which markets a bot's strategy is allowed to trade — the watchlist / AI-auto
/// modes. `All` = scan every market (AI auto-discovery); `Only` = a fixed set.
#[derive(Clone)]
enum SymbolFilter {
    All,
    Only(HashSet<String>),
}
impl SymbolFilter {
    fn allows(&self, sym: &str) -> bool {
        match self {
            SymbolFilter::All => true,
            SymbolFilter::Only(s) => s.contains(sym),
        }
    }
    fn merge(&mut self, other: SymbolFilter) {
        match (&mut *self, other) {
            (SymbolFilter::All, _) => {}
            (slot, SymbolFilter::All) => *slot = SymbolFilter::All,
            (SymbolFilter::Only(a), SymbolFilter::Only(b)) => a.extend(b),
        }
    }
}

/// A live strategy instance plus the markets it may act on.
struct StratEntry {
    strat: Box<dyn strategies::Strategy>,
    filter: SymbolFilter,
}

/// One tenant's live strategy set.
struct TenantStrategies {
    tenant: TenantId,
    strategies: Vec<StratEntry>,
}

/// Build the per-tenant strategy set from this tenant's *running bots*. Bots on
/// the same strategy are merged (their watchlists unioned; any AI-auto bot opens
/// the whole universe) so each (tenant, strategy) evaluates once per candle.
async fn build_tenants(ctx: &Context) -> Vec<TenantStrategies> {
    let mut filters: HashMap<(Uuid, String), SymbolFilter> = HashMap::new();
    for (tenant_uuid, strategy, config, broker) in load_active(ctx).await {
        // A real broker can't trade every market. Tell the user clearly (once)
        // which markets it skips, instead of silently dropping them or spamming
        // failed orders.
        if broker != "paper" && !broker.is_empty() {
            let unsupported: Vec<String> = ctx
                .config
                .symbols
                .iter()
                .filter(|s| !broker_supports(&broker, s))
                .map(|s| s.trim_end_matches("USDT").to_string())
                .collect();
            if !unsupported.is_empty() {
                let label = broker_label(&broker);
                let body = format!(
                    "{label} can’t trade {} — your “{strategy}” bot skips these markets on {label}. \
                     Switch the bot to Paper (or a broker that lists them) to trade every market.",
                    unsupported.join(", ")
                );
                let dedup = format!("unsupported:{tenant_uuid}:{broker}");
                let _ = ctx
                    .store
                    .notify_once(TenantId(tenant_uuid), "broker_unsupported",
                        "Some markets aren’t supported by your broker", &body, "warning", &dedup, 86_400)
                    .await;
            }
        }
        // Restrict the bot to markets its broker can actually trade.
        let f = restrict_to_broker(parse_filter(&config), &broker, &ctx.config.symbols);
        filters
            .entry((tenant_uuid, strategy))
            .and_modify(|e| e.merge(f.clone()))
            .or_insert(f);
    }

    let mut by_tenant: HashMap<Uuid, TenantStrategies> = HashMap::new();
    for ((tenant_uuid, name), filter) in filters {
        if let Some(strat) = strategies::build(&name) {
            by_tenant
                .entry(tenant_uuid)
                .or_insert_with(|| TenantStrategies {
                    tenant: TenantId(tenant_uuid),
                    strategies: Vec::new(),
                })
                .strategies
                .push(StratEntry { strat, filter });
        }
    }
    by_tenant.into_values().collect()
}

/// Parse a bot's `config` JSON into a market filter (mode + watchlist/symbol).
fn parse_filter(config: &str) -> SymbolFilter {
    let v: serde_json::Value = serde_json::from_str(config).unwrap_or(serde_json::Value::Null);
    match v.get("mode").and_then(|m| m.as_str()).unwrap_or("ai_auto") {
        "single_symbol" => match v.get("symbol").and_then(|s| s.as_str()) {
            Some(s) => SymbolFilter::Only(HashSet::from([s.to_string()])),
            None => SymbolFilter::All,
        },
        "custom_watchlist" => {
            let set: HashSet<String> = v
                .get("watchlist")
                .and_then(|w| w.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            if set.is_empty() { SymbolFilter::All } else { SymbolFilter::Only(set) }
        }
        _ => SymbolFilter::All, // ai_auto: scan every market
    }
}

async fn load_active(ctx: &Context) -> Vec<(Uuid, String, String, String)> {
    match ctx.store.active_bots().await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, "could not load active bots; signals idle");
            Vec::new()
        }
    }
}

fn broker_label(b: &str) -> &str {
    match b {
        "alpaca" => "Alpaca",
        "binance" => "Binance",
        "coinbase" => "Coinbase",
        other => other,
    }
}

/// Does this broker support trading `symbol`? Paper has no restriction; real
/// brokers only list a subset (e.g. Alpaca crypto). Keeps bots from emitting
/// orders that would just bounce back as "unsupported asset".
fn broker_supports(broker: &str, symbol: &str) -> bool {
    match broker {
        "" | "paper" => true,
        "alpaca" => matches!(
            symbol.trim_end_matches("USDT"),
            "BTC" | "ETH" | "SOL" | "DOGE" | "AVAX" | "LINK" | "DOT" | "LTC" | "BCH" | "AAVE" | "UNI"
        ),
        _ => true, // other brokers: assume supported until their adapter narrows it
    }
}

/// Narrow a bot's market filter to what its broker can actually trade.
fn restrict_to_broker(filter: SymbolFilter, broker: &str, universe: &[String]) -> SymbolFilter {
    if broker.is_empty() || broker == "paper" {
        return filter;
    }
    let supported: HashSet<String> = universe.iter().filter(|s| broker_supports(broker, s)).cloned().collect();
    match filter {
        SymbolFilter::All => SymbolFilter::Only(supported),
        SymbolFilter::Only(set) => SymbolFilter::Only(set.intersection(&supported).cloned().collect()),
    }
}

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    // Shared so a background task can hot-reload strategies (new bots start
    // trading without a restart).
    let tenants = Arc::new(RwLock::new(build_tenants(&ctx).await));
    let n = tenants.read().await.len();
    tracing::info!(tenants = n, "signals starting");

    {
        let ctx = ctx.clone();
        let tenants = tenants.clone();
        let secs: u64 = std::env::var("SIGNALS_RELOAD_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(secs.max(5)));
            loop {
                tick.tick().await;
                let fresh = build_tenants(&ctx).await;
                *tenants.write().await = fresh;
            }
        });
    }

    let http = reqwest::Client::new();
    let ai_url = ctx.config.ai_engine_url.clone();
    let mut windows: HashMap<String, VecDeque<Candle>> = HashMap::new();

    // Warm windows from recent history so strategies emit on the FIRST live
    // candle after a (re)start — eliminates the ~slow-EMA-minutes cold start
    // that silently halted trading after every deploy.
    {
        let secs: u64 = std::env::var("CANDLE_SECONDS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
        let interval = match secs {
            s if s >= 3600 => "1h",
            s if s >= 900 => "15m",
            s if s >= 300 => "5m",
            _ => "1m",
        };
        let mut warmed = 0usize;
        for symbol in &ctx.config.symbols {
            let candles = warmup_klines(&http, symbol, interval, 180).await;
            if !candles.is_empty() {
                warmed += 1;
                windows.insert(format!("binance:{symbol}"), candles.into_iter().collect());
            }
        }
        tracing::info!(warmed, total = ctx.config.symbols.len(), interval, "signals: warmed candle windows from history");
    }

    let mut sub = ctx.bus.subscribe(subjects::MARKET_CANDLE_ALL).await?;
    while let Some(msg) = sub.next().await {
        let candle: Candle = match decode::<Candle>(&msg.payload) {
            Some(c) if c.closed => c,
            _ => continue,
        };
        let key = format!("{}:{}", candle.exchange, candle.symbol);
        let win = windows.entry(key).or_default();
        win.push_back(candle.clone());
        while win.len() > WINDOW {
            win.pop_front();
        }
        let slice: Vec<Candle> = win.iter().cloned().collect();

        // Evaluate under a short read lock, then release it before awaiting on
        // the (potentially slow) signal emit so reloads are never blocked.
        let to_emit: Vec<(TenantId, &'static str, strategies::RawSignal)> = {
            let guard = tenants.read().await;
            let mut out = Vec::new();
            for ts in guard.iter() {
                for entry in &ts.strategies {
                    if entry.filter.allows(&candle.symbol) {
                        if let Some(raw) = entry.strat.evaluate(&slice) {
                            out.push((ts.tenant, entry.strat.name(), raw));
                        }
                    }
                }
            }
            out
        };
        for (tenant, name, raw) in to_emit {
            emit_signal(&ctx, &http, &ai_url, tenant, name, &candle, raw, &slice).await;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn emit_signal(
    ctx: &Context,
    http: &reqwest::Client,
    ai_url: &str,
    tenant: TenantId,
    strategy: &'static str,
    candle: &Candle,
    raw: strategies::RawSignal,
    slice: &[Candle],
) {
    let started = std::time::Instant::now();
    let side_str = match raw.side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    };
    let closes = indicators::closes(slice);
    let ai_score = ai::score(http, ai_url, strategy, &candle.symbol, side_str, raw.price, raw.strength, &closes).await;

    let signal = Signal {
        id: Uuid::new_v4(),
        tenant,
        strategy: strategy.to_string(),
        exchange: candle.exchange.clone(),
        symbol: candle.symbol.clone(),
        side: raw.side,
        price: raw.price,
        stop: raw.stop,
        take_profit: raw.take_profit,
        strength: raw.strength,
        ai_score,
        ts: chrono::Utc::now(),
    };

    if let Err(e) = ctx.bus.publish(tenant::subj_signals(tenant), &signal).await {
        tracing::warn!(error = %e, "failed to publish signal");
        return;
    }
    metrics::inc("signals_emitted_total");
    metrics::observe_latency("signal_latency_seconds", started.elapsed().as_secs_f64());
    tracing::info!(%tenant, strategy, symbol = %candle.symbol, side = side_str, ?ai_score, "signal");
}

