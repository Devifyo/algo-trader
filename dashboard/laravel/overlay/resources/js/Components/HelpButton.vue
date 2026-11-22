<script setup>
import { Link } from '@inertiajs/vue3';
import Tooltip from './Tooltip.vue';
defineProps({ topic: { type: String, default: '' }, hint: { type: String, default: 'Learn more' } });
</script>

<template>
    <Tooltip :text="hint">
        <Link :href="topic ? `/help?q=${encodeURIComponent(topic)}` : '/help'"
            class="inline-flex h-6 w-6 items-center justify-center rounded-full border border-line text-[11px] font-semibold text-muted hover:text-accent hover:border-accent transition"
            aria-label="Help">?</Link>
    </Tooltip>
</template>
