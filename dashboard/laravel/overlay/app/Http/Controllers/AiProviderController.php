<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Str;
use Inertia\Inertia;

class AiProviderController extends Controller
{
    public function index()
    {
        $providers = rescue(fn () => DB::table('ai_providers')->orderBy('priority')->get(), collect());
        $keys = rescue(fn () => DB::table('ai_provider_keys')->get()->groupBy('provider_id'), collect());

        // 24h cost + request counts per provider.
        $usage = rescue(fn () => DB::table('ai_requests')
            ->selectRaw("provider, count(*) requests, sum(cost_usd) cost, "
                . "avg(latency_ms) latency, sum(case when success then 0 else 1 end) failures")
            ->whereRaw("time >= now() - interval '24 hours'")
            ->groupBy('provider')->get()->keyBy('provider'), collect());

        $live = rescue(fn () => Http::timeout(3)->get(env('AI_ENGINE_URL', 'http://ai-engine:8000') . '/ai/providers/status')->json(), ['providers' => []]);

        $settings = rescue(fn () => DB::table('ai_router_settings')->where('id', 1)->first(), null);

        return Inertia::render('AiProviders', [
            'settings' => [
                'retries_per_key' => (int) ($settings->retries_per_key ?? 1),
                'max_failures' => (int) ($settings->max_failures ?? 3),
                'circuit_open_seconds' => (int) ($settings->circuit_open_seconds ?? 60),
            ],
            'providers' => $providers->map(function ($p) use ($keys, $usage) {
                $u = $usage->get($p->name);
                return [
                    'id' => $p->id,
                    'name' => $p->name,
                    'enabled' => (bool) $p->enabled,
                    'priority' => $p->priority,
                    'weight' => $p->weight,
                    'models' => json_decode($p->models, true),
                    'keys' => ($keys->get($p->id) ?? collect())->map(fn ($k) => [
                        'id' => $k->id, 'label' => $k->label, 'enabled' => (bool) $k->enabled,
                        'status' => $k->status, 'failures' => $k->consecutive_failures,
                        'daily_limit_usd' => $k->daily_limit_usd,
                    ])->values(),
                    'usage_24h' => [
                        'requests' => (int) ($u->requests ?? 0),
                        'cost_usd' => round((float) ($u->cost ?? 0), 4),
                        'avg_latency_ms' => round((float) ($u->latency ?? 0), 0),
                        'failures' => (int) ($u->failures ?? 0),
                    ],
                ];
            })->values(),
            'liveStatus' => $live,
            'mode' => env('AI_LB_MODE', 'cost'),
        ]);
    }

    public function toggleProvider(Request $request, int $id)
    {
        rescue(fn () => DB::table('ai_providers')->where('id', $id)
            ->update(['enabled' => $request->boolean('enabled'), 'updated_at' => now()]));
        return back()->with('success', 'Provider updated.');
    }

    public function setPriority(Request $request, int $id)
    {
        $data = $request->validate(['priority' => ['required', 'integer', 'min:1', 'max:999']]);
        rescue(fn () => DB::table('ai_providers')->where('id', $id)
            ->update(['priority' => $data['priority'], 'updated_at' => now()]));
        return back()->with('success', 'Priority updated — failover order takes effect within ~15s.');
    }

    public function saveSettings(Request $request)
    {
        $data = $request->validate([
            'retries_per_key' => ['required', 'integer', 'min:0', 'max:10'],
            'max_failures' => ['required', 'integer', 'min:1', 'max:50'],
            'circuit_open_seconds' => ['required', 'integer', 'min:5', 'max:3600'],
        ]);
        rescue(fn () => DB::table('ai_router_settings')->updateOrInsert(
            ['id' => 1],
            $data + ['updated_at' => now()],
        ));
        return back()->with('success', 'Failover settings saved — applied live within ~15s.');
    }

    public function toggleKey(Request $request, int $id)
    {
        rescue(fn () => DB::table('ai_provider_keys')->where('id', $id)
            ->update(['enabled' => $request->boolean('enabled')]));
        return back()->with('success', 'API key updated.');
    }

    /**
     * Add a real API key. The secret is written to VAULT (never stored in
     * Postgres); only a `vault:` reference is saved. The AI router resolves the
     * reference at request time.
     */
    public function addKey(Request $request, int $providerId)
    {
        $data = $request->validate([
            'label' => ['required', 'string', 'max:60'],
            'secret' => ['required', 'string', 'min:8'],
            'daily_limit_usd' => ['nullable', 'numeric', 'min:0'],
            'monthly_limit_usd' => ['nullable', 'numeric', 'min:0'],
        ]);

        $provider = DB::table('ai_providers')->where('id', $providerId)->first();
        if (! $provider) {
            return back()->with('error', 'Unknown provider.');
        }

        $slug = Str::slug($data['label']) ?: 'key-' . now()->timestamp;
        $path = "ai/{$provider->name}/{$slug}";

        // Store the secret in Vault (KV v2).
        $vaultAddr = rtrim(env('VAULT_ADDR', 'http://vault:8200'), '/');
        $resp = rescue(fn () => Http::withHeaders(['X-Vault-Token' => env('VAULT_TOKEN', 'dev-only-root-token')])
            ->timeout(5)
            ->put("{$vaultAddr}/v1/secret/data/{$path}", ['data' => ['api_key' => $data['secret']]]), null);

        if (! $resp || ! $resp->successful()) {
            return back()->with('error', 'Could not store the key in Vault. Is the Vault service running?');
        }

        rescue(fn () => DB::table('ai_provider_keys')->updateOrInsert(
            ['provider_id' => $providerId, 'label' => $data['label']],
            [
                'secret_ref' => "vault:{$path}#api_key",
                'enabled' => true,
                'status' => 'healthy',
                'consecutive_failures' => 0,
                'circuit_open_until' => null,
                'daily_limit_usd' => $data['daily_limit_usd'] ?? null,
                'monthly_limit_usd' => $data['monthly_limit_usd'] ?? null,
            ]
        ));

        return back()->with('success', "API key added for {$provider->name} and stored in Vault.");
    }

    public function deleteKey(int $id)
    {
        rescue(fn () => DB::table('ai_provider_keys')->where('id', $id)->delete());
        return back()->with('success', 'API key removed.');
    }

    /**
     * Paginated per-key request log (for the expandable "load more on scroll"
     * drill-down). Cursor by `before` timestamp, newest first. Returns each API
     * request with its status and — if it failed — the error reason.
     */
    public function keyRequests(Request $request, int $id)
    {
        $before = $request->query('before');
        $limit = 25;

        $rows = rescue(function () use ($id, $before, $limit) {
            $q = DB::table('ai_requests')->where('key_id', $id);
            if ($before) {
                $q->where('time', '<', $before);
            }
            return $q->orderByDesc('time')->limit($limit + 1)->get([
                'time', 'model', 'kind', 'latency_ms', 'input_tokens',
                'output_tokens', 'cost_usd', 'success', 'error',
            ]);
        }, collect());

        $hasMore = $rows->count() > $limit;
        $items = $rows->take($limit)->map(fn ($r) => [
            'time' => $r->time,
            'model' => $r->model,
            'kind' => $r->kind,
            'latency_ms' => (int) $r->latency_ms,
            'in_tokens' => (int) $r->input_tokens,
            'out_tokens' => (int) $r->output_tokens,
            'cost' => (float) $r->cost_usd,
            'success' => (bool) $r->success,
            'error' => $r->error,
        ])->values();

        return response()->json([
            'items' => $items,
            'next' => $hasMore ? ($items->last()['time'] ?? null) : null,
        ]);
    }
}
