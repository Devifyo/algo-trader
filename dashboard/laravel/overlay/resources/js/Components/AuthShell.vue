<script setup>
import { Head, Link } from '@inertiajs/vue3';
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';

defineProps({
    title: { type: String, required: true },
    quote: { type: String, default: 'Trade by rules,' },
    quoteG: { type: String, default: 'not by nerves.' },
});

// Live BTC price for the side panel (Binance public mirror, no key).
const closes = ref([]);
let timer;
async function load() {
    try {
        const r = await fetch('https://data-api.binance.vision/api/v3/klines?symbol=BTCUSDT&interval=1h&limit=48');
        if (r.ok) closes.value = (await r.json()).map((k) => +k[4]);
    } catch (e) { /* panel shows without the chart */ }
}
const spark = computed(() => {
    const c = closes.value;
    if (c.length < 2) return null;
    const lo = Math.min(...c), hi = Math.max(...c), span = hi - lo || 1;
    const line = c.map((v, i) => (i ? 'L' : 'M') + ((i / (c.length - 1)) * 200).toFixed(1) + ' ' + (4 + (1 - (v - lo) / span) * 52).toFixed(1)).join(' ');
    const last = c[c.length - 1];
    return { line, area: `${line} L200 60 L0 60 Z`, last, chg: ((last - c[0]) / c[0]) * 100 };
});
onMounted(() => { load(); timer = setInterval(load, 60000); });
onBeforeUnmount(() => clearInterval(timer));
</script>

<template>
    <Head :title="title">
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
        <link href="https://fonts.googleapis.com/css2?family=Poppins:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet" />
        <meta name="robots" content="noindex" />
    </Head>

    <div class="auth">
        <!-- ── Visual side ── -->
        <aside class="side" aria-hidden="true">
            <img class="side-img" src="/img/lp/hero-neon.jpg" alt="" />
            <div class="side-shade"></div>

            <Link href="/" class="brand brand-side">
                <svg class="brand-mark" viewBox="0 0 32 32"><circle cx="16" cy="16" r="15" /><path d="M7 19c4-6 7 2 11-4s5-3 7-5" /><path d="M6 23.5c5-3 9 0 13-2.5s5-2 7-3" opacity=".55" /></svg>
                TradePilot
            </Link>

            <div class="side-body">
                <p class="side-h">{{ quote }}<br /><span class="g">{{ quoteG }}</span></p>

                <div class="spark-card">
                    <div class="sc-top">
                        <span>BTC / USDT · 48h</span>
                        <span v-if="spark" :class="spark.chg >= 0 ? 'up' : 'dn'">{{ spark.chg >= 0 ? '+' : '' }}{{ spark.chg.toFixed(2) }}%</span>
                    </div>
                    <p class="sc-px">{{ spark ? '$' + spark.last.toLocaleString('en-US', { maximumFractionDigits: 0 }) : '—' }}</p>
                    <svg v-if="spark" viewBox="0 0 200 60" preserveAspectRatio="none" class="sc-svg">
                        <defs><linearGradient id="authSpark" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1fdc8f" stop-opacity=".35" /><stop offset="1" stop-color="#1fdc8f" stop-opacity="0" /></linearGradient></defs>
                        <path :d="spark.area" fill="url(#authSpark)" />
                        <path :d="spark.line" class="sc-line" pathLength="1" />
                    </svg>
                </div>

                <ul class="side-facts">
                    <li><i></i>Trade-only API keys — we can’t withdraw</li>
                    <li><i></i>Every order needs a stop-loss</li>
                    <li><i></i>Free paper mode on live prices</li>
                </ul>
            </div>
        </aside>

        <!-- ── Form side ── -->
        <main class="main">
            <header class="main-top">
                <Link href="/" class="brand brand-main">
                    <svg class="brand-mark" viewBox="0 0 32 32" aria-hidden="true"><circle cx="16" cy="16" r="15" /><path d="M7 19c4-6 7 2 11-4s5-3 7-5" /><path d="M6 23.5c5-3 9 0 13-2.5s5-2 7-3" opacity=".55" /></svg>
                    TradePilot
                </Link>
                <Link href="/" class="back">← Back to site</Link>
            </header>

            <div class="main-in">
                <slot />
            </div>

            <footer class="main-foot">
                <Link href="/terms">Terms</Link>
                <Link href="/privacy">Privacy</Link>
                <Link href="/risk-disclosure">Risk disclosure</Link>
            </footer>
        </main>
    </div>
</template>

<style scoped>
.auth {
    --bg: #0c0e0d; --card: #151917; --tx: #eef2f0; --tx-2: #a3aca8; --tx-3: #6b7470;
    --g: #1fdc8f; --rule: rgba(255,255,255,.08); --rule-2: rgba(255,255,255,.14); --red: #f0616d;
    min-height: 100vh; display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    background: var(--bg); color: var(--tx); font-family: 'Poppins', ui-sans-serif, system-ui, sans-serif;
}
@media (max-width: 960px) { .auth { grid-template-columns: 1fr; } }
.g { color: var(--g); } .up { color: var(--g); } .dn { color: var(--red); }

.brand { display: inline-flex; align-items: center; gap: 10px; font-weight: 700; font-size: 1.25rem; letter-spacing: -.02em; color: #fff; }
.brand-mark { width: 32px; height: 32px; }
.brand-mark circle { fill: #fff; }
.brand-mark path { fill: none; stroke: var(--bg); stroke-width: 2.6; stroke-linecap: round; }

/* side */
.side { position: sticky; top: 0; height: 100vh; overflow: hidden; display: flex; flex-direction: column; padding: 36px 44px; border-right: 1px solid var(--rule); }
.side-img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; object-position: 50% 20%; animation: sideIn 1.6s cubic-bezier(.16,1,.3,1) both; }
@keyframes sideIn { from { opacity: 0; transform: scale(1.08); filter: blur(10px); } to { opacity: 1; transform: none; filter: none; } }
.side-shade { position: absolute; inset: 0; background: linear-gradient(180deg, rgba(12,14,13,.55) 0%, rgba(12,14,13,.05) 30%, rgba(12,14,13,.55) 62%, rgba(12,14,13,.96) 100%); }
.brand-side, .side-body { position: relative; z-index: 1; }
.side-body { margin-top: auto; animation: up .9s cubic-bezier(.16,1,.3,1) both .4s; }
.side-h { text-transform: uppercase; font-weight: 700; font-size: clamp(2rem, 3.2vw, 3rem); line-height: 1.04; }
.spark-card { margin-top: 28px; max-width: 340px; background: rgba(21,25,23,.7); backdrop-filter: blur(14px); -webkit-backdrop-filter: blur(14px); border: 1px solid var(--rule-2); border-radius: 14px; padding: 16px 18px 12px; }
.sc-top { display: flex; justify-content: space-between; font-size: .78rem; color: var(--tx-2); font-family: 'JetBrains Mono', monospace; }
.sc-px { margin-top: 6px; font-family: 'JetBrains Mono', monospace; font-size: 1.35rem; font-weight: 500; }
.sc-svg { width: 100%; height: 56px; margin-top: 6px; display: block; }
.sc-line { fill: none; stroke: var(--g); stroke-width: 1.8; vector-effect: non-scaling-stroke; stroke-dasharray: 1; stroke-dashoffset: 1; animation: draw 1.8s cubic-bezier(.45,0,.2,1) forwards .9s; }
.side-facts { margin-top: 26px; display: grid; gap: 10px; }
.side-facts li { display: flex; align-items: center; gap: 12px; font-size: .92rem; color: #dfe6e2; }
@media (max-width: 960px) { .side { display: none; } }
.side-facts i { width: 8px; height: 8px; border-radius: 50%; background: var(--g); box-shadow: 0 0 10px var(--g); flex: none; }

/* main */
.main { display: flex; flex-direction: column; min-height: 100vh; padding: 32px 48px; background: radial-gradient(600px 400px at 70% 10%, rgba(31,220,143,.07), transparent 70%); }
@media (max-width: 640px) { .main { padding: 22px 16px; } }
.main-top { display: flex; justify-content: space-between; align-items: center; }
.brand-main { visibility: hidden; }
@media (max-width: 960px) { .brand-main { visibility: visible; } }
.back { font-size: .88rem; color: var(--tx-2); transition: color .2s; }
.back:hover { color: #fff; }
.main-in { width: 100%; max-width: 440px; margin: auto; padding: 40px 0; animation: up .8s cubic-bezier(.16,1,.3,1) both .1s; }
.main-foot { display: flex; justify-content: center; gap: 22px; font-size: .78rem; color: var(--tx-3); }
.main-foot a:hover { color: var(--tx-2); }

@keyframes up { from { opacity: 0; transform: translateY(18px); } to { opacity: 1; transform: none; } }
@keyframes draw { to { stroke-dashoffset: 0; } }
@media (prefers-reduced-motion: reduce) { .side-img, .side-body, .main-in, .sc-line { animation: none; stroke-dashoffset: 0; } }

/* ── form primitives shared by Login/Register (slot content) ── */
.auth :slotted(.a-h) { font-size: 2rem; font-weight: 600; letter-spacing: -.02em; line-height: 1.15; }
.auth :slotted(.a-sub) { margin-top: 8px; color: var(--tx-2); font-size: .95rem; line-height: 1.6; }
</style>

<!-- Unscoped: form controls rendered by the pages inside the shell. -->
<style>
.auth-form { margin-top: 30px; display: grid; gap: 18px; }
.auth-form .row2 { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
@media (max-width: 480px) { .auth-form .row2 { grid-template-columns: 1fr; } }
.fld { display: block; }
.fld-l { display: flex; justify-content: space-between; align-items: baseline; font-size: .82rem; font-weight: 500; color: #cfd6d3; margin-bottom: 7px; }
.fld-l small { font-weight: 400; color: #6b7470; }
.fld-in { position: relative; }
.fld input { width: 100%; height: 50px; padding: 0 16px; background: #121614; border: 1px solid rgba(255,255,255,.12); border-radius: 12px; color: #eef2f0; font: inherit; font-size: .95rem; transition: border-color .2s, box-shadow .2s, background .2s; }
.fld input::placeholder { color: #5a625e; }
.fld input:hover { border-color: rgba(255,255,255,.22); }
.fld input:focus { outline: none; border-color: #1fdc8f; background: #131a17; box-shadow: 0 0 0 4px rgba(31,220,143,.14); }
.fld.bad input { border-color: #f0616d; }
.fld-err { display: block; margin-top: 6px; font-size: .78rem; color: #f0616d; }
.eye { position: absolute; right: 8px; top: 50%; transform: translateY(-50%); width: 36px; height: 36px; display: grid; place-items: center; border-radius: 8px; color: #8a938f; transition: color .2s, background .2s; }
.eye:hover { color: #fff; background: rgba(255,255,255,.06); }
.eye svg { width: 18px; height: 18px; }
.fld-in input[type='password'], .fld-in input.has-eye { padding-right: 50px; }
.chk { display: inline-flex; align-items: center; gap: 10px; font-size: .88rem; color: #a3aca8; cursor: pointer; user-select: none; }
.chk input { appearance: none; width: 18px; height: 18px; border: 1.5px solid rgba(255,255,255,.25); border-radius: 5px; display: grid; place-items: center; transition: all .2s; cursor: pointer; }
.chk input:checked { background: #1fdc8f; border-color: #1fdc8f; }
.chk input:checked::after { content: ''; width: 5px; height: 9px; border: solid #04140c; border-width: 0 2px 2px 0; transform: rotate(45deg) translate(-1px, -1px); }
.a-btn { position: relative; height: 52px; width: 100%; border-radius: 999px; background: #1fdc8f; color: #04140c; font: inherit; font-weight: 600; font-size: 1rem; display: inline-flex; align-items: center; justify-content: center; gap: 10px; transition: transform .2s, box-shadow .3s, background .2s, opacity .2s; }
.a-btn:hover:not(:disabled) { transform: translateY(-2px); background: #3ae9a1; box-shadow: 0 14px 40px -10px rgba(31,220,143,.6); }
.a-btn:disabled { opacity: .7; cursor: progress; }
.a-btn .spin { width: 18px; height: 18px; border: 2px solid rgba(4,20,12,.3); border-top-color: #04140c; border-radius: 50%; animation: aspin .7s linear infinite; }
@keyframes aspin { to { transform: rotate(360deg); } }
.a-alert { display: flex; gap: 10px; align-items: flex-start; padding: 12px 14px; border-radius: 12px; font-size: .88rem; line-height: 1.5; border: 1px solid rgba(240,97,109,.35); background: rgba(240,97,109,.08); color: #ffb3b9; animation: shake .4s ease; }
.a-alert.ok { border-color: rgba(31,220,143,.35); background: rgba(31,220,143,.08); color: #9ff0c9; animation: none; }
@keyframes shake { 20%, 60% { transform: translateX(-5px); } 40%, 80% { transform: translateX(5px); } }
.a-alt { margin-top: 26px; text-align: center; font-size: .92rem; color: #a3aca8; }
.a-alt a { color: #1fdc8f; font-weight: 600; }
.a-alt a:hover { text-decoration: underline; text-underline-offset: 3px; }
.a-fine { font-size: .78rem; line-height: 1.6; color: #6b7470; text-align: center; }
.a-fine a { color: #a3aca8; text-decoration: underline; text-underline-offset: 2px; }
.a-fine a:hover { color: #fff; }
</style>
