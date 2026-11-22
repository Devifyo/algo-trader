/** @type {import('tailwindcss').Config} */
export default {
    darkMode: 'class',
    content: [
        './resources/views/**/*.blade.php',
        './resources/js/**/*.{vue,js,ts}',
    ],
    theme: {
        extend: {
            fontFamily: {
                // Professional, premium type system.
                display: ['"Plus Jakarta Sans"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
                sans: ['"Plus Jakarta Sans"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
                mono: ['"JetBrains Mono"', 'ui-monospace', 'monospace'],
            },
            colors: {
                ink: 'rgb(var(--ink) / <alpha-value>)',
                surface: 'rgb(var(--surface) / <alpha-value>)',
                panel: 'rgb(var(--panel) / <alpha-value>)',
                'panel-2': 'rgb(var(--panel-2) / <alpha-value>)',
                line: 'rgb(var(--line) / <alpha-value>)',
                'line-strong': 'rgb(var(--line-strong) / <alpha-value>)',
                muted: 'rgb(var(--muted) / <alpha-value>)',
                faint: 'rgb(var(--faint) / <alpha-value>)',
                accent: 'rgb(var(--accent) / <alpha-value>)',
                bull: 'rgb(var(--bull) / <alpha-value>)',
                bear: 'rgb(var(--bear) / <alpha-value>)',
                warn: 'rgb(var(--warn) / <alpha-value>)',
                info: 'rgb(var(--info) / <alpha-value>)',
            },
            borderRadius: { xl2: '1rem' },
            boxShadow: { card: 'var(--card-shadow)' },
        },
    },
    plugins: [],
};
