<script setup>
// Global confirmation popup — the custom replacement for window.confirm().
// Renders the shared confirmState; mount once (in AppLayout). Driven via
// useConfirm() which returns a Promise<boolean>.
import { computed } from 'vue';
import { confirmState, settleConfirm } from '../composables/confirm';

const s = confirmState;
const danger = computed(() => s.tone === 'danger');
const tint = computed(() => (danger.value ? 'bg-bear/10 text-bear' : 'bg-warn/10 text-warn'));
const confirmBtn = computed(() => (danger.value ? 'bg-bear hover:bg-bear/90' : 'grad-accent'));
</script>

<template>
    <div v-if="s.open" class="fixed inset-0 z-[110] flex items-center justify-center p-4">
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="settleConfirm(false)"></div>
        <div class="relative w-full max-w-sm card p-6 text-center" role="alertdialog" aria-modal="true">
            <div :class="['mx-auto flex h-12 w-12 items-center justify-center rounded-full', tint]">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
            </div>
            <h3 class="mt-4 font-display text-lg font-bold text-ink">{{ s.title }}</h3>
            <p class="mt-1.5 text-sm text-muted">{{ s.message }}</p>
            <div class="mt-5 flex gap-2">
                <button @click="settleConfirm(false)" class="flex-1 rounded-xl border border-line px-4 py-2.5 text-sm font-semibold text-muted hover:text-ink">
                    {{ s.cancelText }}
                </button>
                <button @click="settleConfirm(true)" :class="['flex-1 rounded-xl px-4 py-2.5 text-sm font-semibold text-white', confirmBtn]">
                    {{ s.confirmText }}
                </button>
            </div>
        </div>
    </div>
</template>
