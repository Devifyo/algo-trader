# Advanced AI Portfolio Management

Advisory analytics in the Python intelligence service (`portfolio-service`).
Every endpoint returns scores + a recommendation + an explanation. **The risk
engine is final authority; trade execution is never blocked by this layer.**

| Module | Endpoint | What it does |
|--------|----------|--------------|
| AI Risk Manager | `POST /risk/size` | Dynamic, explainable sizing: position size, stop, take-profit, leverage, exposure limit, confidence — scaled by risk profile; auto-reduces/halts on high volatility, major news, exchange outage, black swan. |
| Smart Capital Allocation | `POST /allocation` | Sharpe-led split across strategies/bots/assets/exchanges, drawdown/vol-penalized, capped per item for diversification. Daily/weekly/monthly rebalance. |
| AI Trade Recovery | `POST /recovery` | Recommends one action (hold/exit/reduce/scale-in/move-stop/partial-TP) under hard safety rules (never average down blindly, never exceed limits, risk validation required). |
| Strategy Health Monitor | `POST /health/evaluate` | Health + risk + AI-confidence + market-compat scores; flags decay/degradation/overfitting/market-mismatch. |
| Paper→Live Validation | `POST /health/validate` | Promotion gate: ≥30 paper days, ≥100 trades, ≥55% win rate, ≥1.3 profit factor. |
| Multi-Bot Portfolio | `POST /portfolio/analyze` | Treats bots as one book: exposure, concentration (HHI), correlation, leverage risk, profit/risk contribution; flags conflicting positions / overexposure / bot competition. |
| Strategy Optimizer | `POST /optimize` | Grid + random search over a parameter space; Bayesian/genetic/ML are declared seams. |
| Market Regime Engine | `POST /regime` | Classifies bull/bear/sideways/trend/range/high-vol/low-vol and maps each to preferred strategies. |
| Unified AI Decision Layer | `POST /decision` | Folds risk/news/regime/health/portfolio into bullish/bearish/risk/confidence/market/portfolio scores **with mandatory explainability** (reason, data used, confidence, risk factors, expected outcome). |
| Exchange API Safety | `POST /safety/exchange-key` | Verifies permissions, rejects withdrawal-enabled keys, checks IP allow-list, returns plain-language warnings + fix steps. |

## Marketplace & copy trading

Relational features in the hub: `marketplace_strategies` /
`marketplace_subscriptions` / `strategy_reviews`; `copy_traders` /
`copy_followers` / `copy_trades` with copy modes (fixed/percentage/risk-multiplier)
and max-drawdown protection. Admin/browse UIs live in the Vue dashboard.

## Explainability (mandatory)

Decisions are persisted to `ai_decisions` with the full explanation. The platform
never presents black-box behavior — the dashboard surfaces the reason for every
decision.
