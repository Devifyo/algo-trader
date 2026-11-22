<script setup>
import { ref, reactive, computed } from 'vue';
import { router } from '@inertiajs/vue3';
import { useConfirm } from '../composables/confirm';
import PageHeader from '../Components/PageHeader.vue';
import NavIcon from '../Components/NavIcon.vue';
import EmptyState from '../Components/EmptyState.vue';

const props = defineProps({
    bots: { type: Array, default: () => [] },
    kpi: { type: Object, default: () => ({}) },
    options: { type: Object, default: () => ({ strategies: {}, symbols: [] }) },
});
const filter = ref('all');
const q = ref('');

const brokerList = computed(() => props.options.brokers || [{ id: 'paper', label: 'Paper (simulated)', live: false }]);
const defaultBroker = () => (brokerList.value.find((b) => b.live) || brokerList.value[0] || { id: 'paper' }).id;
const brokerById = (id) => brokerList.value.find((b) => b.id === id);

const symbols = computed(() => props.options.symbols || []);
const modes = computed(() => props.options.modes || { ai_auto: 'AI Auto-Discovery' });

// Each setting explained for the “?” hints so users aren't confused.
const hints = {
    broker: 'Where this bot sends its orders. “Paper (simulated)” uses virtual fills. A connected broker (e.g. Alpaca) routes REAL orders to that account.',
    strategy: 'The trading logic the bot runs — it decides when to buy and sell from live market data.',
    mode: 'How the bot picks markets. AI Auto-Discovery scans every market and trades the best opportunities. Custom Watchlist limits it to markets you choose. Single Symbol trades just one.',
    watchlist: 'The markets this strategy monitors. It continuously scans all of them and trades the strongest setups.',
    allocation: 'Total capital (USD) this bot manages. Total deployed never exceeds it.',
    budget: 'How much of the per-market budget the AI may spend on a single trade. It sizes between your Min and Max based on conviction — stronger setups get more, weaker ones less.',
};

// Create / edit bot modal
const creating = ref(false);
const editingId = ref(null);
const form = reactive({ name: '', strategy: 'trend_following', mode: 'ai_auto', symbol: 'BTCUSDT', watchlist: [], allocation: 1000, min_pct: 25, max_pct: 100, broker: 'paper', saving: false });
function openCreate() {
    editingId.value = null;
    Object.assign(form, { name: '', strategy: 'trend_following', mode: 'ai_auto', symbol: 'BTCUSDT', watchlist: [...symbols.value.slice(0, 6)], allocation: 1000, min_pct: 25, max_pct: 100, broker: defaultBroker(), saving: false });
    creating.value = true;
}
function openEdit(b) {
    editingId.value = b.id;
    Object.assign(form, {
        name: b.name, strategy: b.strategy_key || 'trend_following', mode: b.mode || 'ai_auto',
        symbol: b.symbol || 'BTCUSDT', watchlist: [...(b.watchlist || [])],
        allocation: b.allocation || 1000, min_pct: b.min_pct ?? 25, max_pct: b.max_pct ?? 100,
        broker: b.broker || 'paper', saving: false,
    });
    creating.value = true;
}
const toggleWatch = (s) => {
    const i = form.watchlist.indexOf(s);
    if (i === -1) form.watchlist.push(s); else form.watchlist.splice(i, 1);
};
function saveBot() {
    form.saving = true;
    const payload = {
        name: form.name, strategy: form.strategy, mode: form.mode,
        symbol: form.symbol, watchlist: form.watchlist, allocation: form.allocation,
        min_pct: form.min_pct, max_pct: form.max_pct, broker: form.broker,
    };
    const opts = {
        preserveScroll: true,
        onFinish: () => { form.saving = false; },
        onSuccess: () => { creating.value = false; editingId.value = null; },
    };
    if (editingId.value) router.patch(`/bots/${editingId.value}`, payload, opts);
    else router.post('/bots', payload, opts);
}
const confirm = useConfirm();
const removeBot = async (b) => { if (await confirm(`Delete “${b.name}”? This stops its strategy.`, { title: 'Delete bot', confirmText: 'Delete', tone: 'danger' })) router.delete(`/bots/${b.id}`, { preserveScroll: true }); };
const filters = ['all', 'running', 'paused', 'stopped'];
const usd = (n) => '$' + Number(n || 0).toLocaleString(undefined, { maximumFractionDigits: 2 });
const statusPill = (s) => ({
    running: 'text-bull bg-bull/10', paused: 'text-warn bg-warn/10', stopped: 'text-bear bg-bear/10',
}[s] || 'text-muted bg-panel-2');
const coinInits = (b) => (b.coin || b.name || '?').slice(0, 3).toUpperCase();
const list = computed(() => props.bots.filter((b) =>
    (filter.value === 'all' || b.status === filter.value) &&
    (!q.value || (b.name + b.pair + b.strategy).toLowerCase().includes(q.value.toLowerCase()))));
const toggle = (b) => router.post(`/bots/${b.id}/toggle`, {}, { preserveScroll: true });
const allocPct = (b) => b.alloc_total > 0 ? Math.min(100, b.alloc_used / b.alloc_total * 100) : 0;
</script>

<template>
    <div>
        <PageHeader title="Bots" description="Run multiple bots as one managed portfolio. Start, pause, and monitor every strategy." help-topic="paper trading">
            <template #actions>
                <button @click="openCreate" class="inline-flex items-center gap-1.5 rounded-xl grad-accent px-3.5 py-2 text-sm font-semibold text-white shadow-sm hover:opacity-90">+ New Bot</button>
            </template>
        </PageHeader>

        <!-- KPI row -->
        <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
            <div class="card p-5">
                <div class="flex items-start justify-between"><span class="eyebrow text-[11px] text-muted">Active Bots</span><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-blue text-white"><NavIcon name="bots" /></div></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-ink">{{ kpi.active ?? 0 }}</p>
                <p class="mt-2 text-xs font-medium text-bull">▲ of {{ kpi.total_bots ?? 0 }} total</p>
            </div>
            <div class="card p-5">
                <div class="flex items-start justify-between"><span class="eyebrow text-[11px] text-muted">Total Trades (24h)</span><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-violet text-white"><NavIcon name="dashboard" /></div></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-ink">{{ (kpi.trades_24h ?? 0).toLocaleString() }}</p>
                <p class="mt-2 text-xs text-muted">across all bots</p>
            </div>
            <div class="card p-5">
                <div class="flex items-start justify-between"><span class="eyebrow text-[11px] text-muted">Total Profit (24h)</span><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-emerald text-white"><NavIcon name="portfolio" /></div></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-bull">+{{ usd(kpi.profit_24h) }}</p>
                <p class="mt-2 text-xs text-muted">realized today</p>
            </div>
            <div class="card p-5">
                <div class="flex items-start justify-between"><span class="eyebrow text-[11px] text-muted">Avg Win Rate</span><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-amber text-white"><NavIcon name="admin" /></div></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-ink">{{ kpi.win_rate ?? 0 }}%</p>
                <div class="mt-2.5 h-2 w-full rounded-full bg-panel-2"><div class="h-full rounded-full grad-emerald" :style="{ width: (kpi.win_rate || 0) + '%' }"></div></div>
            </div>
        </div>

        <!-- Search + filters -->
            <div class="mt-5 flex flex-wrap items-center gap-3">
                <div class="relative min-w-[220px] flex-1">
                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-faint" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M21 21l-4-4"/></svg>
                    <input v-model="q" type="search" placeholder="Search bots by name, pair, or strategy…"
                        class="h-10 w-full rounded-xl border border-line bg-panel pl-9 pr-3 text-sm text-ink" />
                </div>
                <div class="flex items-center gap-1.5">
                    <button v-for="f in filters" :key="f" @click="filter = f"
                        :class="['rounded-full px-3.5 py-1.5 text-xs font-semibold capitalize transition',
                            filter === f ? 'grad-accent text-white' : 'border border-line text-muted hover:text-ink']">{{ f }}</button>
                </div>
            </div>

            <!-- Bot cards -->
            <div v-if="list.length" class="mt-5 grid gap-4 lg:grid-cols-2 xl:grid-cols-3">
                <div v-for="b in list" :key="b.id" class="card overflow-hidden p-0">
                    <div class="h-1 w-full" :class="b.grad"></div>
                    <div class="p-5">
                        <div class="flex items-start justify-between">
                            <div class="flex items-center gap-3">
                                <div class="flex h-10 w-10 items-center justify-center rounded-full text-xs font-bold text-white" :class="b.grad">{{ coinInits(b) }}</div>
                                <div>
                                    <p class="font-display text-sm font-bold text-ink">{{ b.name }}</p>
                                    <p class="flex items-center gap-1 text-xs text-muted">
                                        <span v-if="b.mode === 'ai_auto'" class="text-accent">🔍</span>{{ b.markets_label || b.pair }}
                                    </p>
                                </div>
                            </div>
                            <div class="flex flex-col items-end gap-1.5">
                                <span :class="['inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-semibold capitalize', statusPill(b.status)]">
                                    <span class="h-1.5 w-1.5 rounded-full bg-current"></span>{{ b.status }}
                                </span>
                                <span :class="['inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-semibold', b.is_live ? 'bg-accent/10 text-accent' : 'bg-panel-2 text-muted']"
                                    :title="b.is_live ? 'Live order routing to your ' + b.broker_label + ' account' : 'Simulated fills — no real broker'">
                                    {{ b.is_live ? '⚡' : '◷' }} {{ b.broker_label }}
                                </span>
                            </div>
                        </div>

                        <div class="mt-4 grid grid-cols-2 gap-y-3 text-sm">
                            <div><p class="text-xs text-muted">Strategy</p><p class="font-medium text-ink">{{ b.strategy }}</p></div>
                            <div><p class="text-xs text-muted">Trades (24h)</p><p class="font-medium text-ink tnum">{{ b.trades_24h }}</p></div>
                            <div><p class="text-xs text-muted">Profit (24h)</p><p class="font-medium tnum" :class="b.profit_24h > 0 ? 'text-bull' : 'text-ink'">{{ b.profit_24h > 0 ? '+' + usd(b.profit_24h) : usd(b.profit_24h) }}</p></div>
                            <div><p class="text-xs text-muted">Win Rate</p><p class="font-medium text-ink tnum">{{ b.win_rate }}%</p></div>
                        </div>

                        <div class="mt-4">
                            <div class="flex items-center justify-between text-xs"><span class="text-muted">Allocation</span><span class="tnum text-ink">{{ usd(b.alloc_used) }} / {{ usd(b.alloc_total) }}</span></div>
                            <div class="mt-1.5 h-2 w-full rounded-full bg-panel-2"><div class="h-full rounded-full" :class="b.grad" :style="{ width: allocPct(b) + '%' }"></div></div>
                        </div>

                        <div class="mt-4 flex items-center gap-2">
                            <button @click="toggle(b)"
                                :class="['flex-1 rounded-xl py-2.5 text-sm font-semibold text-white shadow-sm',
                                    b.status === 'running' ? 'bg-bear hover:opacity-90' : 'grad-emerald hover:opacity-90']">
                                {{ b.status === 'running' ? '❚❚ Pause' : '▶ Start' }}
                            </button>
                            <button @click="openEdit(b)" class="rounded-xl border border-line p-2.5 text-muted hover:bg-panel-2 hover:text-ink" title="Edit bot">
                                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>
                            </button>
                            <button @click="removeBot(b)" class="rounded-xl border border-line p-2.5 text-muted hover:border-bear/40 hover:text-bear" title="Delete bot">✕</button>
                        </div>
                    </div>
                </div>
            </div>
            <EmptyState v-else class="mt-6" :title="bots.length ? 'No bots match' : 'No bots yet'"
                :description="bots.length ? 'Try a different filter or search term.' : 'Create your first bot to start paper trading a strategy.'">
                <button v-if="!bots.length" @click="openCreate" class="mt-3 rounded-lg grad-accent px-3 py-1.5 text-sm font-semibold text-white">Create your first bot</button>
            </EmptyState>

        <!-- Create bot modal -->
        <div v-if="creating" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="creating = false">
            <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
            <div class="relative w-full max-w-md card p-6">
                <div class="mb-5 flex items-center justify-between">
                    <h3 class="font-display text-base font-bold text-ink">{{ editingId ? 'Edit bot' : 'New bot' }}</h3>
                    <button @click="creating = false" class="rounded-lg p-1 text-muted hover:bg-panel-2 hover:text-ink">✕</button>
                </div>
                <form @submit.prevent="saveBot" class="space-y-3">
                    <label class="block"><span class="text-xs font-medium text-muted">Bot name</span>
                        <input v-model="form.name" required placeholder="e.g. BTC Trend Rider"
                            class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink" /></label>

                    <label class="block">
                        <span class="flex items-center gap-1 text-xs font-medium text-muted">Broker
                            <span class="inline-flex h-3.5 w-3.5 cursor-help items-center justify-center rounded-full border border-line text-[9px] text-muted" :title="hints.broker">?</span></span>
                        <select v-model="form.broker" class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink">
                            <option v-for="b in brokerList" :key="b.id" :value="b.id">{{ b.label }}{{ b.live ? ' — live order routing' : '' }}</option>
                        </select></label>

                    <label class="block">
                        <span class="flex items-center gap-1 text-xs font-medium text-muted">Strategy
                            <span class="inline-flex h-3.5 w-3.5 cursor-help items-center justify-center rounded-full border border-line text-[9px] text-muted" :title="hints.strategy">?</span></span>
                        <select v-model="form.strategy" class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink">
                            <option v-for="(label, key) in options.strategies" :key="key" :value="key">{{ label }}</option>
                        </select></label>

                    <label class="block">
                        <span class="flex items-center gap-1 text-xs font-medium text-muted">Market selection
                            <span class="inline-flex h-3.5 w-3.5 cursor-help items-center justify-center rounded-full border border-line text-[9px] text-muted" :title="hints.mode">?</span></span>
                        <select v-model="form.mode" class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink">
                            <option v-for="(label, key) in modes" :key="key" :value="key">{{ label }}</option>
                        </select></label>

                    <!-- AI auto: scans the whole universe -->
                    <p v-if="form.mode === 'ai_auto'" class="rounded-lg border border-accent/30 bg-accent/10 p-2.5 text-xs text-ink">
                        🔍 Scans <strong>all {{ symbols.length }} markets</strong> and trades the best risk-adjusted opportunities — see the <a href="/markets" class="font-semibold text-accent hover:underline">Markets</a> dashboard for live rankings.
                    </p>
                    <!-- Single symbol -->
                    <label v-else-if="form.mode === 'single_symbol'" class="block">
                        <span class="text-xs font-medium text-muted">Symbol</span>
                        <select v-model="form.symbol" class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink">
                            <option v-for="s in symbols" :key="s" :value="s">{{ s }}</option>
                        </select></label>
                    <!-- Custom watchlist -->
                    <div v-else>
                        <span class="flex items-center gap-1 text-xs font-medium text-muted">Watchlist ({{ form.watchlist.length }})
                            <span class="inline-flex h-3.5 w-3.5 cursor-help items-center justify-center rounded-full border border-line text-[9px] text-muted" :title="hints.watchlist">?</span></span>
                        <div class="mt-1 flex flex-wrap gap-1.5">
                            <button v-for="s in symbols" :key="s" type="button" @click="toggleWatch(s)"
                                :class="['rounded-lg border px-2.5 py-1.5 text-xs font-semibold transition', form.watchlist.includes(s) ? 'grad-accent border-transparent text-white' : 'border-line text-muted hover:text-ink']">{{ s }}</button>
                        </div>
                    </div>
                    <label class="block">
                        <span class="flex items-center gap-1 text-xs font-medium text-muted">Allocation (USD)
                            <span class="inline-flex h-3.5 w-3.5 cursor-help items-center justify-center rounded-full border border-line text-[9px] text-muted" :title="hints.allocation">?</span></span>
                        <input v-model.number="form.allocation" type="number" min="10" step="10" required
                            class="mt-1 w-full rounded-lg border border-line bg-panel-2 px-3 py-2.5 text-sm text-ink" /></label>

                    <div>
                        <span class="flex items-center gap-1 text-xs font-medium text-muted">AI trade budget — Min / Max (% of allocation)
                            <span class="inline-flex h-3.5 w-3.5 cursor-help items-center justify-center rounded-full border border-line text-[9px] text-muted" :title="hints.budget">?</span></span>
                        <div class="mt-1 flex items-center gap-2">
                            <div class="flex items-center gap-1.5 rounded-lg border border-line bg-panel-2 px-3 py-2">
                                <span class="text-[11px] text-muted">Min</span>
                                <input v-model.number="form.min_pct" type="number" min="1" max="100" class="w-12 bg-transparent text-sm text-ink outline-none" />
                                <span class="text-[11px] text-muted">%</span>
                            </div>
                            <div class="flex items-center gap-1.5 rounded-lg border border-line bg-panel-2 px-3 py-2">
                                <span class="text-[11px] text-muted">Max</span>
                                <input v-model.number="form.max_pct" type="number" min="1" max="100" class="w-12 bg-transparent text-sm text-ink outline-none" />
                                <span class="text-[11px] text-muted">%</span>
                            </div>
                            <span class="text-[11px] text-muted">AI picks within this range by conviction</span>
                        </div>
                    </div>

                    <p class="rounded-lg border border-line bg-panel-2 p-2.5 text-xs text-muted">
                        <template v-if="form.broker === 'paper'"><strong class="text-ink">Paper (simulated)</strong> — virtual fills on real market data.</template>
                        <template v-else>Routes <strong class="text-accent">real orders</strong> to your {{ brokerById(form.broker)?.label }} account (Paper/Live per how you connected it).</template>
                        The risk engine sizes and validates every trade.
                    </p>
                    <button type="submit" :disabled="form.saving"
                        class="w-full rounded-lg grad-accent px-3 py-2.5 text-sm font-semibold text-white shadow-sm hover:opacity-90 disabled:opacity-60">
                        {{ form.saving ? 'Saving…' : (editingId ? 'Save changes' : 'Create & start bot') }}
                    </button>
                </form>
            </div>
        </div>
    </div>
</template>
