<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

/**
 * Markets / AI Portfolio intelligence. Renders the live output of the Rust
 * opportunity scanner: per-market scores + factor breakdown (explainability),
 * the current market regime, and the dynamic capital allocation. All values are
 * computed live by the engine — nothing static.
 */
class MarketsController extends Controller
{
    public function index()
    {
        $scores = rescue(fn () => DB::table('market_scores')->orderByDesc('market_score')->get(), collect());
        $regime = rescue(fn () => DB::table('market_regime')->where('id', 1)->first(), null);
        $alloc = rescue(fn () => DB::table('capital_allocation')->orderBy('rank')->get(), collect());

        $rows = $scores->map(fn ($s) => [
            'symbol' => $s->symbol,
            'coin' => preg_replace('/USDT$/', '', $s->symbol),
            'price' => (float) $s->price,
            'market_score' => round((float) $s->market_score, 1),
            'bullish' => round((float) $s->bullish_score, 1),
            'bearish' => round((float) $s->bearish_score, 1),
            'confidence' => round((float) $s->confidence_score, 1),
            'risk' => round((float) $s->risk_score, 1),
            'trend' => round((float) $s->trend_strength, 1),
            'momentum' => round((float) $s->momentum, 2),
            'volatility' => round((float) $s->volatility, 2),
            'sentiment' => round((float) $s->sentiment, 2),
            'bias' => $s->bias,
        ])->values();

        return Inertia::render('Markets', [
            'scores' => $rows,
            'regime' => $regime ? [
                'regime' => $regime->regime,
                'volatility' => $regime->volatility,
                'breadth' => round((float) $regime->breadth, 0),
                'avg_volatility' => round((float) $regime->avg_volatility, 2),
                'favored' => $this->pgArray($regime->favored),
                'updated_at' => $regime->updated_at,
            ] : null,
            'allocation' => $alloc->map(fn ($a) => [
                'symbol' => $a->symbol,
                'weight' => round((float) $a->weight * 100, 1),
                'rank' => (int) $a->rank,
                'reason' => $a->reason,
            ])->values(),
            'opportunities' => $rows->where('bias', 'bullish')->sortByDesc('market_score')->take(5)->values(),
            'universe' => $rows->count(),
        ]);
    }

    private function pgArray($val): array
    {
        if (is_array($val)) {
            return $val;
        }
        $s = trim((string) $val, '{}');
        return $s === '' ? [] : array_map(fn ($x) => trim($x, '"'), str_getcsv($s));
    }
}
