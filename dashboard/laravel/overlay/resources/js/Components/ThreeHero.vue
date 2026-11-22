<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';

const host = ref(null);
let renderer, scene, camera, group, raf, ro, disposed = false;
const mouse = { x: 0, y: 0 };
const reduce = typeof window !== 'undefined' && window.matchMedia
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches : false;

function cssColor(T, varName, fallback) {
    try {
        const v = getComputedStyle(document.documentElement).getPropertyValue(varName).trim();
        const [r, g, b] = v.split(/\s+/).map(Number);
        if ([r, g, b].every((n) => !Number.isNaN(n))) return new T.Color(r / 255, g / 255, b / 255);
    } catch (e) { /* noop */ }
    return new T.Color(fallback);
}

onMounted(async () => {
    let T;
    try { T = await import('three'); } catch (e) { return; } // graceful: CSS aurora stays
    if (disposed || !host.value) return;
    const el = host.value;
    const w = el.clientWidth || 1, h = el.clientHeight || 1;

    scene = new T.Scene();
    camera = new T.PerspectiveCamera(58, w / h, 0.1, 100);
    camera.position.z = 15;
    renderer = new T.WebGLRenderer({ alpha: true, antialias: true, powerPreference: 'high-performance' });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    renderer.setSize(w, h);
    el.appendChild(renderer.domElement);

    group = new T.Group();
    scene.add(group);

    const accent = cssColor(T, '--accent', '#6E56F8');
    const cyan = new T.Color('#22D3EE');

    // ── Globe: points on a fibonacci sphere, colored accent→cyan by latitude ──
    const N = 2800, R = 7;
    const pos = new Float32Array(N * 3), col = new Float32Array(N * 3);
    const golden = Math.PI * (1 + Math.sqrt(5));
    for (let i = 0; i < N; i++) {
        const y = 1 - (i / (N - 1)) * 2;
        const rad = Math.sqrt(1 - y * y);
        const th = golden * i;
        pos[i * 3] = Math.cos(th) * rad * R;
        pos[i * 3 + 1] = y * R;
        pos[i * 3 + 2] = Math.sin(th) * rad * R;
        const c = accent.clone().lerp(cyan, (y + 1) / 2);
        col[i * 3] = c.r; col[i * 3 + 1] = c.g; col[i * 3 + 2] = c.b;
    }
    const geo = new T.BufferGeometry();
    geo.setAttribute('position', new T.BufferAttribute(pos, 3));
    geo.setAttribute('color', new T.BufferAttribute(col, 3));
    const mat = new T.PointsMaterial({ size: 0.12, vertexColors: true, transparent: true, opacity: 0.92, sizeAttenuation: true, blending: T.AdditiveBlending, depthWrite: false });
    const globe = new T.Points(geo, mat);
    group.add(globe);

    // ── Drifting dust field for depth ──
    const M = 700;
    const dp = new Float32Array(M * 3);
    for (let i = 0; i < M; i++) {
        dp[i * 3] = (Math.random() - 0.5) * 34;
        dp[i * 3 + 1] = (Math.random() - 0.5) * 22;
        dp[i * 3 + 2] = (Math.random() - 0.5) * 20 - 4;
    }
    const dgeo = new T.BufferGeometry();
    dgeo.setAttribute('position', new T.BufferAttribute(dp, 3));
    const dmat = new T.PointsMaterial({ size: 0.06, color: accent, transparent: true, opacity: 0.5, blending: T.AdditiveBlending, depthWrite: false });
    const dust = new T.Points(dgeo, dmat);
    scene.add(dust);

    const resize = () => {
        if (!renderer || !host.value) return;
        const nw = host.value.clientWidth || 1, nh = host.value.clientHeight || 1;
        camera.aspect = nw / nh; camera.updateProjectionMatrix(); renderer.setSize(nw, nh);
    };
    ro = new ResizeObserver(resize); ro.observe(el);
    window.addEventListener('mousemove', onMouse, { passive: true });

    if (reduce) { group.rotation.set(0.3, 0.6, 0); renderer.render(scene, camera); return; }

    const loop = () => {
        group.rotation.y += 0.0016;
        group.rotation.x += (mouse.y * 0.35 - group.rotation.x) * 0.05;
        dust.rotation.y -= 0.0006;
        camera.position.x += (mouse.x * 2.4 - camera.position.x) * 0.045;
        camera.position.y += (-mouse.y * 2.0 - camera.position.y) * 0.045;
        camera.lookAt(0, 0, 0);
        renderer.render(scene, camera);
        raf = requestAnimationFrame(loop);
    };
    loop();
});

function onMouse(e) {
    mouse.x = (e.clientX / window.innerWidth) - 0.5;
    mouse.y = (e.clientY / window.innerHeight) - 0.5;
}

onBeforeUnmount(() => {
    disposed = true;
    cancelAnimationFrame(raf);
    window.removeEventListener('mousemove', onMouse);
    ro && ro.disconnect();
    scene && scene.traverse((o) => { if (o.geometry) o.geometry.dispose(); if (o.material) o.material.dispose(); });
    if (renderer) { renderer.dispose(); renderer.domElement && renderer.domElement.remove(); }
});
</script>

<template>
    <div ref="host" class="three-host" aria-hidden="true"></div>
</template>

<style scoped>
.three-host { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; }
.three-host :deep(canvas) { display: block; }
</style>
