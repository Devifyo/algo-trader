<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;
use Inertia\Inertia;

/**
 * Opportunity Engine — button-mode momentum/trend/volume screen + portfolio
 * awareness. Surfaces RESEARCH SIGNALS only (no return forecasts, no orders).
 * The heavy scan runs in the AI engine; this proxies it and adds the tenant's
 * own holdings (RLS-scoped via the default connection).
 */
class OpportunityController extends Controller
{
    public function index()
    {
        $holdings = rescue(fn () => DB::table('positions')->where('status', 'open')
            ->orderByDesc('created_at')
            ->get(['symbol', 'side', 'qty', 'entry_price', 'strategy', 'created_at'])
            ->map(fn ($p) => [
                'symbol' => $p->symbol,
                'side' => $p->side,
                'qty' => (float) $p->qty,
                'entry' => (float) $p->entry_price,
                'strategy' => $p->strategy,
                'opened' => $p->created_at,
            ])->values(), collect());

        return Inertia::render('Opportunities', ['holdings' => $holdings]);
    }

    /** Button-mode: run the live scan now (proxied to the AI engine). */
    public function scan()
    {
        $res = rescue(fn () => Http::timeout(45)
            ->get(env('AI_ENGINE_URL', 'http://ai-engine:8000') . '/ai/opportunities', ['limit' => 14])
            ->json(), null);

        if (! $res || ! ($res['ok'] ?? false)) {
            return response()->json(['ok' => false, 'error' => $res['error'] ?? 'scanner unavailable', 'candidates' => []]);
        }
        return response()->json($res);
    }
}
