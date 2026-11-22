<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';

defineProps({
    title: { type: String, required: true },
    updated: { type: String, default: '' },
    summary: { type: String, default: '' },
});

// Table of contents is built from the document's own <h2>s, so the legal
// text stays the single source of truth.
const body = ref(null);
const toc = ref([]);
const active = ref('');
let io;
onMounted(() => {
    const hs = [...body.value.querySelectorAll('h2')];
    toc.value = hs.map((h, i) => {
        h.id = h.id || 'sec-' + (i + 1);
        return { id: h.id, label: h.textContent.replace(/^\d+\.\s*/, ''), n: String(i + 1).padStart(2, '0') };
    });
    io = new IntersectionObserver((es) => {
        es.forEach((e) => { if (e.isIntersecting) active.value = e.target.id; });
    }, { rootMargin: '-20% 0px -70% 0px' });
    hs.forEach((h) => io.observe(h));
});
onBeforeUnmount(() => io && io.disconnect());
function go(id) {
    const el = document.getElementById(id);
    if (el) window.scrollTo({ top: el.getBoundingClientRect().top + window.scrollY - 100, behavior: 'smooth' });
}
</script>

<template>
    <section class="ld-hero">
        <div class="ps-wrap">
            <p class="ld-kicker">Legal</p>
            <h1 class="ld-h">{{ title }}</h1>
            <p v-if="summary" class="ld-sum">{{ summary }}</p>
            <p v-if="updated" class="ld-up"><i></i>Last updated {{ updated }}</p>
        </div>
    </section>

    <div class="ps-wrap ld-grid">
        <aside class="ld-toc" aria-label="On this page">
            <p class="ld-toc-h">On this page</p>
            <button v-for="t in toc" :key="t.id" class="ld-toc-i" :class="{ on: active === t.id }" @click="go(t.id)">
                <span>{{ t.n }}</span>{{ t.label }}
            </button>
        </aside>
        <article ref="body" class="ld-body">
            <slot />
        </article>
    </div>
</template>

<style>
.ld-hero { position: relative; overflow: hidden; padding: 90px 0 60px; border-bottom: 1px solid var(--rule); }
.ld-hero::before { content: ''; position: absolute; right: -10%; top: -40%; width: 700px; height: 500px; background: radial-gradient(closest-side, rgba(31,220,143,.13), transparent); pointer-events: none; }
.ld-kicker { font-family: var(--mono); font-size: .75rem; letter-spacing: .14em; text-transform: uppercase; color: var(--g); }
.ld-h { margin-top: 14px; text-transform: uppercase; font-weight: 700; font-size: clamp(2.3rem, 6vw, 4.6rem); line-height: 1; letter-spacing: -.01em; animation: ld-in .9s cubic-bezier(.16,1,.3,1) both; }
.ld-sum { margin-top: 20px; max-width: 620px; color: var(--tx-2); line-height: 1.7; animation: ld-in .9s cubic-bezier(.16,1,.3,1) both .1s; }
.ld-up { margin-top: 22px; display: inline-flex; align-items: center; gap: 10px; font-family: var(--mono); font-size: .78rem; color: var(--tx-2); border: 1px solid var(--rule-2); border-radius: 999px; padding: 7px 14px; }
.ld-up i { width: 7px; height: 7px; border-radius: 50%; background: var(--g); box-shadow: 0 0 8px var(--g); }
@keyframes ld-in { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: none; } }

.ld-grid { display: grid; grid-template-columns: 260px minmax(0, 1fr); gap: 70px; padding-top: 60px; padding-bottom: 110px; }
@media (max-width: 960px) { .ld-grid { grid-template-columns: 1fr; gap: 30px; } .ld-toc { display: none !important; } }
.ld-toc { position: sticky; top: 110px; align-self: start; display: grid; gap: 2px; max-height: calc(100vh - 140px); overflow-y: auto; padding-right: 6px; }
.ld-toc-h { font-family: var(--mono); font-size: .72rem; letter-spacing: .1em; text-transform: uppercase; color: var(--tx-3); margin-bottom: 10px; }
.ld-toc-i { display: flex; gap: 12px; text-align: left; padding: 8px 12px; border-radius: 8px; border-left: 2px solid transparent; font-size: .86rem; line-height: 1.4; color: var(--tx-3); transition: color .2s, background .2s, border-color .2s; }
.ld-toc-i span { font-family: var(--mono); font-size: .72rem; padding-top: 2px; }
.ld-toc-i:hover { color: var(--tx); background: rgba(255,255,255,.03); }
.ld-toc-i.on { color: #fff; border-left-color: var(--g); background: rgba(31,220,143,.06); }
.ld-toc-i.on span { color: var(--g); }

.ld-body { max-width: 760px; }
.ld-body > .legal > p:first-child { font-size: 1.08rem; line-height: 1.8; color: #d6ddd9; }
.ld-body h2 { scroll-margin-top: 100px; margin-top: 56px; padding-top: 28px; border-top: 1px solid var(--rule); font-size: 1.35rem; font-weight: 600; color: #fff; letter-spacing: -.01em; }
.ld-body h2:first-of-type { margin-top: 44px; }
.ld-body p { margin-top: 14px; font-size: .97rem; line-height: 1.8; color: var(--tx-2); }
.ld-body strong { color: var(--tx); font-weight: 600; }
.ld-body a { color: var(--g); text-decoration: underline; text-underline-offset: 3px; }
.ld-body ul { margin-top: 12px; padding-left: 20px; list-style: disc; color: var(--tx-2); }
.ld-body li { margin-top: 6px; line-height: 1.7; }
.ld-body .disclaimer { margin-top: 44px; border: 1px solid var(--rule-2); background: var(--card); border-radius: 12px; padding: 18px 20px; font-size: .82rem; color: var(--tx-3); }
.ld-body .legal-warn { display: flex; gap: 14px; border: 1px solid rgba(240,180,60,.4); background: rgba(240,180,60,.07); border-radius: 14px; padding: 18px 20px; font-size: .95rem; line-height: 1.7; color: #f4dfb4; margin-bottom: 36px; }
.ld-body .legal-warn::before { content: '!'; flex: none; width: 26px; height: 26px; border-radius: 50%; display: grid; place-items: center; background: var(--amber); color: #1a1205; font-weight: 700; }
.ld-body .legal-warn strong { color: #fff; }
.ld-body .legal-warn p { color: #f4dfb4; }
</style>
