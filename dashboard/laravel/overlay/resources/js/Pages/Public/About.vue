<script setup>
import { Link } from '@inertiajs/vue3';
import PublicShell from '../../Components/PublicShell.vue';
import SeoHead from '../../Components/SeoHead.vue';
defineOptions({ layout: null });

const values = [
    { t: 'Discipline over hype', d: 'Rules-based automation grounded in published research — not “holy grail” promises.' },
    { t: 'Safety first', d: 'Real protective stops, a kill-switch and risk gates are built in. Paper mode lets you prove a strategy with zero real money.' },
    { t: 'Radical transparency', d: 'Every order, fill, rejection and AI decision is logged and visible. We’re honest about what software can and can’t do.' },
    { t: 'You stay in control', d: 'Your funds stay at your own exchange. Your keys are encrypted. You decide what runs, when, and how much.' },
];
// The real order path, in the order a trade moves through it.
const pipeline = [
    { t: 'Market data', d: 'Exchange feeds, normalised' },
    { t: 'Signals', d: 'Rule-based strategies' },
    { t: 'AI context', d: 'Second opinion, veto only' },
    { t: 'Risk gate', d: '9 checks, then sizing' },
    { t: 'Execution', d: 'Your exchange, trade-only key' },
];
const stack = ['Rust trading core', 'Python AI & analytics', 'Laravel + Vue dashboard', 'TimescaleDB', 'Encrypted secrets vault', 'NATS message bus'];
</script>

<template>
    <PublicShell>
        <SeoHead title="About — Automated Crypto Trading Platform"
            description="TradePilot builds disciplined, rule-based automated crypto trading software with real risk controls and full transparency — trading tools, not advice. Based in India."
            path="/about" />

        <!-- hero -->
        <section class="ab-hero">
            <div class="ps-wrap ab-hero-in">
                <div>
                    <p class="kick">About TradePilot</p>
                    <h1 class="ab-h">Trading software,<br /><span class="g">built like infrastructure.</span></h1>
                    <p class="ab-lead">TradePilot helps people trade crypto the disciplined way — automated, rule-based strategies running on their own exchange accounts, wrapped in real risk controls and full transparency. We’re engineers and market nerds based in <b>India</b>, building the tool we wished existed: powerful, honest, and firmly in the user’s control.</p>
                </div>
                <figure class="ab-photo">
                    <img src="/img/lp/checklist.jpg" alt="A hand ticking items off a written checklist" width="1600" height="1065" />
                    <figcaption>Rules first. Then the trade.</figcaption>
                </figure>
            </div>
        </section>

        <!-- mission -->
        <section class="ps-wrap ab-sec ab-two">
            <div data-in>
                <span class="num">01</span>
                <h2 class="ab-h2">Our mission</h2>
                <p>To make systematic, risk-managed trading accessible — without pretending markets are easy or risk-free. We give you institutional-style tooling and get out of your way.</p>
            </div>
            <div data-in style="--d:.1s">
                <span class="num">02</span>
                <h2 class="ab-h2">What we are (and aren’t)</h2>
                <p>TradePilot is <b>software</b>. We’re not a broker, fund or financial adviser, we don’t hold your money, and we don’t guarantee profits. You connect your own account and make every decision.</p>
            </div>
        </section>

        <!-- values -->
        <section class="ab-band">
            <div class="ps-wrap ab-sec">
                <h2 class="ab-h2 big" data-in>What we <span class="g">value</span></h2>
                <div class="vals">
                    <div v-for="(v, i) in values" :key="v.t" class="val" data-in :style="{ '--d': i * 0.08 + 's' }">
                        <span class="val-n">{{ String(i + 1).padStart(2, '0') }}</span>
                        <h3>{{ v.t }}</h3>
                        <p>{{ v.d }}</p>
                    </div>
                </div>
            </div>
        </section>

        <!-- pipeline -->
        <section class="ps-wrap ab-sec">
            <div class="ab-top" data-in>
                <h2 class="ab-h2 big">How an order <span class="g">actually moves</span></h2>
                <p>A low-latency Rust core does the trading. Python handles AI and analytics. Every hop is logged — this is the real path, not a marketing diagram.</p>
            </div>
            <div class="pipe" data-in>
                <div class="pipe-line" aria-hidden="true"><i></i></div>
                <div v-for="(p, i) in pipeline" :key="p.t" class="pipe-step" :style="{ '--i': i }">
                    <span class="pipe-dot"></span>
                    <b>{{ p.t }}</b>
                    <span>{{ p.d }}</span>
                </div>
            </div>
            <div class="tags" data-in>
                <span v-for="s in stack" :key="s" class="tag">{{ s }}</span>
            </div>
        </section>

        <!-- talk -->
        <section class="ps-wrap ab-sec ab-talk" data-in>
            <h2 class="ab-h2 big">Want to <span class="g">talk?</span></h2>
            <p>Questions, feedback or partnerships — we read everything.</p>
            <div class="ab-ctas">
                <Link href="/contact" class="btn-g">Contact us →</Link>
                <Link href="/register" class="btn-ghost">Get started free</Link>
            </div>
        </section>
    </PublicShell>
</template>

<style scoped>
.kick { font-family: var(--mono); font-size: .75rem; letter-spacing: .14em; text-transform: uppercase; color: var(--g); }
.ab-hero { position: relative; overflow: hidden; border-bottom: 1px solid var(--rule); }
.ab-hero::before { content: ''; position: absolute; left: 30%; top: -30%; width: 800px; height: 600px; background: radial-gradient(closest-side, rgba(31,220,143,.12), transparent); pointer-events: none; }
.ab-hero-in { position: relative; display: grid; grid-template-columns: 1.2fr .8fr; gap: 64px; align-items: center; padding-top: 90px; padding-bottom: 90px; }
@media (max-width: 900px) { .ab-hero-in { grid-template-columns: 1fr; padding-top: 60px; padding-bottom: 60px; gap: 44px; } }
.ab-h { margin-top: 16px; text-transform: uppercase; font-weight: 700; font-size: clamp(2.2rem, 5.2vw, 4.2rem); line-height: 1.02; animation: ab-in .9s cubic-bezier(.16,1,.3,1) both; }
.ab-lead { margin-top: 26px; max-width: 580px; font-size: 1.08rem; line-height: 1.75; color: var(--tx-2); animation: ab-in .9s cubic-bezier(.16,1,.3,1) both .12s; }
.ab-lead b { color: var(--tx); font-weight: 600; }
.ab-photo { position: relative; margin: 0; border-radius: 16px; overflow: hidden; aspect-ratio: 4 / 5; animation: ab-photo 1.4s cubic-bezier(.7,0,.2,1) both .2s; }
.ab-photo img { width: 100%; height: 100%; object-fit: cover; filter: grayscale(.6) contrast(1.05) brightness(.8); }
.ab-photo::after { content: ''; position: absolute; inset: 0; background: linear-gradient(180deg, rgba(31,220,143,.12), rgba(12,14,13,.75)); mix-blend-mode: normal; }
.ab-photo figcaption { position: absolute; left: 20px; bottom: 18px; z-index: 1; font-family: var(--mono); font-size: .8rem; color: #fff; }
@keyframes ab-in { from { opacity: 0; transform: translateY(22px); } to { opacity: 1; transform: none; } }
@keyframes ab-photo { from { clip-path: inset(100% 0 0 0); } to { clip-path: inset(0); } }

.ab-sec { padding-top: 100px; padding-bottom: 100px; }
.ab-two { display: grid; grid-template-columns: 1fr 1fr; gap: 64px; }
@media (max-width: 760px) { .ab-two { grid-template-columns: 1fr; gap: 44px; } }
.num { display: inline-block; font-family: var(--mono); font-size: .8rem; color: var(--g); padding-top: 10px; border-top: 1px solid currentColor; min-width: 40px; }
.ab-h2 { margin-top: 16px; text-transform: uppercase; font-weight: 600; font-size: 1.5rem; line-height: 1.15; }
.ab-h2.big { margin-top: 0; font-size: clamp(1.9rem, 3.8vw, 3rem); }
.ab-two p, .ab-top p, .ab-talk p { margin-top: 14px; line-height: 1.75; color: var(--tx-2); }
.ab-two b { color: var(--tx); }

.ab-band { background: var(--bg-2); border-top: 1px solid var(--rule); border-bottom: 1px solid var(--rule); }
.vals { margin-top: 50px; display: grid; grid-template-columns: repeat(4, 1fr); border: 1px solid var(--rule); border-radius: 16px; overflow: hidden; }
@media (max-width: 960px) { .vals { grid-template-columns: 1fr 1fr; } }
@media (max-width: 560px) { .vals { grid-template-columns: 1fr; } }
.val { padding: 30px 26px 34px; border-right: 1px solid var(--rule); background: var(--card); transition: background .3s; position: relative; }
.val:last-child { border-right: 0; }
@media (max-width: 960px) { .val:nth-child(2n) { border-right: 0; } .val:nth-child(-n+2) { border-bottom: 1px solid var(--rule); } }
@media (max-width: 560px) { .val { border-right: 0; border-bottom: 1px solid var(--rule); } .val:last-child { border-bottom: 0; } }
.val::before { content: ''; position: absolute; left: 0; top: 0; height: 2px; width: 0; background: var(--g); transition: width .5s cubic-bezier(.16,1,.3,1); }
.val:hover { background: var(--card-2); }
.val:hover::before { width: 100%; }
.val-n { font-family: var(--mono); font-size: .8rem; color: var(--g); }
.val h3 { margin-top: 26px; font-weight: 600; font-size: 1.1rem; }
.val p { margin-top: 10px; font-size: .9rem; line-height: 1.65; color: var(--tx-2); }

.ab-top { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 20px 60px; align-items: flex-start; }
.ab-top p { max-width: 420px; margin-top: 0; font-size: .95rem; }
.pipe { position: relative; margin-top: 60px; display: grid; grid-template-columns: repeat(5, 1fr); gap: 16px; }
.pipe-line { position: absolute; left: 10%; right: 10%; top: 9px; height: 2px; background: var(--rule-2); overflow: hidden; }
.pipe-line i { position: absolute; top: 0; left: -20%; width: 20%; height: 100%; background: linear-gradient(90deg, transparent, var(--g), transparent); animation: pipe 3s linear infinite; }
@keyframes pipe { to { left: 100%; } }
.pipe-step { position: relative; text-align: center; display: grid; justify-items: center; gap: 6px; }
.pipe-dot { width: 20px; height: 20px; border-radius: 50%; background: var(--bg); border: 2px solid var(--g); box-shadow: 0 0 0 5px rgba(31,220,143,.1); margin-bottom: 14px; animation: dot 3s ease-in-out infinite; animation-delay: calc(var(--i) * .6s); }
@keyframes dot { 0%, 100% { background: var(--bg); } 10%, 20% { background: var(--g); box-shadow: 0 0 18px var(--g); } }
.pipe-step b { font-weight: 600; font-size: 1rem; }
.pipe-step span:last-child { font-size: .82rem; color: var(--tx-3); }
@media (max-width: 760px) {
    .pipe { grid-template-columns: 1fr; gap: 26px; padding-left: 8px; }
    .pipe-line { left: 17px; right: auto; top: 10px; bottom: 10px; width: 2px; height: auto; }
    .pipe-line i { left: 0; top: -20%; width: 100%; height: 20%; background: linear-gradient(180deg, transparent, var(--g), transparent); animation-name: pipe-v; }
    @keyframes pipe-v { to { top: 100%; } }
    .pipe-step { grid-template-columns: 20px 1fr; text-align: left; justify-items: start; column-gap: 18px; row-gap: 2px; }
    .pipe-dot { grid-row: span 2; margin: 2px 0 0; }
}
.tags { margin-top: 60px; display: flex; flex-wrap: wrap; gap: 10px; justify-content: center; }
.tag { border: 1px solid var(--rule-2); border-radius: 999px; padding: 8px 16px; font-size: .84rem; color: var(--tx-2); font-family: var(--mono); }

.ab-talk { text-align: center; border-top: 1px solid var(--rule); }
.ab-ctas { margin-top: 30px; display: flex; flex-wrap: wrap; gap: 12px; justify-content: center; }

@media (prefers-reduced-motion: reduce) { .ab-h, .ab-lead, .ab-photo, .pipe-line i, .pipe-dot { animation: none; } }
</style>
