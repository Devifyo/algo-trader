<script setup>
import { reactive } from 'vue';
import { router } from '@inertiajs/vue3';
import { useConfirm } from '../composables/confirm';
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import EmptyState from '../Components/EmptyState.vue';
import Tooltip from '../Components/Tooltip.vue';

const props = defineProps({
    providers: { type: Array, default: () => [] },
    liveStatus: { type: Object, default: () => ({}) },
    mode: { type: String, default: 'cost' },
    settings: { type: Object, default: () => ({}) },
});

const forms = reactive({});          // provider.id -> add-key form state
function formFor(id) {
    if (!forms[id]) forms[id] = { open: false, label: '', secret: '', daily_limit_usd: '', monthly_limit_usd: '', saving: false };
    return forms[id];
}
const toggleProvider = (p) => router.post(`/admin/ai-providers/${p.id}/toggle`, { enabled: !p.enabled }, { preserveScroll: true });
const toggleKey = (k) => router.post(`/admin/ai-providers/keys/${k.id}/toggle`, { enabled: !k.enabled }, { preserveScroll: true });
const confirm = useConfirm();
const deleteKey = async (k) => { if (await confirm('Remove this API key? This cannot be undone.', { title: 'Remove API key', confirmText: 'Remove', tone: 'danger' })) router.post(`/admin/ai-providers/keys/${k.id}/delete`, {}, { preserveScroll: true }); };
function addKey(p) {
    const f = formFor(p.id);
    f.saving = true;
    router.post(`/admin/ai-providers/${p.id}/keys`, {
        label: f.label, secret: f.secret,
        daily_limit_usd: f.daily_limit_usd || null, monthly_limit_usd: f.monthly_limit_usd || null,
    }, {
        preserveScroll: true,
        onFinish: () => { f.saving = false; f.secret = ''; f.label = ''; f.open = false; },
    });
}
const modelCount = (m) => Object.keys(m || {}).filter((k) => !k.startsWith('_')).length;

// ── Per-key request drill-down (expandable, infinite scroll) ──────────────
const details = reactive({}); // key.id -> { open, items, next, loading, done, started, error }
function detailFor(id) {
    if (!details[id]) details[id] = { open: false, items: [], next: null, loading: false, done: false, started: false, error: '' };
    return details[id];
}
async function toggleDetails(k) {
    const d = detailFor(k.id);
    d.open = !d.open;
    if (d.open && !d.started) await loadReqs(k);
}
async function loadReqs(k) {
    const d = detailFor(k.id);
    if (d.loading || d.done) return;
    d.loading = true; d.error = '';
    try {
        const url = `/admin/ai-providers/keys/${k.id}/requests` + (d.next ? `?before=${encodeURIComponent(d.next)}` : '');
        const r = await fetch(url, { headers: { Accept: 'application/json' }, credentials: 'same-origin' });
        if (!r.ok) throw new Error('HTTP ' + r.status);
        const data = await r.json();
        d.items.push(...(data.items || []));
        d.next = data.next || null;
        if (!d.next) d.done = true;
        d.started = true;
    } catch (e) {
        d.error = 'Could not load requests.';
    } finally {
        d.loading = false;
    }
}
function onScroll(e, k) {
    const el = e.target;
    if (el.scrollTop + el.clientHeight >= el.scrollHeight - 48) loadReqs(k);
}
const fmtTime = (t) => t ? new Date(t).toLocaleString() : '';
const cost6 = (c) => '$' + Number(c || 0).toFixed(6);

// ── Admin-editable priority + failover settings ───────────────────────────
const prio = reactive({});            // provider.id -> edited priority
props.providers.forEach((p) => { prio[p.id] = p.priority; });
function savePriority(p) {
    router.post(`/admin/ai-providers/${p.id}/priority`, { priority: Number(prio[p.id]) }, { preserveScroll: true });
}
const settingsForm = reactive({
    retries_per_key: props.settings.retries_per_key ?? 1,
    max_failures: props.settings.max_failures ?? 3,
    circuit_open_seconds: props.settings.circuit_open_seconds ?? 60,
    saving: false,
});
function saveSettings() {
    settingsForm.saving = true;
    router.post('/admin/ai-providers/settings', {
        retries_per_key: Number(settingsForm.retries_per_key),
        max_failures: Number(settingsForm.max_failures),
        circuit_open_seconds: Number(settingsForm.circuit_open_seconds),
    }, { preserveScroll: true, onFinish: () => { settingsForm.saving = false; } });
}
</script>

<template>
    <div>
        <PageHeader title="AI Providers"
            description="Add providers and keys. Requests are load-balanced and fail over automatically, so one provider's outage never stops trading."
            help-topic="AI sentiment analysis">
            <template #actions>
                <Tooltip text="How requests are routed across providers right now.">
                    <span class="rounded-full border border-line px-3 py-1.5 text-xs capitalize text-muted">Routing: {{ mode }}</span>
                </Tooltip>
            </template>
        </PageHeader>

        <div class="mb-6 rounded-xl border border-line bg-panel p-4 text-sm text-muted">
            <strong class="text-ink">How keys are stored:</strong> when you add a key here, the secret is written to
            <strong class="text-ink">Vault</strong> and only a reference is saved in the database — the raw key is never
            stored in Postgres. <strong class="text-ink">Recommended:</strong> add at least two providers so failover has
            somewhere to go.
        </div>

        <!-- Failover settings (admin-editable; applied live, no restart) -->
        <Card title="Failover settings" description="How the router retries and recovers. Changes apply within ~15 seconds — no restart." class="mb-6">
            <div class="grid gap-4 sm:grid-cols-3">
                <label class="block">
                    <span class="text-xs font-semibold text-muted">Retries per key</span>
                    <input v-model.number="settingsForm.retries_per_key" type="number" min="0" max="10"
                        class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none" />
                    <span class="mt-1 block text-[11px] text-faint">Extra attempts on the same key before moving to the next one.</span>
                </label>
                <label class="block">
                    <span class="text-xs font-semibold text-muted">Failures before circuit opens</span>
                    <input v-model.number="settingsForm.max_failures" type="number" min="1" max="50"
                        class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none" />
                    <span class="mt-1 block text-[11px] text-faint">Consecutive failures that temporarily disable a single key.</span>
                </label>
                <label class="block">
                    <span class="text-xs font-semibold text-muted">Cooldown period (seconds)</span>
                    <input v-model.number="settingsForm.circuit_open_seconds" type="number" min="5" max="3600"
                        class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none" />
                    <span class="mt-1 block text-[11px] text-faint">How long a key rests before it's automatically retried.</span>
                </label>
            </div>
            <button @click="saveSettings" :disabled="settingsForm.saving"
                class="mt-4 rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-white disabled:opacity-60">
                {{ settingsForm.saving ? 'Saving…' : 'Save settings' }}
            </button>
        </Card>

        <div v-if="providers.length" class="space-y-4">
            <Card v-for="p in providers" :key="p.id" :title="p.name" :description="`${modelCount(p.models)} models · priority ${p.priority} · weight ${p.weight}`">
                <template #actions>
                    <div class="flex items-center gap-2">
                        <Tooltip text="Failover order — lower number is tried first. Applies live.">
                            <span class="flex items-center gap-1 rounded-full border border-line px-2 py-1 text-xs text-muted">
                                Priority
                                <input v-model.number="prio[p.id]" type="number" min="1" max="999"
                                    class="w-12 rounded bg-panel px-1 py-0.5 text-center text-ink focus:outline-none" />
                                <button @click="savePriority(p)" class="font-semibold text-accent hover:underline">Save</button>
                            </span>
                        </Tooltip>
                        <button @click="toggleProvider(p)"
                            :class="['rounded-full px-3 py-1 text-xs font-semibold border transition',
                                p.enabled ? 'border-bull/30 bg-bull/10 text-bull' : 'border-line text-muted hover:text-ink']">
                            {{ p.enabled ? 'Enabled' : 'Disabled' }}
                        </button>
                    </div>
                </template>

                <div class="grid gap-4 sm:grid-cols-4">
                    <div><p class="text-xs text-muted">Requests (24h)</p><p class="font-display text-lg font-semibold tnum text-ink">{{ p.usage_24h.requests }}</p></div>
                    <div><p class="text-xs text-muted">Cost (24h)</p><p class="font-display text-lg font-semibold tnum text-ink">${{ p.usage_24h.cost_usd }}</p></div>
                    <div><p class="text-xs text-muted">Avg latency</p><p class="font-display text-lg font-semibold tnum text-ink">{{ p.usage_24h.avg_latency_ms }}ms</p></div>
                    <div><p class="text-xs text-muted">Failures (24h)</p><p class="font-display text-lg font-semibold tnum" :class="p.usage_24h.failures ? 'text-bear' : 'text-ink'">{{ p.usage_24h.failures }}</p></div>
                </div>

                <div class="mt-4">
                    <div class="mb-2 flex items-center justify-between">
                        <p class="text-xs font-semibold text-muted">API keys</p>
                        <button @click="formFor(p.id).open = !formFor(p.id).open"
                            class="rounded-lg border border-accent/40 bg-accent/10 px-2.5 py-1 text-xs font-semibold text-accent hover:bg-accent/20 transition">
                            {{ formFor(p.id).open ? 'Cancel' : '+ Add key' }}
                        </button>
                    </div>

                    <!-- Add-key form -->
                    <form v-if="formFor(p.id).open" @submit.prevent="addKey(p)"
                        class="mb-3 space-y-3 rounded-lg border border-line bg-surface p-3">
                        <div class="grid gap-3 sm:grid-cols-2">
                            <label class="block"><span class="text-xs text-muted">Label</span>
                                <input v-model="formFor(p.id).label" required placeholder="e.g. key-1"
                                    class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none" /></label>
                            <label class="block"><span class="text-xs text-muted">API key (secret)</span>
                                <input v-model="formFor(p.id).secret" type="password" required placeholder="paste the provider key"
                                    class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none" /></label>
                            <label class="block"><span class="text-xs text-muted">Daily limit $ (optional)</span>
                                <input v-model="formFor(p.id).daily_limit_usd" type="number" step="0.01" min="0"
                                    class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none" /></label>
                            <label class="block"><span class="text-xs text-muted">Monthly limit $ (optional)</span>
                                <input v-model="formFor(p.id).monthly_limit_usd" type="number" step="0.01" min="0"
                                    class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink focus:border-accent focus:outline-none" /></label>
                        </div>
                        <button type="submit" :disabled="formFor(p.id).saving"
                            class="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-white disabled:opacity-60">
                            {{ formFor(p.id).saving ? 'Saving to Vault…' : 'Save key' }}
                        </button>
                        <p class="text-xs text-muted">The secret is written to Vault; only a reference is kept in the database.</p>
                    </form>

                    <ul v-if="p.keys.length" class="divide-y divide-line rounded-lg border border-line">
                        <li v-for="k in p.keys" :key="k.id" class="px-3 py-2.5 text-sm">
                            <div class="flex items-center justify-between gap-3">
                                <button @click="toggleDetails(k)" class="flex items-center gap-2 text-left">
                                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
                                        :class="['text-faint transition-transform', detailFor(k.id).open ? 'rotate-90' : '']"><path d="M9 18l6-6-6-6"/></svg>
                                    <span class="font-medium text-ink">{{ k.label }}</span>
                                    <span :class="['rounded-full px-2 py-0.5 text-xs', k.status === 'healthy' ? 'text-bull bg-bull/10' : 'text-bear bg-bear/10']">{{ k.status }}</span>
                                    <span v-if="k.failures" class="text-xs text-bear">{{ k.failures }} fails</span>
                                </button>
                                <div class="flex items-center gap-2">
                                    <button @click="toggleDetails(k)"
                                        class="rounded-full border border-line px-2.5 py-0.5 text-xs font-semibold text-muted hover:text-ink">
                                        {{ detailFor(k.id).open ? 'Hide' : 'Requests' }}
                                    </button>
                                    <button @click="toggleKey(k)"
                                        :class="['rounded-full px-2.5 py-0.5 text-xs font-semibold border',
                                            k.enabled ? 'border-bull/30 bg-bull/10 text-bull' : 'border-line text-muted']">
                                        {{ k.enabled ? 'On' : 'Off' }}
                                    </button>
                                    <button @click="deleteKey(k)" title="Remove key"
                                        class="rounded-full border border-line px-2 py-0.5 text-xs text-muted hover:text-bear hover:border-bear/40">✕</button>
                                </div>
                            </div>

                            <!-- Expandable request log (load more on scroll) -->
                            <div v-if="detailFor(k.id).open" class="mt-2 overflow-hidden rounded-lg border border-line bg-surface">
                                <div class="max-h-80 overflow-y-auto" @scroll="onScroll($event, k)">
                                    <table v-if="detailFor(k.id).items.length" class="w-full text-xs">
                                        <thead class="sticky top-0 z-10 bg-panel text-left text-[10px] uppercase tracking-wide text-faint">
                                            <tr>
                                                <th class="px-2 py-1.5">Time</th><th>Model</th><th>Kind</th>
                                                <th class="text-right">Latency</th><th class="text-right">Tokens (in/out)</th>
                                                <th class="text-right">Cost</th><th class="pr-2">Status</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-line align-top">
                                            <tr v-for="(r, i) in detailFor(k.id).items" :key="i" class="hover:bg-panel-2/50">
                                                <td class="px-2 py-1.5 whitespace-nowrap text-muted">{{ fmtTime(r.time) }}</td>
                                                <td class="text-muted">{{ r.model }}</td>
                                                <td class="text-muted">{{ r.kind }}</td>
                                                <td class="text-right tnum text-muted">{{ r.latency_ms }}ms</td>
                                                <td class="text-right tnum text-muted">{{ r.in_tokens }}/{{ r.out_tokens }}</td>
                                                <td class="text-right tnum text-muted">{{ cost6(r.cost) }}</td>
                                                <td class="pr-2">
                                                    <span :class="['rounded-full px-2 py-0.5 text-[11px] font-semibold', r.success ? 'text-bull bg-bull/10' : 'text-bear bg-bear/10']">{{ r.success ? 'OK' : 'Failed' }}</span>
                                                    <p v-if="!r.success && r.error" class="mt-1 max-w-sm whitespace-pre-wrap break-words text-[11px] text-bear/90">{{ r.error }}</p>
                                                </td>
                                            </tr>
                                        </tbody>
                                    </table>
                                    <div v-if="detailFor(k.id).loading" class="px-3 py-3 text-center text-xs text-muted">Loading…</div>
                                    <div v-else-if="detailFor(k.id).error" class="px-3 py-3 text-center text-xs text-bear">{{ detailFor(k.id).error }}</div>
                                    <div v-else-if="!detailFor(k.id).items.length" class="px-3 py-4 text-center text-xs text-muted">No requests recorded for this key.</div>
                                    <button v-else-if="!detailFor(k.id).done" @click="loadReqs(k)"
                                        class="w-full py-2 text-xs font-semibold text-accent hover:underline">Load more</button>
                                    <div v-else class="px-3 py-2 text-center text-[11px] text-faint">— end of history —</div>
                                </div>
                            </div>
                        </li>
                    </ul>
                    <p v-else class="rounded-lg border border-dashed border-line px-3 py-3 text-center text-xs text-muted">
                        No keys yet. Click “+ Add key” to connect this provider.
                    </p>
                </div>
            </Card>
        </div>
        <EmptyState v-else title="No providers configured" description="Seed runs on first boot. If you see this, the database may still be initializing." />
    </div>
</template>
