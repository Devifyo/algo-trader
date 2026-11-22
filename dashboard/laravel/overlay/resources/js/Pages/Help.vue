<script setup>
import { ref } from 'vue';
import { router } from '@inertiajs/vue3';
import PageHeader from '../Components/PageHeader.vue';
import Card from '../Components/Card.vue';
import EmptyState from '../Components/EmptyState.vue';

const props = defineProps({ articles: { type: Array, default: () => [] }, q: { type: String, default: '' } });
const query = ref(props.q);
const search = () => router.get('/help', { q: query.value }, { preserveState: true, replace: true });
</script>

<template>
    <div>
        <PageHeader title="Help center"
            description="Plain-language guides for every feature. New to trading? Start with the beginner guides." />

        <div class="mb-6">
            <input v-model="query" @keyup.enter="search" type="search" placeholder="Search help… e.g. 'paper trading'"
                class="w-full rounded-xl border border-line bg-panel px-4 py-3 text-sm text-ink placeholder:text-muted focus:border-accent focus:outline-none"
                aria-label="Search help articles" />
        </div>

        <div v-if="articles.length" class="grid gap-4 sm:grid-cols-2">
            <Card v-for="a in articles" :key="a.slug" :title="a.title" :description="a.category + ' · ' + a.audience">
                <p class="text-sm leading-relaxed text-muted">{{ a.body }}</p>
            </Card>
        </div>
        <EmptyState v-else title="No articles found" :description="`Nothing matched “${q}”. Try a simpler term.`" />
    </div>
</template>
