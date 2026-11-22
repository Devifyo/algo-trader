<script setup>
import PageHeader from '../Components/PageHeader.vue';
import EmptyState from '../Components/EmptyState.vue';

const props = defineProps({
    decisions: { type: Array, default: () => [] },
    kpi: { type: Object, default: () => ({}) },
    ai_mode: { type: String, default: 'heuristic' },
    providers: { type: Array, default: () => [] },
});

const time = (t) => { try { return new Date(t).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }); } catch { return t; } };
const actionCls = (a) => (a === 'traded' ? 'text-bull bg-bull/10' : 'text-muted bg-panel-2');
const sideCls = (s) => (s === 'buy' ? 'text-bull' : 'text-bear');
const reasonText = (r) => ({
    approved: 'Passed all risk gates — order sent',
    no_setup: 'No valid setup after sizing',
    KILL_SWITCH: 'Blocked by global kill-switch',
    TENANT_HALTED: 'Blocked — account halted',
    NO_STOP: 'Rejected — no protective stop',
    STALE_DATA: 'Rejected — market data stale',
    MAX_OPEN_POSITIONS: 'Rejected — max open positions reached',
    DAILY_DRAWDOWN: 'Rejected — daily drawdown limit hit',
    AI_FLOOR: 'Rejected — AI confidence below floor',
    NEWS_VETO: 'Vetoed — strong opposing news sentiment',
    publish_failed: 'Internal: order publish failed',
}[r] || r);
const regimeText = (r) => ({ bull_market: 'Bull', bear_market: 'Bear', sideways_market: 'Sideways' }[r] || r || '—');
</script>

<template>
    <div>
        <PageHeader title="AI Decisions" description="Every signal the risk engine evaluated and exactly why it traded or was skipped — full explainability for each decision." help-topic="paper trading" />

        <!-- AI mode banner -->
        <div class="mt-4 flex items-center gap-3 rounded-xl border p-3.5"
            :class="ai_mode === 'consensus' ? 'border-accent/30 bg-accent/10' : 'border-line bg-panel-2'">
            <span class="text-lg">{{ ai_mode === 'consensus' ? '🧠' : '⚙️' }}</span>
            <div class="flex-1">
                <p class="text-sm font-semibold text-ink">
                    {{ ai_mode === 'consensus' ? 'Multi-provider AI consensus active' : 'Heuristic scoring (no AI keys yet)' }}
                </p>
                <p class="text-xs text-muted">
                    <template v-if="ai_mode === 'consensus'">Decisions blend votes from: {{ providers.join(', ') }}.</template>
                    <template v-else>Confidence is computed locally from price action. Add provider keys in <a href="/admin/ai-providers" class="font-semibold text-accent hover:underline">AI Providers</a> to enable Claude/OpenAI/Gemini consensus per trade.</template>
                </p>
            </div>
        </div>

        <!-- KPIs -->
        <div class="mt-5 grid grid-cols-2 gap-4 lg:grid-cols-4">
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Decisions</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-ink">{{ kpi.total ?? 0 }}</p></div>
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Traded</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-bull">{{ kpi.traded ?? 0 }}</p></div>
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Skipped</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-muted">{{ kpi.skipped ?? 0 }}</p></div>
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Avg AI Confidence</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-ink">{{ kpi.avg_conf != null ? kpi.avg_conf + '%' : '—' }}</p></div>
        </div>

        <EmptyState v-if="!decisions.length" class="mt-6" title="No decisions yet"
            description="Create a running bot — as its strategy evaluates markets, every decision (and the reason) appears here in real time." />

        <!-- Decision feed -->
        <div v-else class="mt-6 card overflow-hidden p-0">
            <div class="overflow-x-auto">
                <table class="w-full min-w-[860px] text-sm">
                    <thead>
                        <tr class="border-b border-line text-left text-[11px] uppercase tracking-wide text-muted">
                            <th class="px-5 py-3 font-semibold">Time</th>
                            <th class="px-3 py-3 font-semibold">Market</th>
                            <th class="px-3 py-3 font-semibold">Strategy</th>
                            <th class="px-3 py-3 font-semibold">Side</th>
                            <th class="px-3 py-3 font-semibold">Decision</th>
                            <th class="px-3 py-3 font-semibold">Why</th>
                            <th class="px-3 py-3 font-semibold" title="Rule signal strength">Strength</th>
                            <th class="px-3 py-3 font-semibold" title="AI confidence">AI</th>
                            <th class="px-3 py-3 font-semibold" title="News sentiment">Sent.</th>
                            <th class="px-5 py-3 font-semibold">Regime</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="d in decisions" :key="d.id" class="border-b border-line/60 hover:bg-panel-2/50">
                            <td class="px-5 py-3 tnum text-muted">{{ time(d.ts) }}</td>
                            <td class="px-3 py-3 font-bold text-ink">{{ d.symbol }}</td>
                            <td class="px-3 py-3 text-muted">{{ d.strategy }}</td>
                            <td class="px-3 py-3 font-semibold capitalize" :class="sideCls(d.side)">{{ d.side }}</td>
                            <td class="px-3 py-3"><span :class="['rounded-full px-2.5 py-0.5 text-[11px] font-semibold capitalize', actionCls(d.action)]">{{ d.action }}</span></td>
                            <td class="px-3 py-3 text-xs text-ink">{{ reasonText(d.reason) }}</td>
                            <td class="px-3 py-3 tnum text-muted">{{ d.strength }}</td>
                            <td class="px-3 py-3 tnum" :class="d.ai_confidence != null ? 'text-ink' : 'text-faint'">{{ d.ai_confidence != null ? d.ai_confidence + '%' : '—' }}</td>
                            <td class="px-3 py-3 tnum" :class="d.sentiment > 0 ? 'text-bull' : d.sentiment < 0 ? 'text-bear' : 'text-muted'">{{ d.sentiment != null ? (d.sentiment > 0 ? '+' : '') + d.sentiment : '—' }}</td>
                            <td class="px-5 py-3 text-xs text-muted">{{ regimeText(d.regime) }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <p class="border-t border-line p-3 text-center text-[11px] text-faint">The risk engine is the final authority — strategies propose, AI + sentiment + regime inform, risk decides. Newest decisions first.</p>
        </div>
    </div>
</template>
