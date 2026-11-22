<script setup>
import { ref, reactive, computed } from 'vue';
import { router } from '@inertiajs/vue3';
import { useConfirm } from '../composables/confirm';
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import EmptyState from '../Components/EmptyState.vue';

const props = defineProps({
    marketsOrder: { type: Array, default: () => ['India', 'Global', 'Crypto'] },
    byMarket: { type: Object, default: () => ({}) },
    connections: { type: Array, default: () => [] },
});

const tab = ref(props.marketsOrder[0]);
const active = ref(null);
const helpActive = ref(null);
const logoStep = reactive({});           // broker_id -> which logo source we're on
const form = reactive({ label: '', creds: {}, environment: 'paper', ip_whitelisted: false, withdraw_disabled_confirmed: false, saving: false });

const openHelp = (b) => { helpActive.value = b; };
function helpFor(b) {
    const ip = '(Recommended) Restrict the key to this server’s IP.';
    const paper = 'Choose Paper to test risk-free, or Live for real funds.';
    if (b.credential_kind === 'api_key_secret' || b.credential_kind === 'api_key_secret_passphrase') {
        return {
            title: `How to connect ${b.name}`,
            steps: [
                `Log in to ${b.name} and open the API / API Management section.`,
                'Create a new API key with TRADING enabled and WITHDRAWALS DISABLED.',
                ...(b.credential_kind === 'api_key_secret_passphrase' ? ['Set a passphrase for the key (this exchange requires one).'] : []),
                b.supports_paper ? paper : ip,
                'Copy the API Key and Secret, paste them here, then Connect.',
            ],
            note: 'Your secret is sent straight to Vault — it is never stored in our database.',
        };
    }
    if (b.credential_kind === 'api_key_totp') {
        return {
            title: `How to connect ${b.name}`,
            steps: [
                `Log in to ${b.name} and enable API access in account settings.`,
                'Generate your API key and secret.',
                'Set up TOTP (e.g. Google Authenticator) and copy the TOTP secret.',
                'Paste the API key, secret, and TOTP secret here, then Connect.',
            ],
            note: 'The TOTP secret lets us generate the daily login token automatically.',
        };
    }
    if (b.credential_kind === 'oauth2') {
        return {
            title: `How to connect ${b.name}`,
            steps: [
                `Open the ${b.name} developer portal and create an app.`,
                'Copy the Client ID and Client Secret.',
                'Set the redirect URI if the broker requires it (optional here).',
                'Paste the Client ID and Secret, then Connect.',
            ],
            note: 'OAuth app credentials are stored in Vault; the sign-in completes on first use.',
        };
    }
    return {
        title: `How to connect ${b.name}`,
        steps: [
            `Log in to ${b.name} and open Manage API Access.`,
            'Generate a personal access token and find your Account ID.',
            'Paste the Account ID and token here, then Connect.',
        ],
        note: 'Use a practice account first to test risk-free.',
    };
}

const FIELD_LABEL = {
    api_key: 'API Key', api_secret: 'API Secret', passphrase: 'Passphrase',
    totp_secret: 'TOTP secret', client_id: 'Client ID', client_secret: 'Client Secret',
    redirect_uri: 'Redirect URI (optional)', account_id: 'Account ID', token: 'Access Token',
};
const MONO_COLORS = ['#4F46E5', '#0E9F6E', '#0891B2', '#7C3AED', '#DB2777', '#EA580C', '#0D9488', '#2563EB'];

// Logo chain: self-hosted (always served by us) → Clearbit → Google favicon → monogram.
const sourcesFor = (b) => [
    `/broker-logos/${b.broker_id}.png`,
    ...(b.logo_domain ? [
        `https://logo.clearbit.com/${b.logo_domain}`,
        `https://www.google.com/s2/favicons?domain=${b.logo_domain}&sz=128`,
    ] : []),
];
function logoSrc(b) {
    const srcs = sourcesFor(b);
    const step = logoStep[b.broker_id] ?? 0;
    return step < srcs.length ? srcs[step] : null;   // null => show monogram
}
const onLogoError = (b) => { logoStep[b.broker_id] = (logoStep[b.broker_id] ?? 0) + 1; };
const initials = (name) => name.replace(/\(.*?\)/g, '').trim().split(/\s+/).slice(0, 2).map((w) => w[0]).join('').toUpperCase();
const monoColor = (id) => MONO_COLORS[[...id].reduce((a, c) => a + c.charCodeAt(0), 0) % MONO_COLORS.length];
const isConnected = (id) => props.connections.some((c) => c.broker_id === id || c.exchange === id);

function openConnect(b) {
    active.value = b;
    form.label = b.name;
    form.creds = Object.fromEntries(b.fields.map((f) => [f, '']));
    form.environment = 'paper';
    form.ip_whitelisted = false;
    form.withdraw_disabled_confirmed = false;
}
function submit() {
    form.saving = true;
    router.post(`/brokers/${active.value.broker_id}/connect`, {
        label: form.label, creds: form.creds, environment: form.environment,
        ip_whitelisted: form.ip_whitelisted, withdraw_disabled_confirmed: form.withdraw_disabled_confirmed,
    }, { preserveScroll: true, onFinish: () => { form.saving = false; }, onSuccess: () => { active.value = null; } });
}
const confirm = useConfirm();
const disconnect = async (c) => { if (await confirm('Disconnect this broker? Its bots will stop routing live orders.', { title: 'Disconnect broker', confirmText: 'Disconnect', tone: 'danger' })) router.post(`/broker-connections/${c.id}/disconnect`, {}, { preserveScroll: true }); };
const list = computed(() => props.byMarket[tab.value] || []);
const marketCount = (m) => (props.byMarket[m] || []).length;
</script>

<template>
    <div>
        <PageHeader title="Brokers & exchanges"
            description="Connect your broker to let bots trade. Credentials are encrypted in Vault — never stored in our database. Use trade-only API keys with withdrawals disabled."
            help-topic="API permissions" />

        <!-- Connected -->
        <Card v-if="connections.length" class="mb-6" title="Connected accounts">
            <ul class="divide-y divide-line">
                <li v-for="c in connections" :key="c.id" class="flex items-center justify-between gap-3 py-3">
                    <div class="flex items-center gap-3">
                        <div class="flex h-9 w-9 items-center justify-center rounded-lg text-xs font-bold text-white"
                            :style="{ background: monoColor(c.broker_id || c.exchange) }">
                            {{ initials(c.label || c.broker_id || c.exchange) }}
                        </div>
                        <div>
                            <p class="text-sm font-semibold text-ink">{{ c.label }}</p>
                            <p class="text-xs text-muted">{{ c.broker_id || c.exchange }} · {{ c.ip_whitelisted ? 'IP-restricted' : 'no IP restriction' }}</p>
                        </div>
                    </div>
                    <div class="flex items-center gap-2">
                        <span :class="['rounded-full px-2.5 py-0.5 text-xs font-semibold', c.safe ? 'text-bull bg-bull/10' : 'text-warn bg-warn/10']">{{ c.safe ? 'Connected' : 'Review' }}</span>
                        <button @click="disconnect(c)" class="rounded-lg border border-line px-2.5 py-1 text-xs font-medium text-muted hover:text-bear hover:border-bear/40">Disconnect</button>
                    </div>
                </li>
            </ul>
        </Card>

        <!-- Market tabs -->
        <div class="mb-5 inline-flex rounded-xl border border-line bg-panel p-1 shadow-sm">
            <button v-for="m in marketsOrder" :key="m" @click="tab = m"
                :class="['flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition',
                    tab === m ? 'bg-accent text-white shadow-sm' : 'text-muted hover:text-ink']">
                {{ m }}
                <span :class="['rounded-full px-1.5 text-[11px]', tab === m ? 'bg-white/20' : 'bg-line text-muted']">{{ marketCount(m) }}</span>
            </button>
        </div>

        <!-- Catalog -->
        <div v-if="list.length" class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
            <div v-for="b in list" :key="b.broker_id"
                class="card group flex flex-col p-5 transition hover:-translate-y-0.5 hover:border-accent/40">
                <div class="flex items-start gap-3">
                    <!-- Logo with monogram fallback -->
                    <div class="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-line"
                        :class="logoSrc(b) ? 'bg-white' : ''"
                        :style="logoSrc(b) ? {} : { background: monoColor(b.broker_id) }">
                        <img v-if="logoSrc(b)" :src="logoSrc(b)" :alt="b.name" referrerpolicy="no-referrer"
                            class="h-7 w-7 object-contain" @error="onLogoError(b)" />
                        <span v-else class="font-display text-sm font-bold text-white">{{ initials(b.name) }}</span>
                    </div>
                    <div class="min-w-0 flex-1">
                        <div class="flex items-center justify-between gap-2">
                            <h3 class="truncate font-display text-sm font-bold text-ink">{{ b.name }}</h3>
                            <span v-if="isConnected(b.broker_id)" class="shrink-0 rounded-full bg-bull/10 px-2 py-0.5 text-[11px] font-semibold text-bull">Connected</span>
                        </div>
                        <p class="mt-0.5 truncate text-xs text-muted">{{ b.exchanges.join(' · ') }}</p>
                    </div>
                </div>

                <p class="mt-4 text-xs text-muted"><span class="font-medium text-ink">Auth</span> · {{ b.auth_type }}</p>
                <div class="mt-2 flex flex-wrap gap-1.5">
                    <span v-for="(f, i) in b.features.slice(0, 3)" :key="i"
                        class="rounded-md bg-panel-2 px-2 py-0.5 text-[11px] font-medium text-muted">{{ f }}</span>
                </div>

                <div class="mt-5 flex items-center justify-between border-t border-line pt-4">
                    <div class="flex items-center gap-2">
                        <button @click="openHelp(b)" title="How to connect"
                            class="inline-flex h-7 w-7 items-center justify-center rounded-full border border-line text-[12px] font-semibold text-muted hover:border-accent hover:text-accent">?</button>
                        <a :href="b.documentation_url" target="_blank" rel="noopener" class="text-xs font-medium text-muted hover:text-accent">Docs ↗</a>
                    </div>
                    <button @click="openConnect(b)"
                        class="rounded-lg grad-accent px-3.5 py-1.5 text-sm font-semibold text-white shadow-sm hover:opacity-90">
                        {{ isConnected(b.broker_id) ? 'Reconnect' : 'Connect' }}
                    </button>
                </div>
            </div>
        </div>
        <EmptyState v-else title="No brokers in this market" />

        <!-- Connect modal -->
        <div v-if="active" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="active = null">
            <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
            <div class="relative w-full max-w-md card p-6">
                <div class="mb-5 flex items-center gap-3">
                    <div class="flex h-11 w-11 items-center justify-center overflow-hidden rounded-xl border border-line"
                        :class="logoSrc(active) ? 'bg-white' : ''"
                        :style="logoSrc(active) ? {} : { background: monoColor(active.broker_id) }">
                        <img v-if="logoSrc(active)" :src="logoSrc(active)" :alt="active.name" referrerpolicy="no-referrer" class="h-6 w-6 object-contain" @error="onLogoError(active)" />
                        <span v-else class="font-display text-sm font-bold text-white">{{ initials(active.name) }}</span>
                    </div>
                    <div class="min-w-0 flex-1">
                        <h3 class="truncate font-display text-base font-bold text-ink">Connect {{ active.name }}</h3>
                        <p class="truncate text-xs text-muted">{{ active.auth_type }}</p>
                    </div>
                    <button @click="active = null" class="rounded-lg p-1 text-muted hover:bg-panel-2 hover:text-ink">✕</button>
                </div>

                <form @submit.prevent="submit" class="space-y-3">
                    <label class="block"><span class="text-xs font-medium text-muted">Label</span>
                        <input v-model="form.label" class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink" /></label>

                    <label v-for="f in active.fields" :key="f" class="block">
                        <span class="text-xs font-medium text-muted">{{ FIELD_LABEL[f] || f }}</span>
                        <input v-model="form.creds[f]" :type="['redirect_uri','account_id','client_id'].includes(f) ? 'text' : 'password'"
                            :required="f !== 'redirect_uri'" autocomplete="off"
                            class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink" />
                    </label>

                    <div v-if="active.supports_paper">
                        <span class="text-xs font-medium text-muted">Environment</span>
                        <div class="mt-1 inline-flex rounded-lg border border-line bg-panel-2 p-1">
                            <button type="button" @click="form.environment = 'paper'" :class="['rounded-md px-4 py-1.5 text-xs font-semibold', form.environment === 'paper' ? 'grad-accent text-white' : 'text-muted']">Paper</button>
                            <button type="button" @click="form.environment = 'live'" :class="['rounded-md px-4 py-1.5 text-xs font-semibold', form.environment === 'live' ? 'grad-accent text-white' : 'text-muted']">Live</button>
                        </div>
                    </div>

                    <label class="flex items-center justify-between text-sm"><span class="text-ink">Restricted to this server's IP</span>
                        <input v-model="form.ip_whitelisted" type="checkbox" class="h-4 w-4 accent-[rgb(var(--accent))]" /></label>

                    <label v-if="active.market === 'Crypto'" class="flex items-start gap-2 rounded-lg border border-warn/30 bg-warn/10 p-3 text-xs text-ink">
                        <input v-model="form.withdraw_disabled_confirmed" type="checkbox" class="mt-0.5 h-4 w-4 accent-[rgb(var(--accent))]" required />
                        <span>I confirm <strong>withdrawals are DISABLED</strong> on this API key. The platform never needs withdrawal access.</span>
                    </label>

                    <p v-if="active.credential_kind === 'oauth2'" class="rounded-lg border border-line bg-panel-2 p-3 text-xs text-muted">
                        Paste your OAuth <strong class="text-ink">app credentials</strong> (client id / secret). The full sign-in handshake for this broker is completed on first use.
                    </p>

                    <button type="submit" :disabled="form.saving"
                        class="w-full rounded-lg grad-accent px-3 py-2.5 text-sm font-semibold text-white shadow-sm hover:opacity-90 disabled:opacity-60">
                        {{ form.saving ? 'Saving to Vault…' : 'Connect broker' }}
                    </button>
                </form>
            </div>
        </div>

        <!-- Help modal -->
        <div v-if="helpActive" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="helpActive = null">
            <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
            <div class="relative w-full max-w-md card p-6">
                <div class="mb-4 flex items-start justify-between gap-3">
                    <h3 class="font-display text-base font-bold text-ink">{{ helpFor(helpActive).title }}</h3>
                    <button @click="helpActive = null" class="rounded-lg p-1 text-muted hover:bg-panel-2 hover:text-ink">✕</button>
                </div>
                <ol class="space-y-3">
                    <li v-for="(s, i) in helpFor(helpActive).steps" :key="i" class="flex gap-3">
                        <span class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full grad-accent text-xs font-bold text-white">{{ i + 1 }}</span>
                        <span class="text-sm text-ink">{{ s }}</span>
                    </li>
                </ol>
                <p v-if="helpFor(helpActive).note" class="mt-4 rounded-lg border border-line bg-panel-2 p-3 text-xs text-muted">{{ helpFor(helpActive).note }}</p>
                <div class="mt-5 flex items-center justify-between">
                    <a :href="helpActive.documentation_url" target="_blank" rel="noopener" class="text-xs font-semibold text-accent hover:underline">Official docs ↗</a>
                    <button @click="openConnect(helpActive); helpActive = null" class="rounded-lg grad-accent px-3.5 py-1.5 text-sm font-semibold text-white">Connect now</button>
                </div>
            </div>
        </div>
    </div>
</template>
