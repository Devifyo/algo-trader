<script setup>
// Lucide-style line icons, 18px, stroke=currentColor.
defineProps({ name: { type: String, required: true } });
const P = {
    dashboard: '<rect x="3" y="3" width="7" height="9" rx="1.5"/><rect x="14" y="3" width="7" height="5" rx="1.5"/><rect x="14" y="12" width="7" height="9" rx="1.5"/><rect x="3" y="16" width="7" height="5" rx="1.5"/>',
    portfolio: '<path d="M3 3v18h18"/><path d="M7 14l4-4 3 3 5-6"/>',
    brokers: '<rect x="3" y="6" width="18" height="14" rx="2"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
    news: '<path d="M4 5h11v14H6a2 2 0 0 1-2-2z"/><path d="M15 7h3a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2"/><path d="M7 9h5M7 12h5M7 15h3"/>',
    marketplace: '<path d="M3 9l1-5h16l1 5"/><path d="M4 9v10a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1V9"/><path d="M9 13h6"/>',
    copy: '<path d="M16 3a3 3 0 1 0 0 6 3 3 0 0 0 0-6z"/><path d="M8 21v-2a4 4 0 0 1 4-4h2"/><circle cx="7" cy="9" r="3"/>',
    help: '<circle cx="12" cy="12" r="9"/><path d="M9.5 9a2.5 2.5 0 1 1 3.5 2.3c-.8.4-1 .9-1 1.7"/><path d="M12 17h.01"/>',
    admin: '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/>',
    tenants: '<rect x="3" y="7" width="7" height="13" rx="1.5"/><rect x="14" y="3" width="7" height="17" rx="1.5"/><path d="M6 11h1M6 14h1M17 7h1M17 11h1"/>',
    providers: '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2"/>',
    flags: '<path d="M5 21V4"/><path d="M5 4h11l-2 4 2 4H5"/>',
    bots: '<rect x="4" y="8" width="16" height="11" rx="3"/><path d="M12 3v5M9 13h.01M15 13h.01"/>',
    logs: '<path d="M8 6h12M8 12h12M8 18h12"/><path d="M3 6h.01M3 12h.01M3 18h.01"/>',
    settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-2.7.7 1.6 1.6 0 0 0-1 1.5V22a2 2 0 1 1-4 0v-.1a1.6 1.6 0 0 0-2.7-1.2l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1A1.6 1.6 0 0 0 4.6 15a1.6 1.6 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.6 1.6 0 0 0 4.6 9a1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1A1.6 1.6 0 0 0 9 4.6a1.6 1.6 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 2.7 1.2l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8 1.6 1.6 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1z"/>',
};
</script>

<template>
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"
        stroke-linecap="round" stroke-linejoin="round" v-html="P[name] || P.dashboard" />
</template>
