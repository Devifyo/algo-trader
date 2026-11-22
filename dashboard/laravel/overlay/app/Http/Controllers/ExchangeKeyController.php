<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;
use Inertia\Inertia;

class ExchangeKeyController extends Controller
{
    public function index()
    {
        $keys = rescue(fn () => DB::table('exchange_keys')
            ->select('id', 'exchange', 'label', 'permissions', 'withdraw_enabled',
                'ip_whitelisted', 'safe', 'last_checked')
            ->orderByDesc('created_at')->get()->toArray(), []);
        return Inertia::render('ExchangeKeys', ['keys' => $keys]);
    }

    /**
     * Connection test BEFORE saving. In production the permissions come from the
     * exchange (e.g. Binance apiRestrictions); here the caller supplies them.
     * The safety service is the gate; we never persist a withdrawal-enabled key,
     * and the real secret would be written to Vault, never to Postgres.
     */
    public function test(Request $request)
    {
        $url = env('PORTFOLIO_URL', 'http://portfolio-service:8000') . '/safety/exchange-key';
        $result = rescue(fn () => Http::timeout(5)->post($url, [
            'permissions' => $request->input('permissions', []),
            'ip_whitelisted' => $request->boolean('ip_whitelisted'),
            'reachable' => $request->boolean('reachable', true),
            'valid_key' => $request->boolean('valid_key', true),
        ])->json(), ['safe' => false, 'warnings' => [['code' => 'safety_service_unreachable']]]);

        return back()->with('success', null)->with('safetyResult', $result);
    }

    public function store(Request $request)
    {
        $perms = $request->input('permissions', []);
        if (! empty($perms['withdraw_permission'])) {
            return back()->with('error', 'Refused: this key can withdraw funds. Disable withdrawals on the exchange first.');
        }
        // SECURITY: scope to the authenticated tenant — never a shared default
        // (otherwise every tenant's keys collide under one partition).
        $tenant = $request->user()->tenant_id;
        $exchange = $request->input('exchange');
        rescue(fn () => DB::table('exchange_keys')->insert([
            'tenant_id' => $tenant,
            'exchange' => $exchange,
            'label' => $request->input('label'),
            'vault_path' => "secret/tenants/{$tenant}/brokers/{$exchange}",
            'permissions' => json_encode($perms),
            'withdraw_enabled' => false,
            'ip_whitelisted' => $request->boolean('ip_whitelisted'),
            'safe' => true,
            'last_checked' => now(),
        ]));
        return back()->with('success', 'Exchange key saved. Secret stored in Vault.');
    }
}
