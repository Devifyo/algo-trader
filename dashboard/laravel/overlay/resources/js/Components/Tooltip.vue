<script setup>
defineProps({ text: { type: String, required: true } });
</script>

<template>
    <span class="relative inline-flex items-center group">
        <slot />
        <span role="tooltip"
            class="pointer-events-none absolute left-1/2 bottom-full z-30 mb-2 w-max max-w-xs -translate-x-1/2 rounded-lg border border-line bg-panel px-3 py-1.5 text-xs text-ink opacity-0 shadow-lg transition group-hover:opacity-100 group-focus-within:opacity-100">
            {{ text }}
        </span>
    </span>
</template>
