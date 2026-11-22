import './bootstrap';
import '../css/app.css';

import { createApp, h } from 'vue';
import { createInertiaApp } from '@inertiajs/vue3';
import AppLayout from './Layouts/AppLayout.vue';

const pages = import.meta.glob('./Pages/**/*.vue');

createInertiaApp({
    title: (title) => (title ? `${title} · tradepilot` : 'tradepilot'),
    resolve: async (name) => {
        const page = (await pages[`./Pages/${name}.vue`]()).default;
        // Wrap every page in the shared app layout unless it explicitly opts out
        // with `defineOptions({ layout: null })` (e.g. the auth screens).
        if (page.layout === undefined) page.layout = AppLayout;
        return page;
    },
    setup({ el, App, props, plugin }) {
        const app = createApp({ render: () => h(App, props) }).use(plugin);

        // Global scroll-reveal directive — one shared IntersectionObserver, fires once.
        // Usage: v-reveal  or  v-reveal="{ delay: 80 }" (ms) for staggered children.
        const reduce = typeof window !== 'undefined' && window.matchMedia
            && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        const obs = typeof IntersectionObserver !== 'undefined'
            ? new IntersectionObserver((entries, o) => {
                entries.forEach((e) => {
                    if (e.isIntersecting) { e.target.classList.add('reveal-in'); o.unobserve(e.target); }
                });
            }, { threshold: 0.15, rootMargin: '0px 0px -7% 0px' })
            : null;
        app.directive('reveal', {
            mounted(elx, binding) {
                if (reduce || !obs) { elx.classList.add('reveal', 'reveal-in'); return; }
                elx.classList.add('reveal');
                const d = typeof binding.value === 'object' ? binding.value && binding.value.delay : binding.value;
                if (typeof d === 'number') elx.style.transitionDelay = d + 'ms';
                obs.observe(elx);
            },
            unmounted(elx) { obs && obs.unobserve(elx); },
        });

        app.mount(el);
    },
    progress: { color: '#7C94FF' },
});
