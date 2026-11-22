<script setup>
import { Head, Link } from '@inertiajs/vue3';
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import CoinIcon from '../../Components/CoinIcon.vue';
import SeoHead from '../../Components/SeoHead.vue';
import CookieBanner from '../../Components/CookieBanner.vue';

defineOptions({ layout: null });

const reduce = typeof window !== 'undefined' && window.matchMedia
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches : false;
const year = new Date().getFullYear();

// Binance's public market-data mirror: no key, CORS-open, not geo-blocked.
const API = 'https://data-api.binance.vision/api/v3';
const fmt = (p) => p >= 1000 ? p.toLocaleString('en-US', { maximumFractionDigits: 0 }) : p >= 1 ? p.toFixed(2) : p.toFixed(4);
const fmt2 = (p) => p.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: p >= 1 ? 2 : 4 });

/* ── Exchanges (top marquee + logo strip) ─────────────────────────────── */
const brokers = [
    ['binance', 'Binance'], ['coinbase', 'Coinbase'], ['kraken', 'Kraken'], ['bybit', 'Bybit'],
    ['okx', 'OKX'], ['deribit', 'Deribit'], ['alpaca', 'Alpaca'], ['interactive_brokers', 'IBKR'],
    ['oanda', 'OANDA'], ['upstox', 'Upstox'], ['zerodha', 'Zerodha'], ['angelone', 'Angel One'],
];
const marquee = [...brokers, ...brokers];

/* ── Price ticker ─────────────────────────────────────────────────────── */
const coins = ['BTC', 'ETH', 'SOL', 'BNB', 'XRP', 'ADA', 'AVAX', 'DOGE', 'LINK', 'DOT'];
const tickerItems = [...coins, ...coins];
const prices = ref({});
async function loadPrices() {
    try {
        const r = await fetch(`${API}/ticker/24hr?symbols=` + encodeURIComponent(JSON.stringify(coins.map((s) => s + 'USDT'))));
        if (!r.ok) return;
        const out = {};
        (await r.json()).forEach((t) => { out[t.symbol.replace('USDT', '')] = { price: +t.lastPrice, chg: +t.priceChangePercent }; });
        prices.value = out;
    } catch (e) { /* ticker just shows symbols */ }
}

/* ── Hero card: BTC daily change, last 7 days (real) ─────────────────── */
const week = ref([]);
async function loadWeek() {
    try {
        const r = await fetch(`${API}/klines?symbol=BTCUSDT&interval=1d&limit=7`);
        if (!r.ok) return;
        week.value = (await r.json()).map((k) => ({
            day: new Date(k[0]).toLocaleDateString('en-US', { weekday: 'short' }).slice(0, 2),
            chg: ((+k[4] - +k[1]) / +k[1]) * 100,
        }));
    } catch (e) { /* card shows placeholder bars */ }
}
const weekMax = computed(() => Math.max(1, ...week.value.map((d) => Math.abs(d.chg))));

/* ── Hero card: a single order walking the risk gate ─────────────────── */
const gateRows = ['Stop-loss defined', 'Price data fresh', 'Drawdown within limit', 'Position sized'];
const gateStep = ref(reduce ? gateRows.length : 0);
let gateTimer;
function loopGate() {
    gateTimer = setInterval(() => { gateStep.value = gateStep.value > gateRows.length + 2 ? 0 : gateStep.value + 1; }, 700);
}

/* ── Big live chart ───────────────────────────────────────────────────── */
const symbols = ['BTC', 'ETH', 'SOL'];
const ranges = [
    { k: '1D', interval: '15m', limit: 96 },
    { k: '1W', interval: '1h', limit: 168 },
    { k: '1M', interval: '4h', limit: 180 },
    { k: '3M', interval: '1d', limit: 90 },
];
const sym = ref('BTC');
const range = ref('1W');
const candles = ref([]);
const chartLoading = ref(true);
async function loadChart() {
    chartLoading.value = true;
    const rg = ranges.find((r) => r.k === range.value);
    try {
        const r = await fetch(`${API}/klines?symbol=${sym.value}USDT&interval=${rg.interval}&limit=${rg.limit}`);
        if (r.ok) candles.value = (await r.json()).map((k) => ({ t: k[0], c: +k[4], v: +k[5] }));
    } catch (e) { /* keep previous series */ }
    chartLoading.value = false;
}
function pick(s, r) { if (s) sym.value = s; if (r) range.value = r; hover.value = -1; loadChart(); }

const CW = 1000, CH = 300, VH = 56;
const big = computed(() => {
    const d = candles.value;
    if (d.length < 2) return null;
    const cs = d.map((x) => x.c);
    // 20-period simple moving average: the kind of line a rule actually reads.
    const sma = cs.map((_, i) => i < 19 ? null : cs.slice(i - 19, i + 1).reduce((a, b) => a + b, 0) / 20);
    const lo = Math.min(...cs), hi = Math.max(...cs), pad = (hi - lo) * 0.08 || 1;
    const y0 = lo - pad, y1 = hi + pad;
    const X = (i) => (i / (d.length - 1)) * CW;
    const Y = (v) => (1 - (v - y0) / (y1 - y0)) * CH;
    const line = cs.map((v, i) => (i ? 'L' : 'M') + X(i).toFixed(1) + ' ' + Y(v).toFixed(1)).join(' ');
    const smaLine = sma.map((v, i) => v == null ? '' : ((i === 19 ? 'M' : 'L') + X(i).toFixed(1) + ' ' + Y(v).toFixed(1))).join(' ');
    const vmax = Math.max(...d.map((x) => x.v));
    const bw = CW / d.length;
    const vols = d.map((x, i) => ({ x: X(i) - bw * 0.35, w: bw * 0.7, h: (x.v / vmax) * VH, up: i === 0 || x.c >= d[i - 1].c }));
    const yTicks = [0, 1, 2, 3, 4].map((i) => { const v = y0 + ((y1 - y0) * i) / 4; return { y: Y(v), label: fmt(v) }; });
    const fmtT = range.value === '1D'
        ? (t) => new Date(t).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false })
        : (t) => new Date(t).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    const xTicks = [0, 1, 2, 3, 4, 5, 6].map((j) => { const i = Math.round((j / 6) * (d.length - 1)); return { x: X(i), label: fmtT(d[i].t) }; });
    const first = cs[0], last = cs[cs.length - 1];
    return { line, area: `${line} L${CW} ${CH} L0 ${CH} Z`, smaLine, vols, yTicks, xTicks, last, chg: ((last - first) / first) * 100, X, Y, fmtT };
});
const hover = ref(-1);
function onChartMove(e) {
    if (!big.value) return;
    const r = e.currentTarget.getBoundingClientRect();
    const i = Math.round(((e.clientX - r.left) / r.width) * (candles.value.length - 1));
    hover.value = Math.max(0, Math.min(candles.value.length - 1, i));
}
const hoverPt = computed(() => {
    if (hover.value < 0 || !big.value) return null;
    const c = candles.value[hover.value];
    return { x: big.value.X(hover.value), y: big.value.Y(c.c), price: c.c, time: big.value.fmtT(c.t), pct: (big.value.X(hover.value) / CW) * 100 };
});

/* ── Facts (count up) — every number here is literally true ──────────── */
const stats = ref([
    { n: 9, label: 'Risk checks per order', shown: 0 },
    { n: 12, label: 'Exchanges & brokers', shown: 0 },
    { n: 5, label: 'Built-in strategies', shown: 0 },
    { n: 0, label: 'Withdrawal access', shown: 0, word: 'Zero' },
]);
const statsEl = ref(null);
function countUp() {
    stats.value.forEach((s) => {
        if (reduce) { s.shown = s.n; return; }
        const t0 = performance.now();
        const step = (t) => { const p = Math.min(1, (t - t0) / 1400); s.shown = Math.round(s.n * (1 - Math.pow(1 - p, 3))); if (p < 1) requestAnimationFrame(step); };
        requestAnimationFrame(step);
    });
}

/* ── The nine checks (mirrors crates/risk, in order) ──────────────────── */
const checks = [
    ['Kill-switch is off', 'If you’ve hit the big red button, nothing goes through.'],
    ['Your account isn’t halted', 'Per-account pause — separate from everyone else’s.'],
    ['A stop-loss is defined', 'No stop, no order. We don’t open trades without an exit.'],
    ['The price is fresh', 'Stale market data gets the signal thrown out.'],
    ['Under your max open positions', 'You set the ceiling. The engine respects it.'],
    ['Daily drawdown not breached', 'A bad day ends early instead of becoming a bad week.'],
    ['AI confidence above the floor', 'A second opinion on context — it can veto, never force.'],
    ['Regional rules respected', 'Compliance checks for where you trade from.'],
    ['Sized to your risk', 'Position size comes from the stop distance.'],
];
const checklistEl = ref(null);
const ticked = ref(reduce ? checks.length : 0);
let tickTimer;
function runChecklist() {
    if (reduce) return;
    tickTimer = setInterval(() => { ticked.value++; if (ticked.value >= checks.length) clearInterval(tickTimer); }, 240);
}

/* ── Product screenshots ──────────────────────────────────────────────── */
const shots = [
    { key: 'markets', label: 'Market scanner', img: '/product-markets.png', cap: 'Every market scored from price action, trend, momentum and volatility. When nothing looks good, it holds cash.' },
    { key: 'dashboard', label: 'Dashboard', img: '/product-dashboard.png', cap: 'Positions, allocation and P&L on one screen. Paper account shown — illustrative, not a performance claim.' },
    { key: 'bots', label: 'Bots', img: '/product-bots.png', cap: 'Start, pause and watch each strategy as its own bot. Paper account shown — illustrative, not a performance claim.' },
];
const activeShot = ref('markets');
const current = computed(() => shots.find((s) => s.key === activeShot.value));

const safety = [
    ['Your exchange, your account', 'Funds sit on Binance, Coinbase, Kraken or wherever you already trade. They never pass through us.'],
    ['Trade-only keys', 'Create the API key with withdrawals off. We refuse any key that can withdraw.'],
    ['Keys live in a vault', 'Encrypted at rest, pulled into memory for a single order, then dropped.'],
    ['One button stops everything', 'The kill-switch closes positions and blocks new entries — from any device.'],
];

const faqs = [
    { q: 'Can TradePilot withdraw or move my money?', a: 'No. You connect API keys with trading permission only — withdrawals stay switched off on the exchange side. Your funds never leave your own account, and we can only place the orders your strategy defines.' },
    { q: 'Do I have to use real money?', a: 'No. Paper mode is free and runs any strategy on live prices with simulated funds. Go live only when you’ve seen enough.' },
    { q: 'Is it a black box?', a: 'No. Strategies are plain rules built on standard indicators (EMA, RSI, ATR, ADX). Every signal, risk check, order and fill is written to a log you can read.' },
    { q: 'Which exchanges work?', a: 'Binance, Coinbase, Kraken, Bybit, OKX, Alpaca and others — you connect your own account over API.' },
    { q: 'Will it make me money?', a: 'We can’t promise that, and you should be wary of anyone who does. Strategies have losing trades and losing months. What we promise is that the rules you pick are followed, every time, with a stop in place.' },
];
const openFaq = ref(0);

/* ── Scroll: nav, parallax, reveals ──────────────────────────────────── */
const scrolled = ref(false);
let raf = 0, io;
function onScroll() {
    if (raf) return;
    raf = requestAnimationFrame(() => {
        raf = 0;
        scrolled.value = window.scrollY > 10;
        if (reduce) return;
        const vh = window.innerHeight;
        document.querySelectorAll('[data-parallax]').forEach((el) => {
            const r = el.parentElement.getBoundingClientRect();
            if (r.bottom < 0 || r.top > vh) return;
            const p = (r.top + r.height / 2 - vh / 2) / vh;
            el.style.transform = `translate3d(0, ${(p * -(+el.dataset.parallax || 40)).toFixed(1)}px, 0)`;
        });
    });
}

let priceTimer;
onMounted(() => {
    loadPrices(); loadWeek(); loadChart();
    priceTimer = setInterval(loadPrices, 30000);
    if (!reduce) setTimeout(loopGate, 1600);
    io = new IntersectionObserver((entries) => {
        entries.forEach((e) => {
            if (!e.isIntersecting) return;
            e.target.classList.add('in');
            if (e.target === checklistEl.value) runChecklist();
            if (e.target === statsEl.value) countUp();
            io.unobserve(e.target);
        });
    }, { threshold: 0.2 });
    document.querySelectorAll('.lp [data-in]').forEach((el) => io.observe(el));
    window.addEventListener('scroll', onScroll, { passive: true });
    onScroll();
});
onBeforeUnmount(() => {
    clearInterval(priceTimer); clearInterval(tickTimer); clearInterval(gateTimer);
    io && io.disconnect();
    window.removeEventListener('scroll', onScroll);
});
</script>

<template>
    <SeoHead title="Automated Crypto Trading Bots — Non-Custodial & In Your Control"
        description="Automate crypto trading without giving up control. Non-custodial, trade-only API keys (no withdrawals), real protective stops, and a free paper mode. It's rule-based logic, not a black box."
        path="/" />
    <Head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
        <link href="https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />
        <link rel="preload" as="image" href="/img/lp/hero-neon.jpg" />
    </Head>

    <div class="lp">
        <!-- ── Exchange marquee ── -->
        <div class="strip" aria-label="Supported exchanges">
            <div class="strip-track">
                <span v-for="(b, i) in marquee" :key="i" class="strip-item">
                    <img :src="`/broker-logos/${b[0]}.png`" alt="" />{{ b[1] }}<i class="spark">✦</i>
                </span>
            </div>
        </div>

        <!-- ── Nav ── -->
        <header class="nav" :class="{ solid: scrolled }">
            <div class="wrap nav-in">
                <Link href="/" class="brand">
                    <svg class="brand-mark" viewBox="0 0 32 32" aria-hidden="true"><circle cx="16" cy="16" r="15" /><path d="M7 19c4-6 7 2 11-4s5-3 7-5" /><path d="M6 23.5c5-3 9 0 13-2.5s5-2 7-3" opacity=".55" /></svg>
                    TradePilot
                </Link>
                <nav class="menu">
                    <a href="#top" class="mi on">Home</a>
                    <a href="#market" class="mi">Live market</a>
                    <a href="#how" class="mi">How it works</a>
                    <Link href="/about" class="mi">About <span class="mi-ar">↪</span></Link>
                </nav>
                <div class="nav-r">
                    <Link href="/login" class="signin">Sign in</Link>
                    <Link href="/register" class="btn btn-sm">Get started</Link>
                </div>
            </div>
        </header>

        <!-- ── Hero ── -->
        <section id="top" class="hero">
            <div class="hero-glow" aria-hidden="true"></div>
            <h1 class="hero-h">
                <span class="hh hh-1">Trade by rules</span>
                <span class="hh hh-2" aria-hidden="true">Not by nerves</span>
                <span class="sr-only">— not by nerves.</span>
            </h1>

            <figure class="hero-photo" aria-hidden="true">
                <img src="/img/lp/hero-neon.jpg" data-parallax="24" alt="" width="1400" height="2114" />
            </figure>

            <div class="wrap hero-grid">
                <div class="callout">
                    <p class="callout-n">9</p>
                    <p class="callout-l">risk checks on every order</p>
                    <svg class="callout-line" viewBox="0 0 300 120" aria-hidden="true">
                        <path d="M0 118 H120 L290 12" pathLength="1" />
                        <circle cx="290" cy="12" r="6" />
                    </svg>
                </div>

                <div class="hero-copy">
                    <p>Automated crypto trading that runs on <b>your own exchange</b>, through keys that can trade but <b>can’t withdraw</b>. Every order needs a stop. Every decision is logged.</p>
                    <Link href="/register" class="btn btn-lg">Get started <span class="arr">→</span></Link>
                    <span class="hero-fine">Free paper mode · no card</span>
                </div>
            </div>

            <!-- floating cards -->
            <div class="wrap hero-cards">
                <div class="fcard fcard-bars">
                    <div class="fc-h"><span>BTC · daily change</span><em>7 days</em></div>
                    <div class="bars">
                        <template v-if="week.length">
                            <div v-for="(d, i) in week" :key="i" class="bar-col">
                                <span class="bar" :class="d.chg >= 0 ? 'b-up' : 'b-dn'" :style="{ height: (12 + (Math.abs(d.chg) / weekMax) * 78) + '%', '--i': i }" :title="d.chg.toFixed(2) + '%'"></span>
                                <span class="bar-d">{{ d.day }}</span>
                            </div>
                        </template>
                        <template v-else>
                            <div v-for="i in 7" :key="i" class="bar-col"><span class="bar b-ph" :style="{ height: (20 + i * 9) + '%' }"></span><span class="bar-d">·</span></div>
                        </template>
                    </div>
                </div>

                <div class="fcard fcard-gate">
                    <div class="fc-h"><span>Risk gate</span><em class="pill">Live</em></div>
                    <ul class="gate">
                        <li v-for="(g, i) in gateRows" :key="g" :class="{ ok: i < gateStep }"><i></i>{{ g }}</li>
                    </ul>
                    <p class="gate-out" :class="{ show: gateStep >= gateRows.length }">→ order sent</p>
                </div>
            </div>
        </section>

        <!-- ── Ticker ── -->
        <div class="ticker" aria-label="Live prices">
            <div class="track">
                <span v-for="(s, i) in tickerItems" :key="i" class="tk">
                    <CoinIcon :symbol="s" :size="18" /><b>{{ s }}</b>
                    <span>{{ prices[s] ? '$' + fmt(prices[s].price) : '—' }}</span>
                    <span v-if="prices[s]" :class="prices[s].chg >= 0 ? 'up' : 'dn'">{{ prices[s].chg >= 0 ? '▲' : '▼' }} {{ Math.abs(prices[s].chg).toFixed(1) }}%</span>
                </span>
            </div>
        </div>

        <!-- ── Live market chart ── -->
        <section id="market" class="wrap sec">
            <div class="sec-top" data-in>
                <h2 class="h2">Live market<br /><span class="g">&amp; real data</span></h2>
                <p class="sec-aside">This chart is pulled straight from Binance when you load the page. The faint line is a 20-period moving average — the kind of plain, inspectable input our strategies read. No mock data, no cherry-picked backtest.</p>
            </div>

            <div class="chartbox" data-in>
                <div class="cb-head">
                    <div>
                        <p class="cb-title">Chart</p>
                        <div class="cb-syms">
                            <button v-for="s in symbols" :key="s" :class="{ on: sym === s }" @click="pick(s)"><CoinIcon :symbol="s" :size="16" />{{ s }}/USDT</button>
                        </div>
                    </div>
                    <div class="cb-ranges">
                        <button v-for="r in ranges" :key="r.k" :class="{ on: range === r.k }" @click="pick(null, r.k)">{{ r.k }}</button>
                    </div>
                </div>

                <p v-if="big" class="cb-px">
                    ${{ fmt2(hoverPt ? hoverPt.price : big.last) }}
                    <span v-if="!hoverPt" :class="big.chg >= 0 ? 'up' : 'dn'">{{ big.chg >= 0 ? '+' : '' }}{{ big.chg.toFixed(2) }}% · {{ range }}</span>
                    <span v-else class="cb-time">{{ hoverPt.time }}</span>
                </p>

                <div class="cb-plot" :class="{ loading: chartLoading }" @mousemove="onChartMove" @mouseleave="hover = -1">
                    <svg v-if="big" :key="sym + range" class="cb-svg" :viewBox="`0 0 ${CW} ${CH + VH + 30}`" preserveAspectRatio="none" role="img" :aria-label="`${sym} price chart, ${range}`">
                        <defs>
                            <linearGradient id="cbArea" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="0" stop-color="#1fdc8f" stop-opacity=".32" />
                                <stop offset="1" stop-color="#1fdc8f" stop-opacity="0" />
                            </linearGradient>
                        </defs>
                        <g class="grid">
                            <line v-for="(t, i) in big.yTicks" :key="i" x1="0" :x2="CW" :y1="t.y" :y2="t.y" />
                        </g>
                        <path class="cb-area" :d="big.area" fill="url(#cbArea)" />
                        <path class="cb-sma" :d="big.smaLine" />
                        <path class="cb-line" :d="big.line" pathLength="1" />
                        <g :transform="`translate(0 ${CH + VH + 4})`">
                            <rect v-for="(v, i) in big.vols" :key="i" :x="v.x" :y="-v.h" :width="v.w" :height="v.h" :class="v.up ? 'v-up' : 'v-dn'" />
                        </g>
                        <g v-if="hoverPt">
                            <line class="xh" :x1="hoverPt.x" :x2="hoverPt.x" y1="0" :y2="CH + VH + 4" />
                            <circle class="xh-dot" :cx="hoverPt.x" :cy="hoverPt.y" r="5" />
                        </g>
                    </svg>
                    <div v-if="big" class="ylabels"><span v-for="(t, i) in big.yTicks" :key="i" :style="{ top: (t.y / (CH + VH + 30)) * 100 + '%' }">{{ t.label }}</span></div>
                    <div v-if="big" class="xlabels"><span v-for="(t, i) in big.xTicks" :key="i" :style="{ left: (t.x / CW) * 100 + '%' }">{{ t.label }}</span></div>
                    <div v-if="!big" class="cb-empty">Loading market data…</div>
                </div>
            </div>
        </section>

        <!-- ── Who it's for ── -->
        <section class="wrap sec who">
            <div class="who-text" data-in>
                <h2 class="h2">Who it’s <span class="g">for</span></h2>
                <p>For people who already trade — or want to — and are tired of doing it on adrenaline. You pick the strategy and the limits. TradePilot watches the market around the clock and only acts when every rule agrees.</p>
                <p>It won’t make you rich overnight, and we won’t pretend otherwise. It will stop you from moving a stop-loss at 2 a.m.</p>
            </div>
            <div class="who-media" data-in>
                <div class="who-frame">
                    <img src="/img/lp/trader.jpg" data-parallax="30" alt="A person in glasses watching screens, lit by green light" loading="lazy" width="1000" height="1513" />
                </div>
                <span class="badge badge-b"><CoinIcon symbol="BTC" :size="26" /></span>
                <span class="badge badge-d">$</span>
            </div>
        </section>

        <div ref="statsEl" class="wrap stats" data-in>
            <div v-for="s in stats" :key="s.label" class="stat">
                <p class="stat-n">{{ s.word && s.n === 0 ? s.word : s.shown }}</p>
                <p class="stat-l">{{ s.label }}</p>
            </div>
        </div>

        <!-- ── Nine checks ── -->
        <section id="how" class="sec band">
            <div class="wrap checks-grid">
                <div data-in>
                    <h2 class="h2">Nine checks.<br /><span class="g">Every single order.</span></h2>
                    <p class="body">A strategy only <em>suggests</em> a trade — the risk engine decides. If one check fails, the signal is dropped and the reason goes to your log.</p>
                    <p class="checks-done" :class="{ show: ticked >= checks.length }">→ All clear. Order sent to your exchange.</p>
                </div>
                <ol ref="checklistEl" class="checks" data-in>
                    <li v-for="(c, i) in checks" :key="i" :class="{ ok: i < ticked }">
                        <span class="ck-n">{{ String(i + 1).padStart(2, '0') }}</span>
                        <div><b>{{ c[0] }}</b><span>{{ c[1] }}</span></div>
                        <span class="ck-box"><svg viewBox="0 0 20 20"><path d="M4 10.5 8.2 14.5 16 5.5" /></svg></span>
                    </li>
                </ol>
            </div>
        </section>

        <!-- ── Product ── -->
        <section class="wrap sec">
            <div class="sec-top" data-in>
                <h2 class="h2">The actual <span class="g">app</span></h2>
                <p class="sec-aside">Real screenshots, not mockups. Figures come from a paper account and are illustrative only.</p>
            </div>
            <div class="tabs" data-in>
                <button v-for="s in shots" :key="s.key" class="tab" :class="{ on: activeShot === s.key }" @click="activeShot = s.key">{{ s.label }}</button>
            </div>
            <div class="shot" data-in>
                <transition name="swap" mode="out-in">
                    <img :key="current.key" :src="current.img" :alt="`TradePilot — ${current.label}`" loading="lazy" width="1440" height="1000" />
                </transition>
            </div>
            <p class="caption">{{ current.cap }}</p>
        </section>

        <!-- ── Safety ── -->
        <section class="wrap sec safety">
            <div class="safety-media" data-in>
                <img src="/img/lp/phone.jpg" data-parallax="40" alt="A phone showing crypto market prices" loading="lazy" width="1600" height="2400" />
            </div>
            <div>
                <h2 class="h2" data-in>Your money <span class="g">stays put</span></h2>
                <p class="body" data-in>This corner of the internet has a scam problem, so here’s exactly how it works.</p>
                <div class="safety-list">
                    <div v-for="(s, i) in safety" :key="i" class="sf" data-in :style="{ '--d': i * 0.08 + 's' }">
                        <span class="sf-n">{{ String(i + 1).padStart(2, '0') }}</span>
                        <div><b>{{ s[0] }}</b><p>{{ s[1] }}</p></div>
                    </div>
                </div>
            </div>
        </section>

        <!-- ── Risk ── -->
        <section class="wrap sec-tight">
            <div class="risk" data-in>
                <span class="risk-tag">Risk notice</span>
                <p>Trading is risky and you can lose some or all of the money you put in. Every strategy has losing trades and drawdowns. Anything shown from a paper account is illustrative — <b>not</b> a track record or a promise. TradePilot is software, not investment advice.</p>
                <Link href="/risk-disclosure" class="link">Full risk disclosure →</Link>
            </div>
        </section>

        <!-- ── FAQ ── -->
        <section class="wrap sec faq-sec">
            <h2 class="h2 center" data-in>Questions <span class="g">first</span></h2>
            <div class="faqs">
                <div v-for="(f, i) in faqs" :key="i" class="faq" :class="{ open: openFaq === i }" data-in>
                    <button class="faq-q" :aria-expanded="openFaq === i" @click="openFaq = openFaq === i ? -1 : i">
                        <span>{{ f.q }}</span><i class="plus" aria-hidden="true"></i>
                    </button>
                    <div class="faq-a"><div><p>{{ f.a }}</p></div></div>
                </div>
            </div>
        </section>

        <!-- ── Closing ── -->
        <section class="closing">
            <img class="closing-bg" src="/img/lp/candles.jpg" data-parallax="60" alt="" loading="lazy" aria-hidden="true" />
            <div class="wrap closing-in" data-in>
                <h2 class="closing-h">Markets don’t sleep.<br /><span class="g">Your rules don’t either.</span></h2>
                <p>Set it up on paper tonight. Read the log tomorrow. Decide from there.</p>
                <Link href="/register" class="btn btn-lg">Start in paper mode <span class="arr">→</span></Link>
            </div>
        </section>

        <!-- ── Footer ── -->
        <footer class="foot">
            <div class="wrap foot-in">
                <div>
                    <p class="brand">
                        <svg class="brand-mark" viewBox="0 0 32 32" aria-hidden="true"><circle cx="16" cy="16" r="15" /><path d="M7 19c4-6 7 2 11-4s5-3 7-5" /><path d="M6 23.5c5-3 9 0 13-2.5s5-2 7-3" opacity=".55" /></svg>
                        TradePilot
                    </p>
                    <p class="foot-tag">Automated trading software for crypto markets. Tools, not advice — you stay in control, and the trading risk is yours.</p>
                </div>
                <nav class="foot-links">
                    <Link href="/about">About</Link>
                    <Link href="/contact">Contact</Link>
                    <Link href="/terms">Terms &amp; Conditions</Link>
                    <Link href="/privacy">Privacy Policy</Link>
                    <Link href="/risk-disclosure">Risk Disclosure</Link>
                    <Link href="/login">Sign in</Link>
                </nav>
            </div>
            <div class="wrap foot-legal">
                <p>© {{ year }} TradePilot. TradePilot provides software only and is not a broker-dealer, investment adviser, or financial institution. Nothing here is financial, investment, legal, or tax advice. Trading involves substantial risk of loss.</p>
                <p>Photography from Unsplash. Market data from Binance public API.</p>
            </div>
        </footer>

        <CookieBanner />
    </div>
</template>

<style scoped>
.lp {
    --bg: #0c0e0d; --bg-2: #111413; --card: #151917; --card-2: #1b201e;
    --line: 255 255 255; --rule: rgba(255,255,255,.08); --rule-2: rgba(255,255,255,.14);
    --tx: #eef2f0; --tx-2: #a3aca8; --tx-3: #6b7470;
    --g: #1fdc8f; --g-2: #0fae6c; --g-dim: #0f3a29; --red: #f0616d; --lime: #d4f04c;
    /* theme tokens for shared children (CookieBanner, CoinIcon) */
    --surface: 12 14 13; --panel: 21 25 23; --panel-2: 27 32 30; --ink: 238 242 240; --muted: 163 172 168; --faint: 107 116 112; --accent: 31 220 143;
    --font: 'Poppins', ui-sans-serif, system-ui, sans-serif;
    --mono: 'JetBrains Mono', ui-monospace, monospace;
    background: var(--bg); color: var(--tx); font-family: var(--font); min-height: 100vh; overflow-x: clip;
}
.wrap { max-width: 1240px; margin: 0 auto; padding: 0 28px; }
@media (max-width: 640px) { .wrap { padding: 0 16px; } }
.g { color: var(--g); }
.up { color: var(--g); } .dn { color: var(--red); }
.center { text-align: center; }
.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }

/* ── exchange strip ── */
.strip { background: #080a09; border-bottom: 1px solid var(--rule); overflow: hidden; }
.strip-track { display: flex; width: max-content; animation: marquee 60s linear infinite; padding: 9px 0; }
.strip-item { display: inline-flex; align-items: center; gap: 9px; font-weight: 600; font-size: .9rem; font-style: italic; color: #cfd6d3; white-space: nowrap; }
.strip-item img { width: 20px; height: 20px; border-radius: 50%; object-fit: cover; filter: grayscale(1) brightness(1.6); }
.spark { font-style: normal; margin: 0 26px; color: var(--tx-3); }

/* ── nav ── */
.nav { position: sticky; top: 0; z-index: 40; border-bottom: 1px solid var(--rule); transition: background .3s; }
.nav.solid { background: rgba(12,14,13,.85); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); }
.nav-in { display: flex; align-items: center; height: 82px; }
.brand { display: inline-flex; align-items: center; gap: 10px; font-weight: 700; font-size: 1.35rem; letter-spacing: -.02em; color: #fff; }
.brand-mark { width: 34px; height: 34px; }
.brand-mark circle { fill: #fff; }
.brand-mark path { fill: none; stroke: var(--bg); stroke-width: 2.6; stroke-linecap: round; }
.menu { margin: 0 auto; display: flex; }
.mi { padding: 6px 30px; font-size: 1rem; color: var(--tx-2); border-left: 1px solid var(--rule-2); transition: color .2s; }
.mi:first-child { border-left: 0; }
.mi.on, .mi:hover { color: #fff; }
.mi-ar { margin-left: 8px; }
.nav-r { display: flex; align-items: center; gap: 22px; }
.signin { color: var(--tx-2); font-size: .95rem; } .signin:hover { color: #fff; }
@media (max-width: 1020px) { .menu { display: none; } .nav-r { margin-left: auto; } }
@media (max-width: 420px) { .brand { font-size: 1.15rem; } .brand-mark { width: 28px; height: 28px; } .nav-r { gap: 14px; } }

/* ── buttons ── */
.btn { display: inline-flex; align-items: center; gap: 10px; border-radius: 999px; background: var(--g); color: #04140c; font-weight: 600; padding: .8rem 1.5rem; font-size: .95rem; box-shadow: 0 0 0 0 rgba(31,220,143,.0); transition: transform .2s, box-shadow .3s, background .2s; }
.btn:hover { transform: translateY(-2px); background: #3ae9a1; box-shadow: 0 12px 40px -8px rgba(31,220,143,.55); }
.btn:active { transform: translateY(0) scale(.98); }
.btn-sm { padding: .6rem 1.2rem; font-size: .88rem; }
.btn-lg { padding: 1.15rem 2.4rem; font-size: 1.2rem; }
.arr { transition: transform .2s; } .btn:hover .arr { transform: translateX(4px); }

/* ── hero ── */
.hero { position: relative; min-height: 860px; padding-top: 70px; overflow: hidden; }
.hero-glow { position: absolute; left: 50%; top: 30%; width: 900px; height: 700px; transform: translate(-30%, -20%); background: radial-gradient(closest-side, rgba(31,220,143,.16), transparent); pointer-events: none; }
.hero-h { position: relative; z-index: 1; text-transform: uppercase; font-weight: 800; letter-spacing: -.01em; line-height: .92; padding: 0 3.5vw; }
.hh { display: block; white-space: nowrap; font-size: clamp(3rem, 9.4vw, 9.6rem); }
.hh-1 { background: linear-gradient(180deg, #43f0aa 0%, #13b873 100%); -webkit-background-clip: text; background-clip: text; color: transparent; animation: hIn 1.1s cubic-bezier(.16,1,.3,1) both .1s; }
.hh-2 { margin-top: .22em; padding-left: .45em; background: linear-gradient(180deg, rgba(31,220,143,.32), rgba(31,220,143,.02) 85%); -webkit-background-clip: text; background-clip: text; color: transparent; animation: hIn 1.1s cubic-bezier(.16,1,.3,1) both .3s; }
@keyframes hIn { from { opacity: 0; transform: translateY(40px); letter-spacing: .04em; } to { opacity: 1; transform: none; } }

.hero-photo { position: absolute; z-index: 2; left: 50%; top: 40px; width: min(600px, 48vw); height: 900px; margin: 0; transform: translateX(-46%);
    -webkit-mask-image: radial-gradient(ellipse 50% 46% at 50% 44%, #000 50%, transparent 100%); mask-image: radial-gradient(ellipse 50% 46% at 50% 44%, #000 50%, transparent 100%);
    animation: photoIn 1.6s cubic-bezier(.16,1,.3,1) both .25s; }
.hero-photo img { width: 100%; height: 100%; object-fit: cover; object-position: 50% 18%; will-change: transform; }
@keyframes photoIn { from { opacity: 0; filter: blur(14px) brightness(.4); } to { opacity: 1; filter: none; } }

.hero-grid { position: relative; z-index: 3; display: grid; grid-template-columns: 1fr 1fr; margin-top: -40px; }
.callout { position: relative; padding-top: 30px; padding-left: 90px; }
.callout-n { font-size: 4.6rem; font-weight: 600; line-height: 1; color: #fff; animation: rise .8s ease both 1.2s; }
.callout-l { font-size: 1.45rem; color: #fff; margin-top: 4px; animation: rise .8s ease both 1.3s; }
.callout-line { position: absolute; left: 160px; top: 90px; width: 300px; height: 120px; overflow: visible; }
.callout-line path { fill: none; stroke: #e6ece9; stroke-width: 2; stroke-dasharray: 1; stroke-dashoffset: 1; animation: draw 1.2s cubic-bezier(.6,.1,.3,1) forwards 1.6s; }
.callout-line circle { fill: #fff; opacity: 0; animation: fadein .3s forwards 2.6s, ping 2.2s ease-out infinite 2.9s; transform-box: fill-box; transform-origin: center; }
.hero-copy { justify-self: end; max-width: 380px; padding-top: 50px; animation: rise 1s cubic-bezier(.16,1,.3,1) both .9s; }
.hero-copy p { font-size: 1.22rem; line-height: 1.6; color: #e3e9e6; text-shadow: 0 2px 20px rgba(0,0,0,.7); }
.hero-copy b { color: #fff; font-weight: 600; }
.hero-copy .btn { margin-top: 34px; }
.hero-fine { display: block; margin-top: 14px; font-size: .82rem; color: var(--tx-3); }

.hero-cards { position: relative; z-index: 3; display: flex; justify-content: space-between; align-items: flex-end; margin-top: 30px; padding-bottom: 40px; }
.fcard { background: rgba(21,25,23,.82); backdrop-filter: blur(14px); -webkit-backdrop-filter: blur(14px); border: 1px solid var(--rule-2); border-radius: 14px; padding: 18px 20px; box-shadow: 0 30px 60px -30px #000; animation: rise 1s cubic-bezier(.16,1,.3,1) both 1.5s; }
.fc-h { display: flex; justify-content: space-between; align-items: center; gap: 24px; font-size: 1.05rem; font-weight: 500; }
.fc-h em { font-style: normal; font-size: .75rem; color: var(--tx-3); }
.fc-h .pill { background: var(--g); color: #04140c; font-weight: 600; padding: 3px 12px; border-radius: 6px; }
.fcard-bars { width: 400px; }
.bars { display: flex; gap: 14px; align-items: flex-end; height: 120px; margin-top: 16px; }
.bar-col { flex: 1; height: 100%; display: flex; flex-direction: column; justify-content: flex-end; align-items: center; gap: 6px; }
.bar { width: 100%; max-width: 26px; border-radius: 3px 3px 0 0; transform-origin: bottom; animation: grow .9s cubic-bezier(.16,1,.3,1) both; animation-delay: calc(1.8s + var(--i, 0) * .08s); }
.b-up { background: linear-gradient(180deg, var(--g), var(--g-2)); }
.b-dn { background: linear-gradient(180deg, #ffffff55, #ffffff22); }
.b-ph { background: var(--card-2); animation: none; }
.bar-d { font-size: .68rem; color: var(--tx-3); font-family: var(--mono); }
@keyframes grow { from { transform: scaleY(0); } to { transform: scaleY(1); } }
.fcard-gate { width: 300px; animation-delay: 1.7s; }
.gate { margin-top: 14px; }
.gate li { display: flex; align-items: center; gap: 10px; padding: 7px 0; font-size: .88rem; color: var(--tx-3); transition: color .3s; }
.gate li i { width: 16px; height: 16px; border-radius: 50%; border: 1.5px solid var(--tx-3); transition: all .3s; flex: none; }
.gate li.ok { color: var(--tx); }
.gate li.ok i { background: var(--g); border-color: var(--g); box-shadow: 0 0 12px rgba(31,220,143,.6); }
.gate-out { margin-top: 6px; font-family: var(--mono); font-size: .8rem; color: var(--g); opacity: 0; transition: opacity .3s; }
.gate-out.show { opacity: 1; }

@media (max-width: 980px) {
    .hero { min-height: 0; padding-top: 40px; }
    .hh { font-size: 10.4vw; }
    .hh-2 { padding-left: 0; }
    .hero-photo { position: relative; left: auto; top: auto; transform: none; width: 100%; height: 520px; margin: -30px auto 0; -webkit-mask-image: linear-gradient(180deg, transparent, #000 18%, #000 70%, transparent); mask-image: linear-gradient(180deg, transparent, #000 18%, #000 70%, transparent); }
    .hero-grid { grid-template-columns: 1fr; margin-top: -90px; }
    .callout { padding-left: 0; } .callout-line { display: none; }
    .callout-n { font-size: 3.4rem; } .callout-l { font-size: 1.15rem; }
    .hero-copy { justify-self: start; max-width: none; padding-top: 22px; }
    .hero-copy p { font-size: 1.05rem; }
    .hero-cards { flex-direction: column; align-items: stretch; gap: 14px; margin-top: 36px; }
    .fcard-bars, .fcard-gate { width: auto; }
}

/* ── ticker ── */
.ticker { border-top: 1px solid var(--rule); border-bottom: 1px solid var(--rule); background: var(--bg-2); overflow: hidden; }
.track { display: flex; gap: 40px; width: max-content; padding: 14px 0; animation: marquee 55s linear infinite; font-family: var(--mono); font-size: .82rem; }
.ticker:hover .track { animation-play-state: paused; }
.tk { display: inline-flex; align-items: center; gap: 8px; white-space: nowrap; color: var(--tx-2); }
.tk b { color: #fff; font-weight: 500; }

/* ── sections ── */
.sec { padding-top: 120px; padding-bottom: 120px; }
.sec-tight { padding-top: 40px; padding-bottom: 40px; }
.h2 { text-transform: uppercase; font-weight: 600; font-size: clamp(2rem, 4vw, 3.3rem); line-height: 1.08; letter-spacing: -.01em; }
.sec-top { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: flex-start; gap: 24px 60px; }
.sec-aside { max-width: 400px; font-size: .92rem; line-height: 1.65; color: var(--tx-2); }
.body { margin-top: 20px; max-width: 460px; line-height: 1.7; color: var(--tx-2); }

/* chart */
.chartbox { margin-top: 50px; background: linear-gradient(180deg, #121715, #0e1110); border: 1px solid var(--rule); border-radius: 18px; padding: 28px 28px 40px; }
@media (max-width: 640px) { .chartbox { padding: 18px 12px 34px; } }
.cb-head { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 16px; align-items: flex-start; }
.cb-title { font-weight: 600; font-size: 1.05rem; }
.cb-syms { margin-top: 10px; display: flex; gap: 6px; flex-wrap: wrap; }
.cb-syms button, .cb-ranges button { display: inline-flex; align-items: center; gap: 6px; font-size: .78rem; font-family: var(--mono); color: var(--tx-2); border: 1px solid var(--rule-2); padding: 6px 12px; border-radius: 8px; transition: all .2s; }
.cb-syms button:hover, .cb-ranges button:hover { color: #fff; border-color: var(--tx-3); }
.cb-syms button.on { color: #fff; border-color: var(--g); background: rgba(31,220,143,.08); }
.cb-ranges { display: flex; gap: 6px; }
.cb-ranges button { border-radius: 999px; }
.cb-ranges button.on { background: var(--g); color: #04140c; border-color: var(--g); font-weight: 500; }
.cb-px { margin-top: 18px; font-family: var(--mono); font-size: 1.6rem; font-weight: 500; display: flex; align-items: baseline; gap: 14px; flex-wrap: wrap; }
.cb-px span { font-size: .85rem; }
.cb-time { color: var(--tx-3); }
.cb-plot { position: relative; margin-top: 18px; margin-left: 58px; height: 400px; cursor: crosshair; transition: opacity .3s; }
.cb-plot.loading { opacity: .45; }
@media (max-width: 640px) { .cb-plot { height: 280px; margin-left: 44px; } }
.cb-svg { width: 100%; height: 100%; overflow: visible; display: block; }
.grid line { stroke: rgba(255,255,255,.06); stroke-width: 1; vector-effect: non-scaling-stroke; }
.cb-line { fill: none; stroke: var(--g); stroke-width: 2; vector-effect: non-scaling-stroke; stroke-linejoin: round; stroke-dasharray: 1; stroke-dashoffset: 1; animation: draw 1.8s cubic-bezier(.45,0,.2,1) forwards .1s; }
.cb-sma { fill: none; stroke: rgba(255,255,255,.35); stroke-width: 1.3; stroke-dasharray: 4 4; vector-effect: non-scaling-stroke; opacity: 0; animation: fadein .8s forwards 1.4s; }
.cb-area { opacity: 0; animation: fadein 1s forwards .9s; }
.v-up { fill: rgba(31,220,143,.45); } .v-dn { fill: rgba(255,255,255,.14); }
.xh { stroke: rgba(255,255,255,.3); stroke-dasharray: 3 3; vector-effect: non-scaling-stroke; }
.xh-dot { fill: var(--bg); stroke: var(--g); stroke-width: 2.5; vector-effect: non-scaling-stroke; }
.ylabels span { position: absolute; left: -58px; width: 52px; text-align: right; transform: translateY(-50%); font-family: var(--mono); font-size: .68rem; color: var(--tx-3); }
@media (max-width: 640px) { .ylabels span { left: -46px; width: 42px; font-size: .6rem; } }
.xlabels span { position: absolute; bottom: -26px; transform: translateX(-50%); font-family: var(--mono); font-size: .68rem; color: var(--tx-3); white-space: nowrap; }
.xlabels span:first-child { transform: none; } .xlabels span:last-child { transform: translateX(-100%); }
@media (max-width: 640px) { .xlabels span:nth-child(even) { display: none; } }
.cb-empty { position: absolute; inset: 0; display: grid; place-items: center; color: var(--tx-3); font-size: .9rem; }

/* who */
.who { display: grid; grid-template-columns: 1.15fr .85fr; gap: 70px; align-items: center; padding-bottom: 70px; }
.who-text p { margin-top: 22px; max-width: 560px; line-height: 1.75; color: var(--tx-2); }
.who-media { position: relative; max-width: 380px; justify-self: end; width: 100%; }
.who-frame { aspect-ratio: 4 / 5; border-radius: 14px; overflow: hidden; background: linear-gradient(135deg, var(--g-2), var(--lime)); padding: 0; box-shadow: 0 40px 80px -40px rgba(31,220,143,.45); }
.who-frame img { width: 100%; height: 115%; object-fit: cover; margin-top: -7%; mix-blend-mode: normal; will-change: transform; }
.badge { position: absolute; display: grid; place-items: center; width: 44px; height: 44px; border-radius: 10px; background: #fff; color: #0c0e0d; font-weight: 700; font-size: 1.15rem; box-shadow: 0 14px 30px -10px rgba(0,0,0,.6); }
.badge-b { top: -16px; right: -14px; transform: rotate(12deg); animation: bob 5s ease-in-out infinite; }
.badge-d { bottom: 60px; left: -20px; transform: rotate(-14deg); background: var(--g); animation: bob 6s ease-in-out infinite .8s; }
@keyframes bob { 0%,100% { translate: 0 -5px; } 50% { translate: 0 5px; } }
@media (max-width: 860px) { .who { grid-template-columns: 1fr; gap: 50px; } .who-media { justify-self: center; } }

.stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; padding-bottom: 110px; }
.stat { text-align: center; padding: 10px; border-left: 1px solid var(--rule); }
.stat:first-child { border-left: 0; }
.stat-n { font-size: clamp(2.2rem, 4vw, 3rem); font-weight: 600; color: var(--g); }
.stat-l { margin-top: 4px; font-size: .9rem; color: var(--tx-2); }
@media (max-width: 760px) { .stats { grid-template-columns: repeat(2, 1fr); row-gap: 34px; } .stat:nth-child(3) { border-left: 0; } }

/* checks */
.band { background: var(--bg-2); border-top: 1px solid var(--rule); border-bottom: 1px solid var(--rule); }
.checks-grid { display: grid; grid-template-columns: .85fr 1.15fr; gap: 70px; align-items: start; }
.checks-grid > div:first-child { position: sticky; top: 120px; }
@media (max-width: 900px) { .checks-grid { grid-template-columns: 1fr; gap: 40px; } .checks-grid > div:first-child { position: static; } }
.checks-done { margin-top: 26px; font-family: var(--mono); font-size: .88rem; color: var(--g); opacity: 0; transition: opacity .5s; }
.checks-done.show, .checks-done.in.show { opacity: 1; }
.checks li { display: grid; grid-template-columns: 44px 1fr 26px; gap: 16px; align-items: center; padding: 18px 20px; border: 1px solid var(--rule); border-radius: 12px; margin-bottom: 10px; background: var(--card); transition: border-color .35s, background .35s; }
.checks li.ok { border-color: rgba(31,220,143,.35); background: linear-gradient(90deg, rgba(31,220,143,.07), var(--card) 60%); }
.ck-n { font-family: var(--mono); font-size: .8rem; color: var(--tx-3); }
.checks b { display: block; font-weight: 500; font-size: 1rem; }
.checks li div span { display: block; margin-top: 3px; font-size: .85rem; color: var(--tx-3); }
.ck-box { width: 26px; height: 26px; border-radius: 50%; border: 1.5px solid var(--rule-2); display: grid; place-items: center; transition: all .3s; }
.ck-box svg { width: 15px; height: 15px; }
.ck-box path { fill: none; stroke: #04140c; stroke-width: 2.6; stroke-linecap: round; stroke-linejoin: round; stroke-dasharray: 20; stroke-dashoffset: 20; transition: stroke-dashoffset .3s .05s; }
.checks li.ok .ck-box { background: var(--g); border-color: var(--g); box-shadow: 0 0 14px rgba(31,220,143,.5); }
.checks li.ok .ck-box path { stroke-dashoffset: 0; }

/* product */
.tabs { margin-top: 40px; display: flex; gap: 8px; flex-wrap: wrap; }
.tab { padding: 9px 20px; border-radius: 999px; border: 1px solid var(--rule-2); color: var(--tx-2); font-size: .9rem; transition: all .2s; }
.tab:hover { color: #fff; }
.tab.on { background: var(--g); border-color: var(--g); color: #04140c; font-weight: 500; }
.shot { margin-top: 26px; border: 1px solid var(--rule-2); border-radius: 16px; overflow: hidden; background: var(--card); box-shadow: 0 50px 100px -50px rgba(31,220,143,.3); }
.shot img { display: block; width: 100%; height: auto; }
.caption { margin-top: 16px; color: var(--tx-3); font-size: .9rem; }
.swap-enter-active, .swap-leave-active { transition: opacity .25s, transform .25s; }
.swap-enter-from { opacity: 0; transform: scale(.99); } .swap-leave-to { opacity: 0; }

/* safety */
.safety { display: grid; grid-template-columns: .8fr 1.2fr; gap: 80px; align-items: center; padding-top: 30px; }
.safety-media { aspect-ratio: 3 / 4; border-radius: 16px; overflow: hidden; position: relative; }
.safety-media img { width: 100%; height: 118%; object-fit: cover; margin-top: -9%; filter: saturate(.6) hue-rotate(-20deg); will-change: transform; }
.safety-media::after { content: ''; position: absolute; inset: 0; background: linear-gradient(180deg, transparent 50%, rgba(12,14,13,.7)), linear-gradient(135deg, rgba(31,220,143,.18), transparent 60%); }
.safety-list { margin-top: 34px; }
.sf { display: flex; gap: 22px; padding: 22px 0; border-top: 1px solid var(--rule); }
.sf-n { font-family: var(--mono); color: var(--g); font-size: .85rem; padding-top: 3px; }
.sf b { font-weight: 500; font-size: 1.12rem; }
.sf p { margin-top: 6px; color: var(--tx-2); line-height: 1.6; font-size: .95rem; }
@media (max-width: 900px) { .safety { grid-template-columns: 1fr; gap: 50px; } .safety-media { max-width: 420px; } }

/* risk */
.risk { border: 1px solid rgba(240,180,60,.3); background: rgba(240,180,60,.05); border-radius: 16px; padding: 30px 34px; display: grid; grid-template-columns: auto 1fr auto; gap: 30px; align-items: center; }
.risk-tag { font-family: var(--mono); font-size: .72rem; letter-spacing: .12em; text-transform: uppercase; color: #f0b43c; border: 1px solid rgba(240,180,60,.5); padding: 6px 12px; border-radius: 6px; white-space: nowrap; }
.risk p { color: var(--tx-2); line-height: 1.65; font-size: .92rem; }
.risk b { color: #fff; }
.link { color: #fff; font-weight: 500; font-size: .9rem; white-space: nowrap; border-bottom: 1px solid var(--g); padding-bottom: 2px; }
@media (max-width: 860px) { .risk { grid-template-columns: 1fr; gap: 16px; padding: 24px 20px; } .risk-tag { justify-self: start; } }

/* faq */
.faqs { max-width: 860px; margin: 50px auto 0; }
.faq { border: 1px solid var(--rule); border-radius: 14px; margin-bottom: 10px; background: var(--card); transition: border-color .3s; }
.faq.open { border-color: rgba(31,220,143,.35); }
.faq-q { width: 100%; display: flex; justify-content: space-between; align-items: center; gap: 20px; padding: 22px 24px; text-align: left; font-size: 1.05rem; font-weight: 500; color: #fff; }
.plus { position: relative; flex: none; width: 30px; height: 30px; border-radius: 50%; background: var(--card-2); transition: background .3s; }
.plus::before, .plus::after { content: ''; position: absolute; left: 9px; top: 14px; width: 12px; height: 2px; background: #fff; transition: transform .35s cubic-bezier(.16,1,.3,1); }
.plus::after { transform: rotate(90deg); }
.faq.open .plus { background: var(--g); }
.faq.open .plus::before, .faq.open .plus::after { background: #04140c; }
.faq.open .plus::after { transform: rotate(0); }
.faq-a { display: grid; grid-template-rows: 0fr; transition: grid-template-rows .4s cubic-bezier(.16,1,.3,1); }
.faq.open .faq-a { grid-template-rows: 1fr; }
.faq-a > div { overflow: hidden; }
.faq-a p { padding: 0 24px 24px; color: var(--tx-2); line-height: 1.7; }

/* closing */
.closing { position: relative; overflow: hidden; border-top: 1px solid var(--rule); }
.closing-bg { position: absolute; left: 0; top: -10%; width: 100%; height: 120%; object-fit: cover; opacity: .28; filter: hue-rotate(-15deg) saturate(.8); will-change: transform; }
.closing::after { content: ''; position: absolute; inset: 0; background: radial-gradient(ellipse at 20% 50%, rgba(12,14,13,.6), rgba(12,14,13,.95) 80%); }
.closing-in { position: relative; z-index: 1; padding-top: 150px; padding-bottom: 150px; }
.closing-h { text-transform: uppercase; font-weight: 700; font-size: clamp(2.2rem, 5.2vw, 4.6rem); line-height: 1.02; }
.closing-in p { margin: 24px 0 38px; color: var(--tx-2); font-size: 1.1rem; }

/* footer */
.foot { border-top: 1px solid var(--rule); padding: 60px 0 30px; background: #090b0a; }
.foot-in { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 34px; }
.foot-tag { margin-top: 14px; max-width: 360px; font-size: .9rem; line-height: 1.65; color: var(--tx-2); }
.foot-links { display: grid; grid-template-columns: repeat(2, auto); gap: 12px 56px; font-size: .9rem; color: var(--tx-2); }
.foot-links a:hover { color: var(--g); }
.foot-legal { margin-top: 44px; padding-top: 22px; border-top: 1px solid var(--rule); font-size: .76rem; line-height: 1.65; color: var(--tx-3); }
.foot-legal p + p { margin-top: 6px; }

/* cookie banner: green instead of the app's violet */
.lp :deep(.btn-accept) { background: var(--g); color: #04140c; box-shadow: none; }

/* ── motion ── */
[data-in] { opacity: 0; transform: translateY(26px); transition: opacity .9s cubic-bezier(.16,1,.3,1) var(--d, 0s), transform .9s cubic-bezier(.16,1,.3,1) var(--d, 0s); }
[data-in].in { opacity: 1; transform: none; }
.who-media[data-in] .who-frame, .safety-media[data-in] { clip-path: inset(100% 0 0 0 round 14px); transition: clip-path 1.3s cubic-bezier(.7,0,.2,1) .1s, opacity .9s, transform .9s; }
.who-media[data-in].in .who-frame, .safety-media[data-in].in { clip-path: inset(0 0 0 0 round 14px); }
@keyframes rise { from { opacity: 0; transform: translateY(24px); } to { opacity: 1; transform: none; } }
@keyframes draw { to { stroke-dashoffset: 0; } }
@keyframes fadein { to { opacity: 1; } }
@keyframes ping { 0% { transform: scale(1); } 60% { transform: scale(1.8); opacity: .3; } 100% { transform: scale(1); opacity: 1; } }
@keyframes marquee { to { transform: translateX(-50%); } }

@media (prefers-reduced-motion: reduce) {
    .hh-1, .hh-2, .hero-photo, .callout-n, .callout-l, .hero-copy, .fcard, .bar, .cb-area, .cb-sma, .callout-line circle, .badge { animation: none !important; opacity: 1 !important; }
    .callout-line path, .cb-line { animation: none; stroke-dashoffset: 0; }
    .strip-track, .track { animation: none; }
    [data-in], .who-media[data-in] .who-frame, .safety-media[data-in] { opacity: 1; transform: none; clip-path: none; transition: none; }
}
</style>
