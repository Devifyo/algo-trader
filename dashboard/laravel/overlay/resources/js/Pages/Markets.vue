<script setup>
import { computed } from 'vue';
import PageHeader from '../Components/PageHeader.vue';
import EmptyState from '../Components/EmptyState.vue';

const props = defineProps({
    scores: { type: Array, default: () => [] },
    regime: { type: Object, default: null },
    allocation: { type: Array, default: () => [] },
    opportunities: { type: Array, default: () => [] },
    universe: { type: Number, default: 0 },
});

const usd = (n) => '$' + Number(n || 0).toLocaleString(undefined, { maximumFractionDigits: n < 10 ? 4 : 2 });
const biasCls = (b) => ({ bullish: 'text-bull bg-bull/10', bearish: 'text-bear bg-bear/10' }[b] || 'text-muted bg-panel-2');
const scoreCls = (s) => (s >= 66 ? 'text-bull' : s >= 45 ? 'text-warn' : 'text-bear');
const regimeLabel = computed(() => ({
    bull_market: 'Bull Market', bear_market: 'Bear Market', sideways_market: 'Sideways',
}[props.regime?.regime] || '—'));
const regimeCls = computed(() => ({
    bull_market: 'grad-emerald', bear_market: 'grad-rose', sideways_market: 'grad-amber',
}[props.regime?.regime] || 'grad-violet'));
const stratLabel = (k) => ({
    trend_following: 'Trend Following', momentum: 'Momentum', mean_reversion: 'Mean Reversion',
    breakout: 'Breakout', vol_expansion: 'Volatility Expansion',
}[k] || k);
</script>

<template>
    <div>
        <PageHeader title="Markets" description="AI opportunity scanner — every market scored live from price action, trend, momentum, volatility and news. Capital flows to the best risk-adjusted opportunities." help-topic="paper trading" />

        <EmptyState v-if="!scores.length" class="mt-6" title="Scanner warming up"
            description="The opportunity scanner needs ~30 candles per market before it can score them. Live rankings appear within a minute or two of the engine starting." />

        <template v-else>
            <!-- Regime · Allocation · Opportunities -->
            <div class="mt-6 grid gap-4 lg:grid-cols-3">
                <!-- Market regime -->
                <div class="card p-5">
                    <p class="eyebrow text-[11px] text-muted">Market Regime</p>
                    <div class="mt-3 flex items-center gap-3">
                        <span :class="['inline-flex items-center rounded-xl px-3 py-1.5 text-sm font-bold text-white', regimeCls]">{{ regimeLabel }}</span>
                        <span class="text-xs font-semibold capitalize text-muted">{{ (regime?.volatility || '').replace('_', ' ') }}</span>
                    </div>
                    <div class="mt-4">
                        <div class="flex justify-between text-xs"><span class="text-muted">Bullish breadth</span><span class="tnum text-ink">{{ regime?.breadth ?? 0 }}%</span></div>
                        <div class="mt-1.5 h-2 w-full rounded-full bg-panel-2"><div class="h-full rounded-full grad-emerald" :style="{ width: (regime?.breadth || 0) + '%' }"></div></div>
                    </div>
                    <p class="mt-4 text-[11px] text-muted">Strategies suited to this regime</p>
                    <div class="mt-1.5 flex flex-wrap gap-1.5">
                        <span v-for="s in regime?.favored || []" :key="s" class="rounded-lg bg-panel-2 px-2 py-1 text-[11px] font-semibold text-ink">{{ stratLabel(s) }}</span>
                    </div>
                </div>

                <!-- Dynamic capital allocation -->
                <div class="card p-5">
                    <p class="eyebrow text-[11px] text-muted">Dynamic Capital Allocation</p>
                    <div class="mt-3 space-y-2.5">
                        <div v-for="a in allocation" :key="a.symbol">
                            <div class="flex items-center justify-between text-xs">
                                <span class="font-semibold" :class="a.symbol === 'cash_reserve' ? 'text-muted' : 'text-ink'">{{ a.symbol === 'cash_reserve' ? 'Cash reserve' : a.symbol }}</span>
                                <span class="tnum text-ink">{{ a.weight }}%</span>
                            </div>
                            <div class="mt-1 h-1.5 w-full rounded-full bg-panel-2"><div class="h-full rounded-full" :class="a.symbol === 'cash_reserve' ? 'bg-faint' : 'grad-accent'" :style="{ width: a.weight + '%' }"></div></div>
                            <p v-if="a.reason && a.symbol !== 'cash_reserve'" class="mt-0.5 text-[10px] text-faint">{{ a.reason }}</p>
                        </div>
                    </div>
                </div>

                <!-- Top opportunities -->
                <div class="card p-5">
                    <p class="eyebrow text-[11px] text-muted">Top Opportunities</p>
                    <div class="mt-3 space-y-2">
                        <div v-for="(o, i) in opportunities" :key="o.symbol" class="flex items-center gap-3 rounded-lg border border-line p-2.5">
                            <span class="flex h-6 w-6 items-center justify-center rounded-full grad-accent text-[11px] font-bold text-white">{{ i + 1 }}</span>
                            <div class="flex-1"><p class="text-sm font-bold text-ink">{{ o.symbol }}</p><p class="text-[11px] text-muted">conf {{ o.confidence }} · risk {{ o.risk }}</p></div>
                            <span class="tnum text-sm font-bold" :class="scoreCls(o.market_score)">{{ o.market_score }}</span>
                        </div>
                        <p v-if="!opportunities.length" class="text-xs text-muted">No bullish setups right now — the scanner is holding cash.</p>
                    </div>
                </div>
            </div>

            <!-- Market rankings -->
            <div class="mt-6 card overflow-hidden p-0">
                <div class="flex items-center justify-between border-b border-line p-5">
                    <div><h3 class="font-display text-base font-bold text-ink">Market Rankings</h3><p class="text-xs text-muted">{{ universe }} markets scanned · ranked by opportunity quality</p></div>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full min-w-[820px] text-sm">
                        <thead>
                            <tr class="border-b border-line text-left text-[11px] uppercase tracking-wide text-muted">
                                <th class="px-5 py-3 font-semibold">#</th>
                                <th class="px-3 py-3 font-semibold">Market</th>
                                <th class="px-3 py-3 font-semibold">Price</th>
                                <th class="px-3 py-3 font-semibold">Bias</th>
                                <th class="px-3 py-3 font-semibold">Score</th>
                                <th class="px-3 py-3 font-semibold" title="Trend strength from EMA fast/slow">Trend</th>
                                <th class="px-3 py-3 font-semibold" title="Rate of change %">Mom.</th>
                                <th class="px-3 py-3 font-semibold" title="ATR % of price">Vol.</th>
                                <th class="px-3 py-3 font-semibold" title="News sentiment -1..1">Sent.</th>
                                <th class="px-3 py-3 font-semibold" title="Confidence in the read">Conf.</th>
                                <th class="px-5 py-3 font-semibold" title="Higher = riskier">Risk</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="(s, i) in scores" :key="s.symbol" class="border-b border-line/60 hover:bg-panel-2/50">
                                <td class="px-5 py-3 tnum text-muted">{{ i + 1 }}</td>
                                <td class="px-3 py-3"><span class="font-bold text-ink">{{ s.symbol }}</span></td>
                                <td class="px-3 py-3 tnum text-muted">{{ usd(s.price) }}</td>
                                <td class="px-3 py-3"><span :class="['rounded-full px-2 py-0.5 text-[11px] font-semibold capitalize', biasCls(s.bias)]">{{ s.bias }}</span></td>
                                <td class="px-3 py-3">
                                    <div class="flex items-center gap-2">
                                        <span class="tnum font-bold" :class="scoreCls(s.market_score)">{{ s.market_score }}</span>
                                        <div class="h-1.5 w-14 rounded-full bg-panel-2"><div class="h-full rounded-full grad-accent" :style="{ width: s.market_score + '%' }"></div></div>
                                    </div>
                                </td>
                                <td class="px-3 py-3 tnum" :class="s.trend > 0 ? 'text-bull' : 'text-bear'">{{ s.trend > 0 ? '+' : '' }}{{ s.trend }}</td>
                                <td class="px-3 py-3 tnum" :class="s.momentum > 0 ? 'text-bull' : 'text-bear'">{{ s.momentum > 0 ? '+' : '' }}{{ s.momentum }}%</td>
                                <td class="px-3 py-3 tnum text-muted">{{ s.volatility }}%</td>
                                <td class="px-3 py-3 tnum" :class="s.sentiment > 0 ? 'text-bull' : s.sentiment < 0 ? 'text-bear' : 'text-muted'">{{ s.sentiment > 0 ? '+' : '' }}{{ s.sentiment }}</td>
                                <td class="px-3 py-3 tnum text-ink">{{ s.confidence }}</td>
                                <td class="px-5 py-3 tnum" :class="s.risk > 60 ? 'text-bear' : 'text-muted'">{{ s.risk }}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
                <p class="border-t border-line p-3 text-center text-[11px] text-faint">Every number is computed live by the engine's opportunity scanner — trend, momentum and volatility from price action, sentiment from real news. Phase 2 adds on-chain, funding & whale flow.</p>
            </div>
        </template>
    </div>
</template>
