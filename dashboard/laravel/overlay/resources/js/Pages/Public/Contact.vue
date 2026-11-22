<script setup>
import { computed } from 'vue';
import { useForm, usePage } from '@inertiajs/vue3';
import PublicShell from '../../Components/PublicShell.vue';
import SeoHead from '../../Components/SeoHead.vue';
defineOptions({ layout: null });

const flash = computed(() => usePage().props.flash || {});
const form = useForm({ name: '', email: '', subject: '', message: '' });
const submit = () => form.post('/contact', { preserveScroll: true, onSuccess: () => form.reset() });

const channels = [
    { ic: 'mail', t: 'General & support', v: 'support@tradepilot.devifyo.cloud' },
    { ic: 'scale', t: 'Legal', v: 'legal@tradepilot.devifyo.cloud' },
    { ic: 'lock', t: 'Privacy & data', v: 'privacy@tradepilot.devifyo.cloud' },
];
const topics = ['Getting started', 'Connecting an exchange', 'Billing', 'Partnership', 'Something else'];
</script>

<template>
    <PublicShell>
        <SeoHead title="Contact — Crypto Trading Bot Support"
            description="Contact TradePilot for support, partnerships or questions about our automated crypto trading bots and platform."
            path="/contact" />

        <section class="ct-hero">
            <div class="ps-wrap">
                <p class="kick">Contact</p>
                <h1 class="ct-h">Get in <span class="g">touch</span></h1>
                <p class="ct-lead">Questions, feedback or partnership ideas — send us a message and we’ll get back to you by email.</p>
            </div>
        </section>

        <section class="ps-wrap ct-grid">
            <div class="ct-card" data-in>
                <div v-if="flash.success" class="ct-ok" role="status">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12.5l4.2 4L19 7" /></svg>
                    {{ flash.success }}
                </div>

                <form class="ct-form" @submit.prevent="submit">
                    <div class="row2">
                        <label class="fld" :class="{ bad: form.errors.name }">
                            <span class="fld-l">Name</span>
                            <input v-model="form.name" type="text" autocomplete="name" placeholder="Your name" />
                            <span v-if="form.errors.name" class="fld-err">{{ form.errors.name }}</span>
                        </label>
                        <label class="fld" :class="{ bad: form.errors.email }">
                            <span class="fld-l">Email</span>
                            <input v-model="form.email" type="email" autocomplete="email" placeholder="you@example.com" />
                            <span v-if="form.errors.email" class="fld-err">{{ form.errors.email }}</span>
                        </label>
                    </div>

                    <div class="fld">
                        <span class="fld-l">What’s it about?</span>
                        <div class="chips">
                            <button v-for="t in topics" :key="t" type="button" class="chip" :class="{ on: form.subject === t }" @click="form.subject = t">{{ t }}</button>
                        </div>
                    </div>

                    <label class="fld" :class="{ bad: form.errors.subject }">
                        <span class="fld-l">Subject</span>
                        <input v-model="form.subject" type="text" placeholder="How can we help?" />
                        <span v-if="form.errors.subject" class="fld-err">{{ form.errors.subject }}</span>
                    </label>

                    <label class="fld" :class="{ bad: form.errors.message }">
                        <span class="fld-l">Message <small>{{ form.message.length }} chars</small></span>
                        <textarea v-model="form.message" rows="6" placeholder="Tell us a bit more…"></textarea>
                        <span v-if="form.errors.message" class="fld-err">{{ form.errors.message }}</span>
                    </label>

                    <button type="submit" class="btn-g ct-send" :disabled="form.processing">
                        <span v-if="form.processing" class="spin" aria-hidden="true"></span>
                        {{ form.processing ? 'Sending…' : 'Send message' }}
                        <span v-if="!form.processing" aria-hidden="true">→</span>
                    </button>
                    <p class="ct-note">By sending, you agree to our <a href="/privacy">Privacy Policy</a>. We’ll only use your details to reply.</p>
                </form>
            </div>

            <aside class="ct-side">
                <a v-for="(c, i) in channels" :key="c.t" :href="`mailto:${c.v}`" class="chan" data-in :style="{ '--d': i * 0.08 + 's' }">
                    <span class="chan-ic" aria-hidden="true">
                        <svg v-if="c.ic === 'mail'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="14" rx="2" /><path d="M3 7l9 6 9-6" /></svg>
                        <svg v-else-if="c.ic === 'scale'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v18M5 21h14M6 7h12M6 7l-3 7a3 3 0 0 0 6 0zM18 7l-3 7a3 3 0 0 0 6 0z" /></svg>
                        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="11" width="16" height="9" rx="2" /><path d="M8 11V8a4 4 0 0 1 8 0v3" /></svg>
                    </span>
                    <span class="chan-tx">
                        <span class="chan-t">{{ c.t }}</span>
                        <span class="chan-v">{{ c.v }}</span>
                    </span>
                    <span class="chan-ar" aria-hidden="true">↗</span>
                </a>

                <div class="loc" data-in style="--d:.24s">
                    <img src="/img/lp/desk.jpg" alt="" loading="lazy" />
                    <div class="loc-tx">
                        <span class="chan-t">Based in</span>
                        <span class="loc-v">India</span>
                        <span class="loc-s">Fully online — no offices to visit.</span>
                    </div>
                </div>
            </aside>
        </section>
    </PublicShell>
</template>

<style scoped>
.kick { font-family: var(--mono); font-size: .75rem; letter-spacing: .14em; text-transform: uppercase; color: var(--g); }
.ct-hero { position: relative; overflow: hidden; padding: 90px 0 60px; border-bottom: 1px solid var(--rule); }
.ct-hero::before { content: ''; position: absolute; right: 0; top: -40%; width: 700px; height: 500px; background: radial-gradient(closest-side, rgba(31,220,143,.13), transparent); pointer-events: none; }
.ct-h { margin-top: 14px; text-transform: uppercase; font-weight: 700; font-size: clamp(2.6rem, 7vw, 5.4rem); line-height: 1; animation: ct-in .9s cubic-bezier(.16,1,.3,1) both; }
.ct-lead { margin-top: 20px; max-width: 520px; color: var(--tx-2); line-height: 1.7; font-size: 1.05rem; animation: ct-in .9s cubic-bezier(.16,1,.3,1) both .1s; }
@keyframes ct-in { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: none; } }

.ct-grid { display: grid; grid-template-columns: 1.35fr .9fr; gap: 40px; padding-top: 60px; padding-bottom: 110px; align-items: start; }
@media (max-width: 960px) { .ct-grid { grid-template-columns: 1fr; } }
.ct-card { background: var(--card); border: 1px solid var(--rule); border-radius: 20px; padding: 34px; }
@media (max-width: 640px) { .ct-card { padding: 22px 16px; } }
.ct-ok { display: flex; align-items: center; gap: 10px; margin-bottom: 22px; padding: 12px 14px; border-radius: 12px; border: 1px solid rgba(31,220,143,.35); background: rgba(31,220,143,.08); color: #9ff0c9; font-size: .9rem; }
.ct-ok svg { width: 18px; height: 18px; flex: none; }
.ct-form { display: grid; gap: 20px; }
.row2 { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
@media (max-width: 520px) { .row2 { grid-template-columns: 1fr; } }
.fld { display: block; }
.fld-l { display: flex; justify-content: space-between; font-size: .82rem; font-weight: 500; color: #cfd6d3; margin-bottom: 7px; }
.fld-l small { font-weight: 400; color: var(--tx-3); font-family: var(--mono); font-size: .72rem; }
.fld input, .fld textarea { width: 100%; padding: 14px 16px; background: #101412; border: 1px solid rgba(255,255,255,.12); border-radius: 12px; color: var(--tx); font: inherit; font-size: .95rem; transition: border-color .2s, box-shadow .2s; resize: vertical; }
.fld input::placeholder, .fld textarea::placeholder { color: #5a625e; }
.fld input:hover, .fld textarea:hover { border-color: rgba(255,255,255,.22); }
.fld input:focus, .fld textarea:focus { outline: none; border-color: var(--g); box-shadow: 0 0 0 4px rgba(31,220,143,.14); }
.fld.bad input, .fld.bad textarea { border-color: var(--red); }
.fld-err { display: block; margin-top: 6px; font-size: .78rem; color: var(--red); }
.chips { display: flex; flex-wrap: wrap; gap: 8px; }
.chip { padding: 8px 14px; border-radius: 999px; border: 1px solid var(--rule-2); font-size: .82rem; color: var(--tx-2); transition: all .2s; }
.chip:hover { color: #fff; border-color: var(--tx-3); }
.chip.on { background: var(--g); border-color: var(--g); color: #04140c; font-weight: 500; }
.ct-send { height: 52px; font-size: 1rem; }
.ct-send:disabled { opacity: .7; }
.spin { width: 18px; height: 18px; border: 2px solid rgba(4,20,12,.3); border-top-color: #04140c; border-radius: 50%; animation: spin .7s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.ct-note { font-size: .78rem; color: var(--tx-3); text-align: center; }
.ct-note a { color: var(--g); text-decoration: underline; text-underline-offset: 2px; }

.ct-side { display: grid; gap: 12px; }
.chan { display: flex; align-items: center; gap: 16px; padding: 18px; border-radius: 16px; border: 1px solid var(--rule); background: var(--card); transition: border-color .25s, transform .25s, background .25s; }
.chan:hover { border-color: rgba(31,220,143,.4); background: var(--card-2); transform: translateX(4px); }
.chan-ic { width: 46px; height: 46px; flex: none; display: grid; place-items: center; border-radius: 12px; background: rgba(31,220,143,.1); color: var(--g); }
.chan-ic svg { width: 21px; height: 21px; }
.chan-tx { display: grid; min-width: 0; }
.chan-t { font-family: var(--mono); font-size: .7rem; letter-spacing: .08em; text-transform: uppercase; color: var(--tx-3); }
.chan-v { margin-top: 3px; font-size: .92rem; font-weight: 500; color: var(--tx); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.chan-ar { margin-left: auto; color: var(--tx-3); transition: color .2s, transform .2s; }
.chan:hover .chan-ar { color: var(--g); transform: translate(2px, -2px); }
.loc { position: relative; overflow: hidden; border-radius: 16px; border: 1px solid var(--rule); min-height: 200px; display: flex; align-items: flex-end; }
.loc img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; filter: grayscale(.7) brightness(.45); }
.loc::after { content: ''; position: absolute; inset: 0; background: linear-gradient(180deg, rgba(31,220,143,.1), rgba(12,14,13,.85)); }
.loc-tx { position: relative; z-index: 1; display: grid; padding: 20px; }
.loc-v { font-size: 1.8rem; font-weight: 700; text-transform: uppercase; }
.loc-s { font-size: .85rem; color: var(--tx-2); }

@media (prefers-reduced-motion: reduce) { .ct-h, .ct-lead { animation: none; } }
</style>
