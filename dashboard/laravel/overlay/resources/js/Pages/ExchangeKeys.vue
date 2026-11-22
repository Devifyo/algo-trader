<script setup>
import { reactive, computed } from 'vue';
import { router, usePage } from '@inertiajs/vue3';
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import EmptyState from '../Components/EmptyState.vue';

defineProps({ keys: { type: Array, default: () => [] } });
const page = usePage();
const safety = computed(() => page.props.flash?.safetyResult || null);

const form = reactive({
    exchange: 'binance', label: 'main',
    ip_whitelisted: false, reachable: true, valid_key: true,
    permissions: { read_permission: true, spot_permission: true, trading_permission: true,
        margin_permission: false, futures_permission: false, withdraw_permission: false },
});
const test = () => router.post('/exchange-keys/test', form, { preserveScroll: true });
const save = () => router.post('/exchange-keys', form, { preserveScroll: true });
const labels = {
    read_permission: 'Read account data', spot_permission: 'Spot trading',
    margin_permission: 'Margin trading', futures_permission: 'Futures trading',
    trading_permission: 'Place orders', withdraw_permission: 'Withdraw funds',
};
</script>

<template>
    <div>
        <PageHeader title="Exchange keys"
            description="Connect a trade-only API key. We verify permissions and refuse any key that can withdraw funds."
            help-topic="API permissions" />

        <div class="mb-6 rounded-xl border border-warn/30 bg-warn/10 p-4 text-sm text-ink">
            <strong>Safety first.</strong> On your exchange, create an API key with trading enabled and
            <strong>withdrawals DISABLED</strong>, and restrict it to this server's IP. The platform never needs
            withdrawal access and will reject keys that have it.
        </div>

        <div class="grid gap-6 lg:grid-cols-2">
            <Card title="Connect a key" description="We test before saving — no key is stored until it passes.">
                <div class="space-y-3">
                    <div class="grid grid-cols-2 gap-3">
                        <label class="block"><span class="text-xs text-muted">Exchange</span>
                            <select v-model="form.exchange" class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink">
                                <option>binance</option><option>bybit</option><option>okx</option><option>coinbase</option>
                            </select></label>
                        <label class="block"><span class="text-xs text-muted">Label</span>
                            <input v-model="form.label" class="mt-1 w-full rounded-lg border border-line bg-panel px-3 py-2 text-sm text-ink" /></label>
                    </div>
                    <fieldset class="rounded-lg border border-line p-3">
                        <legend class="px-1 text-xs text-muted">Key permissions</legend>
                        <label v-for="(lbl, key) in labels" :key="key" class="flex items-center justify-between py-1 text-sm">
                            <span :class="key === 'withdraw_permission' ? 'text-bear font-medium' : 'text-ink'">{{ lbl }}</span>
                            <input type="checkbox" v-model="form.permissions[key]" class="h-4 w-4 accent-[rgb(var(--accent))]" />
                        </label>
                    </fieldset>
                    <label class="flex items-center justify-between text-sm"><span class="text-ink">Restricted to this server's IP</span>
                        <input type="checkbox" v-model="form.ip_whitelisted" class="h-4 w-4 accent-[rgb(var(--accent))]" /></label>
                    <div class="flex gap-2 pt-1">
                        <button @click="test" class="rounded-lg border border-line px-3 py-2 text-sm font-semibold text-ink hover:bg-surface">Test connection</button>
                        <button @click="save" :disabled="form.permissions.withdraw_permission"
                            class="rounded-lg bg-accent px-3 py-2 text-sm font-semibold text-white disabled:opacity-50">Save key</button>
                    </div>
                </div>

                <!-- Safety result -->
                <div v-if="safety" class="mt-4 rounded-lg border p-3 text-sm"
                    :class="safety.safe ? 'border-bull/30 bg-bull/10' : 'border-bear/30 bg-bear/10'">
                    <p class="font-semibold" :class="safety.safe ? 'text-bull' : 'text-bear'">
                        {{ safety.safe ? 'Safe to save ✓' : 'Fix these before saving' }}
                    </p>
                    <ul class="mt-2 space-y-1">
                        <li v-for="(wn, i) in safety.warnings" :key="i" class="text-muted">• {{ wn.explanation || wn.code }}</li>
                    </ul>
                    <ul v-if="safety.fix_steps?.length" class="mt-2 list-inside list-disc text-xs text-muted">
                        <li v-for="(f, i) in safety.fix_steps" :key="i">{{ f }}</li>
                    </ul>
                </div>
            </Card>

            <Card title="Connected keys" description="Stored references only — secrets live in Vault.">
                <ul v-if="keys.length" class="divide-y divide-line">
                    <li v-for="k in keys" :key="k.id" class="flex items-center justify-between py-3 text-sm">
                        <div><p class="font-medium text-ink">{{ k.exchange }} · {{ k.label }}</p>
                            <p class="text-xs text-muted">{{ k.ip_whitelisted ? 'IP-restricted' : 'No IP restriction' }}</p></div>
                        <span :class="['rounded-full px-2.5 py-0.5 text-xs font-semibold', k.safe ? 'text-bull bg-bull/10' : 'text-bear bg-bear/10']">{{ k.safe ? 'Safe' : 'Review' }}</span>
                    </li>
                </ul>
                <EmptyState v-else title="No keys connected" description="Add a trade-only key to let bots place orders." />
            </Card>
        </div>
    </div>
</template>
