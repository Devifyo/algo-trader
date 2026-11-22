<script setup>
import { computed } from 'vue';
const props = defineProps({ level: { type: String, default: 'medium' } });
const map = {
    low: ['Low risk', 'text-bull bg-bull/10 border-bull/30'],
    medium: ['Medium risk', 'text-warn bg-warn/10 border-warn/30'],
    high: ['High risk', 'text-bear bg-bear/10 border-bear/30'],
    critical: ['Critical', 'text-bear bg-bear/15 border-bear/40'],
};
const m = computed(() => map[(props.level || 'medium').toLowerCase()] || map.medium);
</script>

<template>
    <span :class="['inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-semibold', m[1]]">
        <span class="h-1.5 w-1.5 rounded-full bg-current"></span>{{ m[0] }}
    </span>
</template>
