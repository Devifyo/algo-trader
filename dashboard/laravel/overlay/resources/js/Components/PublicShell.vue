<script setup>
import { Head, Link, usePage } from '@inertiajs/vue3';
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import CookieBanner from './CookieBanner.vue';

const year = new Date().getFullYear();
const path = computed(() => (usePage().url || '/').split('?')[0].split('#')[0]);

const brokers = [
    ['binance', 'Binance'], ['coinbase', 'Coinbase'], ['kraken', 'Kraken'], ['bybit', 'Bybit'],
    ['okx', 'OKX'], ['deribit', 'Deribit'], ['alpaca', 'Alpaca'], ['interactive_brokers', 'IBKR'],
    ['oanda', 'OANDA'], ['upstox', 'Upstox'], ['zerodha', 'Zerodha'], ['angelone', 'Angel One'],
];
const marquee = [...brokers, ...brokers];
const menu = [
    { href: '/', label: 'Home' },
    { href: '/#market', label: 'Live market' },
    { href: '/#how', label: 'How it works' },
    { href: '/about', label: 'About' },
    { href: '/contact', label: 'Contact' },
];
const isOn = (href) => href === path.value;

const scrolled = ref(false);
const progress = ref(0);
const open = ref(false);
function onScroll() {
    const st = window.scrollY;
    scrolled.value = st > 10;
    const h = document.documentElement.scrollHeight - window.innerHeight;
    progress.value = h > 0 ? Math.min(100, (st / h) * 100) : 0;
}

let io;
onMounted(() => {
    onScroll();
    window.addEventListener('scroll', onScroll, { passive: true });
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    io = new IntersectionObserver((es) => es.forEach((e) => { if (e.isIntersecting) { e.target.classList.add('in'); io.unobserve(e.target); } }), { threshold: 0.15 });
    document.querySelectorAll('.ps [data-in]').forEach((el) => reduce ? el.classList.add('in') : io.observe(el));
});
onBeforeUnmount(() => { window.removeEventListener('scroll', onScroll); io && io.disconnect(); });
</script>

<template>
    <Head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
        <link href="https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />
    </Head>

    <div class="ps">
        <div class="ps-progress" :style="{ width: progress + '%' }" aria-hidden="true"></div>

        <div class="ps-strip" aria-label="Supported exchanges">
            <div class="ps-strip-track">
                <span v-for="(b, i) in marquee" :key="i" class="ps-strip-item">
                    <img :src="`/broker-logos/${b[0]}.png`" alt="" />{{ b[1] }}<i>✦</i>
                </span>
            </div>
        </div>

        <header class="ps-nav" :class="{ solid: scrolled || open }">
            <div class="ps-wrap ps-nav-in">
                <Link href="/" class="ps-brand">
                    <svg class="ps-mark" viewBox="0 0 32 32" aria-hidden="true"><circle cx="16" cy="16" r="15" /><path d="M7 19c4-6 7 2 11-4s5-3 7-5" /><path d="M6 23.5c5-3 9 0 13-2.5s5-2 7-3" opacity=".55" /></svg>
                    TradePilot
                </Link>
                <nav class="ps-menu">
                    <Link v-for="m in menu" :key="m.href" :href="m.href" class="ps-mi" :class="{ on: isOn(m.href) }">{{ m.label }}</Link>
                </nav>
                <div class="ps-right">
                    <Link href="/login" class="ps-signin">Sign in</Link>
                    <Link href="/register" class="ps-btn">Get started</Link>
                    <button class="ps-burger" :class="{ x: open }" :aria-expanded="open" aria-label="Menu" @click="open = !open"><i></i><i></i></button>
                </div>
            </div>
            <transition name="ps-drop">
                <nav v-if="open" class="ps-mobile">
                    <Link v-for="m in menu" :key="m.href" :href="m.href" class="ps-mm" :class="{ on: isOn(m.href) }" @click="open = false">{{ m.label }}</Link>
                    <Link href="/login" class="ps-mm" @click="open = false">Sign in</Link>
                </nav>
            </transition>
        </header>

        <main><slot /></main>

        <footer class="ps-foot">
            <div class="ps-wrap ps-foot-cta" data-in>
                <p>Try it on paper first. <span>It’s free, and nothing real is at stake.</span></p>
                <Link href="/register" class="ps-btn ps-btn-lg">Start in paper mode <span aria-hidden="true">→</span></Link>
            </div>
            <div class="ps-wrap ps-foot-in">
                <div>
                    <p class="ps-brand">
                        <svg class="ps-mark" viewBox="0 0 32 32" aria-hidden="true"><circle cx="16" cy="16" r="15" /><path d="M7 19c4-6 7 2 11-4s5-3 7-5" /><path d="M6 23.5c5-3 9 0 13-2.5s5-2 7-3" opacity=".55" /></svg>
                        TradePilot
                    </p>
                    <p class="ps-tag">Automated trading software for crypto markets. Tools, not advice — you stay in control, and the trading risk is yours.</p>
                </div>
                <div class="ps-cols">
                    <div>
                        <p class="ps-col-h">Product</p>
                        <Link href="/#market">Live market</Link>
                        <Link href="/#how">How it works</Link>
                        <Link href="/register">Get started</Link>
                        <Link href="/login">Sign in</Link>
                    </div>
                    <div>
                        <p class="ps-col-h">Company</p>
                        <Link href="/about">About</Link>
                        <Link href="/contact">Contact</Link>
                    </div>
                    <div>
                        <p class="ps-col-h">Legal</p>
                        <Link href="/terms">Terms &amp; Conditions</Link>
                        <Link href="/privacy">Privacy Policy</Link>
                        <Link href="/risk-disclosure">Risk Disclosure</Link>
                    </div>
                </div>
            </div>
            <div class="ps-wrap ps-legal">
                <p>© {{ year }} TradePilot. TradePilot provides software only and is not a broker-dealer, investment adviser, or financial institution. Nothing here is financial, investment, legal, or tax advice. Trading involves substantial risk of loss.</p>
                <p>Photography from Unsplash. Market data from Binance public API.</p>
            </div>
        </footer>

        <CookieBanner />
    </div>
</template>

<style>
/* Unscoped on purpose: these tokens and helpers are shared by every public
   page rendered inside the shell (About, Contact, legal docs, Landing). */
.ps {
    --bg: #0c0e0d; --bg-2: #111413; --card: #151917; --card-2: #1b201e;
    --rule: rgba(255,255,255,.08); --rule-2: rgba(255,255,255,.14);
    --tx: #eef2f0; --tx-2: #a3aca8; --tx-3: #6b7470;
    --g: #1fdc8f; --g-2: #0fae6c; --red: #f0616d; --amber: #f0b43c; --lime: #d4f04c;
    --surface: 12 14 13; --panel: 21 25 23; --panel-2: 27 32 30; --line: 40 46 43; --line-strong: 60 68 64;
    --ink: 238 242 240; --muted: 163 172 168; --faint: 107 116 112; --accent: 31 220 143; --warn: 240 180 60;
    --font: 'Poppins', ui-sans-serif, system-ui, sans-serif;
    --mono: 'JetBrains Mono', ui-monospace, monospace;
    background: var(--bg); color: var(--tx); font-family: var(--font); min-height: 100vh; overflow-x: clip;
}
.ps .ps-wrap { max-width: 1240px; margin-left: auto; margin-right: auto; padding-left: 28px; padding-right: 28px; }
@media (max-width: 640px) { .ps .ps-wrap { padding-left: 16px; padding-right: 16px; } }
.ps .g { color: var(--g); }
.ps [data-in] { opacity: 0; transform: translateY(24px); transition: opacity .9s cubic-bezier(.16,1,.3,1) var(--d, 0s), transform .9s cubic-bezier(.16,1,.3,1) var(--d, 0s); }
.ps [data-in].in { opacity: 1; transform: none; }
.ps :is(.btn-g, .ps-btn) { display: inline-flex; align-items: center; justify-content: center; gap: 10px; border-radius: 999px; background: var(--g); color: #04140c; font-weight: 600; padding: .65rem 1.25rem; font-size: .9rem; transition: transform .2s, box-shadow .3s, background .2s; }
.ps :is(.btn-g, .ps-btn):hover { transform: translateY(-2px); background: #3ae9a1; box-shadow: 0 12px 40px -8px rgba(31,220,143,.55); }
.ps .btn-ghost { display: inline-flex; align-items: center; gap: 10px; border-radius: 999px; border: 1px solid var(--rule-2); color: var(--tx); font-weight: 500; padding: .65rem 1.25rem; font-size: .9rem; transition: border-color .2s, background .2s; }
.ps .btn-ghost:hover { border-color: var(--tx-3); background: rgba(255,255,255,.04); }
.ps .cookie-inner .btn-accept { background: var(--g); color: #04140c; box-shadow: none; }

.ps-progress { position: fixed; top: 0; left: 0; height: 2px; z-index: 60; background: var(--g); box-shadow: 0 0 10px var(--g); transition: width .1s linear; }

.ps-strip { background: #080a09; border-bottom: 1px solid var(--rule); overflow: hidden; }
.ps-strip-track { display: flex; width: max-content; animation: ps-marquee 60s linear infinite; padding: 9px 0; }
.ps-strip-item { display: inline-flex; align-items: center; gap: 9px; font-weight: 600; font-size: .9rem; font-style: italic; color: #cfd6d3; white-space: nowrap; }
.ps-strip-item img { width: 20px; height: 20px; border-radius: 50%; object-fit: cover; filter: grayscale(1) brightness(1.6); }
.ps-strip-item i { font-style: normal; margin: 0 26px; color: var(--tx-3); }
@keyframes ps-marquee { to { transform: translateX(-50%); } }

.ps-nav { position: sticky; top: 0; z-index: 40; border-bottom: 1px solid var(--rule); transition: background .3s; }
.ps-nav.solid { background: rgba(12,14,13,.88); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); }
.ps-nav-in { display: flex; align-items: center; height: 78px; }
.ps-brand { display: inline-flex; align-items: center; gap: 10px; font-weight: 700; font-size: 1.3rem; letter-spacing: -.02em; color: #fff; }
.ps-mark { width: 32px; height: 32px; flex: none; }
.ps-mark circle { fill: #fff; }
.ps-mark path { fill: none; stroke: var(--bg); stroke-width: 2.6; stroke-linecap: round; }
.ps-menu { margin: 0 auto; display: flex; }
.ps-mi { padding: 6px 24px; font-size: .95rem; color: var(--tx-2); border-left: 1px solid var(--rule-2); transition: color .2s; }
.ps-mi:first-child { border-left: 0; }
.ps-mi.on, .ps-mi:hover { color: #fff; }
.ps-right { display: flex; align-items: center; gap: 20px; }
.ps-signin { color: var(--tx-2); font-size: .92rem; } .ps-signin:hover { color: #fff; }
.ps-burger { display: none; width: 40px; height: 40px; border-radius: 10px; border: 1px solid var(--rule-2); position: relative; }
.ps-burger i { position: absolute; left: 11px; width: 18px; height: 1.8px; background: #fff; transition: transform .3s, top .3s; }
.ps-burger i:first-child { top: 15px; } .ps-burger i:last-child { top: 23px; }
.ps-burger.x i:first-child { top: 19px; transform: rotate(45deg); } .ps-burger.x i:last-child { top: 19px; transform: rotate(-45deg); }
.ps-mobile { display: grid; padding: 8px 16px 18px; border-top: 1px solid var(--rule); }
.ps-mm { padding: 14px 4px; border-bottom: 1px solid var(--rule); color: var(--tx-2); font-size: 1rem; }
.ps-mm.on { color: var(--g); }
.ps-drop-enter-active, .ps-drop-leave-active { transition: opacity .2s, transform .2s; }
.ps-drop-enter-from, .ps-drop-leave-to { opacity: 0; transform: translateY(-6px); }
@media (max-width: 1060px) { .ps-menu { display: none; } .ps-right { margin-left: auto; } .ps-burger { display: block; } }
@media (max-width: 520px) { .ps-signin { display: none; } .ps-brand { font-size: 1.15rem; } .ps-right { gap: 10px; } .ps-nav-in > .ps-right > .ps-btn { padding: .55rem 1rem; font-size: .84rem; } }
@media (min-width: 1061px) { .ps-mobile { display: none; } }

.ps-foot { border-top: 1px solid var(--rule); background: #090b0a; }
.ps-foot-cta { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 24px; padding-top: 64px; padding-bottom: 64px; border-bottom: 1px solid var(--rule); }
.ps-foot-cta p { font-size: clamp(1.5rem, 3vw, 2.2rem); font-weight: 600; line-height: 1.2; max-width: 640px; }
.ps-foot-cta p span { color: var(--g); }
.ps .ps-btn-lg { padding: 1rem 1.9rem; font-size: 1.02rem; }
.ps-foot-in { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 40px; padding-top: 52px; }
.ps-tag { margin-top: 14px; max-width: 340px; font-size: .9rem; line-height: 1.65; color: var(--tx-2); }
.ps-cols { display: flex; flex-wrap: wrap; gap: 36px 72px; }
.ps-cols div { display: grid; gap: 10px; align-content: start; font-size: .9rem; }
.ps-cols a { color: var(--tx-2); transition: color .2s; }
.ps-cols a:hover { color: var(--g); }
.ps-col-h { font-family: var(--mono); font-size: .72rem; letter-spacing: .1em; text-transform: uppercase; color: var(--tx-3); margin-bottom: 4px; }
.ps-legal { margin-top: 44px; padding-top: 22px; padding-bottom: 30px; border-top: 1px solid var(--rule); font-size: .76rem; line-height: 1.65; color: var(--tx-3); }
.ps-legal p + p { margin-top: 6px; }

@media (prefers-reduced-motion: reduce) {
    .ps-strip-track { animation: none; }
    .ps [data-in] { opacity: 1; transform: none; transition: none; }
}
</style>
