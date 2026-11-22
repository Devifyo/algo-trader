<script setup>
import { ref } from 'vue';

// Open-source coin logos (spothq/cryptocurrency-icons) via the jsDelivr CDN.
// Falls back to the ticker text in a tinted circle if a logo is missing/offline.
const props = defineProps({
    symbol: { type: String, required: true },
    size: { type: Number, default: 32 },
});
const err = ref(false);
const src = `https://cdn.jsdelivr.net/npm/cryptocurrency-icons@0.18.1/svg/color/${props.symbol.toLowerCase()}.svg`;
</script>

<template>
    <span class="cico" :style="{ width: size + 'px', height: size + 'px' }">
        <img v-if="!err" :src="src" :alt="symbol" :width="size" :height="size" loading="lazy" referrerpolicy="no-referrer" @error="err = true" />
        <span v-else class="cf">{{ symbol.slice(0, 3) }}</span>
    </span>
</template>

<style scoped>
.cico { display: inline-grid; place-items: center; border-radius: 50%; background: rgb(var(--panel-2)); overflow: hidden; flex: none; box-shadow: 0 2px 6px -3px rgba(0,0,0,.5); }
.cico img { width: 100%; height: 100%; object-fit: contain; display: block; }
.cf { font-size: .5rem; font-weight: 800; color: rgb(var(--ink)); letter-spacing: -.02em; }
</style>
