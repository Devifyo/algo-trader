<script setup>
import PageHeader from '../Components/PageHeader.vue';
import EmptyState from '../Components/EmptyState.vue';

defineProps({ notifications: { type: Array, default: () => [] } });

const when = (t) => { try { return new Date(t).toLocaleString(); } catch { return t; } };
const dot = (s) => ({ critical: 'bg-bear', warning: 'bg-warn' }[s] || 'bg-accent');
const ring = (s) => ({ critical: 'border-bear/30', warning: 'border-warn/30' }[s] || 'border-line');
const icon = (k) => ({ allocation_full: '💰', system: '⚙️' }[k] || '🔔');
</script>

<template>
    <div>
        <PageHeader title="Notifications" description="Alerts from your bots and the platform — allocation limits, halts, and system events. New alerts also email you." />

        <EmptyState v-if="!notifications.length" class="mt-6" title="No notifications"
            description="You're all caught up. Bot and platform alerts (like a bot hitting its allocation limit) will appear here and in your inbox." />

        <div v-else class="mt-6 space-y-3">
            <div v-for="n in notifications" :key="n.id" :class="['card flex items-start gap-4 p-4', ring(n.severity), n.read ? '' : 'ring-1 ring-accent/20']">
                <div class="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-panel-2 text-lg">{{ icon(n.kind) }}</div>
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                        <span :class="['h-2 w-2 rounded-full', dot(n.severity)]"></span>
                        <p class="font-display text-sm font-bold text-ink">{{ n.title }}</p>
                        <span v-if="!n.read" class="rounded-full bg-accent/10 px-2 py-0.5 text-[10px] font-semibold text-accent">new</span>
                    </div>
                    <p class="mt-1 text-sm text-muted">{{ n.body }}</p>
                    <p class="mt-1.5 text-[11px] text-faint">{{ when(n.created_at) }}</p>
                </div>
            </div>
        </div>
    </div>
</template>
