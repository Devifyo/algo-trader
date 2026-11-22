<script setup>
import { ref, onMounted } from 'vue';
import { Link } from '@inertiajs/vue3';

const KEY = 'tp_cookie_consent';
const show = ref(false);

onMounted(() => {
    try { show.value = !localStorage.getItem(KEY); } catch (e) { show.value = true; }
});
function choose(v) {
    try { localStorage.setItem(KEY, v); } catch (e) { /* noop */ }
    show.value = false;
}
</script>

<template>
    <transition name="cookie">
        <div v-if="show" class="cookie" role="dialog" aria-label="Cookie consent">
            <div class="cookie-inner">
                <p class="cookie-text">
                    We use strictly necessary cookies to keep you signed in, plus optional analytics cookies to improve TradePilot.
                    See our <Link href="/privacy" class="cookie-link">Privacy Policy</Link>.
                </p>
                <div class="cookie-actions">
                    <button class="btn-decline" @click="choose('necessary')">Necessary only</button>
                    <button class="btn-accept" @click="choose('all')">Accept all</button>
                </div>
            </div>
        </div>
    </transition>
</template>

<style scoped>
.cookie { position: fixed; left: 0; right: 0; bottom: 0; z-index: 60; padding: 0 1rem 1rem; }
.cookie-inner { margin: 0 auto; max-width: 64rem; display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; border: 1px solid rgb(var(--line)); background: rgb(var(--panel)/.92); backdrop-filter: blur(12px); border-radius: 16px; padding: 1rem 1.25rem; box-shadow: 0 20px 50px -20px rgba(0,0,0,.5); }
.cookie-text { flex: 1 1 18rem; font-size: .82rem; line-height: 1.5; color: rgb(var(--muted)); }
.cookie-link { color: rgb(var(--accent)); text-decoration: underline; }
.cookie-actions { display: flex; gap: .6rem; margin-left: auto; }
.btn-decline { border-radius: .65rem; border: 1px solid rgb(var(--line)); padding: .5rem .9rem; font-size: .8rem; font-weight: 600; color: rgb(var(--ink)); }
.btn-decline:hover { background: rgb(var(--panel-2)); }
.btn-accept { border-radius: .65rem; padding: .5rem 1.1rem; font-size: .8rem; font-weight: 700; color: #fff; background: linear-gradient(120deg, #6E56F8, #8B5CF6); box-shadow: 0 8px 20px -8px rgba(110,86,248,.7); }
.btn-accept:hover { transform: translateY(-1px); }
.cookie-enter-active, .cookie-leave-active { transition: opacity .35s ease, transform .35s ease; }
.cookie-enter-from, .cookie-leave-to { opacity: 0; transform: translateY(16px); }
</style>
