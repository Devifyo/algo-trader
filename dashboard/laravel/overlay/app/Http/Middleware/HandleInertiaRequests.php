<?php

namespace App\Http\Middleware;

use Illuminate\Http\Request;
use Inertia\Middleware;

class HandleInertiaRequests extends Middleware
{
    protected $rootView = 'app';

    public function share(Request $request): array
    {
        $user = $request->user();
        $isAdmin = $user && $user->isAdmin();

        $userNav = [
            ['group' => 'Trading', 'items' => [
                ['key' => 'dashboard', 'label' => 'Dashboard', 'href' => '/dashboard', 'icon' => 'dashboard'],
                ['key' => 'markets', 'label' => 'Markets', 'href' => '/markets', 'icon' => 'portfolio'],
                ['key' => 'bots', 'label' => 'Bots', 'href' => '/bots', 'icon' => 'bots'],
                ['key' => 'portfolio', 'label' => 'Portfolio', 'href' => '/portfolio', 'icon' => 'portfolio'],
                ['key' => 'brokers', 'label' => 'Brokers', 'href' => '/brokers', 'icon' => 'brokers'],
                ['key' => 'logs', 'label' => 'Execution Logs', 'href' => '/logs', 'icon' => 'logs'],
            ]],
            ['group' => 'Intelligence', 'items' => [
                ['key' => 'opportunities', 'label' => 'Opportunities', 'href' => '/opportunities', 'icon' => 'portfolio'],
                ['key' => 'ai-decisions', 'label' => 'AI Decisions', 'href' => '/ai-decisions', 'icon' => 'providers'],
                ['key' => 'news', 'label' => 'News & Sentiment', 'href' => '/news', 'icon' => 'news'],
            ]],
            ['group' => 'Community', 'items' => [
                ['key' => 'marketplace', 'label' => 'Marketplace', 'href' => '/marketplace', 'icon' => 'marketplace'],
                ['key' => 'copy', 'label' => 'Copy Trading', 'href' => '/copy-trading', 'icon' => 'copy'],
            ]],
            ['group' => 'Support', 'items' => [
                ['key' => 'help', 'label' => 'Help Center', 'href' => '/help', 'icon' => 'help'],
            ]],
        ];
        $adminNav = [
            ['group' => 'Platform', 'items' => [
                ['key' => 'admin', 'label' => 'Overview', 'href' => '/admin', 'icon' => 'admin'],
                ['key' => 'tenants', 'label' => 'Tenants', 'href' => '/admin/tenants', 'icon' => 'tenants'],
                ['key' => 'strategies', 'label' => 'Strategies', 'href' => '/admin/strategies', 'icon' => 'bots'],
            ]],
            ['group' => 'AI', 'items' => [
                ['key' => 'providers', 'label' => 'AI Providers', 'href' => '/admin/ai-providers', 'icon' => 'providers'],
            ]],
            ['group' => 'Config', 'items' => [
                ['key' => 'flags', 'label' => 'Feature Flags', 'href' => '/admin/feature-flags', 'icon' => 'flags'],
            ]],
        ];

        return array_merge(parent::share($request), [
            'app' => [
                'name' => config('app.name', 'algo-trader'),
                'tradingMode' => env('TRADING_MODE', 'paper'),
                'region' => env('REGION', 'us-east-1'),
            ],
            'auth' => [
                'user' => $user ? [
                    'name' => $user->name,
                    'email' => $user->email,
                    'role' => $user->role,
                    'isAdmin' => $isAdmin,
                ] : null,
                // True when an admin is currently impersonating this tenant user.
                'impersonating' => $request->session()->has('impersonator_id'),
            ],
            'nav' => $isAdmin ? $adminNav : $userNav,
            // Unread notification count for the topbar bell. Use the owner
            // connection with an explicit tenant filter — the per-request RLS
            // GUC isn't reliably set this early in the response lifecycle.
            'unreadNotifications' => $user
                ? (int) rescue(fn () => \Illuminate\Support\Facades\DB::connection('pgsql_admin')
                    ->table('notifications')->where('tenant_id', $user->tenant_id)
                    ->whereNull('read_at')->count(), 0)
                : 0,
            'flash' => [
                'success' => fn () => $request->session()->get('success'),
                'error' => fn () => $request->session()->get('error'),
                'safetyResult' => fn () => $request->session()->get('safetyResult'),
            ],
        ]);
    }
}
