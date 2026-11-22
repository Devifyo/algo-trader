<script setup>
import { ref, computed } from 'vue';
import { router } from '@inertiajs/vue3';
import PageHeader from '../Components/PageHeader.vue';
import NavIcon from '../Components/NavIcon.vue';
import EmptyState from '../Components/EmptyState.vue';
import Tooltip from '../Components/Tooltip.vue';

// Column headers with a plain-language explanation shown on hover.
const COLUMNS = [
    { label: 'Timestamp', cls: 'px-4 py-3', help: 'When the order was placed (shown in your local time).' },
    { label: 'Log ID', cls: '', help: 'Unique reference for this order.' },
    { label: 'Bot', cls: '', help: 'The strategy/bot that placed the order.' },
    { label: 'Type', cls: '', help: '“trade” = an order that went to the venue; “error” = the venue rejected it.' },
    { label: 'Pair', cls: '', help: 'The market traded, e.g. BTC = Bitcoin against USD.' },
    { label: 'Side', cls: '', help: 'BUY = opening or adding to a position. SELL = closing or reducing it.' },
    { label: 'Amount', cls: 'text-right', help: 'Quantity of the asset (coins) — e.g. 0.00014 BTC. Small numbers are normal for pricey coins.' },
    { label: 'Price', cls: 'text-right', help: 'Price per 1 unit of the asset at the time of the order.' },
    { label: 'Value', cls: 'text-right', help: 'How much was bought/sold in USD = Amount × Price. This is the total size of the trade.' },
    { label: 'Status', cls: '', help: 'Filled = executed. Pending = sent, awaiting fill. Failed = venue rejected it. Canceled = skipped before sending.' },
    { label: 'Actions', cls: 'text-right pr-4', help: 'View the full details of this order.' },
];

const props = defineProps({
    logs: { type: Array, default: () => [] },
    kpi: { type: Object, default: () => ({}) },
    counts: { type: Object, default: () => ({}) },
    pagination: { type: Object, default: () => ({}) },
    filters: { type: Object, default: () => ({}) },
    options: { type: Object, default: () => ({}) },
});
const tab = ref(props.filters.tab || 'all');
const q = ref(props.filters.q || '');
// Full filter set (all server-side, so they apply across the entire history).
const fStatus = ref(props.filters.status || 'all');
const fSide = ref(props.filters.side || 'all');
const fBroker = ref(props.filters.broker || 'all');
const fSymbol = ref(props.filters.symbol || 'all');
const fFrom = ref(props.filters.from || '');
const fTo = ref(props.filters.to || '');
const tabs = computed(() => [
    { key: 'all', label: 'All Logs', count: props.counts.all ?? 0 },
    { key: 'trades', label: 'Trades', count: props.counts.trades ?? 0 },
    { key: 'errors', label: 'Errors', count: props.counts.errors ?? 0 },
]);
const activeFilters = computed(() =>
    (fStatus.value !== 'all') + (fSide.value !== 'all') + (fBroker.value !== 'all') +
    (fSymbol.value !== 'all') + (!!fFrom.value) + (!!fTo.value) + (!!q.value));

// Server-driven: every filter + paging reloads from the server, so the table
// pages through the FULL history instead of a client-side slice.
const reload = (params = {}) => router.get('/logs', {
    tab: tab.value,
    status: fStatus.value !== 'all' ? fStatus.value : undefined,
    side: fSide.value !== 'all' ? fSide.value : undefined,
    broker: fBroker.value !== 'all' ? fBroker.value : undefined,
    symbol: fSymbol.value !== 'all' ? fSymbol.value : undefined,
    from: fFrom.value || undefined,
    to: fTo.value || undefined,
    q: q.value || undefined,
    ...params,
}, { preserveState: true, preserveScroll: true, replace: true });
const setTab = (key) => { tab.value = key; reload({ page: 1 }); };
const applyFilters = () => reload({ page: 1 });
const resetFilters = () => {
    tab.value = 'all'; q.value = ''; fStatus.value = 'all'; fSide.value = 'all';
    fBroker.value = 'all'; fSymbol.value = 'all'; fFrom.value = ''; fTo.value = '';
    reload({ page: 1 });
};
let searchTimer = null;
const onSearch = () => { clearTimeout(searchTimer); searchTimer = setTimeout(() => reload({ page: 1 }), 350); };
const goPage = (p) => reload({ page: p });
const usd = (n) => '$' + Number(n || 0).toLocaleString(undefined, { maximumFractionDigits: 2 });
const num = (n) => Number(n || 0).toLocaleString(undefined, { maximumFractionDigits: 8 });
const when = (t) => t ? new Date(t).toLocaleString() : '';
const reasonText = (r) => {
    if (!r) return '';
    // Engine-side skips (order never sent to the venue).
    if (r.startsWith('BELOW_MIN_NOTIONAL')) return 'Order value below the broker’s minimum order size — skipped';
    if (r.startsWith('SHORT_NOT_SUPPORTED')) return 'This broker can’t short — short entry skipped';
    if (r.startsWith('ALPACA_UNSUPPORTED')) return 'Alpaca does not support this crypto — order skipped';
    // Venue rejections (translated tags, raw message appended after the colon).
    if (r.startsWith('ALPACA_BELOW_MIN_NOTIONAL')) return 'Below Alpaca’s $10 minimum order size';
    if (r.startsWith('ALPACA_INSUFFICIENT_FUNDS')) return 'Alpaca rejected: insufficient buying power';
    if (r.startsWith('ALPACA_NO_SHORT')) return 'Alpaca can’t short crypto — order rejected';
    if (r.startsWith('ALPACA_422')) return 'Alpaca rejected: invalid order parameters';
    if (r.startsWith('ALPACA_HTTP')) return 'Could not reach Alpaca';
    if (r.startsWith('ALPACA_403')) return 'Alpaca rejected: below $10 minimum or insufficient balance';
    if (r.startsWith('ALPACA_')) return 'Rejected by Alpaca';
    return ({
        VAULT_KEY_LOAD_FAILED: 'Could not load your broker keys from Vault',
        BROKER_LOOKUP_FAILED: 'Could not resolve the bot’s broker',
        KILL_SWITCH_OR_TENANT_HALT: 'Blocked — kill-switch or account halt',
    }[r] || r);
};
const statusPill = (s) => ({
    filled: ['Filled', 'text-bull bg-bull/10'], submitted: ['Pending', 'text-warn bg-warn/10'],
    rejected: ['Failed', 'text-bear bg-bear/10'],
}[s] || [s, 'text-muted bg-panel-2']);
const typePill = (k) => k === 'error' ? 'text-bear bg-bear/10' : 'text-accent bg-accent/10';
// Server already filtered + paginated; render what it returned.
const list = computed(() => props.logs);
const refresh = () => reload();

// View detail modal
const selected = ref(null);
const openView = (l) => { selected.value = l; };

// Export the current view to CSV
function exportCsv() {
    const cols = ['time', 'log_id', 'bot', 'kind', 'pair', 'side', 'qty', 'price', 'status', 'broker', 'reason'];
    const esc = (v) => '"' + String(v ?? '').replace(/"/g, '""') + '"';
    const rows = [cols.join(',')].concat(list.value.map((l) => cols.map((c) => esc(l[c])).join(',')));
    const blob = new Blob([rows.join('\n')], { type: 'text/csv' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `execution-logs-${tab.value}.csv`;
    a.click();
    URL.revokeObjectURL(a.href);
}
</script>

<template>
    <div>
        <PageHeader title="Execution Logs" description="Real-time monitoring of all bot trades and system operations.">
            <template #actions>
                <button @click="refresh" class="inline-flex items-center gap-1.5 rounded-xl border border-line px-3 py-1.5 text-sm font-semibold text-muted hover:text-ink">⟳ Refresh</button>
                <button @click="exportCsv" class="inline-flex items-center gap-1.5 rounded-xl border border-line px-3 py-1.5 text-sm font-semibold text-muted hover:text-ink">⤓ Export</button>
            </template>
        </PageHeader>

        <!-- KPI -->
        <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
            <div class="card p-5">
                <div class="flex items-start justify-between"><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-blue text-white"><NavIcon name="dashboard" /></div><span class="rounded-full bg-bull/10 px-2 py-0.5 text-xs font-semibold text-bull">live</span></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-ink">{{ (kpi.total ?? 0).toLocaleString() }}</p>
                <p class="mt-1 text-sm text-muted">Total Executions</p>
            </div>
            <div class="card p-5">
                <div class="flex items-start justify-between"><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-emerald text-white"><NavIcon name="portfolio" /></div><span class="rounded-full bg-bull/10 px-2 py-0.5 text-xs font-semibold text-bull">{{ kpi.success_rate ?? 0 }}%</span></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-ink">{{ kpi.success_rate ?? 0 }}%</p>
                <p class="mt-1 text-sm text-muted">Success Rate</p>
            </div>
            <div class="card p-5">
                <div class="flex items-start justify-between"><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-rose text-white"><NavIcon name="admin" /></div></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-ink">{{ kpi.failed ?? 0 }}</p>
                <p class="mt-1 text-sm text-muted">Failed Trades</p>
            </div>
            <div class="card p-5">
                <div class="flex items-start justify-between"><div class="flex h-10 w-10 items-center justify-center rounded-xl grad-violet text-white"><NavIcon name="brokers" /></div></div>
                <p class="mt-3 font-display text-3xl font-extrabold tnum text-ink">{{ usd(kpi.volume) }}</p>
                <p class="mt-1 text-sm text-muted">Total Volume</p>
            </div>
        </div>

        <!-- Tabs -->
        <div class="mt-6 flex flex-wrap gap-1.5">
            <button v-for="t in tabs" :key="t.key" @click="setTab(t.key)"
                :class="['inline-flex items-center gap-2 rounded-xl px-3.5 py-2 text-sm font-semibold transition',
                    tab === t.key ? 'grad-accent text-white' : 'border border-line text-muted hover:text-ink']">
                {{ t.label }}
                <span :class="['rounded-full px-1.5 text-[11px]', tab === t.key ? 'bg-white/20' : 'bg-panel-2']">{{ t.count }}</span>
            </button>
        </div>

        <!-- Search -->
        <div class="mt-4">
            <div class="relative">
                <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-faint" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M21 21l-4-4"/></svg>
                <input v-model="q" @input="onSearch" type="search" placeholder="Search by ID, pair, or strategy…"
                    class="h-11 w-full rounded-xl border border-line bg-panel pl-9 pr-3 text-sm text-ink" />
            </div>
        </div>

        <!-- Filters -->
        <div class="mt-3 flex flex-wrap items-end gap-3">
            <label class="flex flex-col gap-1">
                <span class="text-[11px] font-semibold uppercase tracking-wide text-faint">Status</span>
                <select v-model="fStatus" @change="applyFilters" class="h-10 rounded-xl border border-line bg-panel px-2.5 text-sm text-ink">
                    <option value="all">All</option>
                    <option v-for="s in (options.statuses || [])" :key="s" :value="s" class="capitalize">{{ s }}</option>
                </select>
            </label>
            <label class="flex flex-col gap-1">
                <span class="text-[11px] font-semibold uppercase tracking-wide text-faint">Side</span>
                <select v-model="fSide" @change="applyFilters" class="h-10 rounded-xl border border-line bg-panel px-2.5 text-sm text-ink">
                    <option value="all">All</option>
                    <option value="buy">Buy</option>
                    <option value="sell">Sell</option>
                </select>
            </label>
            <label class="flex flex-col gap-1">
                <span class="text-[11px] font-semibold uppercase tracking-wide text-faint">Venue</span>
                <select v-model="fBroker" @change="applyFilters" class="h-10 rounded-xl border border-line bg-panel px-2.5 text-sm text-ink capitalize">
                    <option value="all">All</option>
                    <option v-for="b in (options.brokers || [])" :key="b" :value="b">{{ b }}</option>
                </select>
            </label>
            <label class="flex flex-col gap-1">
                <span class="text-[11px] font-semibold uppercase tracking-wide text-faint">Market</span>
                <select v-model="fSymbol" @change="applyFilters" class="h-10 rounded-xl border border-line bg-panel px-2.5 text-sm text-ink">
                    <option value="all">All</option>
                    <option v-for="s in (options.symbols || [])" :key="s" :value="s">{{ s }}</option>
                </select>
            </label>
            <label class="flex flex-col gap-1">
                <span class="text-[11px] font-semibold uppercase tracking-wide text-faint">From</span>
                <input v-model="fFrom" @change="applyFilters" type="date" class="h-10 rounded-xl border border-line bg-panel px-2.5 text-sm text-ink" />
            </label>
            <label class="flex flex-col gap-1">
                <span class="text-[11px] font-semibold uppercase tracking-wide text-faint">To</span>
                <input v-model="fTo" @change="applyFilters" type="date" class="h-10 rounded-xl border border-line bg-panel px-2.5 text-sm text-ink" />
            </label>
            <button v-if="activeFilters" @click="resetFilters"
                class="h-10 rounded-xl border border-line px-3 text-sm font-semibold text-muted hover:text-ink">
                ✕ Clear ({{ activeFilters }})
            </button>
        </div>

        <!-- Table -->
        <div class="card mt-4 overflow-hidden">
            <div v-if="list.length" class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="border-b border-line text-left text-[11px] uppercase tracking-wide text-faint">
                        <tr>
                            <th v-for="c in COLUMNS" :key="c.label" :class="['py-3', c.cls]">
                                <Tooltip :text="c.help">
                                    <span class="inline-flex items-center gap-1 cursor-help">
                                        {{ c.label }}
                                        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="opacity-50"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
                                    </span>
                                </Tooltip>
                            </th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-line">
                        <tr v-for="(l, i) in list" :key="i" class="hover:bg-panel-2/60">
                            <td class="px-4 py-3 text-xs text-muted tnum">{{ when(l.time) }}</td>
                            <td><span class="rounded-md bg-panel-2 px-2 py-0.5 font-mono text-[11px] text-muted">#{{ l.log_id }}</span></td>
                            <td class="font-medium text-ink">{{ l.bot }}</td>
                            <td><span :class="['rounded-md px-2 py-0.5 text-[11px] font-semibold capitalize', typePill(l.kind)]">{{ l.kind }}</span></td>
                            <td class="font-medium text-ink">{{ l.pair }}</td>
                            <td><span :class="l.side === 'BUY' ? 'text-bull' : 'text-bear'" class="text-xs font-semibold">{{ l.side }}</span></td>
                            <td class="text-right tnum text-muted">{{ num(l.qty) }}</td>
                            <td class="text-right tnum text-muted">{{ usd(l.price) }}</td>
                            <td class="text-right tnum font-semibold text-ink">{{ usd(l.qty * l.price) }}</td>
                            <td><span :class="['rounded-full px-2 py-0.5 text-[11px] font-semibold', statusPill(l.status)[1]]">{{ statusPill(l.status)[0] }}</span></td>
                            <td class="pr-4 text-right">
                                <button @click="openView(l)" class="rounded-lg p-1.5 text-muted hover:bg-line hover:text-ink" title="View details">
                                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z"/><circle cx="12" cy="12" r="3"/></svg>
                                </button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <EmptyState v-else title="No logs in this view" description="Execution logs appear here as bots trade. The engine writes a row per order." />
        </div>

        <!-- Pagination -->
        <div v-if="(pagination.total ?? 0) > 0" class="mt-4 flex items-center justify-between gap-3 text-sm">
            <p class="text-muted">
                Showing <span class="text-ink tnum">{{ pagination.from ?? 0 }}–{{ pagination.to ?? 0 }}</span>
                of <span class="text-ink tnum">{{ (pagination.total ?? 0).toLocaleString() }}</span>
            </p>
            <div class="flex items-center gap-2">
                <button @click="goPage((pagination.current_page ?? 1) - 1)" :disabled="(pagination.current_page ?? 1) <= 1"
                    class="rounded-xl border border-line px-3 py-1.5 font-semibold text-muted hover:text-ink disabled:opacity-40 disabled:cursor-not-allowed">‹ Prev</button>
                <span class="text-muted tnum">Page {{ pagination.current_page ?? 1 }} / {{ pagination.last_page ?? 1 }}</span>
                <button @click="goPage((pagination.current_page ?? 1) + 1)" :disabled="(pagination.current_page ?? 1) >= (pagination.last_page ?? 1)"
                    class="rounded-xl border border-line px-3 py-1.5 font-semibold text-muted hover:text-ink disabled:opacity-40 disabled:cursor-not-allowed">Next ›</button>
            </div>
        </div>

        <!-- View detail modal -->
        <div v-if="selected" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="selected = null">
            <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
            <div class="relative w-full max-w-lg card p-6">
                <div class="mb-4 flex items-start justify-between gap-3">
                    <div>
                        <h3 class="font-display text-base font-bold text-ink">Execution #{{ selected.log_id }}</h3>
                        <p class="text-xs text-muted">{{ when(selected.time) }}</p>
                    </div>
                    <button @click="selected = null" class="rounded-lg p-1 text-muted hover:bg-panel-2 hover:text-ink">✕</button>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
                    <div><p class="text-xs text-muted">Status</p><span :class="['mt-1 inline-block rounded-full px-2 py-0.5 text-[11px] font-semibold', statusPill(selected.status)[1]]">{{ statusPill(selected.status)[0] }}</span></div>
                    <div><p class="text-xs text-muted">Bot</p><p class="font-medium text-ink">{{ selected.bot }}</p></div>
                    <div><p class="text-xs text-muted">Market</p><p class="font-medium text-ink">{{ selected.pair }}</p></div>
                    <div><p class="text-xs text-muted">Side</p><p class="font-semibold" :class="selected.side === 'BUY' ? 'text-bull' : 'text-bear'">{{ selected.side }}</p></div>
                    <div><p class="text-xs text-muted">Quantity (coins)</p><p class="tnum text-ink">{{ num(selected.qty) }}</p></div>
                    <div><p class="text-xs text-muted">Price (per unit)</p><p class="tnum text-ink">{{ usd(selected.price) }}</p></div>
                    <div><p class="text-xs text-muted">Total value</p><p class="tnum font-semibold text-ink">{{ usd(selected.qty * selected.price) }}</p></div>
                    <div><p class="text-xs text-muted">Order type</p><p class="text-ink capitalize">{{ selected.type || 'market' }}</p></div>
                    <div><p class="text-xs text-muted">Stop price</p><p class="tnum text-ink">{{ selected.stop_price != null ? usd(selected.stop_price) : '—' }}</p></div>
                    <div><p class="text-xs text-muted">Routed to</p><p class="text-ink capitalize">{{ selected.broker || '—' }}</p></div>
                    <div><p class="text-xs text-muted">Broker order ID</p><p class="font-mono text-[11px] text-ink break-all">{{ selected.exchange_oid || '—' }}</p></div>
                    <div class="col-span-2"><p class="text-xs text-muted">Client ID</p><p class="font-mono text-[11px] text-muted break-all">{{ selected.client_id }}</p></div>
                    <div v-if="selected.reason" class="col-span-2"><p class="text-xs text-muted">Reason</p><p class="rounded-lg border border-bear/30 bg-bear/10 p-2 text-xs text-ink">{{ reasonText(selected.reason) }}</p><p class="mt-1 font-mono text-[10px] text-faint">{{ selected.reason }}</p></div>
                </div>
            </div>
        </div>
    </div>
</template>
