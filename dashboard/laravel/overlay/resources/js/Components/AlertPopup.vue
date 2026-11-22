<script setup>
// Global flash-message popup. Renders Laravel flash (`flash.error` / `flash.success`)
// shared via HandleInertiaRequests as a centered, dismissible modal — a warning
// popup for errors, a success popup for confirmations. Re-pops whenever a new
// flash message arrives. Drop <AlertPopup /> once inside a layout.
import { computed, ref, watch } from 'vue';
import { usePage } from '@inertiajs/vue3';

const page = usePage();
const flash = computed(() => page.props.flash || {});

const open = ref(false);
const kind = ref('warning'); // 'warning' (errors/alerts) | 'success'
const message = ref('');

function evaluate() {
    const f = flash.value || {};
    if (f.error) {
        kind.value = 'warning';
        message.value = f.error;
        open.value = true;
    } else if (f.success) {
        kind.value = 'success';
        message.value = f.success;
        open.value = true;
    }
}
// Show on first load and re-show whenever a new flash message comes back.
watch(() => [flash.value.error, flash.value.success], evaluate, { immediate: true });

const close = () => { open.value = false; };
const title = computed(() => (kind.value === 'success' ? 'Success' : 'Warning'));
</script>

<template>
    <div v-if="open && message" class="fixed inset-0 z-[100] flex items-center justify-center p-4">
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="close"></div>
        <div class="relative w-full max-w-sm card p-6 text-center" role="alertdialog" aria-modal="true">
            <div :class="['mx-auto flex h-12 w-12 items-center justify-center rounded-full',
                kind === 'success' ? 'bg-bull/10 text-bull' : 'bg-warn/10 text-warn']">
                <svg v-if="kind === 'warning'" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
            </div>
            <h3 class="mt-4 font-display text-lg font-bold text-ink">{{ title }}</h3>
            <p class="mt-1.5 text-sm text-muted">{{ message }}</p>
            <button @click="close" :class="['mt-5 w-full rounded-xl px-4 py-2.5 text-sm font-semibold text-white',
                kind === 'success' ? 'grad-emerald' : 'grad-accent']">
                OK
            </button>
        </div>
    </div>
</template>
