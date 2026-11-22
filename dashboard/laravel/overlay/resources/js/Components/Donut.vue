<script setup>
import { computed } from 'vue';
const props = defineProps({
    slices: { type: Array, default: () => [] }, // [{label, value}]
    center: { type: String, default: '' },
    centerSub: { type: String, default: '' },
    size: { type: Number, default: 200 },
});
const COLORS = ['#7C5CFF', '#38BDF8', '#34D399', '#FBBF24', '#FB7185', '#C084FC', '#22D3EE', '#F472B6'];
const total = computed(() => props.slices.reduce((a, s) => a + Number(s.value || 0), 0) || 1);
const withColor = computed(() => props.slices.map((s, i) => ({ ...s, color: COLORS[i % COLORS.length], pct: Number(s.value || 0) / total.value * 100 })));
const gradient = computed(() => {
    if (!props.slices.length) return 'conic-gradient(rgb(var(--line)) 0% 100%)';
    let acc = 0;
    const stops = withColor.value.map((s) => {
        const start = acc; acc += s.pct; return `${s.color} ${start.toFixed(2)}% ${acc.toFixed(2)}%`;
    });
    return `conic-gradient(${stops.join(',')})`;
});
const hole = computed(() => Math.round(props.size * 0.62));
</script>

<template>
    <div class="flex flex-col items-center">
        <div class="relative" :style="{ width: size + 'px', height: size + 'px' }">
            <div class="h-full w-full rounded-full" :style="{ background: gradient }"></div>
            <div class="absolute rounded-full bg-panel"
                :style="{ width: hole + 'px', height: hole + 'px', top: ((size - hole) / 2) + 'px', left: ((size - hole) / 2) + 'px' }">
                <div class="flex h-full w-full flex-col items-center justify-center">
                    <span class="font-display text-2xl font-extrabold tnum text-ink">{{ center }}</span>
                    <span class="text-xs text-muted">{{ centerSub }}</span>
                </div>
            </div>
        </div>
        <ul class="mt-5 w-full space-y-2">
            <li v-for="s in withColor" :key="s.label" class="flex items-center justify-between text-sm">
                <span class="flex items-center gap-2 text-ink">
                    <span class="h-2.5 w-2.5 rounded-full" :style="{ background: s.color }"></span>{{ s.label }}
                </span>
                <span class="tnum font-medium text-muted">{{ s.pct.toFixed(1) }}%</span>
            </li>
        </ul>
    </div>
</template>
