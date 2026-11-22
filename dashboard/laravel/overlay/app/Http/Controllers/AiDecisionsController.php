<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

/**
 * AI Decisions / Explainability feed. Shows every signal the risk engine
 * evaluated and WHY it traded or skipped — rule strength, AI confidence, news
 * sentiment, market regime, and the deciding gate. Per-tenant (RLS).
 */
class AiDecisionsController extends Controller
{
    private const STRAT = [
        'trend_following' => 'Trend Following', 'momentum' => 'Momentum',
        'mean_reversion' => 'Mean Reversion', 'breakout' => 'Breakout',
        'vol_expansion' => 'Volatility Expansion',
    ];

    public function index()
    {
        $rows = rescue(fn () => DB::table('trade_decisions')->orderByDesc('ts')->limit(120)->get(), collect());

        $decisions = $rows->map(fn ($d) => [
            'id' => $d->id,
            'ts' => $d->ts,
            'symbol' => $d->symbol,
            'strategy' => self::STRAT[$d->strategy] ?? $d->strategy,
            'side' => $d->side,
            'strength' => round((float) $d->signal_strength, 2),
            'ai_confidence' => $d->ai_confidence !== null ? (int) round((float) $d->ai_confidence * 100) : null,
            'ai_mode' => $d->ai_mode,
            'sentiment' => $d->sentiment !== null ? (int) round((float) $d->sentiment) : null,
            'regime' => $d->regime,
            'action' => $d->action,
            'reason' => $d->reason,
        ])->values();

        $providers = rescue(fn () => DB::table('ai_provider_keys')->where('enabled', true)
            ->distinct()->pluck('provider')->all(), []);
        $confRows = $rows->whereNotNull('ai_confidence');

        return Inertia::render('AiDecisions', [
            'decisions' => $decisions,
            'kpi' => [
                'total' => $rows->count(),
                'traded' => $rows->where('action', 'traded')->count(),
                'skipped' => $rows->where('action', 'skipped')->count(),
                'avg_conf' => $confRows->count() ? (int) round($confRows->avg('ai_confidence') * 100) : null,
            ],
            'ai_mode' => count($providers) ? 'consensus' : 'heuristic',
            'providers' => array_values($providers),
        ]);
    }
}
