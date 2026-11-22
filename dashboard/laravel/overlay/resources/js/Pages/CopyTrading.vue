<script setup>
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import RiskBadge from '../Components/RiskBadge.vue';
import EmptyState from '../Components/EmptyState.vue';

defineProps({ traders: { type: Array, default: () => [] }, following: { type: Array, default: () => [] } });
const pct = (n) => (n == null ? '—' : (n >= 0 ? '+' : '') + Number(n).toFixed(1) + '%');
</script>

<template>
    <div>
        <PageHeader title="Copy trading"
            description="Mirror verified traders automatically. Built-in max-drawdown protection stops copying if losses exceed your limit."
            help-topic="how copy trading works" />

        <Card v-if="following.length" class="mb-6" title="You're following">
            <ul class="divide-y divide-line">
                <li v-for="f in following" :key="f.id" class="flex items-center justify-between py-3 text-sm">
                    <div><p class="font-medium text-ink">{{ f.display_name }}</p>
                        <p class="text-xs text-muted">{{ f.copy_mode.replace('_', ' ') }} · stop at {{ f.max_drawdown_stop }}% drawdown</p></div>
                    <span class="rounded-full border border-line px-2.5 py-0.5 text-xs capitalize text-muted">{{ f.status }}</span>
                </li>
            </ul>
        </Card>

        <h2 class="mb-3 font-display text-sm font-semibold text-ink">Top traders</h2>
        <div v-if="traders.length" class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <Card v-for="t in traders" :key="t.id">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <span class="font-display font-semibold text-ink">{{ t.display_name }}</span>
                        <span v-if="t.verified" class="rounded-full bg-accent/10 px-2 py-0.5 text-xs text-accent">Verified</span>
                    </div>
                    <RiskBadge :level="t.risk_rating || 'medium'" />
                </div>
                <div class="mt-3 grid grid-cols-3 gap-2 text-sm">
                    <div><p class="text-xs text-muted">Monthly</p><p class="tnum font-semibold" :class="(t.monthly_return||0) >= 0 ? 'text-bull' : 'text-bear'">{{ pct(t.monthly_return) }}</p></div>
                    <div><p class="text-xs text-muted">Drawdown</p><p class="tnum font-semibold text-ink">{{ pct(t.max_drawdown) }}</p></div>
                    <div><p class="text-xs text-muted">Followers</p><p class="tnum font-semibold text-ink">{{ t.followers }}</p></div>
                </div>
                <button class="mt-4 w-full rounded-lg bg-accent px-3 py-1.5 text-sm font-semibold text-white">Copy trader</button>
            </Card>
        </div>
        <EmptyState v-else title="No traders to copy yet" description="Verified traders with a track record will appear here." />
    </div>
</template>
