# News Intelligence & Sentiment

A dedicated, always-on service that collects, dedupes, categorizes, AI-analyzes,
and scores financial/crypto news — and publishes **advisory** sentiment.

> **News never triggers a trade.** It contributes to sentiment, risk, confidence,
> and regime. The risk engine is final authority.

## Pipeline

```
News Service → Redis queue → AI Analysis (router) → Consensus → Sentiment
            → NATS sentiment.{asset} + Redis sentiment:{asset} → Risk Engine
```

- **Collection:** RSS (CoinDesk, Cointelegraph, Decrypt, CryptoSlate, The Block,
  CNBC, MarketWatch, Yahoo) out of the box; Reuters/Bloomberg/economic/social/
  on-chain declared with adapters (stubbed until keys). `NEWS_SOURCE=synthetic`
  runs a self-contained offline stream (default).
- **Dedup:** unique SHA-256 of the article URL.
- **Categorize:** keyword classifier → 16 categories + affected assets +
  critical-event detection (ETF approval/rejection, hack, bankruptcy, regulation,
  rate decision, CPI, Fed, black swan).
- **Analyze:** each article gets a multi-provider **consensus** sentiment, mapped
  to the output schema: sentiment (−100..+100), confidence (0..100), market
  impact, duration, affected assets, risk level.
- **Economic calendar:** seeds CPI/FOMC/NFP/GDP/PPI; raises a Redis
  `macro:event_soon` flag in-window the risk engine reads to lower conviction.

## Consumption by the risk engine (read-only)

The Rust `risk` service reads `sentiment:{asset}` and `macro:event_soon` from
Redis. It can **veto or tighten** an entry (e.g. block a long when confident news
is strongly bearish, or raise the AI floor during a macro window) — it never
creates a trade from news. See `risk/src/lib.rs` steps 6b/6c.

## Storage

`news_articles`, `news_ai_analysis`, `news_sentiments`, `market_events`,
`economic_events`, `social_mentions`, `onchain_metrics`, `sentiment_history`.

## Reliability

Multi-provider failover (via the AI router), Redis-queued processing, per-feed
isolation (one bad feed can't stall others), dedup, and fail-open analysis (a
model outage yields a neutral verdict rather than stalling the pipeline).
