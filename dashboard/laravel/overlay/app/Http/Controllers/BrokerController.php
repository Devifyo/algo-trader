<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Str;
use Inertia\Inertia;

/**
 * Broker catalog + per-tenant connections. The connect form adapts to each
 * broker's `credential_kind`; all secrets are written to the tenant's Vault
 * partition and only a reference is stored in Postgres.
 */
class BrokerController extends Controller
{
    // Brand domains used to fetch broker logos in the UI.
    private array $domains = [
        'dhan' => 'dhan.co', 'zerodha' => 'zerodha.com', 'upstox' => 'upstox.com',
        'angelone' => 'angelone.in', 'fyers' => 'fyers.in', 'shoonya' => 'shoonya.com',
        'flattrade' => 'flattrade.in', 'fivepaisa' => '5paisa.com', 'alpaca' => 'alpaca.markets',
        'tradier' => 'tradier.com', 'schwab' => 'schwab.com', 'tastytrade' => 'tastytrade.com',
        'interactive_brokers' => 'interactivebrokers.com', 'oanda' => 'oanda.com',
        'binance' => 'binance.com', 'bybit' => 'bybit.com', 'okx' => 'okx.com',
        'coinbase' => 'coinbase.com', 'deribit' => 'deribit.com', 'kraken' => 'kraken.com',
    ];

    // Brokers that offer a paper/sandbox environment (shows a Paper/Live toggle).
    private array $paper = ['alpaca' => true, 'tradier' => true, 'tastytrade' => true, 'oanda' => true];

    // Which secret fields each credential kind requires.
    private array $fields = [
        'api_key_secret' => ['api_key', 'api_secret'],
        'api_key_secret_passphrase' => ['api_key', 'api_secret', 'passphrase'],
        'api_key_totp' => ['api_key', 'api_secret', 'totp_secret'],
        'oauth2' => ['client_id', 'client_secret', 'redirect_uri'],
        'bearer_token' => ['account_id', 'token'],
    ];

    public function index()
    {
        $brokers = rescue(fn () => DB::table('brokers')->where('enabled', true)
            ->orderBy('market')->orderBy('name')->get(), collect());

        $byMarket = $brokers->groupBy('market')->map(fn ($g) => $g->map(fn ($b) => [
            'broker_id' => $b->broker_id, 'name' => $b->name, 'market' => $b->market,
            'exchanges' => $this->pgArray($b->exchanges), 'auth_type' => $b->auth_type,
            'credential_kind' => $b->credential_kind, 'rate_limit_rpm' => $b->rate_limit_rpm,
            'features' => $this->pgArray($b->features), 'documentation_url' => $b->documentation_url,
            'fields' => $this->fields[$b->credential_kind] ?? ['api_key', 'api_secret'],
            'logo_domain' => $this->domains[$b->broker_id] ?? null,
            'supports_paper' => $this->paper[$b->broker_id] ?? false,
        ])->values());

        $connections = rescue(fn () => DB::table('exchange_keys')
            ->select('id', 'broker_id', 'exchange', 'label', 'ip_whitelisted', 'safe', 'last_checked')
            ->orderByDesc('created_at')->get(), []);

        return Inertia::render('Brokers', [
            'marketsOrder' => ['India', 'Global', 'Crypto'],
            'byMarket' => $byMarket,
            'connections' => $connections,
        ]);
    }

    public function connect(Request $request, string $brokerId)
    {
        $broker = rescue(fn () => DB::table('brokers')->where('broker_id', $brokerId)->where('enabled', true)->first(), null);
        if (! $broker) {
            return back()->with('error', 'Unknown or disabled broker.');
        }

        $needed = $this->fields[$broker->credential_kind] ?? ['api_key', 'api_secret'];
        // redirect_uri is optional for oauth2; everything else is required.
        $rules = ['label' => ['nullable', 'string', 'max:60']];
        foreach ($needed as $f) {
            $rules["creds.$f"] = $f === 'redirect_uri' ? ['nullable', 'string'] : ['required', 'string'];
        }
        $data = $request->validate($rules);

        // Crypto safety: refuse unless the user confirms withdrawals are disabled.
        if ($broker->market === 'Crypto' && ! $request->boolean('withdraw_disabled_confirmed')) {
            return back()->with('error', 'Please confirm that withdrawals are DISABLED on this API key before connecting.');
        }

        $tenant = Auth::user()->tenant_id;
        $path = "tenants/{$tenant}/brokers/{$brokerId}";
        $creds = collect($needed)->mapWithKeys(fn ($f) => [$f => $request->input("creds.$f")])->filter()->all();
        if ($this->paper[$brokerId] ?? false) {
            $creds['environment'] = $request->input('environment') === 'live' ? 'live' : 'paper';
        }

        $vaultAddr = rtrim(env('VAULT_ADDR', 'http://vault:8200'), '/');
        $resp = rescue(fn () => Http::withHeaders(['X-Vault-Token' => env('VAULT_TOKEN', 'dev-only-root-token')])
            ->timeout(5)->put("{$vaultAddr}/v1/secret/data/{$path}", ['data' => $creds]), null);
        if (! $resp || ! $resp->successful()) {
            return back()->with('error', 'Could not store credentials in Vault. Is the Vault service running?');
        }

        rescue(fn () => DB::table('exchange_keys')->updateOrInsert(
            ['tenant_id' => $tenant, 'exchange' => $brokerId],
            [
                'broker_id' => $brokerId,
                'label' => $data['label'] ?? $broker->name,
                'vault_path' => "secret/{$path}",
                'permissions' => json_encode(['market' => $broker->market, 'credential_kind' => $broker->credential_kind]),
                'withdraw_enabled' => false,
                'ip_whitelisted' => $request->boolean('ip_whitelisted'),
                'safe' => true,
                'last_checked' => now(),
            ]
        ));

        return back()->with('success', "{$broker->name} connected. Credentials stored in Vault.");
    }

    public function disconnect(int $id)
    {
        rescue(fn () => DB::table('exchange_keys')->where('id', $id)->delete());
        return back()->with('success', 'Broker disconnected.');
    }

    private function pgArray($val): array
    {
        if (is_array($val)) {
            return $val;
        }
        $s = trim((string) $val, '{}');
        if ($s === '') {
            return [];
        }
        return array_map(fn ($x) => trim($x, '"'), str_getcsv($s));
    }
}
