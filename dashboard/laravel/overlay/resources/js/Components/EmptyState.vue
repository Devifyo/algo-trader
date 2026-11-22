<script setup>
defineProps({
    title: { type: String, default: 'Nothing here yet' },
    description: { type: String, default: '' },
});
</script>

<template>
    <div class="flex flex-col items-center justify-center rounded-xl border border-dashed border-line py-10 text-center">
        <div class="mb-3 flex h-11 w-11 items-center justify-center rounded-full border border-line text-muted">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7l9-4 9 4-9 4-9-4zM3 7v10l9 4 9-4V7"/></svg>
        </div>
        <p class="font-display text-sm font-semibold text-ink">{{ title }}</p>
        <p v-if="description" class="mt-1 max-w-sm text-sm text-muted">{{ description }}</p>
        <slot />
    </div>
</template>
