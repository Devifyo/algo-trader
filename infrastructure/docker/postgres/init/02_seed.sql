-- Minimal seed so a fresh stack has something to trade in paper mode.
INSERT INTO exchanges (name, testnet, enabled) VALUES
    ('binance', true, true),
    ('bybit',   true, false),
    ('okx',     true, false),
    ('coinbase',true, false)
ON CONFLICT (name) DO NOTHING;

INSERT INTO symbols (exchange_id, symbol, base, quote, tick_size, step_size, min_notional)
SELECT e.id, v.symbol, v.base, v.quote, v.tick, v.step, v.min_notional
FROM exchanges e
JOIN (VALUES
    ('BTCUSDT','BTC','USDT',0.1,0.00001,5),
    ('ETHUSDT','ETH','USDT',0.01,0.0001,5),
    ('SOLUSDT','SOL','USDT',0.01,0.01,5)
) AS v(symbol,base,quote,tick,step,min_notional) ON true
WHERE e.name = 'binance'
ON CONFLICT (exchange_id, symbol) DO NOTHING;

INSERT INTO strategies (name, kind, enabled, params) VALUES
    ('trend_following', 'trend',        true,  '{"fast_ema":21,"slow_ema":55,"atr_period":14,"atr_stop_mult":2.0}'),
    ('momentum',        'momentum',     false, '{"lookback":20,"threshold":0.02}'),
    ('mean_reversion',  'mean_reversion',false,'{"bb_period":20,"bb_std":2.0,"rsi_period":14}'),
    ('breakout',        'breakout',     false, '{"channel":55,"atr_period":14}'),
    ('stat_arb',        'stat_arb',     false, '{"pair":["BTCUSDT","ETHUSDT"],"zscore":2.0,"window":120}'),
    ('vol_expansion',   'vol_expansion',false, '{"atr_period":14,"squeeze_pct":0.5}')
ON CONFLICT (tenant_id, name) DO NOTHING;
