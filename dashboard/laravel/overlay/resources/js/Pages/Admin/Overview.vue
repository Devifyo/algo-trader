<script setup>
import { router } from '@inertiajs/vue3';
import { useConfirm } from '../../composables/confirm';
import PageHeader from '../../Components/PageHeader.vue';
import Card from '../../Components/Card.vue';
import StatCard from '../../Components/StatCard.vue';
import EmptyState from '../../Components/EmptyState.vue';

const props = defineProps({ stats: { type: Object, default: () => ({}) }, providers: { type: Array, default: () => [] }, recentAudit: { type: Array, default: () => [] } });
const confirm = useConfirm();
const toggleKill = async () => {
    const on = !props.stats.kill_switch;
    if (on && !(await confirm('Engage the GLOBAL kill switch across ALL tenants? Every tenant stops placing new entries.', { title: 'Global kill switch', confirmText: 'Engage globally', tone: 'danger' }))) return;
    router.post('/kill-switch', { on }, { preserveScroll: true });
};
const when = (t) => t ? new Date(t).toLocaleString() : '';
</script>

<template>
    <div>
        <PageHeader title="Platform overview" description="Everything across every tenant. You are the SaaS operator here.">
            <template #actions>
                <button @click="toggleKill"
                    :class="['rounded-lg border px-3 py-1.5 text-sm font-semibold transition',
                        stats.kill_switch ? 'border-bull/40 bg-bull/10 text-bull' : 'border-bear/40 bg-bear/10 text-bear']">
                    {{ stats.kill_switch ? 'Clear global kill switch' : 'Global kill switch' }}
                </button>
            </template>
        </PageHeader>

        <div v-if="stats.kill_switch" class="mb-6 rounded-xl border border-bear/40 bg-bear/10 px-4 py-3 text-sm text-bear">
            <strong>Global kill switch engaged.</strong> No tenant is placing new entries.
        </div>

        <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
            <StatCard label="Tenants" :value="stats.tenants ?? 0" hint="Total registered workspaces." />
            <StatCard label="Users" :value="stats.users ?? 0" />
            <StatCard label="Active bots" :value="stats.bots ?? 0" />
            <StatCard label="Open positions" :value="stats.open_positions ?? 0" />
            <StatCard label="AI spend (24h)" :value="'$' + (stats.ai_cost_24h ?? 0)" hint="Token cost across all providers in the last 24h." />
        </div>

        <div class="mt-6 grid gap-6 lg:grid-cols-2">
            <Card title="AI providers" description="Global provider health.">
                <ul v-if="providers.length" class="space-y-2">
                    <li v-for="p in providers" :key="p.name" class="flex items-center justify-between text-sm">
                        <span class="font-medium text-ink capitalize">{{ p.name }}</span>
                        <span :class="['rounded-full px-2 py-0.5 text-xs', p.enabled ? 'text-bull bg-bull/10' : 'text-muted bg-surface']">{{ p.enabled ? 'enabled' : 'disabled' }}</span>
                    </li>
                </ul>
                <EmptyState v-else title="No providers" />
            </Card>

            <Card title="Recent activity" description="Audit log across the platform.">
                <ul v-if="recentAudit.length" class="space-y-2 text-sm">
                    <li v-for="(a, i) in recentAudit" :key="i" class="flex items-center justify-between">
                        <span class="text-ink">{{ a.action }}</span>
                        <span class="text-xs text-muted">{{ a.actor }} · {{ when(a.at) }}</span>
                    </li>
                </ul>
                <EmptyState v-else title="No activity yet" />
            </Card>
        </div>
    </div>
</template>
