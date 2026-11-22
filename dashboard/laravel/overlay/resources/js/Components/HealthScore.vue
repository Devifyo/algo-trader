<script setup>
import { computed } from 'vue';
const props = defineProps({
    score: { type: Number, default: 0 },
    label: { type: String, default: 'Health' },
    size: { type: Number, default: 56 },
});
const color = computed(() => props.score >= 70 ? 'var(--bull)' : props.score >= 45 ? 'var(--warn)' : 'var(--bear)');
const dash = computed(() => {
    const r = (props.size - 8) / 2;
    const c = 2 * Math.PI * r;
    return { r, c, offset: c * (1 - Math.max(0, Math.min(100, props.score)) / 100) };
});
</script>

<template>
    <div class="inline-flex flex-col items-center gap-1" :title="`${label}: ${Math.round(score)} / 100`">
        <svg :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`" class="-rotate-90">
            <circle :cx="size/2" :cy="size/2" :r="dash.r" fill="none" stroke="rgb(var(--line))" stroke-width="6" />
            <circle :cx="size/2" :cy="size/2" :r="dash.r" fill="none" :stroke="`rgb(${color})`" stroke-width="6"
                stroke-linecap="round" :stroke-dasharray="dash.c" :stroke-dashoffset="dash.offset"
                style="transition: stroke-dashoffset .6s ease" />
        </svg>
        <span class="text-xs text-muted">{{ label }}</span>
    </div>
</template>
