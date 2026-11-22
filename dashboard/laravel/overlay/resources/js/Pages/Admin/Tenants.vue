<script setup>
import { router } from '@inertiajs/vue3';
import { useConfirm } from '../../composables/confirm';
import PageHeader from '../../Components/PageHeader.vue';
import Card from '../../Components/Card.vue';
import EmptyState from '../../Components/EmptyState.vue';

defineProps({ tenants: { type: Array, default: () => [] } });
const confirm = useConfirm();
const setStatus = (t, status) => router.post(`/admin/tenants/${t.id}/status`, { status }, { preserveScroll: true });
const impersonate = async (t) => {
    if (await confirm(`Sign in as ${t.name} to see their workspace? You can return to admin anytime.`, { title: 'Impersonate tenant', confirmText: 'Sign in', tone: 'default' }))
        router.post(`/admin/tenants/${t.id}/impersonate`);
};
const when = (t) => t ? new Date(t).toLocaleDateString() : '';
</script>

<template>
    <div>
        <PageHeader title="Tenants" description="Every workspace on the platform. Suspend a tenant to immediately stop its trading." />

        <Card>
            <div v-if="tenants.length" class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="text-left text-xs text-muted">
                        <tr><th class="py-2">Workspace</th><th>Owner</th><th>Region</th><th>Plan</th><th>Users</th><th>Status</th><th>Joined</th><th></th></tr>
                    </thead>
                    <tbody class="divide-y divide-line">
                        <tr v-for="t in tenants" :key="t.id">
                            <td class="py-2.5 font-medium text-ink">{{ t.name }}</td>
                            <td class="text-muted">{{ t.owner_email || '—' }}</td>
                            <td class="text-muted">{{ t.region }}</td>
                            <td class="capitalize text-muted">{{ t.plan }}</td>
                            <td class="tnum text-muted">{{ t.users }}</td>
                            <td><span :class="['rounded-full px-2 py-0.5 text-xs', t.status === 'active' ? 'text-bull bg-bull/10' : 'text-bear bg-bear/10']">{{ t.status }}</span></td>
                            <td class="text-xs text-muted">{{ when(t.created_at) }}</td>
                            <td class="text-right">
                                <div class="flex items-center justify-end gap-2">
                                    <button @click="impersonate(t)" class="rounded-lg border border-accent/40 px-2.5 py-1 text-xs font-semibold text-accent hover:bg-accent/10">Impersonate</button>
                                    <button v-if="t.status === 'active'" @click="setStatus(t, 'suspended')" class="rounded-lg border border-bear/40 px-2.5 py-1 text-xs font-semibold text-bear hover:bg-bear/10">Suspend</button>
                                    <button v-else @click="setStatus(t, 'active')" class="rounded-lg border border-bull/40 px-2.5 py-1 text-xs font-semibold text-bull hover:bg-bull/10">Reactivate</button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <EmptyState v-else title="No tenants yet" description="Tenants appear here as users register." />
        </Card>
    </div>
</template>
