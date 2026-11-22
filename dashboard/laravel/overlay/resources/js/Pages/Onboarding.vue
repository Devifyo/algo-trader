<script setup>
import { computed } from 'vue';
import { router } from '@inertiajs/vue3';
import PageHeader from '../Components/PageHeader.vue';

const props = defineProps({ steps: { type: Array, default: () => [] }, progress: { type: Number, default: 0 }, total: { type: Number, default: 6 } });
const pct = computed(() => Math.round((props.progress / Math.max(1, props.total)) * 100));
const complete = (step) => router.post('/onboarding/complete', { step }, { preserveScroll: true });
</script>

<template>
    <div class="mx-auto max-w-2xl">
        <PageHeader title="Get started"
            description="Six steps from zero to a paper-trading bot. No money is at risk until you explicitly go live." />

        <div class="mb-6">
            <div class="flex items-center justify-between text-xs text-muted"><span>Progress</span><span>{{ progress }} / {{ total }}</span></div>
            <div class="mt-2 h-2 w-full overflow-hidden rounded-full bg-line">
                <div class="h-full rounded-full bg-accent transition-all" :style="{ width: pct + '%' }"></div>
            </div>
        </div>

        <ol class="space-y-3">
            <li v-for="(s, i) in steps" :key="s.key" class="card flex items-start gap-4 p-4">
                <div :class="['flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-semibold',
                    s.completed ? 'bg-bull/15 text-bull' : 'border border-line text-muted']">
                    <span v-if="s.completed">✓</span><span v-else>{{ i + 1 }}</span>
                </div>
                <div class="min-w-0 flex-1">
                    <p class="font-display text-sm font-semibold text-ink">{{ s.title }}</p>
                    <p class="mt-0.5 text-sm text-muted">{{ s.desc }}</p>
                </div>
                <button v-if="!s.completed" @click="complete(s.key)"
                    class="rounded-lg border border-line px-3 py-1.5 text-xs font-semibold text-muted hover:text-ink hover:bg-surface transition">Mark done</button>
                <span v-else class="text-xs text-bull">Done</span>
            </li>
        </ol>
    </div>
</template>
