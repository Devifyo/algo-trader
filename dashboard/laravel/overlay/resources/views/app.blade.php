<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="dark light">
    <meta name="theme-color" content="#0A0A0F">
    <title inertia>{{ config('app.name', 'tradepilot') }}</title>

    {{-- Favicons / app icons --}}
    <link rel="icon" type="image/svg+xml" href="/favicon.svg">
    <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png">
    <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
    <link rel="manifest" href="/site.webmanifest">

    {{-- SEO defaults (per-page <SeoHead> overrides title/description/og:title/url) --}}
    <meta name="description" content="TradePilot is automated crypto trading software — run trend-following & momentum bots on your own exchange with real protective stops, AI market context and a free paper mode.">
    <meta name="keywords" content="automated crypto trading, crypto trading bot, algorithmic trading software, crypto trading platform, trend following bot, momentum trading, Bitcoin trading bot, Ethereum trading bot, crypto trading automation, paper trading, AI trading bot, crypto trading strategies">
    <meta name="author" content="TradePilot">
    <meta name="robots" content="index, follow, max-image-preview:large">
    <link rel="canonical" href="{{ config('app.url') }}{{ request()->getPathInfo() }}">

    {{-- Open Graph --}}
    <meta property="og:site_name" content="TradePilot">
    <meta property="og:type" content="website">
    <meta property="og:locale" content="en_US">
    <meta property="og:title" content="TradePilot — Automated Crypto Trading, on Autopilot">
    <meta property="og:description" content="Rule-based crypto trading bots on your own exchange — real protective stops, AI market context, and a free paper mode.">
    <meta property="og:image" content="{{ config('app.url') }}/og.png">
    <meta property="og:image:width" content="1200">
    <meta property="og:image:height" content="630">
    <meta property="og:image:alt" content="TradePilot — Automated Crypto Trading, on Autopilot">

    {{-- Twitter --}}
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="TradePilot — Automated Crypto Trading, on Autopilot">
    <meta name="twitter:description" content="Rule-based crypto trading bots on your own exchange — real protective stops, AI market context, and a free paper mode.">
    <meta name="twitter:image" content="{{ config('app.url') }}/og.png">

    {{-- Structured data (rich results) --}}
    <script type="application/ld+json">
    {!! json_encode([
        '@context' => 'https://schema.org',
        '@graph' => [
            [
                '@type' => 'Organization',
                'name' => 'TradePilot',
                'url' => config('app.url'),
                'logo' => config('app.url') . '/icon-512.png',
                'description' => 'Automated crypto trading software with rule-based bots, real protective stops and AI market context.',
                'foundingLocation' => ['@type' => 'Place', 'address' => ['@type' => 'PostalAddress', 'addressCountry' => 'IN']],
            ],
            [
                '@type' => 'WebSite',
                'name' => 'TradePilot',
                'url' => config('app.url'),
            ],
            [
                '@type' => 'SoftwareApplication',
                'name' => 'TradePilot',
                'applicationCategory' => 'FinanceApplication',
                'operatingSystem' => 'Web',
                'description' => 'Automated crypto trading bots (trend-following, momentum, breakout, mean-reversion) that run on your own exchange with real stops, AI context and a free paper mode.',
                'offers' => ['@type' => 'Offer', 'price' => '0', 'priceCurrency' => 'USD', 'description' => 'Free paper-trading mode'],
            ],
        ],
    ], JSON_UNESCAPED_SLASHES | JSON_PRETTY_PRINT) !!}
    </script>

    {{-- Dark is the default; only switch to light if the user chose it. --}}
    <script>
        (function () {
            try {
                if (localStorage.getItem('theme') === 'light') {
                    document.documentElement.setAttribute('data-theme', 'light');
                }
            } catch (e) {}
        })();
    </script>
    @vite(['resources/css/app.css', 'resources/js/app.js'])
    @inertiaHead
</head>
<body class="app-bg antialiased">
    @inertia
</body>
</html>
