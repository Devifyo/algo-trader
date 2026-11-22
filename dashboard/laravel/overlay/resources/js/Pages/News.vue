<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import PageHeader from '../Components/PageHeader.vue';
import EmptyState from '../Components/EmptyState.vue';

const props = defineProps({
    latest: { type: Array, default: () => [] },
    sentiment: { type: Array, default: () => [] },
    events: { type: Array, default: () => [] },
    econ: { type: Array, default: () => [] },
    stats: { type: Object, default: () => ({}) },
});

const q = ref('');
const asset = ref('all');
const selected = ref(null);

// Server-driven feed with infinite scroll (newest first; scales to lots of news).
const items = ref([...props.latest]);
const page = ref(2);                       // initial render is page 1
const done = ref(props.latest.length < 20);
const loading = ref(false);
const sentinel = ref(null);
let observer = null, debounce = null;

async function loadFeed(reset = false) {
    if (loading.value) return;
    if (reset) { page.value = 1; items.value = []; done.value = false; }
    if (done.value) return;
    loading.value = true;
    try {
        const params = new URLSearchParams({ page: page.value, asset: asset.value, q: q.value });
        const res = await fetch(`/news/feed?${params}`, { headers: { Accept: 'application/json' }, credentials: 'same-origin' });
        const data = await res.json();
        items.value.push(...(data.articles || []));
        if (data.next) page.value = data.next; else done.value = true;
    } catch (e) { done.value = true; }
    loading.value = false;
}

watch(q, () => { clearTimeout(debounce); debounce = setTimeout(() => loadFeed(true), 350); });
watch(asset, () => loadFeed(true));

onMounted(() => {
    observer = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting) loadFeed();
    }, { rootMargin: '400px' });
    if (sentinel.value) observer.observe(sentinel.value);
});
onBeforeUnmount(() => observer && observer.disconnect());

const assetChips = ['all', 'BTC', 'ETH', 'SOL', 'BNB', 'XRP', 'DOGE', 'AVAX', 'LINK', 'ADA', 'DOT', 'LTC', 'ATOM'];

const ago = (t) => {
    if (!t) return '';
    const s = Math.max(0, (Date.now() - new Date(t).getTime()) / 1000);
    if (s < 60) return 'just now';
    if (s < 3600) return Math.floor(s / 60) + 'm ago';
    if (s < 86400) return Math.floor(s / 3600) + 'h ago';
    return Math.floor(s / 86400) + 'd ago';
};
const biasCls = (b) => ({ bullish: 'text-bull bg-bull/10', bearish: 'text-bear bg-bear/10' }[b] || 'text-muted bg-panel-2');
const biasIcon = (b) => ({ bullish: '▲', bearish: '▼' }[b] || '◆');
const sentCls = (v) => (v > 10 ? 'text-bull' : v < -10 ? 'text-bear' : 'text-muted');

</script>

<template>
    <div>
        <PageHeader title="News Intelligence"
            description="Live market news scraped from multiple sources, AI-scored for sentiment. This informs the bots' confidence and risk — it never triggers a trade by itself."
            help-topic="AI sentiment analysis" />

        <!-- KPIs -->
        <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Articles</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-ink">{{ stats.total ?? 0 }}</p><p class="mt-1 text-xs text-muted">{{ stats.analyzed ?? 0 }} AI-analyzed</p></div>
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Sources</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-ink">{{ stats.sources ?? 0 }}</p><p class="mt-1 text-xs text-muted">scraped continuously</p></div>
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Bullish</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-bull">{{ stats.bullish ?? 0 }}</p><p class="mt-1 text-xs text-muted">positive headlines</p></div>
            <div class="card p-5"><p class="eyebrow text-[11px] text-muted">Bearish</p><p class="mt-2 font-display text-3xl font-extrabold tnum text-bear">{{ stats.bearish ?? 0 }}</p><p class="mt-1 text-xs text-muted">negative headlines</p></div>
        </div>

        <div class="mt-6 grid gap-6 lg:grid-cols-3">
            <!-- Feed -->
            <div class="lg:col-span-2">
                <div class="mb-4 flex flex-wrap items-center gap-3">
                    <div class="relative min-w-[200px] flex-1">
                        <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-faint" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M21 21l-4-4"/></svg>
                        <input v-model="q" type="search" placeholder="Search headlines…" class="h-10 w-full rounded-xl border border-line bg-panel pl-9 pr-3 text-sm text-ink" />
                    </div>
                    <div class="flex items-center gap-1.5 overflow-x-auto">
                        <button v-for="a in assetChips.slice(0, 10)" :key="a" @click="asset = a"
                            :class="['rounded-full px-3 py-1.5 text-xs font-semibold transition', asset === a ? 'grad-accent text-white' : 'border border-line text-muted hover:text-ink']">{{ a === 'all' ? 'All' : a }}</button>
                    </div>
                </div>

                <div v-if="items.length" class="space-y-3">
                    <button v-for="a in items" :key="a.id" @click="selected = a"
                        class="card w-full p-4 text-left transition hover:border-accent/40">
                        <div class="flex items-start justify-between gap-3">
                            <div class="min-w-0 flex-1">
                                <div class="mb-1.5 flex flex-wrap items-center gap-2 text-[11px] text-muted">
                                    <span class="rounded bg-panel-2 px-1.5 py-0.5 font-semibold text-ink">{{ a.source }}</span>
                                    <span>{{ ago(a.published_at) }}</span>
                                    <span v-for="x in (a.assets || []).slice(0, 3)" :key="x" class="rounded-full bg-accent/10 px-1.5 py-0.5 font-semibold text-accent">{{ x }}</span>
                                </div>
                                <p class="font-display text-sm font-bold leading-snug text-ink">{{ a.title }}</p>
                                <p v-if="a.summary" class="mt-1 line-clamp-2 text-xs text-muted">{{ a.summary }}</p>
                            </div>
                            <span v-if="a.bias" :class="['shrink-0 rounded-full px-2 py-0.5 text-[11px] font-semibold', biasCls(a.bias)]">{{ biasIcon(a.bias) }} {{ a.bias }}</span>
                            <span v-else class="shrink-0 text-[11px] text-faint">analyzing…</span>
                        </div>
                    </button>
                </div>
                <EmptyState v-else-if="!loading" class="mt-2" title="No matching news" description="The collector scrapes new articles continuously — try a different filter." />

                <!-- infinite scroll -->
                <div ref="sentinel" class="h-6"></div>
                <p v-if="loading" class="py-3 text-center text-xs text-muted">Loading more…</p>
                <p v-else-if="done && items.length" class="py-3 text-center text-[11px] text-faint">— You're all caught up —</p>
            </div>

            <!-- Side rail -->
            <div class="space-y-6">
                <div class="card p-5">
                    <h3 class="font-display text-sm font-bold text-ink">Market mood</h3>
                    <p class="text-xs text-muted">Rolling sentiment by asset (24h)</p>
                    <ul v-if="sentiment.length" class="mt-3 space-y-2.5">
                        <li v-for="s in sentiment" :key="s.asset" class="flex items-center justify-between">
                            <div><span class="font-semibold text-ink">{{ s.asset }}</span><span class="ml-2 text-[11px] text-muted">{{ s.news_count }} art.</span></div>
                            <span class="tnum text-sm font-bold" :class="sentCls(s.sentiment_score)">{{ s.sentiment_score > 0 ? '+' : '' }}{{ Math.round(s.sentiment_score) }}</span>
                        </li>
                    </ul>
                    <EmptyState v-else title="Warming up" description="Sentiment appears as articles are analyzed." />
                </div>

                <div class="card p-5">
                    <h3 class="font-display text-sm font-bold text-ink">Critical events</h3>
                    <p class="text-xs text-muted">Auto-flagged market movers</p>
                    <ul v-if="events.length" class="mt-3 space-y-2.5">
                        <li v-for="(e, i) in events" :key="i" class="flex items-start gap-2">
                            <span :class="['mt-1 h-2 w-2 shrink-0 rounded-full', e.severity === 'critical' ? 'bg-bear' : 'bg-warn']"></span>
                            <div class="min-w-0"><p class="line-clamp-2 text-xs text-ink">{{ e.headline }}</p><p class="text-[11px] text-muted">{{ e.event_type }} · {{ e.asset }} · {{ ago(e.detected_at) }}</p></div>
                        </li>
                    </ul>
                    <EmptyState v-else title="No critical events" />
                </div>

                <div class="card p-5">
                    <h3 class="font-display text-sm font-bold text-ink">Economic calendar</h3>
                    <p class="text-xs text-muted">Risk awareness near major prints</p>
                    <ul v-if="econ.length" class="mt-3 space-y-2.5">
                        <li v-for="(e, i) in econ" :key="i" class="flex items-center justify-between text-sm">
                            <div class="min-w-0"><p class="truncate font-medium text-ink">{{ e.name }}</p><p class="text-[11px] text-muted">{{ e.country }}</p></div>
                            <span class="shrink-0 rounded-full px-2 py-0.5 text-[10px] font-semibold capitalize"
                                :class="e.importance === 'high' ? 'bg-bear/10 text-bear' : e.importance === 'medium' ? 'bg-warn/10 text-warn' : 'bg-panel-2 text-muted'">{{ e.importance }}</span>
                        </li>
                    </ul>
                    <EmptyState v-else title="No upcoming events" />
                </div>
            </div>
        </div>

        <!-- Article detail modal -->
        <div v-if="selected" class="fixed inset-0 z-50 flex items-center justify-center p-4" @click.self="selected = null">
            <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
            <div class="relative w-full max-w-xl card p-6">
                <div class="mb-3 flex items-start justify-between gap-3">
                    <div class="flex flex-wrap items-center gap-2 text-[11px] text-muted">
                        <span class="rounded bg-panel-2 px-2 py-0.5 font-semibold text-ink">{{ selected.source }}</span>
                        <span>{{ ago(selected.published_at) }}</span>
                        <span v-for="x in selected.assets" :key="x" class="rounded-full bg-accent/10 px-1.5 py-0.5 font-semibold text-accent">{{ x }}</span>
                    </div>
                    <button @click="selected = null" class="rounded-lg p-1 text-muted hover:bg-panel-2 hover:text-ink">✕</button>
                </div>
                <h2 class="font-display text-lg font-bold leading-snug text-ink">{{ selected.title }}</h2>
                <p v-if="selected.summary" class="mt-3 text-sm leading-relaxed text-muted">{{ selected.summary }}</p>

                <div v-if="selected.bias" class="mt-4 rounded-xl border border-line bg-panel-2 p-3">
                    <div class="flex items-center justify-between">
                        <span class="text-xs text-muted">AI sentiment</span>
                        <span :class="['rounded-full px-2 py-0.5 text-[11px] font-semibold', biasCls(selected.bias)]">{{ biasIcon(selected.bias) }} {{ selected.bias }} · {{ Math.round(selected.score || 0) }}</span>
                    </div>
                    <div class="mt-2"><div class="flex justify-between text-[11px] text-muted"><span>Confidence</span><span>{{ Math.round(selected.confidence || 0) }}%</span></div>
                        <div class="mt-1 h-1.5 w-full rounded-full bg-panel"><div class="h-full rounded-full grad-accent" :style="{ width: (selected.confidence || 0) + '%' }"></div></div></div>
                </div>

                <div class="mt-5 flex items-center justify-between">
                    <span class="text-[11px] text-faint">Sentiment is advisory — it never places a trade.</span>
                    <a v-if="selected.url" :href="selected.url" target="_blank" rel="noopener"
                        class="rounded-lg grad-accent px-3.5 py-2 text-sm font-semibold text-white hover:opacity-90">Read full article ↗</a>
                </div>
            </div>
        </div>
    </div>
</template>
