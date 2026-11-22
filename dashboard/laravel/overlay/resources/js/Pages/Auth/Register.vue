<script setup>
import { useForm, usePage, Link } from '@inertiajs/vue3';
import { ref, computed } from 'vue';
import AuthShell from '../../Components/AuthShell.vue';

defineOptions({ layout: null });
const form = useForm({ name: '', org: '', email: '', password: '', password_confirmation: '', risk_profile: 'balanced' });
const flash = computed(() => usePage().props.flash || {});
const show = ref(false);
const submit = () => form.post('/register', { onFinish: () => form.reset('password', 'password_confirmation') });

const profiles = [
    { v: 'conservative', t: 'Conservative', d: 'Small size, low leverage', lvl: 1 },
    { v: 'balanced', t: 'Balanced', d: 'Sensible default', lvl: 2 },
    { v: 'aggressive', t: 'Aggressive', d: 'Bigger size, higher risk', lvl: 3 },
];

// Rough strength hint only — the server enforces the real rule (min 8).
const strength = computed(() => {
    const p = form.password;
    if (!p) return 0;
    let s = p.length >= 8 ? 1 : 0;
    if (p.length >= 12) s++;
    if (/[A-Z]/.test(p) && /[a-z]/.test(p)) s++;
    if (/\d/.test(p) && /[^A-Za-z0-9]/.test(p)) s++;
    return Math.max(1, s);
});
const strengthLabel = computed(() => ['', 'Weak', 'Okay', 'Good', 'Strong'][strength.value]);
const mismatch = computed(() => form.password_confirmation && form.password !== form.password_confirmation);
</script>

<template>
    <AuthShell title="Create your account" quote="Start on paper." quote-g="Go live when you’re sure.">
        <h1 class="a-h">Create your account</h1>
        <p class="a-sub">Free paper mode on live prices. No card, no real money until you decide.</p>

        <form class="auth-form" novalidate @submit.prevent="submit">
            <div v-if="flash.error" class="a-alert" role="alert">{{ flash.error }}</div>

            <div class="row2">
                <label class="fld" :class="{ bad: form.errors.name }">
                    <span class="fld-l">Your name</span>
                    <input v-model="form.name" autocomplete="name" placeholder="Alex Morgan" required autofocus />
                    <span v-if="form.errors.name" class="fld-err">{{ form.errors.name }}</span>
                </label>
                <label class="fld" :class="{ bad: form.errors.org }">
                    <span class="fld-l">Workspace <small>or company</small></span>
                    <input v-model="form.org" autocomplete="organization" placeholder="Personal" required />
                    <span v-if="form.errors.org" class="fld-err">{{ form.errors.org }}</span>
                </label>
            </div>

            <label class="fld" :class="{ bad: form.errors.email }">
                <span class="fld-l">Email</span>
                <input v-model="form.email" type="email" autocomplete="email" placeholder="you@example.com" required />
                <span v-if="form.errors.email" class="fld-err">{{ form.errors.email }}</span>
            </label>

            <label class="fld" :class="{ bad: form.errors.password }">
                <span class="fld-l">Password <small>8+ characters</small></span>
                <span class="fld-in">
                    <input v-model="form.password" :type="show ? 'text' : 'password'" class="has-eye" autocomplete="new-password" placeholder="Create a password" required />
                    <button type="button" class="eye" :aria-label="show ? 'Hide password' : 'Show password'" @click="show = !show">
                        <svg v-if="!show" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" /><circle cx="12" cy="12" r="3" /></svg>
                        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M3 3l18 18M10.6 5.1A10.4 10.4 0 0 1 12 5c6.5 0 10 7 10 7a17 17 0 0 1-3.2 4.2M6.6 6.6C3.9 8.4 2 12 2 12s3.5 7 10 7c1.8 0 3.4-.5 4.8-1.3" /><path d="M9.9 9.9a3 3 0 0 0 4.2 4.2" /></svg>
                    </button>
                </span>
                <span v-if="form.password" class="meter" :data-s="strength">
                    <i v-for="n in 4" :key="n" :class="{ on: n <= strength }"></i>
                    <em>{{ strengthLabel }}</em>
                </span>
                <span v-if="form.errors.password" class="fld-err">{{ form.errors.password }}</span>
            </label>

            <label class="fld" :class="{ bad: mismatch }">
                <span class="fld-l">Confirm password</span>
                <input v-model="form.password_confirmation" :type="show ? 'text' : 'password'" autocomplete="new-password" placeholder="Type it again" required />
                <span v-if="mismatch" class="fld-err">Passwords don’t match yet.</span>
            </label>

            <div class="prof" role="group" aria-label="Risk profile">
                <p class="fld-l">Risk profile <small>change anytime</small></p>
                <div class="prof-grid">
                    <button v-for="p in profiles" :key="p.v" type="button" class="prof-o" :class="{ on: form.risk_profile === p.v }" :aria-pressed="form.risk_profile === p.v" @click="form.risk_profile = p.v">
                        <span class="prof-bars" aria-hidden="true"><i v-for="n in 3" :key="n" :class="{ f: n <= p.lvl }"></i></span>
                        <b>{{ p.t }}</b>
                        <span>{{ p.d }}</span>
                    </button>
                </div>
            </div>

            <button type="submit" class="a-btn" :disabled="form.processing">
                <span v-if="form.processing" class="spin" aria-hidden="true"></span>
                {{ form.processing ? 'Creating your workspace…' : 'Create free account' }}
                <span v-if="!form.processing" aria-hidden="true">→</span>
            </button>

            <p class="a-fine">
                By creating an account you agree to our <Link href="/terms">Terms</Link>, <Link href="/privacy">Privacy Policy</Link>
                and <Link href="/risk-disclosure">Risk Disclosure</Link>. Trading involves substantial risk of loss.
            </p>
        </form>

        <p class="a-alt">Already have an account? <Link href="/login">Sign in</Link></p>
    </AuthShell>
</template>

<style scoped>
.meter { display: flex; align-items: center; gap: 5px; margin-top: 9px; }
.meter i { flex: 1; height: 4px; border-radius: 2px; background: rgba(255,255,255,.1); transition: background .3s; }
.meter[data-s='1'] i.on { background: #f0616d; }
.meter[data-s='2'] i.on { background: #f0b43c; }
.meter[data-s='3'] i.on, .meter[data-s='4'] i.on { background: #1fdc8f; }
.meter em { font-style: normal; font-size: .74rem; color: #8a938f; min-width: 46px; text-align: right; }

.prof { border: 0; padding: 0; margin: 0; }
.prof-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
@media (max-width: 480px) { .prof-grid { grid-template-columns: 1fr; } }
.prof-o { text-align: left; padding: 14px; border-radius: 12px; border: 1px solid rgba(255,255,255,.12); background: #121614; transition: border-color .2s, background .2s, transform .2s; }
.prof-o:hover { border-color: rgba(255,255,255,.25); transform: translateY(-2px); }
.prof-o.on { border-color: #1fdc8f; background: rgba(31,220,143,.08); box-shadow: 0 0 0 3px rgba(31,220,143,.12); }
.prof-o b { display: block; margin-top: 10px; font-weight: 600; font-size: .9rem; color: #eef2f0; }
.prof-o > span:last-child { display: block; margin-top: 2px; font-size: .75rem; color: #8a938f; }
.prof-bars { display: flex; gap: 3px; align-items: flex-end; height: 16px; }
.prof-bars i { width: 5px; border-radius: 2px; background: rgba(255,255,255,.15); }
.prof-bars i:nth-child(1) { height: 6px; } .prof-bars i:nth-child(2) { height: 11px; } .prof-bars i:nth-child(3) { height: 16px; }
.prof-bars i.f { background: #1fdc8f; }
</style>
