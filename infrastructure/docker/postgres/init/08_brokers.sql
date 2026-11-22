-- ════════════════════════════════════════════════════════════════════════
--  Broker / exchange catalog. Tenants connect to these; the connect flow
--  adapts its credential fields to `credential_kind`. Secrets always go to
--  Vault (secret/tenants/{id}/brokers/{broker_id}); only references are stored.
-- ════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS brokers (
    broker_id        TEXT PRIMARY KEY,
    name             TEXT NOT NULL,
    market           TEXT NOT NULL,                 -- India | Global | Crypto
    exchanges        TEXT[] NOT NULL DEFAULT '{}',
    auth_type        TEXT NOT NULL,
    -- drives the connect form: api_key_secret | api_key_secret_passphrase |
    -- api_key_totp | oauth2 | bearer_token
    credential_kind  TEXT NOT NULL,
    rate_limit_rpm   INT NOT NULL DEFAULT 600,
    features         TEXT[] NOT NULL DEFAULT '{}',
    documentation_url TEXT,
    enabled          BOOLEAN NOT NULL DEFAULT true,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
GRANT SELECT ON brokers TO algotrader_app;

INSERT INTO brokers (broker_id,name,market,exchanges,auth_type,credential_kind,rate_limit_rpm,features,documentation_url) VALUES
 ('dhan','Dhan (DhanHQ V2)','India',ARRAY['NSE','BSE','MCX'],'OAuth2','oauth2',600,ARRAY['WebSockets (V2)','Sub-100ms Execution','Webhook Integration','Free API'],'https://docs.dhanhq.co/'),
 ('zerodha','Zerodha (Kite Connect)','India',ARRAY['NSE','BSE','MCX'],'OAuth2','oauth2',600,ARRAY['WebSockets','Highest Liquidity','Historical Data API','Developer Fee Required'],'https://kite.trade/'),
 ('upstox','Upstox (Uplink V2)','India',ARRAY['NSE','BSE','MCX'],'OAuth2','oauth2',600,ARRAY['Protobuf WebSockets V3','REST API','Free API','Sandbox Mode'],'https://upstox.com/developer/api-documentation/'),
 ('angelone','Angel One (SmartAPI)','India',ARRAY['NSE','BSE','MCX'],'API Key + TOTP (Daily Token)','api_key_totp',1200,ARRAY['WebSocket Streaming 2.0','Historical Data API','Free API','Multi-Language SDKs'],'https://smartapi.angelone.in/docs'),
 ('fyers','Fyers (API v3)','India',ARRAY['NSE','BSE','MCX'],'OAuth2','oauth2',600,ARRAY['WebSockets','Options Support','Webhook / TradingView Alerts','Free API'],'https://myapi.fyers.in/docsv3'),
 ('shoonya','Finvasia (Shoonya)','India',ARRAY['NSE','BSE','MCX','NFO','CDS'],'API Key + TOTP','api_key_totp',600,ARRAY['WebSockets','Free API','Zero Brokerage','Multi-Segment'],'https://www.shoonya.com/api-documentation'),
 ('flattrade','Flattrade','India',ARRAY['NSE','BSE','MCX','NFO','CDS'],'API Key + TOTP','api_key_totp',600,ARRAY['WebSockets','Free API','Zero Brokerage','REST API'],'https://flattrade.in/api'),
 ('fivepaisa','5paisa (XTS API)','India',ARRAY['NSE','BSE','MCX'],'OAuth2','oauth2',600,ARRAY['WebSockets','REST API','Historical Data','Low-Cost Brokerage'],'https://www.5paisa.com/developerapi/overview'),
 ('alpaca','Alpaca','Global',ARRAY['NYSE','NASDAQ','Crypto'],'API Key + Secret (Paper/Live)','api_key_secret',12000,ARRAY['Fractional Shares','Paper Trading API','REST + WebSocket','Commission-free'],'https://docs.alpaca.markets/'),
 ('tradier','Tradier','Global',ARRAY['NYSE','NASDAQ','US Options'],'OAuth2','oauth2',120,ARRAY['Advanced Options Routing','Partner Revenue Sharing','Sandbox Environment'],'https://docs.tradier.com/'),
 ('schwab','Charles Schwab (Trader API)','Global',ARRAY['NYSE','NASDAQ','US Options'],'OAuth2','oauth2',7200,ARRAY['WebSocket Streaming','Options Chains','Historical Data','Replaced TD Ameritrade'],'https://developer.schwab.com/'),
 ('tastytrade','tastytrade','Global',ARRAY['NYSE','NASDAQ','US Options','Futures'],'OAuth2','oauth2',600,ARRAY['Advanced Options','Streaming Market Data','Sandbox Environment','Futures Support'],'https://developer.tastytrade.com/'),
 ('interactive_brokers','Interactive Brokers (IBKR)','Global',ARRAY['US Stocks','EU Stocks','Asian Markets','Forex','Futures'],'OAuth2 (Client Portal Web API)','oauth2',3000,ARRAY['Global Reach','Institutional Liquidity','Complex Order Types'],'https://interactivebrokers.github.io/cpwebapi/'),
 ('oanda','OANDA (v20 API)','Global',ARRAY['Forex','CFDs'],'Bearer Token','bearer_token',7200,ARRAY['Streaming Prices','REST API','Practice Account','Forex Specialist'],'https://developer.oanda.com/rest-live-v20/introduction/'),
 ('binance','Binance','Crypto',ARRAY['Spot','USDT-M Futures','COIN-M Futures'],'API Key / Secret (HMAC, RSA, Ed25519)','api_key_secret',3000,ARRAY['WebSockets','Deepest Crypto Liquidity','High Rate Limits'],'https://binance-docs.github.io/apidocs/'),
 ('bybit','Bybit (V5 API)','Crypto',ARRAY['Spot','Perpetuals','Options'],'API Key / Secret (Static)','api_key_secret',7200,ARRAY['Unified V5 Interface','Ultra-Low Latency','Excellent Testnet'],'https://bybit-exchange.github.io/docs/v5/intro'),
 ('okx','OKX (V5 API)','Crypto',ARRAY['Spot','Perpetuals','Options'],'API Key / Secret (Static)','api_key_secret_passphrase',3600,ARRAY['High Frequency Trading Friendly','Unified Account Margin','Demo Trading'],'https://www.okx.com/docs-v5/en/'),
 ('coinbase','Coinbase (Advanced Trade API)','Crypto',ARRAY['Spot'],'API Key / Secret (Ed25519 / JWT)','api_key_secret_passphrase',1800,ARRAY['WebSockets','US-Compliant','REST API','Deep USD Liquidity'],'https://docs.cdp.coinbase.com/advanced-trade/docs/welcome'),
 ('deribit','Deribit','Crypto',ARRAY['Options','Perpetuals','Futures'],'API Key / Secret (OAuth2)','api_key_secret',12000,ARRAY['Leading Crypto Options Venue','WebSockets / FIX','Ultra-Low Latency','Testnet'],'https://docs.deribit.com/'),
 ('kraken','Kraken','Crypto',ARRAY['Spot','Futures'],'API Key / Secret (Static)','api_key_secret',3600,ARRAY['WebSockets','US/EU Compliant','Staking','REST API'],'https://docs.kraken.com/api/')
ON CONFLICT (broker_id) DO NOTHING;

-- Tie a connected exchange_keys row back to the catalog (broker_id).
ALTER TABLE exchange_keys ADD COLUMN IF NOT EXISTS broker_id TEXT;
