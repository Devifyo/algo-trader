<script setup>
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import HealthScore from '../Components/HealthScore.vue';
import EmptyState from '../Components/EmptyState.vue';

defineProps({ bots: { type: Array, default: () => [] }, allocations: { type: Array, default: () => [] }, health: { type: Array, default: () => [] }, decisions: { type: Array, default: () => [] } });
</script>

<template>
    <div>
        <PageHeader title="Portfolio"
            description="Your bots, capital allocation, strategy health, and the AI's explained decisions — managed as one book."
            help-topic="risk management" />

        <div class="grid gap-6 lg:grid-cols-2">
            <Card title="Capital allocation" description="AI-suggested split across strategies and bots by risk-adjusted performance.">
                <ul v-if="allocations.length" class="space-y-3">
                    <li v-for="(a, i) in allocations" :key="i">
                        <div class="flex items-center justify-between text-sm"><span class="text-ink">{{ a.target_id }}</span><span class="tnum text-muted">{{ Math.round(a.weight * 100) }}%</span></div>
                        <div class="mt-1 h-1.5 w-full rounded-full bg-line"><div class="h-full rounded-full bg-accent" :style="{ width: (a.weight * 100) + '%' }"></div></div>
                    </li>
                </ul>
                <EmptyState v-else title="No allocations yet" description="Allocations appear once strategies have a track record." />
            </Card>

            <Card title="Strategy health" description="Decay, degradation, and overfitting are flagged automatically.">
                <ul v-if="health.length" class="space-y-4">
                    <li v-for="(h, i) in health" :key="i" class="flex items-center gap-4">
                        <HealthScore :score="h.health_score || 0" :label="h.strategy" :size="48" />
                        <div class="flex flex-wrap gap-1">
                            <span v-for="(f, j) in (h.flags || [])" :key="j" class="rounded-full bg-warn/10 px-2 py-0.5 text-xs text-warn">{{ f.replace('_', ' ') }}</span>
                            <span v-if="!(h.flags || []).length" class="text-xs text-bull">healthy</span>
                        </div>
                    </li>
                </ul>
                <EmptyState v-else title="No health data" description="Strategies are evaluated continuously once they trade." />
            </Card>
        </div>

        <Card class="mt-6" title="AI decisions" description="Every decision is explained — never a black box." help-topic="AI sentiment analysis">
            <ul v-if="decisions.length" class="divide-y divide-line">
                <li v-for="(d, i) in decisions" :key="i" class="py-3">
                    <div class="flex items-center justify-between">
                        <span class="font-medium text-ink">{{ d.symbol || '—' }}</span>
                        <span :class="['rounded-full px-2.5 py-0.5 text-xs font-semibold',
                            d.decision === 'allow' ? 'text-bull bg-bull/10' : d.decision === 'reject' ? 'text-bear bg-bear/10' : 'text-warn bg-warn/10']">{{ d.decision }}</span>
                    </div>
                    <p class="mt-1 text-sm text-muted">{{ (JSON.parse(d.explanation || '{}').reason) || 'No explanation recorded.' }}</p>
                </li>
            </ul>
            <EmptyState v-else title="No decisions yet" description="The unified AI decision layer logs each call here with its reasoning." />
        </Card>

        <Card class="mt-6" title="Bots">
            <div v-if="bots.length" class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="text-left text-xs text-muted"><tr><th class="py-2">Name</th><th>Type</th><th>Status</th></tr></thead>
                    <tbody class="divide-y divide-line">
                        <tr v-for="b in bots" :key="b.id"><td class="py-2.5 font-medium text-ink">{{ b.name }}</td><td class="text-muted">{{ b.type }}</td><td class="capitalize text-muted">{{ b.status }}</td></tr>
                    </tbody>
                </table>
            </div>
            <EmptyState v-else title="No bots" description="Create a bot during onboarding to start paper trading." />
        </Card>
    </div>
</template>
