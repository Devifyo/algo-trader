<script setup>
import { computed } from 'vue';
const props = defineProps({ bias: { type: String, default: 'neutral' }, score: { type: Number, default: null } });
const m = computed(() => {
    const b = (props.bias || 'neutral').toLowerCase();
    if (b === 'bullish') return ['Bullish', 'text-bull bg-bull/10 border-bull/30', '▲'];
    if (b === 'bearish') return ['Bearish', 'text-bear bg-bear/10 border-bear/30', '▼'];
    return ['Neutral', 'text-muted bg-surface border-line', '—'];
});
</script>

<template>
    <span :class="['inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-semibold tnum', m[1]]">
        <span>{{ m[2] }}</span>{{ m[0] }}<span v-if="score !== null" class="opacity-70">{{ Math.round(score) }}</span>
    </span>
</template>
