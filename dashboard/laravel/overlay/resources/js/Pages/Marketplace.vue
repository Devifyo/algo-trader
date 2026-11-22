<script setup>
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import RiskBadge from '../Components/RiskBadge.vue';
import EmptyState from '../Components/EmptyState.vue';

defineProps({ strategies: { type: Array, default: () => [] } });
const riskLevel = (s) => s >= 70 ? 'high' : s >= 45 ? 'medium' : 'low';
const pct = (n) => (n == null ? '—' : (n >= 0 ? '+' : '') + Number(n).toFixed(1) + '%');
</script>

<template>
    <div>
        <PageHeader title="Strategy marketplace"
            description="Browse verified strategies you can subscribe to, rent, or buy. Past performance never guarantees future results."
            help-topic="how copy trading works" />

        <div v-if="strategies.length" class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <Card v-for="s in strategies" :key="s.id" :title="s.name" :description="s.description">
                <template #actions><RiskBadge :level="riskLevel(s.risk_score || 50)" /></template>
                <div class="grid grid-cols-2 gap-3 text-sm">
                    <div><p class="text-xs text-muted">Monthly</p><p class="tnum font-semibold" :class="(s.monthly_return||0) >= 0 ? 'text-bull' : 'text-bear'">{{ pct(s.monthly_return) }}</p></div>
                    <div><p class="text-xs text-muted">Max drawdown</p><p class="tnum font-semibold text-ink">{{ pct(s.max_drawdown) }}</p></div>
                    <div><p class="text-xs text-muted">Win rate</p><p class="tnum font-semibold text-ink">{{ s.win_rate != null ? Math.round(s.win_rate * 100) + '%' : '—' }}</p></div>
                    <div><p class="text-xs text-muted">Subscribers</p><p class="tnum font-semibold text-ink">{{ s.subscribers }}</p></div>
                </div>
                <div class="mt-4 flex items-center justify-between">
                    <span class="text-sm font-semibold text-ink">{{ s.price_usd > 0 ? '$' + s.price_usd + '/mo' : 'Free' }}</span>
                    <button class="rounded-lg bg-accent px-3 py-1.5 text-sm font-semibold text-white">Subscribe</button>
                </div>
            </Card>
        </div>
        <EmptyState v-else title="No strategies listed yet"
            description="Be the first — publish a strategy that meets the validation bar and earn from subscribers." />
    </div>
</template>
