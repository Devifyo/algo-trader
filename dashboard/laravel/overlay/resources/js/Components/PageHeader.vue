<script setup>
import HelpButton from './HelpButton.vue';
defineProps({
    title: { type: String, required: true },
    description: { type: String, default: '' },
    helpTopic: { type: String, default: '' },
});
</script>

<template>
    <div class="mb-6 flex flex-wrap items-end justify-between gap-3">
        <div>
            <div class="flex items-center gap-2">
                <h1 class="font-display text-xl font-semibold text-ink">{{ title }}</h1>
                <HelpButton v-if="helpTopic" :topic="helpTopic" />
            </div>
            <p v-if="description" class="mt-1 max-w-2xl text-sm text-muted">{{ description }}</p>
        </div>
        <div class="flex items-center gap-2"><slot name="actions" /></div>
    </div>
</template>
