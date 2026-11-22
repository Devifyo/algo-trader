"""News source registry + category taxonomy. RSS feeds work with no API key;
paywalled/API sources (Bloomberg, on-chain providers, social) are declared with
their `kind` so the collector knows to use the right adapter (stubbed until keys
are provisioned)."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class Source:
    name: str
    kind: str          # rss | api | social | onchain
    url: str
    asset_class: str   # crypto | financial | economic | social | onchain


# Free RSS feeds — collected out of the box.
SOURCES: list[Source] = [
    # ── Crypto ──
    Source("CoinDesk", "rss", "https://www.coindesk.com/arc/outboundfeeds/rss/", "crypto"),
    Source("Cointelegraph", "rss", "https://cointelegraph.com/rss", "crypto"),
    Source("Decrypt", "rss", "https://decrypt.co/feed", "crypto"),
    Source("CryptoSlate", "rss", "https://cryptoslate.com/feed/", "crypto"),
    Source("TheBlock", "rss", "https://www.theblock.co/rss.xml", "crypto"),
    Source("NewsBTC", "rss", "https://www.newsbtc.com/feed/", "crypto"),
    Source("CryptoPotato", "rss", "https://cryptopotato.com/feed/", "crypto"),
    Source("BeInCrypto", "rss", "https://beincrypto.com/feed/", "crypto"),
    Source("Bitcoinist", "rss", "https://bitcoinist.com/feed/", "crypto"),
    Source("BitcoinMagazine", "rss", "https://bitcoinmagazine.com/feed", "crypto"),
    Source("AMBCrypto", "rss", "https://ambcrypto.com/feed/", "crypto"),
    # ── Open / free JSON news APIs (no key required) ──
    Source("CryptoCompare", "json_api", "https://min-api.cryptocompare.com/data/v2/news/?lang=EN", "crypto"),
    Source("GDELT", "json_api", "https://api.gdeltproject.org/api/v2/doc/doc?query=(bitcoin%20OR%20ethereum%20OR%20cryptocurrency%20OR%20markets)&mode=ArtList&format=json&maxrecords=50&sort=DateDesc", "financial"),
    # ── Financial ──
    Source("CNBC", "rss", "https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114", "financial"),
    Source("MarketWatch", "rss", "https://feeds.content.dowjones.io/public/rss/mw_topstories", "financial"),
    Source("YahooFinance", "rss", "https://finance.yahoo.com/news/rssindex", "financial"),
    Source("Investing", "rss", "https://www.investing.com/rss/news.rss", "financial"),
    Source("SeekingAlpha", "rss", "https://seekingalpha.com/market_currents.xml", "financial"),
    # ── High-impact regulator press pages (scraped via Scrapling) ──
    # US-government works = public domain → safe to use commercially. Feed the
    # defensive event-risk signal (regulatory actions, rate/macro decisions).
    Source("SEC", "scrape", "https://www.sec.gov/news/pressreleases", "regulatory"),
    Source("CFTC", "scrape", "https://www.cftc.gov/PressRoom/PressReleases", "regulatory"),
    Source("FederalReserve", "scrape", "https://www.federalreserve.gov/newsevents/pressreleases.htm", "economic"),
    Source("USTreasury", "scrape", "https://home.treasury.gov/news/press-releases", "economic"),
    # Reuters/Bloomberg require licensed APIs — declared, adapter stubbed.
    Source("Reuters", "api", "https://api.reuters.example/markets", "financial"),
    Source("Bloomberg", "api", "https://api.bloomberg.example/markets", "financial"),
    # ── Economic ──
    Source("TradingEconomics", "api", "https://api.tradingeconomics.com/calendar", "economic"),
    Source("InvestingCalendar", "api", "https://api.investing.example/calendar", "economic"),
    # ── Social (require auth) ──
    Source("X", "social", "https://api.x.example/2/tweets/search/recent", "social"),
    Source("Reddit", "social", "https://www.reddit.com/r/CryptoCurrency/.rss", "social"),
    Source("Telegram", "social", "https://api.telegram.example/channels", "social"),
    # ── On-chain (require auth) ──
    Source("Glassnode", "onchain", "https://api.glassnode.example/v1/metrics", "onchain"),
    Source("CryptoQuant", "onchain", "https://api.cryptoquant.example/v1", "onchain"),
    Source("Santiment", "onchain", "https://api.santiment.example/graphql", "onchain"),
    Source("Coinglass", "onchain", "https://api.coinglass.example/api", "onchain"),
]

CATEGORIES = [
    "Bitcoin", "Ethereum", "Solana", "Altcoins", "ETF News", "SEC News",
    "Regulations", "Macro Economics", "Federal Reserve", "Interest Rates",
    "Inflation", "Exchange News", "Security Breaches", "Stablecoins",
    "AI & Crypto", "Institutional Adoption",
]

# keyword → category (lowercased substring match; an article may match many)
CATEGORY_KEYWORDS: dict[str, list[str]] = {
    "Bitcoin": ["bitcoin", "btc"],
    "Ethereum": ["ethereum", "eth", "ether"],
    "Solana": ["solana", "sol"],
    "Altcoins": ["altcoin", "xrp", "cardano", "dogecoin", "avalanche"],
    "ETF News": ["etf", "spot etf", "exchange-traded fund"],
    "SEC News": ["sec", "securities and exchange"],
    "Regulations": ["regulation", "regulator", "lawsuit", "ban", "compliance"],
    "Macro Economics": ["gdp", "recession", "macro", "economy"],
    "Federal Reserve": ["federal reserve", "fed ", "fomc", "powell"],
    "Interest Rates": ["interest rate", "rate cut", "rate hike", "basis points"],
    "Inflation": ["inflation", "cpi", "ppi", "consumer price"],
    "Exchange News": ["binance", "coinbase", "kraken", "exchange"],
    "Security Breaches": ["hack", "exploit", "breach", "stolen", "drained"],
    "Stablecoins": ["stablecoin", "usdt", "usdc", "tether", "circle"],
    "AI & Crypto": ["ai ", "artificial intelligence", "machine learning"],
    "Institutional Adoption": ["institutional", "blackrock", "fidelity", "microstrategy", "treasury"],
}

# Asset extraction for sentiment aggregation.
ASSET_KEYWORDS = {
    "BTC": ["bitcoin", "btc"],
    "ETH": ["ethereum", "eth", "ether"],
    "SOL": ["solana", "sol"],
    "BNB": ["binance coin", "bnb"],
    "XRP": ["xrp", "ripple"],
    "DOGE": ["dogecoin", "doge"],
    "AVAX": ["avalanche", "avax"],
    "LTC": ["litecoin", "ltc"],
    # Specific keywords only — avoid bare ambiguous tickers (link/dot/ada/atom).
    "LINK": ["chainlink"],
    "ADA": ["cardano"],
    "DOT": ["polkadot"],
    "ATOM": ["cosmos atom", "cosmos network", "cosmos hub"],
}

# Critical-event detection → flagged as market_events.
EVENT_KEYWORDS = {
    "etf_approval": ["etf approv", "approves etf", "approved bitcoin etf"],
    "etf_rejection": ["etf reject", "denies etf", "rejected etf"],
    "exchange_hack": ["hack", "exploit", "drained", "stolen funds"],
    "exchange_bankruptcy": ["bankruptcy", "insolvent", "files chapter 11", "halts withdrawals"],
    "regulation": ["sues", "lawsuit", "ban ", "crackdown", "enforcement action"],
    "rate_decision": ["rate cut", "rate hike", "raises rates", "cuts rates"],
    "cpi_data": ["cpi", "consumer price index"],
    "fed_statement": ["fomc", "powell", "federal reserve statement"],
    "black_swan": ["collapse", "contagion", "depeg", "flash crash"],
}
