<script setup>
import HelpButton from './HelpButton.vue';
defineProps({
    title: { type: String, default: '' },
    description: { type: String, default: '' },
    helpTopic: { type: String, default: '' },
});
</script>

<template>
    <section class="card p-5">
        <header v-if="title" class="mb-4 flex items-start justify-between gap-3">
            <div>
                <h3 class="font-display text-sm font-semibold text-ink">{{ title }}</h3>
                <p v-if="description" class="mt-0.5 text-xs text-muted">{{ description }}</p>
            </div>
            <div class="flex items-center gap-2">
                <slot name="actions" />
                <HelpButton v-if="helpTopic" :topic="helpTopic" />
            </div>
        </header>
        <slot />
    </section>
</template>
