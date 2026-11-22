<script setup>
import { useForm, usePage, Link } from '@inertiajs/vue3';
import { ref, computed } from 'vue';
import AuthShell from '../../Components/AuthShell.vue';

defineOptions({ layout: null });
const form = useForm({ email: '', password: '', remember: false });
const flash = computed(() => usePage().props.flash || {});
const show = ref(false);
const submit = () => form.post('/login', { onFinish: () => form.reset('password') });
</script>

<template>
    <AuthShell title="Sign in" quote="Welcome back." quote-g="Your rules kept working.">
        <h1 class="a-h">Sign in</h1>
        <p class="a-sub">Good to see you. Pick up where your bots left off.</p>

        <form class="auth-form" novalidate @submit.prevent="submit">
            <div v-if="flash.error" class="a-alert" role="alert">{{ flash.error }}</div>
            <div v-else-if="flash.success" class="a-alert ok" role="status">{{ flash.success }}</div>

            <label class="fld" :class="{ bad: form.errors.email }">
                <span class="fld-l">Email</span>
                <input v-model="form.email" type="email" autocomplete="email" placeholder="you@example.com" required autofocus />
                <span v-if="form.errors.email" class="fld-err">{{ form.errors.email }}</span>
            </label>

            <label class="fld" :class="{ bad: form.errors.password }">
                <span class="fld-l">Password</span>
                <span class="fld-in">
                    <input v-model="form.password" :type="show ? 'text' : 'password'" class="has-eye" autocomplete="current-password" placeholder="Your password" required />
                    <button type="button" class="eye" :aria-label="show ? 'Hide password' : 'Show password'" @click="show = !show">
                        <svg v-if="!show" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" /><circle cx="12" cy="12" r="3" /></svg>
                        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M3 3l18 18M10.6 5.1A10.4 10.4 0 0 1 12 5c6.5 0 10 7 10 7a17 17 0 0 1-3.2 4.2M6.6 6.6C3.9 8.4 2 12 2 12s3.5 7 10 7c1.8 0 3.4-.5 4.8-1.3" /><path d="M9.9 9.9a3 3 0 0 0 4.2 4.2" /></svg>
                    </button>
                </span>
                <span v-if="form.errors.password" class="fld-err">{{ form.errors.password }}</span>
            </label>

            <label class="chk"><input v-model="form.remember" type="checkbox" /> Keep me signed in</label>

            <button type="submit" class="a-btn" :disabled="form.processing">
                <span v-if="form.processing" class="spin" aria-hidden="true"></span>
                {{ form.processing ? 'Signing in…' : 'Sign in' }}
                <span v-if="!form.processing" aria-hidden="true">→</span>
            </button>
        </form>

        <p class="a-alt">New to TradePilot? <Link href="/register">Create a free account</Link></p>
    </AuthShell>
</template>
