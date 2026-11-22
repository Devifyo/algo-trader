<script setup>
import { computed, ref } from 'vue';
import { router } from '@inertiajs/vue3';
import PageHeader from '../../Components/PageHeader.vue';
import Card from '../../Components/Card.vue';
import EmptyState from '../../Components/EmptyState.vue';
import Tooltip from '../../Components/Tooltip.vue';

const props = defineProps({
    strategies: { type: Array, default: () => [] },
    recommended: { type: String, default: null },
    defaultStrategy: { type: String, default: null },
    totalClosed: { type: Number, default: 0 },
    since: { type: String, default: null },
    researchRuns: { type: Array, default: () => [] },
    backtests: { type: Array, default: () => [] },
    forwardTest: { type: Object, default: () => ({}) },
});
const sinceLabel = computed(() => props.since ? new Date(props.since).toLocaleString() : null);
const when = (t) => t ? new Date(t).toLocaleString() : '—';
const runPill = (s) => ({
    running: ['Running', 'text-warn bg-warn/10'],
    completed: ['Completed', 'text-bull bg-bull/10'],
    failed: ['Failed', 'text-bear bg-bear/10'],
}[s] || [s, 'text-muted bg-panel-2']);

const researching = ref(false);
function rerunResearch() {
    researching.value = true;
    router.post('/admin/strategies/research', {}, {
        preserveScroll: true,
        onFinish: () => { researching.value = false; },
    });
}

const usd = (n) => (n < 0 ? '-$' : '$') + Math.abs(Number(n || 0)).toLocaleString(undefined, { maximumFractionDigits: 2 });
const verdictPill = (v) => ({
    winning: ['Winning', 'text-bull bg-bull/10'],
    losing: ['Losing', 'text-bear bg-bear/10'],
    breakeven: ['Break-even', 'text-warn bg-warn/10'],
    insufficient: ['Not enough data', 'text-muted bg-panel-2'],
}[v] || [v, 'text-muted bg-panel-2']);
const best = computed(() => props.strategies.find((s) => s.strategy === props.recommended));
const num = (n) => (n === null || n === undefined ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));

function setDefault(strategy) {
    router.post('/admin/strategies/default', { strategy }, { preserveScroll: true });
}

const bt = ref({ strategy: 'trend_following', symbol: 'BTCUSDT', timeframe: '1h' });
const backtesting = ref(false);
function runBacktest() {
    backtesting.value = true;
    router.post('/admin/strategies/backtest', bt.value, {
        preserveScroll: true,
        onFinish: () => { backtesting.value = false; },
    });
}

// NautilusTrader — institutional engine, backtest-only validation service.
const nbt = ref({ symbol: 'BTCUSDT', interval: '1d', fast: 10, slow: 20 });
const nbtRunning = ref(false);
function nautBacktest() {
    nbtRunning.value = true;
    router.post('/admin/strategies/nautilus-backtest', nbt.value, {
        preserveScroll: true,
        onFinish: () => { nbtRunning.value = false; },
    });
}
</script>

<template>
    <div>
        <PageHeader title="Strategy Performance"
            description="Real, cross-tenant results per strategy — which are winning and which are losing — so you can decide what to trade." />

        <div v-if="sinceLabel" class="mb-4 rounded-lg border border-line bg-panel px-3 py-2 text-xs text-muted">
            Showing real-market results since <strong class="text-ink">{{ sinceLabel }}</strong> (the switch to live Binance data). Earlier synthetic-test history is excluded so verdicts reflect real performance.
        </div>

        <!-- Live status: forward-test + research run -->
        <div class="mb-6 grid gap-4 sm:grid-cols-2">
            <div class="rounded-xl border p-4" :class="forwardTest.active ? 'border-warn/30 bg-warn/10' : 'border-line bg-panel'">
                <div class="flex items-center gap-2">
                    <span v-if="forwardTest.active" class="inline-block h-2 w-2 animate-pulse rounded-full bg-warn"></span>
                    <p class="text-xs font-semibold uppercase tracking-wide" :class="forwardTest.active ? 'text-warn' : 'text-muted'">Forward-test</p>
                </div>
                <p class="mt-1 text-sm font-semibold text-ink">{{ forwardTest.active ? 'Running on live data' : 'Not started' }}</p>
                <p class="text-xs text-muted">{{ forwardTest.closed_trades || 0 }} real trades closed since {{ sinceLabel || '—' }}. Verdicts below update as trades close.</p>
            </div>
            <div class="rounded-xl border border-line bg-panel p-4">
                <p class="text-xs font-semibold uppercase tracking-wide text-muted">Latest strategy research</p>
                <template v-if="researchRuns.length">
                    <div class="mt-1 flex items-center gap-2">
                        <span :class="['rounded-full px-2 py-0.5 text-[11px] font-semibold', runPill(researchRuns[0].status)[1]]">{{ runPill(researchRuns[0].status)[0] }}</span>
                        <span class="text-xs text-muted">started {{ when(researchRuns[0].started_at) }}</span>
                    </div>
                    <p class="mt-1 text-xs text-muted">{{ researchRuns[0].recommended ? ('Recommends: ' + researchRuns[0].recommended) : 'Researching latest strategies…' }}</p>
                </template>
                <p v-else class="mt-1 text-sm text-muted">No research runs yet.</p>
            </div>
        </div>

        <!-- Data-driven recommendation -->
        <div v-if="best" class="mb-6 rounded-xl border border-bull/30 bg-bull/10 p-4">
            <p class="text-xs font-semibold uppercase tracking-wide text-bull">Best performer (data-driven)</p>
            <p class="mt-1 text-lg font-bold text-ink">{{ best.label }}</p>
            <p class="text-sm text-muted">Profit factor {{ best.profit_factor }} · win rate {{ best.win_rate }}% · net {{ usd(best.net_pnl) }} over {{ best.trades }} closed trades.</p>
        </div>
        <div v-else class="mb-6 rounded-xl border border-line bg-panel p-4 text-sm text-muted">
            No strategy has enough closed trades yet to crown a winner. Recommendations appear once a strategy clears ~10 closed trades with a profit factor ≥ 1.1.
        </div>

        <div v-if="defaultStrategy" class="mb-6 flex items-center gap-2 rounded-xl border border-accent/30 bg-accent/10 px-4 py-3 text-sm">
            <span class="text-accent">●</span>
            <span class="text-ink">Platform default strategy: <strong>{{ defaultStrategy.replace('_', ' ') }}</strong></span>
            <span class="text-muted">— set manually below; new bots default to this.</span>
        </div>

        <Card title="Per-strategy report" :description="`${totalClosed} closed trades analyzed across all tenants`">
            <div v-if="strategies.length" class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="border-b border-line text-left text-[11px] uppercase tracking-wide text-faint">
                        <tr>
                            <th class="px-3 py-2">Strategy</th>
                            <th>Verdict</th>
                            <th class="text-right">Trades</th>
                            <th class="text-right">Win rate</th>
                            <th class="text-right">Net P&L</th>
                            <th class="text-right"><Tooltip text="Gross profit ÷ gross loss. >1 = profitable; <1 = losing."><span class="cursor-help underline decoration-dotted">Profit factor</span></Tooltip></th>
                            <th class="text-right"><Tooltip text="Average profit per trade (net P&L ÷ trades)."><span class="cursor-help underline decoration-dotted">Expectancy</span></Tooltip></th>
                            <th class="text-right"><Tooltip text="Per-trade Sharpe — mean P&L ÷ its standard deviation. Higher = steadier returns."><span class="cursor-help underline decoration-dotted">Sharpe</span></Tooltip></th>
                            <th class="text-right"><Tooltip text="Like Sharpe but penalizes only losing trades (downside deviation)."><span class="cursor-help underline decoration-dotted">Sortino</span></Tooltip></th>
                            <th class="text-right"><Tooltip text="Max drawdown — largest peak-to-trough drop of cumulative realized P&L ($)."><span class="cursor-help underline decoration-dotted">Max DD</span></Tooltip></th>
                            <th class="text-right">Open</th>
                            <th class="text-right">Fills 24h</th>
                            <th class="text-right pr-3">Default</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-line">
                        <tr v-for="s in strategies" :key="s.strategy" class="hover:bg-panel-2/50"
                            :class="s.strategy === recommended ? 'bg-bull/5' : ''">
                            <td class="px-3 py-2.5 font-medium text-ink">
                                {{ s.label }}
                                <span v-if="s.strategy === recommended" class="ml-1 rounded bg-bull/15 px-1.5 text-[10px] font-semibold text-bull">★ best</span>
                            </td>
                            <td><span :class="['rounded-full px-2 py-0.5 text-[11px] font-semibold', verdictPill(s.verdict)[1]]">{{ verdictPill(s.verdict)[0] }}</span></td>
                            <td class="text-right tnum text-muted">{{ s.trades }} <span class="text-faint">({{ s.wins }}W/{{ s.losses }}L)</span></td>
                            <td class="text-right tnum text-muted">{{ s.win_rate }}%</td>
                            <td class="text-right tnum font-semibold" :class="s.net_pnl >= 0 ? 'text-bull' : 'text-bear'">{{ usd(s.net_pnl) }}</td>
                            <td class="text-right tnum" :class="s.profit_factor >= 1 ? 'text-bull' : 'text-bear'">{{ s.profit_factor }}</td>
                            <td class="text-right tnum" :class="s.expectancy >= 0 ? 'text-ink' : 'text-bear'">{{ usd(s.expectancy) }}</td>
                            <td class="text-right tnum" :class="s.sharpe >= 0 ? 'text-ink' : 'text-bear'">{{ num(s.sharpe) }}</td>
                            <td class="text-right tnum" :class="s.sortino >= 0 ? 'text-ink' : 'text-bear'">{{ num(s.sortino) }}</td>
                            <td class="text-right tnum text-bear">{{ s.max_dd ? usd(s.max_dd) : '—' }}</td>
                            <td class="text-right tnum text-muted">{{ s.open_positions }}</td>
                            <td class="text-right tnum text-muted">{{ s.fills_24h }}</td>
                            <td class="text-right pr-3">
                                <span v-if="s.strategy === defaultStrategy" class="rounded bg-accent/15 px-2 py-0.5 text-[11px] font-semibold text-accent">✓ default</span>
                                <button v-else @click="setDefault(s.strategy)" class="rounded border border-line px-2 py-0.5 text-[11px] font-medium text-muted hover:text-ink hover:border-accent">Set default</button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <EmptyState v-else title="No strategy data yet" description="Metrics appear here as bots open and close positions." />
        </Card>

        <!-- Quick backtest (indicative) -->
        <Card title="Quick backtest" description="Single-pass, long-only test on real Binance history with fees — an indicative sanity check, NOT walk-forward/CPCV validation." class="mt-6">
            <div class="flex flex-wrap items-end gap-3">
                <label class="text-xs text-muted">Strategy
                    <select v-model="bt.strategy" class="mt-1 block rounded-lg border border-line bg-panel px-2 py-1.5 text-sm text-ink">
                        <option v-for="s in strategies" :key="s.strategy" :value="s.strategy">{{ s.label }}</option>
                    </select>
                </label>
                <label class="text-xs text-muted">Symbol
                    <select v-model="bt.symbol" class="mt-1 block rounded-lg border border-line bg-panel px-2 py-1.5 text-sm text-ink">
                        <option>BTCUSDT</option><option>ETHUSDT</option><option>SOLUSDT</option><option>BNBUSDT</option>
                    </select>
                </label>
                <label class="text-xs text-muted">Timeframe
                    <select v-model="bt.timeframe" class="mt-1 block rounded-lg border border-line bg-panel px-2 py-1.5 text-sm text-ink">
                        <option>5m</option><option>15m</option><option>1h</option><option>4h</option><option>1d</option>
                    </select>
                </label>
                <button @click="runBacktest" :disabled="backtesting"
                    class="rounded-lg bg-accent px-3 py-1.5 text-sm font-semibold text-white disabled:opacity-60">
                    {{ backtesting ? 'Running…' : 'Run backtest' }}
                </button>
            </div>
            <div v-if="backtests.length" class="mt-4 overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="border-b border-line text-left text-[11px] uppercase tracking-wide text-faint">
                        <tr>
                            <th class="px-3 py-2">When</th><th>Strategy</th><th>Symbol</th><th>TF</th>
                            <th class="text-right">Trades</th><th class="text-right">Win%</th>
                            <th class="text-right">Net %</th><th class="text-right">PF</th>
                            <th class="text-right">Sharpe</th><th class="text-right pr-3">Max DD%</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-line">
                        <tr v-for="b in backtests" :key="b.id" class="hover:bg-panel-2/50">
                            <td class="px-3 py-2 text-muted whitespace-nowrap">{{ when(b.created_at) }}</td>
                            <td class="text-ink">{{ b.strategy }}</td>
                            <td class="text-muted">{{ b.symbol }}</td>
                            <td class="text-muted">{{ b.timeframe }}</td>
                            <td class="text-right tnum text-muted">{{ b.metrics.trades ?? '—' }}</td>
                            <td class="text-right tnum text-muted">{{ b.metrics.win_rate ?? '—' }}</td>
                            <td class="text-right tnum font-semibold" :class="(b.metrics.net_return_pct ?? 0) >= 0 ? 'text-bull' : 'text-bear'">{{ num(b.metrics.net_return_pct) }}</td>
                            <td class="text-right tnum" :class="(b.metrics.profit_factor ?? 0) >= 1 ? 'text-bull' : 'text-bear'">{{ num(b.metrics.profit_factor) }}</td>
                            <td class="text-right tnum text-muted">{{ num(b.metrics.sharpe) }}</td>
                            <td class="text-right tnum text-bear pr-3">{{ num(b.metrics.max_dd_pct) }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <p v-else class="mt-3 text-sm text-muted">No backtests yet — run one above.</p>
        </Card>

        <!-- NautilusTrader — institutional engine -->
        <Card title="NautilusTrader backtest (institutional engine)"
            description="Validate on the professional NautilusTrader engine — same code path as live, realistic maker/taker fees, long-only (spot reality). Backtest ONLY: this engine is not wired to live trading or customer funds."
            class="mt-6">
            <div class="mb-4 rounded-xl border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-muted">
                <strong class="text-ink">Why this exists.</strong> A great engine is not an edge — it just makes the
                test trustworthy. Use this to validate a strategy <em>before</em> any customer money touches it.
                Results land in the table above (engine <code class="text-ink">nautilus_long_ema</code>).
            </div>
            <div class="flex flex-wrap items-end gap-3">
                <label class="text-xs text-muted">Symbol
                    <select v-model="nbt.symbol" class="mt-1 block rounded-lg border border-line bg-panel px-2 py-1.5 text-sm text-ink">
                        <option>BTCUSDT</option><option>ETHUSDT</option>
                    </select>
                </label>
                <label class="text-xs text-muted">Interval
                    <select v-model="nbt.interval" class="mt-1 block rounded-lg border border-line bg-panel px-2 py-1.5 text-sm text-ink">
                        <option>1d</option><option>4h</option><option>1h</option><option>15m</option><option>5m</option>
                    </select>
                </label>
                <label class="text-xs text-muted">Fast EMA
                    <input type="number" min="2" max="200" v-model.number="nbt.fast"
                        class="mt-1 block w-20 rounded-lg border border-line bg-panel px-2 py-1.5 text-sm text-ink" />
                </label>
                <label class="text-xs text-muted">Slow EMA
                    <input type="number" min="3" max="400" v-model.number="nbt.slow"
                        class="mt-1 block w-20 rounded-lg border border-line bg-panel px-2 py-1.5 text-sm text-ink" />
                </label>
                <button @click="nautBacktest" :disabled="nbtRunning"
                    class="rounded-lg bg-accent px-3 py-1.5 text-sm font-semibold text-white disabled:opacity-60">
                    {{ nbtRunning ? 'Running on NautilusTrader…' : 'Run on NautilusTrader' }}
                </button>
            </div>
            <p class="mt-3 text-xs text-faint">Supported symbols: BTCUSDT, ETHUSDT (provider-backed instruments).
                A single backtest is indicative, <strong class="text-muted">not validated</strong> — promotion still
                requires OOS + robustness + multiple-testing.</p>
        </Card>

        <!-- AI strategy research runs -->
        <Card title="AI strategy research" description="Deep research on current market regimes → AI recommendation for the platform-wide strategy (gated behind a sandbox forward-test)." class="mt-6">
            <div v-if="researchRuns.length" class="overflow-x-auto">
                <table class="w-full text-sm">
                    <thead class="border-b border-line text-left text-[11px] uppercase tracking-wide text-faint">
                        <tr><th class="px-3 py-2">Started</th><th>Source</th><th>Status</th><th>Recommends</th><th class="pr-3">Completed</th></tr>
                    </thead>
                    <tbody class="divide-y divide-line">
                        <tr v-for="r in researchRuns" :key="r.id">
                            <td class="px-3 py-2 text-muted whitespace-nowrap">{{ when(r.started_at) }}</td>
                            <td class="text-muted">{{ r.source }}</td>
                            <td><span :class="['rounded-full px-2 py-0.5 text-[11px] font-semibold', runPill(r.status)[1]]">{{ runPill(r.status)[0] }}</span></td>
                            <td class="text-ink">{{ r.recommended || '—' }}</td>
                            <td class="text-muted pr-3 whitespace-nowrap">{{ r.completed_at ? when(r.completed_at) : '—' }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            <p v-else class="text-sm text-muted">No research runs yet.</p>
            <div class="mt-3 flex flex-col gap-1 text-xs text-muted">
                <p>The “best performer” table above comes purely from <strong class="text-ink">real closed-trade results</strong>. The button below runs an <strong class="text-ink">AI analysis</strong> (via your configured providers, with failover) over those live stats and records a recommended default.</p>
                <button @click="rerunResearch" :disabled="researching"
                    class="mt-1 w-fit rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-white disabled:opacity-60">
                    {{ researching ? 'Running AI research…' : '↻ Re-run research now' }}
                </button>
            </div>
        </Card>
    </div>
</template>
