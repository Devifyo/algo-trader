<script setup>
import { computed, ref } from 'vue';
import { Link } from '@inertiajs/vue3';
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import StatCard from '../Components/StatCard.vue';
import Donut from '../Components/Donut.vue';
import SentimentBadge from '../Components/SentimentBadge.vue';
import RiskBadge from '../Components/RiskBadge.vue';
import EmptyState from '../Components/EmptyState.vue';
import NavIcon from '../Components/NavIcon.vue';

const props = defineProps({ widgets: { type: Object, default: () => ({}) } });
const w = computed(() => props.widgets || {});
const usd = (n) => '$' + Number(n || 0).toLocaleString(undefined, { maximumFractionDigits: 0 });
const signed = (n) => (Number(n || 0) >= 0 ? '+' : '') + usd(n);
const equity = computed(() => Number(w.value.portfolio?.equity_usd || 0));
const dailyPnl = computed(() => Number(w.value.dailyPnl || 0));
const changePct = computed(() => equity.value > 0 ? (dailyPnl.value / equity.value) * 100 : 0);
const riskLevel = (s) => s >= 70 ? 'high' : s >= 45 ? 'medium' : 'low';
const biasFor = (s) => s > 10 ? 'bullish' : s < -10 ? 'bearish' : 'neutral';

const tab = ref('positions');
const tabs = [
    { key: 'positions', label: 'Positions' },
    { key: 'bots', label: 'Bots' },
    { key: 'activity', label: 'Activity' },
];
const tiles = computed(() => [
    { label: "Today's P&L", value: signed(dailyPnl.value), tone: dailyPnl.value >= 0 ? 'bull' : 'bear' },
    { label: 'Realized', value: signed(w.value.portfolio?.realized_usd), tone: (w.value.portfolio?.realized_usd || 0) >= 0 ? 'bull' : 'bear' },
    { label: 'Unrealized', value: signed(w.value.portfolio?.unrealized_usd), tone: (w.value.portfolio?.unrealized_usd || 0) >= 0 ? 'bull' : 'bear' },
    { label: 'Open positions', value: String(w.value.portfolio?.open_positions || 0), tone: 'neutral' },
]);
const toneText = { bull: 'text-bull', bear: 'text-bear', neutral: 'text-ink' };
</script>

<template>
    <div>
        <PageHeader title="Dashboard"
            description="Your portfolio at a glance. Everything is advisory — the risk engine has the final say on every trade."
            help-topic="risk management" />

        <div v-if="w.killSwitch" class="mb-6 flex items-center gap-2 rounded-2xl border border-bear/40 bg-bear/10 px-4 py-3 text-sm text-bear">
            <strong>Kill switch engaged.</strong> No new entries are being placed. Clear it from the top bar to resume.
        </div>

        <!-- Hero portfolio card -->
        <div class="card mb-6 p-6">
            <div class="grid gap-6 lg:grid-cols-[1.4fr_1fr]">
                <div class="flex items-start gap-4">
                    <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl grad-violet text-white shadow-sm"><NavIcon name="portfolio" /></div>
                    <div>
                        <p class="eyebrow text-xs text-muted">Total Portfolio Value</p>
                        <p class="mt-1 font-display text-[40px] font-extrabold leading-none tnum text-ink">{{ usd(equity) }}</p>
                        <span :class="['mt-3 inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold tnum',
                            dailyPnl >= 0 ? 'text-bull bg-bull/10' : 'text-bear bg-bear/10']">
                            {{ dailyPnl >= 0 ? '▲' : '▼' }} {{ signed(dailyPnl) }} ({{ changePct.toFixed(2) }}%)
                            <span class="text-muted">· today</span>
                        </span>
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-2 xl:grid-cols-4">
                    <div v-for="t in tiles" :key="t.label" class="rounded-xl border border-line bg-panel-2 p-3">
                        <p class="eyebrow text-[10px] text-muted">{{ t.label }}</p>
                        <p :class="['mt-1 font-display text-lg font-bold tnum', toneText[t.tone]]">{{ t.value }}</p>
                    </div>
                </div>
            </div>
        </div>

        <!-- Gradient stat cards -->
        <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
            <StatCard label="Open positions" icon="bots" grad="grad-blue" :value="w.portfolio?.open_positions || 0"
                pill="Live" pill-tone="accent" hint="Positions currently open across your bots." />
            <StatCard label="AI confidence" icon="providers" grad="grad-violet" :value="Math.round(w.aiConfidence || 0) + '%'"
                hint="How confident the AI decision layer is right now." />
            <StatCard label="Risk score" icon="admin" :grad="(w.riskScore||0) >= 45 ? 'grad-rose' : 'grad-emerald'" :value="Math.round(w.riskScore || 0)"
                :tone="(w.riskScore||0) >= 45 ? 'bear' : 'bull'" hint="Lower is calmer. The platform de-risks automatically." />
            <StatCard label="Today's P&L" icon="portfolio" :grad="dailyPnl >= 0 ? 'grad-emerald' : 'grad-rose'" :value="signed(dailyPnl)"
                :tone="dailyPnl >= 0 ? 'bull' : 'bear'" hint="Realized profit/loss since midnight UTC." />
        </div>

        <!-- Main: tabs + allocation -->
        <div class="mt-6 grid gap-6 lg:grid-cols-[1.6fr_1fr]">
            <Card>
                <div class="mb-4 flex items-center justify-between">
                    <div class="inline-flex rounded-xl border border-line bg-panel-2 p-1">
                        <button v-for="t in tabs" :key="t.key" @click="tab = t.key"
                            :class="['rounded-lg px-3.5 py-1.5 text-sm font-semibold transition', tab === t.key ? 'grad-accent text-white' : 'text-muted hover:text-ink']">
                            {{ t.label }}
                        </button>
                    </div>
                    <Link href="/brokers" class="rounded-lg border border-line px-3 py-1.5 text-xs font-semibold text-muted hover:text-ink">Connect broker</Link>
                </div>

                <!-- Positions -->
                <div v-if="tab === 'positions'">
                    <div v-if="(w.positions || []).length" class="overflow-x-auto">
                        <table class="w-full text-sm">
                            <thead class="text-left text-[11px] uppercase tracking-wide text-faint">
                                <tr><th class="py-2">Symbol</th><th>Side</th><th class="text-right">Qty</th><th class="text-right">Entry</th><th class="text-right">Notional</th><th class="text-right">Lev</th></tr>
                            </thead>
                            <tbody class="divide-y divide-line">
                                <tr v-for="(p, i) in w.positions" :key="i">
                                    <td class="py-3 font-semibold text-ink">{{ p.symbol }}</td>
                                    <td><span :class="['rounded-full px-2 py-0.5 text-xs font-semibold', p.side === 'long' ? 'text-bull bg-bull/10' : 'text-bear bg-bear/10']">{{ p.side }}</span></td>
                                    <td class="text-right tnum text-muted">{{ Number(p.qty).toLocaleString(undefined, { maximumFractionDigits: 4 }) }}</td>
                                    <td class="text-right tnum text-muted">{{ usd(p.entry_price) }}</td>
                                    <td class="text-right tnum font-medium text-ink">{{ usd(p.notional) }}</td>
                                    <td class="text-right tnum text-muted">{{ Number(p.leverage).toFixed(0) }}×</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                    <EmptyState v-else title="No open positions" description="Connect a broker and enable a strategy to start paper trading.">
                        <Link href="/onboarding" class="mt-3 rounded-lg grad-accent px-3 py-1.5 text-sm font-semibold text-white">Start setup</Link>
                    </EmptyState>
                </div>

                <!-- Bots -->
                <div v-else-if="tab === 'bots'">
                    <div v-if="(w.bots || []).length" class="overflow-x-auto">
                        <table class="w-full text-sm">
                            <thead class="text-left text-[11px] uppercase tracking-wide text-faint"><tr><th class="py-2">Name</th><th>Type</th><th>Exchange</th><th>Status</th></tr></thead>
                            <tbody class="divide-y divide-line">
                                <tr v-for="b in w.bots" :key="b.id">
                                    <td class="py-3 font-semibold text-ink">{{ b.name }}</td>
                                    <td class="text-muted">{{ b.type }}</td>
                                    <td class="text-muted">{{ b.exchange || '—' }}</td>
                                    <td><span class="rounded-full border border-line px-2 py-0.5 text-xs capitalize text-muted">{{ b.status }}</span></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                    <EmptyState v-else title="No bots yet" description="Create a bot during onboarding to start paper trading." />
                </div>

                <!-- Activity -->
                <div v-else>
                    <ul v-if="(w.notifications || []).length" class="divide-y divide-line">
                        <li v-for="(n, i) in w.notifications" :key="i" class="flex items-start gap-3 py-3">
                            <RiskBadge :level="n.severity === 'critical' ? 'critical' : 'medium'" />
                            <div class="min-w-0"><p class="truncate text-sm text-ink">{{ n.headline }}</p><p class="text-xs text-muted">{{ n.event_type }} · {{ n.asset }}</p></div>
                        </li>
                    </ul>
                    <EmptyState v-else title="All quiet" description="No market-moving events right now." />
                </div>
            </Card>

            <!-- Allocation + market mood -->
            <div class="space-y-6">
                <Card title="Portfolio allocation" description="By open-position notional.">
                    <Donut v-if="(w.allocation || []).length"
                        :slices="w.allocation.map(a => ({ label: a.label, value: a.value }))"
                        :center="String(w.portfolio?.open_positions || 0)" center-sub="positions" />
                    <EmptyState v-else title="No allocation yet" description="Appears once positions are open." />
                </Card>

                <Card title="Market mood" description="News sentiment by asset." help-topic="AI sentiment analysis">
                    <ul v-if="(w.market?.sentiment || []).length" class="space-y-2">
                        <li v-for="s in w.market.sentiment" :key="s.asset" class="flex items-center justify-between">
                            <span class="font-medium text-ink">{{ s.asset }}</span>
                            <SentimentBadge :bias="biasFor(s.sentiment_score)" :score="s.sentiment_score" />
                        </li>
                    </ul>
                    <EmptyState v-else title="No sentiment yet" description="The news engine is warming up." />
                </Card>
            </div>
        </div>
    </div>
</template>
