<script setup>
import { router } from '@inertiajs/vue3';
import PageHeader from '../../Components/PageHeader.vue';
import Card from '../../Components/Card.vue';
import EmptyState from '../../Components/EmptyState.vue';

defineProps({ flags: { type: Array, default: () => [] } });
const toggle = (f) => router.post(`/admin/feature-flags/${f.id}/toggle`, { enabled: !f.enabled }, { preserveScroll: true });
</script>

<template>
    <div>
        <PageHeader title="Feature flags" description="Turn platform capabilities on or off. Changes take effect immediately." />

        <Card>
            <ul v-if="flags.length" class="divide-y divide-line">
                <li v-for="f in flags" :key="f.id" class="flex items-center justify-between gap-4 py-3">
                    <div>
                        <p class="font-medium text-ink">{{ f.flag.replace('_', ' ') }}</p>
                        <p class="text-xs text-muted">{{ f.description }} · scope: {{ f.scope }}</p>
                    </div>
                    <button @click="toggle(f)"
                        :class="['rounded-full px-3 py-1 text-xs font-semibold border transition',
                            f.enabled ? 'border-bull/30 bg-bull/10 text-bull' : 'border-line text-muted hover:text-ink']">
                        {{ f.enabled ? 'Enabled' : 'Disabled' }}
                    </button>
                </li>
            </ul>
            <EmptyState v-else title="No feature flags" />
        </Card>
    </div>
</template>
