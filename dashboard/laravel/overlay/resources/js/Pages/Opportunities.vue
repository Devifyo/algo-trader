<script setup>
import { ref } from 'vue';
import axios from 'axios';
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import EmptyState from '../Components/EmptyState.vue';

defineProps({ holdings: { type: Array, default: () => [] } });

const loading = ref(false);
const error = ref('');
const candidates = ref([]);
const asof = ref('');
const universe = ref(0);

async function scan() {
    loading.value = true; error.value = '';
    try {
        const { data } = await axios.get('/opportunities/scan');
        if (!data.ok) { error.value = data.error || 'Scanner unavailable.'; candidates.value = []; }
        else { candidates.value = data.candidates || []; asof.value = data.asof || ''; universe.value = data.universe_scanned || 0; }
    } catch (e) { error.value = 'Could not reach the scanner.'; }
    finally { loading.value = false; }
}

const usd = (n) => (n >= 1000 ? n.toLocaleString(undefined, { maximumFractionDigits: 0 }) : n >= 1 ? n.toFixed(2) : n.toFixed(4));
const age = (iso) => { if (!iso) return '—'; const d = (Date.now() - new Date(iso).getTime()) / 86400000; return d < 1 ? '<1d' : `${Math.floor(d)}d`; };
</script>

<template>
    <div>
        <PageHeader title="Opportunity Engine"
            description="A live momentum, trend & volume screen of the tradable universe — research signals, surfaced on demand." />

        <!-- Honest disclaimer (compliance-forward) -->
        <div class="mb-5 rounded-xl border border-warn/30 bg-warn/10 px-4 py-3 text-sm text-muted">
            <strong class="text-ink">Research signal — not advice, not a forecast.</strong> These are markets showing
            momentum/trend/volume right now; ranking is <em>not</em> a prediction of future gains, and high momentum
            often means high risk. No orders are placed. Validate in <strong class="text-ink">paper mode</strong> and
            decide for yourself.
        </div>

        <div class="mb-6 flex flex-wrap items-center gap-3">
            <button @click="scan" :disabled="loading"
                class="rounded-lg bg-accent px-5 py-2.5 text-sm font-bold text-white disabled:opacity-60">
                {{ loading ? 'Scanning the market…' : '⚡ Scan markets now' }}
            </button>
            <span v-if="asof" class="text-xs text-faint">Scanned {{ universe }} liquid pairs · {{ new Date(asof).toLocaleTimeString() }}</span>
            <span v-if="error" class="text-xs text-bear">{{ error }}</span>
        </div>

        <!-- Candidates -->
        <div v-if="candidates.length" class="grid gap-4 lg:grid-cols-2">
            <div v-for="c in candidates" :key="c.symbol" class="rounded-xl border border-line bg-panel p-4">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="font-display text-base font-bold text-ink">{{ c.symbol }}</p>
                        <p class="text-xs text-muted">${{ usd(c.price) }} ·
                            <span :class="c.chg24 >= 0 ? 'text-bull' : 'text-bear'">{{ c.chg24 >= 0 ? '+' : '' }}{{ c.chg24 }}% 24h</span></p>
                    </div>
                    <div class="text-right">
                        <p class="text-[10px] uppercase tracking-wide text-faint">Signal score</p>
                        <p class="font-display text-2xl font-extrabold text-accent">{{ c.score }}</p>
                    </div>
                </div>
                <div class="mt-2 h-1.5 w-full overflow-hidden rounded-full bg-panel-2">
                    <div class="h-full rounded-full" :style="{ width: c.score + '%', background: 'linear-gradient(90deg,#6E56F8,#22D3EE)' }"></div>
                </div>
                <div class="mt-3 flex flex-wrap gap-1.5">
                    <span v-for="r in c.reasons" :key="r" class="rounded bg-bull/10 px-2 py-0.5 text-[11px] font-medium text-bull">{{ r }}</span>
                </div>
                <div class="mt-1.5 flex flex-wrap gap-1.5">
                    <span v-for="r in c.risks" :key="r" class="rounded bg-warn/10 px-2 py-0.5 text-[11px] font-medium text-warn">⚠ {{ r }}</span>
                </div>
            </div>
        </div>
        <EmptyState v-else-if="!loading" title="No scan yet"
            description="Hit “Scan markets now” to rank the live universe by momentum, trend strength and volume." />

        <!-- Portfolio awareness -->
        <Card title="Your open positions" description="What you're holding — review winners to ride and laggards to free up buying power." class="mt-8">
            <div v-if="holdings.length" class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="border-b border-line text-left text-[11px] uppercase tracking-wide text-faint">
                        <tr><th class="px-3 py-2">Market</th><th>Side</th><th class="text-right">Qty</th><th class="text-right">Entry</th><th>Strategy</th><th class="pr-3">Held</th></tr>
                    </thead>
                    <tbody class="divide-y divide-line">
                        <tr v-for="(h, i) in holdings" :key="i" class="hover:bg-panel-2/50">
                            <td class="px-3 py-2 font-medium text-ink">{{ h.symbol }}</td>
                            <td class="text-muted">{{ h.side }}</td>
                            <td class="text-right tnum text-muted">{{ h.qty }}</td>
                            <td class="text-right tnum text-muted">${{ usd(h.entry) }}</td>
                            <td class="text-muted">{{ h.strategy }}</td>
                            <td class="pr-3 text-muted">{{ age(h.opened) }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <EmptyState v-else title="No open positions" description="Connect a broker (paper mode is free) and enable a strategy to start." />
        </Card>
    </div>
</template>
