<script setup>
import { Head } from '@inertiajs/vue3';

// Per-page SEO: title (gets "· tradepilot" suffix from app.js), meta description,
// canonical URL, and Open Graph / Twitter overrides. Site-wide constants
// (favicon, og:image, JSON-LD, twitter:card) live in app.blade.php.
const props = defineProps({
    title: { type: String, required: true },
    description: { type: String, required: true },
    path: { type: String, default: '/' },
});
const base = 'https://tradepilot.devifyo.cloud';
const url = base + props.path;
</script>

<template>
    <Head :title="title">
        <meta name="description" :content="description" head-key="desc" />
        <meta property="og:title" :content="title + ' · TradePilot'" head-key="ogt" />
        <meta property="og:description" :content="description" head-key="ogd" />
        <meta property="og:url" :content="url" head-key="ogu" />
        <meta name="twitter:title" :content="title + ' · TradePilot'" head-key="twt" />
        <meta name="twitter:description" :content="description" head-key="twd" />
    </Head>
</template>
