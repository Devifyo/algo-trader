<script setup>
import { computed, ref } from 'vue';
import { Link, usePage, router } from '@inertiajs/vue3';
import ThemeToggle from '../Components/ThemeToggle.vue';
import NavIcon from '../Components/NavIcon.vue';
import AlertPopup from '../Components/AlertPopup.vue';
import ConfirmDialog from '../Components/ConfirmDialog.vue';
import { useConfirm } from '../composables/confirm';

const confirm = useConfirm();

const page = usePage();
const nav = computed(() => page.props.nav || []);
const app = computed(() => page.props.app || {});
const auth = computed(() => page.props.auth?.user || null);
const impersonating = computed(() => page.props.auth?.impersonating || false);
const unread = computed(() => page.props.unreadNotifications || 0);
const current = computed(() => page.url.split('?')[0]);
const drawer = ref(false);
const search = ref('');

const isActive = (href) => href === '/' ? current.value === '/' : current.value.startsWith(href);
const initials = (n) => (n || '?').split(/\s+/).slice(0, 2).map((w) => w[0]).join('').toUpperCase();
const logout = () => router.post('/logout');
const stopImpersonating = () => router.post('/impersonate/stop');
const doSearch = () => { if (search.value.trim()) router.get('/help', { q: search.value.trim() }); };
async function toggleKill() {
    const on = !window.__killOn;
    if (on && !(await confirm('Halt all new entries now? This stops the engine from placing any new orders.', { title: 'Halt new entries', confirmText: 'Halt now', tone: 'danger' }))) return;
    router.post('/kill-switch', { on }, { preserveScroll: true });
    window.__killOn = on;
}
</script>

<template>
    <div class="min-h-screen lg:flex">
        <!-- Sidebar (sticky: stays in place while the page content scrolls) -->
        <aside class="hidden lg:flex lg:w-[260px] lg:flex-col lg:border-r lg:border-line lg:bg-panel lg:sticky lg:top-0 lg:h-screen lg:self-start">
            <div class="flex h-16 items-center gap-2.5 px-5">
                <div class="flex h-9 w-9 items-center justify-center rounded-xl grad-accent font-display text-base font-extrabold text-white shadow-sm">α</div>
                <div class="leading-tight">
                    <p class="font-display text-[15px] font-bold tracking-tight text-ink">tradepilot</p>
                    <p class="text-[11px] text-faint">AI Trading Platform</p>
                </div>
            </div>

            <nav class="flex-1 space-y-5 overflow-y-auto px-3 py-3">
                <div v-for="grp in nav" :key="grp.group">
                    <p class="eyebrow px-3 pb-1.5 text-[10px] text-faint">{{ grp.group }}</p>
                    <div class="space-y-0.5">
                        <Link v-for="item in grp.items" :key="item.key" :href="item.href"
                            :class="['flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition',
                                isActive(item.href) ? 'grad-accent text-white shadow-sm' : 'text-muted hover:bg-panel-2 hover:text-ink']">
                            <NavIcon :name="item.icon" />
                            {{ item.label }}
                        </Link>
                    </div>
                </div>
            </nav>

            <!-- Promo / status card -->
            <div class="p-3">
                <div class="rounded-2xl border border-line bg-gradient-to-br from-accent/[0.10] to-panel-2 p-4">
                    <div class="mb-2 flex h-8 w-8 items-center justify-center rounded-lg bg-accent/15 text-accent"><NavIcon name="bots" /></div>
                    <p class="text-sm font-semibold text-ink">{{ app.tradingMode === 'live' ? 'Live trading' : 'Paper mode' }}</p>
                    <p class="mt-0.5 text-xs text-muted">{{ app.tradingMode === 'live' ? 'Real capital at work.' : 'Practicing risk-free with virtual funds.' }}</p>
                    <Link href="/onboarding" class="mt-3 inline-block rounded-lg bg-accent px-3 py-1.5 text-xs font-semibold text-[#06120E]">Setup guide</Link>
                </div>
            </div>
        </aside>

        <div class="flex min-w-0 flex-1 flex-col">
            <!-- Topbar -->
            <header class="sticky top-0 z-20 flex h-16 items-center gap-3 border-b border-line bg-surface/80 px-4 backdrop-blur-md lg:px-6">
                <button class="lg:hidden text-muted" @click="drawer = !drawer" aria-label="Menu">
                    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M3 12h18M3 18h18"/></svg>
                </button>

                <div class="relative hidden sm:block">
                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-faint" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M21 21l-4-4"/></svg>
                    <input v-model="search" @keyup.enter="doSearch" type="search" placeholder="Search help…"
                        class="h-9 w-56 rounded-xl border border-line bg-panel-2 pl-9 pr-3 text-sm text-ink" aria-label="Search" />
                </div>

                <div class="ml-auto flex items-center gap-2">
                    <span class="hidden items-center gap-2 rounded-full border border-line bg-panel-2 px-3 py-1.5 text-xs font-semibold text-ink sm:inline-flex">
                        <span class="pulse h-2 w-2 rounded-full bg-accent"></span>
                        Live · <span class="capitalize text-muted">{{ app.tradingMode }}</span>
                    </span>
                    <button @click="toggleKill"
                        class="rounded-xl border border-bear/40 bg-bear/10 px-3 py-1.5 text-xs font-semibold text-bear hover:bg-bear/20 transition"
                        title="Emergency: halt all new entries">Kill switch</button>
                    <button class="hidden h-9 w-9 items-center justify-center rounded-xl border border-line text-muted hover:bg-panel-2 hover:text-ink sm:inline-flex" title="Quick actions" aria-label="Quick actions">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2 3 14h7l-1 8 10-12h-7z"/></svg>
                    </button>
                    <Link href="/notifications" class="relative inline-flex h-9 w-9 items-center justify-center rounded-xl border border-line text-muted hover:bg-panel-2 hover:text-ink" title="Alerts" aria-label="Alerts">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8a6 6 0 1 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.7 21a2 2 0 0 1-3.4 0"/></svg>
                        <span v-if="unread > 0" class="absolute -right-1 -top-1 inline-flex h-4 min-w-[16px] items-center justify-center rounded-full bg-bear px-1 text-[10px] font-bold text-white ring-2 ring-panel">{{ unread > 9 ? '9+' : unread }}</span>
                    </Link>
                    <ThemeToggle />
                    <div v-if="auth" class="flex items-center gap-2 rounded-xl border border-line bg-panel-2 py-1 pl-1 pr-1.5">
                        <div class="flex h-7 w-7 items-center justify-center rounded-lg grad-pink text-xs font-bold text-white">{{ initials(auth.name) }}</div>
                        <span class="hidden text-sm font-medium text-ink md:inline">{{ auth.name }}</span>
                        <button @click="logout" title="Sign out" aria-label="Sign out"
                            class="ml-0.5 rounded-lg p-1.5 text-muted hover:bg-line hover:text-ink">
                            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="M16 17l5-5-5-5M21 12H9"/></svg>
                        </button>
                    </div>
                </div>
            </header>

            <!-- Impersonation banner -->
            <div v-if="impersonating" class="flex items-center justify-between gap-3 bg-warn/15 px-4 py-2.5 text-sm text-warn lg:px-6">
                <span><strong>Viewing as {{ auth?.email }}</strong> — you're impersonating this tenant.</span>
                <button @click="stopImpersonating" class="rounded-lg border border-warn/40 px-3 py-1 text-xs font-semibold hover:bg-warn/20">Return to admin</button>
            </div>

            <!-- Flash alerts surface as a warning/success popup (AlertPopup). -->
            <AlertPopup />
            <!-- Custom confirmation popup, replacing native window.confirm(). -->
            <ConfirmDialog />

            <main class="flex-1 px-4 py-6 lg:px-8"><slot /></main>
        </div>

        <!-- Mobile drawer -->
        <div v-if="drawer" class="fixed inset-0 z-40 lg:hidden" @click="drawer = false">
            <div class="absolute inset-0 bg-black/60"></div>
            <nav class="absolute left-0 top-0 h-full w-[260px] space-y-5 overflow-y-auto border-r border-line bg-panel p-3" @click.stop>
                <div class="flex h-12 items-center gap-2.5 px-2">
                    <div class="flex h-8 w-8 items-center justify-center rounded-xl grad-accent font-display font-extrabold text-white">α</div>
                    <span class="font-display font-bold text-ink">tradepilot</span>
                </div>
                <div v-for="grp in nav" :key="grp.group">
                    <p class="eyebrow px-3 pb-1.5 text-[10px] text-faint">{{ grp.group }}</p>
                    <Link v-for="item in grp.items" :key="item.key" :href="item.href" @click="drawer = false"
                        :class="['flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium',
                            isActive(item.href) ? 'grad-accent text-white' : 'text-muted hover:bg-panel-2']">
                        <NavIcon :name="item.icon" /> {{ item.label }}
                    </Link>
                </div>
                <button v-if="auth" @click="logout" class="w-full rounded-xl px-3 py-2.5 text-left text-sm text-muted hover:bg-panel-2">Sign out</button>
            </nav>
        </div>
    </div>
</template>
