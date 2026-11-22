<script setup>
import Tooltip from './Tooltip.vue';
import NavIcon from './NavIcon.vue';
defineProps({
    label: { type: String, required: true },
    value: { type: [String, Number], default: '—' },
    sub: { type: String, default: '' },
    tone: { type: String, default: 'neutral' },   // neutral|bull|bear|warn
    hint: { type: String, default: '' },
    icon: { type: String, default: '' },
    grad: { type: String, default: 'grad-accent' }, // gradient class for the icon chip
    pill: { type: String, default: '' },            // small pill under the number
    pillTone: { type: String, default: 'neutral' }, // bull|bear|accent|neutral
    delta: { type: Number, default: null },
});
const toneClass = { neutral: 'text-ink', bull: 'text-bull', bear: 'text-bear', warn: 'text-warn' };
const pillClass = {
    bull: 'text-bull bg-bull/10', bear: 'text-bear bg-bear/10',
    accent: 'text-accent bg-accent/10', neutral: 'text-muted bg-panel-2',
};
</script>

<template>
    <div class="card p-5">
        <div class="flex items-start justify-between gap-3">
            <div class="flex items-center gap-1.5">
                <span class="eyebrow text-[11px] text-muted">{{ label }}</span>
                <Tooltip v-if="hint" :text="hint"><span class="cursor-help text-[11px] text-faint">ⓘ</span></Tooltip>
            </div>
            <div v-if="icon" :class="['flex h-10 w-10 items-center justify-center rounded-xl text-white shadow-sm', grad]">
                <NavIcon :name="icon" />
            </div>
        </div>
        <div :class="['mt-3 font-display text-[30px] font-extrabold leading-none tnum', toneClass[tone]]">{{ value }}</div>
        <div class="mt-2.5 flex items-center gap-2">
            <span v-if="delta !== null"
                :class="['inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-semibold tnum', delta >= 0 ? 'text-bull bg-bull/10' : 'text-bear bg-bear/10']">
                {{ delta >= 0 ? '▲' : '▼' }} {{ Math.abs(delta).toFixed(1) }}%
            </span>
            <span v-if="pill" :class="['inline-flex items-center rounded-full px-2 py-0.5 text-xs font-semibold', pillClass[pillTone]]">{{ pill }}</span>
            <span v-if="sub" class="text-xs text-muted">{{ sub }}</span>
        </div>
    </div>
</template>
