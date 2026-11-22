<script setup>
import CoinIcon from './CoinIcon.vue';

// Stacked-cards scroll (sticky section): each card locks at the top and the next
// scrolls up to cover it like a deck. Pure CSS (position: sticky) — no JS scroll math.
const cards = [
    {
        eyebrow: 'Step 01', tone: 'a',
        title: 'Connect your broker — or start in paper',
        body: 'Link your own exchange via API, or run everything in free paper mode with zero real money. Your keys are encrypted in a secrets vault and used only to place your orders.',
        chips: ['Binance', 'Coinbase', 'Kraken', 'Alpaca'],
        kind: 'connect',
    },
    {
        eyebrow: 'Step 02', tone: 'b',
        title: 'Pick a strategy that fits the market',
        body: 'Trend-following, momentum, breakout, mean-reversion or vol-expansion — choose a proven, rule-based strategy and the markets you want it to trade.',
        chips: ['Trend', 'Momentum', 'Breakout', 'Mean-reversion'],
        kind: 'strategy',
    },
    {
        eyebrow: 'Step 03', tone: 'c',
        title: 'Automated, protected execution',
        body: 'Bots run 24/7 with real protective stops — stop-loss, take-profit and an ATR trailing stop enforced server-side, plus AI market-context vetoes.',
        chips: ['Stop-loss', 'Take-profit', 'ATR trailing', 'AI veto'],
        kind: 'execute',
    },
    {
        eyebrow: 'Step 04', tone: 'd',
        title: 'Stay in full control',
        body: 'Watch every order, fill and decision in real time. Per-strategy performance, drawdown limits, and a one-tap kill-switch to flatten everything instantly.',
        chips: ['Live logs', 'Kill-switch', 'Drawdown caps', 'Full transparency'],
        kind: 'control',
    },
];
</script>

<template>
    <section class="story">
        <div class="story-head">
            <h2 v-reveal class="sec-title">How TradePilot works</h2>
            <p v-reveal class="sec-sub">Scroll — each step locks in as the next slides over it.</p>
        </div>

        <div class="deck">
            <article v-for="(c, i) in cards" :key="i" class="scard" :class="'tone-' + c.tone" :style="{ '--i': i, zIndex: i + 1 }">
                <div class="scard-inner">
                    <div class="scard-copy">
                        <span class="eyebrow">{{ c.eyebrow }}</span>
                        <h3 class="scard-title">{{ c.title }}</h3>
                        <p class="scard-body">{{ c.body }}</p>
                        <div class="chips">
                            <span v-for="ch in c.chips" :key="ch" class="chip">{{ ch }}</span>
                        </div>
                    </div>

                    <!-- visual -->
                    <div class="scard-visual">
                        <div v-if="c.kind === 'connect'" class="vpanel">
                            <div class="vrow" v-for="s in ['BTC','ETH','SOL']" :key="s"><CoinIcon :symbol="s" :size="28" /><span class="vname">{{ s }} / USD</span><span class="vtag ok">connected</span></div>
                            <div class="vtoggle"><span>Paper mode</span><span class="switch"><i></i></span></div>
                        </div>
                        <div v-else-if="c.kind === 'strategy'" class="vpanel">
                            <div class="vstrat on"><span>Trend-following</span><span class="vtag ok">selected</span></div>
                            <div class="vstrat"><span>Momentum</span></div>
                            <div class="vstrat"><span>Breakout</span></div>
                            <div class="vstrat"><span>Mean-reversion</span></div>
                        </div>
                        <div v-else-if="c.kind === 'execute'" class="vpanel">
                            <svg class="vchart" viewBox="0 0 320 110" preserveAspectRatio="none"><defs><linearGradient id="sg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="rgba(110,86,248,.4)" /><stop offset="1" stop-color="rgba(110,86,248,0)" /></linearGradient></defs><path d="M0,80 L40,70 L80,86 L120,54 L160,62 L200,38 L240,46 L280,24 L320,30 L320,110 L0,110 Z" fill="url(#sg)" /><path d="M0,80 L40,70 L80,86 L120,54 L160,62 L200,38 L240,46 L280,24 L320,30" fill="none" stroke="#6E56F8" stroke-width="2.5" stroke-linecap="round" /></svg>
                            <div class="vchips"><span class="vtag ok">▲ Stop armed</span><span class="vtag accent">● ATR trailing</span></div>
                        </div>
                        <div v-else class="vpanel">
                            <div class="vlog" v-for="l in [['BUY','BTC','#16c784'],['SELL','ETH','#16c784'],['BUY','SOL','#ef4444']]" :key="l[0]+l[1]"><b :style="{ color: l[2] }">{{ l[0] }}</b><span>{{ l[1] }}</span><span class="vtime">now</span></div>
                            <button class="vkill">Kill-switch</button>
                        </div>
                    </div>
                </div>
            </article>
        </div>
    </section>
</template>

<style scoped>
.story-head { text-align: center; padding: 4rem 1rem 1rem; }
.sec-title { font-family: var(--font-display, inherit); font-size: clamp(1.6rem, 3.4vw, 2.3rem); font-weight: 800; letter-spacing: -.02em; color: rgb(var(--ink)); }
.sec-sub { margin-top: .6rem; color: rgb(var(--muted)); font-size: .95rem; }

.deck { position: relative; padding-bottom: 8vh; }
.scard { position: sticky; top: calc(4.5rem + var(--i) * 1.4rem); min-height: 78vh; display: grid; place-items: center; padding: 1rem; }
.scard-inner { width: 100%; max-width: 64rem; min-height: 62vh; display: grid; gap: 2rem; align-items: center; grid-template-columns: 1fr; padding: clamp(1.75rem, 4vw, 3.25rem); border: 1px solid rgb(var(--line)); border-radius: 28px; box-shadow: 0 -10px 50px -20px rgba(0,0,0,.55), 0 30px 60px -40px rgba(0,0,0,.6); overflow: hidden; }
@media (min-width: 880px) { .scard-inner { grid-template-columns: 1.1fr .9fr; gap: 3rem; } }

/* opaque tinted backgrounds so each card fully covers the one beneath */
.tone-a .scard-inner { background: linear-gradient(135deg, rgb(var(--panel)), rgb(var(--surface))); }
.tone-b .scard-inner { background: linear-gradient(135deg, rgb(var(--panel)), rgb(var(--surface))); }
.tone-c .scard-inner { background: linear-gradient(135deg, rgb(var(--panel)), rgb(var(--surface))); }
.tone-d .scard-inner { background: linear-gradient(135deg, rgb(var(--panel)), rgb(var(--surface))); }
.scard-inner::before { content: ''; position: absolute; top: -40%; right: -10%; width: 60%; height: 120%; background: radial-gradient(circle, rgba(110,86,248,.16), transparent 70%); pointer-events: none; }

.eyebrow { font-size: .72rem; font-weight: 700; letter-spacing: .08em; text-transform: uppercase; color: rgb(var(--accent)); }
.scard-title { margin-top: .8rem; font-family: var(--font-display, inherit); font-size: clamp(1.5rem, 3.2vw, 2.4rem); font-weight: 800; letter-spacing: -.02em; line-height: 1.1; color: rgb(var(--ink)); }
.scard-body { margin-top: 1rem; font-size: 1rem; line-height: 1.6; color: rgb(var(--muted)); max-width: 32rem; }
.chips { margin-top: 1.4rem; display: flex; flex-wrap: wrap; gap: .5rem; }
.chip { font-size: .76rem; font-weight: 600; color: rgb(var(--ink)); border: 1px solid rgb(var(--line)); background: rgb(var(--surface)); border-radius: 999px; padding: .35rem .8rem; }

/* visuals */
.scard-visual { position: relative; }
.vpanel { border: 1px solid rgb(var(--line)); background: rgb(var(--surface)/.7); backdrop-filter: blur(6px); border-radius: 18px; padding: 1.1rem; display: flex; flex-direction: column; gap: .6rem; box-shadow: 0 30px 60px -40px rgba(0,0,0,.6); }
.vrow { display: flex; align-items: center; gap: .6rem; padding: .55rem; border: 1px solid rgb(var(--line)); border-radius: 12px; }
.vname { font-size: .85rem; font-weight: 600; color: rgb(var(--ink)); }
.vtag { margin-left: auto; font-size: .68rem; font-weight: 700; padding: .22rem .55rem; border-radius: 999px; }
.vtag.ok { color: #16c784; background: rgba(22,199,132,.14); }
.vtag.accent { color: rgb(var(--accent)); background: rgb(var(--accent)/.14); }
.vtoggle { display: flex; align-items: center; justify-content: space-between; padding: .6rem .8rem; font-size: .85rem; font-weight: 600; color: rgb(var(--ink)); border: 1px dashed rgb(var(--line)); border-radius: 12px; }
.switch { width: 38px; height: 22px; border-radius: 999px; background: rgb(var(--accent)); position: relative; }
.switch i { position: absolute; top: 2px; right: 2px; width: 18px; height: 18px; border-radius: 50%; background: #fff; }
.vstrat { display: flex; align-items: center; padding: .7rem .85rem; border: 1px solid rgb(var(--line)); border-radius: 12px; font-size: .88rem; font-weight: 600; color: rgb(var(--muted)); }
.vstrat.on { color: rgb(var(--ink)); border-color: rgb(var(--accent)/.5); background: rgb(var(--accent)/.08); }
.vchart { width: 100%; height: 110px; }
.vchips { display: flex; gap: .5rem; margin-top: .4rem; }
.vlog { display: flex; align-items: center; gap: .6rem; padding: .5rem .2rem; border-bottom: 1px solid rgb(var(--line)); font-size: .85rem; color: rgb(var(--ink)); }
.vlog b { width: 2.6rem; font-weight: 800; } .vtime { margin-left: auto; font-size: .72rem; color: rgb(var(--faint)); }
.vkill { margin-top: .8rem; align-self: flex-start; font-size: .8rem; font-weight: 700; color: #fff; background: linear-gradient(120deg,#ef4444,#f97316); padding: .5rem .9rem; border-radius: 10px; }

/* Progressive enhancement: gently scale a card down as it gets covered (Chrome). */
@supports (animation-timeline: view()) {
    @media (prefers-reduced-motion: no-preference) and (min-width: 880px) {
        .scard-inner { animation: coverShrink linear both; animation-timeline: view(); animation-range: exit -10% exit 90%; }
        @keyframes coverShrink { to { transform: scale(.93); opacity: .55; } }
    }
}
</style>
