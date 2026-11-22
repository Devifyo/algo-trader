<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Http;
use Inertia\Inertia;

/**
 * Strategy performance monitor (platform admin). Aggregates REAL results across
 * all tenants — closed positions (realized P&L) + fills — into per-strategy
 * metrics and a winning/losing verdict, plus a data-driven "best" pick. Uses the
 * owner connection so it sees every tenant.
 */
class StrategyController extends Controller
{
    private const VALID = ['trend_following', 'momentum', 'mean_reversion', 'breakout', 'vol_expansion'];

    private function db()
    {
        return DB::connection('pgsql_admin');
    }

    /**
     * Risk-adjusted metrics from a per-trade P&L series (chronological):
     *  - Sharpe  = mean / stdev of trade P&L (per-trade)
     *  - Sortino = mean / downside-deviation (penalizes only losers)
     *  - Max DD  = largest peak-to-trough drop of the cumulative equity curve ($)
     */
    private function riskMetrics(array $pnls): array
    {
        $n = count($pnls);
        if ($n < 2) {
            return ['sharpe' => null, 'sortino' => null, 'max_dd' => round(array_sum(array_map(fn ($p) => min($p, 0), $pnls)) * -1, 2)];
        }
        $mean = array_sum($pnls) / $n;
        $var = 0.0;
        $dvar = 0.0;
        $dn = 0;
        foreach ($pnls as $p) {
            $var += ($p - $mean) ** 2;
            if ($p < 0) {
                $dvar += $p ** 2;
                $dn++;
            }
        }
        $sd = sqrt($var / ($n - 1));
        $dd = $dn ? sqrt($dvar / $dn) : 0.0;
        $cum = 0.0;
        $peak = 0.0;
        $maxdd = 0.0;
        foreach ($pnls as $p) {
            $cum += $p;
            $peak = max($peak, $cum);
            $maxdd = max($maxdd, $peak - $cum);
        }
        return [
            'sharpe' => $sd > 0 ? round($mean / $sd, 2) : null,
            'sortino' => $dd > 0 ? round($mean / $dd, 2) : ($mean > 0 ? 99.0 : null),
            'max_dd' => round($maxdd, 2),
        ];
    }

    /** Admin designates the platform default strategy (shown across the app). */
    public function setDefault(Request $request)
    {
        $s = (string) $request->input('strategy');
        if (! in_array($s, self::VALID, true)) {
            return back()->with('error', 'Unknown strategy.');
        }
        rescue(fn () => $this->db()->table('platform_strategy')->where('id', 1)
            ->update(['default_strategy' => $s, 'updated_at' => now()]));
        return back()->with('success', 'Default strategy set to ' . ucwords(str_replace('_', ' ', $s)) . '.');
    }

    /** Run a quick (indicative, single-pass) backtest on real Binance data. */
    public function runBacktest(Request $request)
    {
        $strategy = (string) $request->input('strategy', 'trend_following');
        $symbol = strtoupper((string) $request->input('symbol', 'BTCUSDT'));
        $timeframe = (string) $request->input('timeframe', '1h');
        if (! in_array($strategy, self::VALID, true)) {
            return back()->with('error', 'Unknown strategy.');
        }
        $res = rescue(fn () => Http::timeout(70)
            ->post(env('AI_ENGINE_URL', 'http://ai-engine:8000') . '/ai/backtest', [
                'strategy' => $strategy, 'symbol' => $symbol, 'timeframe' => $timeframe,
            ])->json(), null);

        if ($res && ($res['ok'] ?? false)) {
            rescue(fn () => $this->db()->table('strategy_backtests')->insert([
                'strategy' => $strategy, 'symbol' => $symbol, 'timeframe' => $timeframe,
                'bars' => $res['bars'] ?? null, 'metrics' => json_encode($res['metrics'] ?? []), 'created_at' => now(),
            ]));
            $m = $res['metrics'] ?? [];
            return back()->with('success', "Backtest {$symbol} {$timeframe}: net " . ($m['net_return_pct'] ?? '?') . '%, PF ' . ($m['profit_factor'] ?? '?') . ', ' . ($m['trades'] ?? 0) . ' trades.');
        }
        return back()->with('error', $res['error'] ?? 'Backtest failed (data/AI engine unavailable).');
    }

    /**
     * Run a backtest on the NautilusTrader engine — the institutional,
     * backtest-only validation microservice (same code path as live, realistic
     * fees). Records the result alongside the quick backtests. BACKTEST ONLY.
     */
    public function nautilusBacktest(Request $request)
    {
        $symbol = strtoupper((string) $request->input('symbol', 'BTCUSDT'));
        $interval = (string) $request->input('interval', '1d');
        $fast = (int) $request->input('fast', 10);
        $slow = (int) $request->input('slow', 20);

        $res = rescue(fn () => Http::timeout(160)
            ->get(env('NAUTILUS_BACKTEST_URL', 'http://nautilus-backtest:8000') . '/backtest', [
                'symbol' => $symbol, 'interval' => $interval, 'fast' => $fast, 'slow' => $slow,
            ])->json(), null);

        if ($res && ($res['ok'] ?? false)) {
            rescue(fn () => $this->db()->table('strategy_backtests')->insert([
                'strategy' => $res['strategy'] ?? 'nautilus_long_ema', 'symbol' => $symbol, 'timeframe' => $interval,
                'bars' => $res['bars'] ?? null,
                'metrics' => json_encode([
                    'net_return_pct' => $res['return_pct'] ?? null,
                    'win_rate' => $res['win_rate_pct'] ?? null,
                    'trades' => $res['closed_trades'] ?? null,
                    'net_pnl_usd' => $res['net_pnl_usd'] ?? null,
                    'engine' => 'nautilus_trader',
                    'validated' => false,
                ]),
                'created_at' => now(),
            ]));

            return back()->with('success', "NautilusTrader {$symbol} {$interval}: net "
                . ($res['return_pct'] ?? '?') . '%, ' . ($res['closed_trades'] ?? 0) . ' trades, win '
                . ($res['win_rate_pct'] ?? '?') . '% — backtest only, NOT validated.');
        }

        return back()->with('error', $res['error'] ?? 'NautilusTrader engine unavailable.');
    }

    /**
     * Run an AI strategy-research pass NOW: feeds live performance context to the
     * multi-provider LLM router (with failover), gets a recommended default
     * strategy, and records it as a research run visible on this page.
     */
    public function rerunResearch()
    {
        $id = rescue(fn () => $this->db()->table('strategy_research_runs')->insertGetId([
            'source' => 'ai-analysis', 'status' => 'running', 'started_at' => now(),
        ]), null);

        // Brief live context to ground the recommendation — REAL-data-era only
        // (exclude synthetic history so the AI doesn't reason on fake results).
        $since = env('STRATEGY_STATS_SINCE');
        $ctx = rescue(function () use ($since) {
            $lines = ['Live high-liquidity crypto (BTC/ETH/SOL/BNB) strategy stats (real market data only):'];
            $closed = $this->db()->table('positions')->where('status', 'closed')
                ->when($since, fn ($q) => $q->where('closed_at', '>=', $since))
                ->selectRaw('strategy, count(*) t, COALESCE(sum(realized_pnl),0) pnl')
                ->groupBy('strategy')->get();
            if ($closed->isEmpty()) {
                $lines[] = 'No real closed trades yet (forward-test just started) — recommend the most robust default on evidence, not on this thin sample.';
            }
            foreach ($closed as $c) {
                $lines[] = "{$c->strategy}: {$c->t} closed trades, net pnl {$c->pnl}";
            }
            foreach ($this->db()->table('orders')->where('status', 'filled')
                ->whereRaw("created_at >= now() - interval '24 hours'")
                ->selectRaw('strategy, count(*) c')->groupBy('strategy')->pluck('c', 'strategy') as $s => $c) {
                $lines[] = "{$s}: {$c} fills/24h";
            }
            return implode("\n", $lines);
        }, 'No live stats available yet.');

        $res = rescue(fn () => Http::timeout(45)
            ->post(env('AI_ENGINE_URL', 'http://ai-engine:8000') . '/ai/strategy-research', ['context' => $ctx])
            ->json(), null);

        if ($id && $res && ($res['available'] ?? false)) {
            $text = trim((string) ($res['text'] ?? ''));
            $rec = null;
            $sum = $text;
            if (preg_match('/\{.*\}/s', $text, $m)) {
                $j = json_decode($m[0], true);
                if (is_array($j)) {
                    $rec = $j['recommended'] ?? null;
                    $sum = $j['summary'] ?? $text;
                }
            }
            $provider = $res['provider'] ?? 'ai';
            rescue(fn () => $this->db()->table('strategy_research_runs')->where('id', $id)->update([
                'status' => 'completed', 'recommended' => $rec,
                'summary' => mb_substr(trim($sum), 0, 480) . " [via {$provider}]",
                'completed_at' => now(),
            ]));
            return back()->with('success', 'AI research complete' . ($rec ? ": recommends {$rec}." : '.'));
        }

        if ($id) {
            rescue(fn () => $this->db()->table('strategy_research_runs')->where('id', $id)->update([
                'status' => 'failed', 'summary' => 'No AI provider available — check AI Providers.', 'completed_at' => now(),
            ]));
        }
        return back()->with('error', 'AI providers unavailable — could not run research.');
    }

    public function index()
    {
        // Only count trades from the REAL-data era. Earlier closed positions came
        // from the synthetic random-walk feed (+ phantom positions) and produce
        // nonsensical metrics (e.g. 96% win / PF 675) that would give false
        // confidence. STRATEGY_STATS_SINCE marks when real market data started.
        $since = env('STRATEGY_STATS_SINCE');

        $closed = rescue(fn () => $this->db()->table('positions')
            ->where('status', 'closed')
            ->when($since, fn ($q) => $q->where('closed_at', '>=', $since))
            ->selectRaw(
                "strategy,
                 count(*) trades,
                 count(*) FILTER (WHERE realized_pnl > 0) wins,
                 count(*) FILTER (WHERE realized_pnl < 0) losses,
                 COALESCE(sum(realized_pnl), 0) net_pnl,
                 COALESCE(sum(realized_pnl) FILTER (WHERE realized_pnl > 0), 0) gross_profit,
                 COALESCE(abs(sum(realized_pnl) FILTER (WHERE realized_pnl < 0)), 0) gross_loss,
                 COALESCE(avg(realized_pnl) FILTER (WHERE realized_pnl > 0), 0) avg_win,
                 COALESCE(avg(realized_pnl) FILTER (WHERE realized_pnl < 0), 0) avg_loss,
                 max(closed_at) last_closed"
            )->groupBy('strategy')->get()->keyBy('strategy'), collect());

        $open = rescue(fn () => $this->db()->table('positions')->where('status', 'open')
            ->selectRaw("strategy, count(*) c, COALESCE(sum(qty * entry_price), 0) notional")
            ->groupBy('strategy')->get()->keyBy('strategy'), collect());

        $fills = rescue(fn () => $this->db()->table('orders')->where('status', 'filled')
            ->whereRaw("created_at >= now() - interval '24 hours'")
            ->selectRaw("strategy, count(*) c")->groupBy('strategy')->pluck('c', 'strategy'), collect());

        // Per-trade P&L series (chronological) → Sharpe / Sortino / Max DD.
        $pnlRows = rescue(fn () => $this->db()->table('positions')->where('status', 'closed')
            ->when($since, fn ($q) => $q->where('closed_at', '>=', $since))
            ->orderBy('closed_at')->get(['strategy', 'realized_pnl']), collect());
        $pnlSeries = [];
        foreach ($pnlRows as $r) {
            $pnlSeries[$r->strategy][] = (float) $r->realized_pnl;
        }

        $names = collect(['trend_following', 'momentum', 'mean_reversion', 'breakout', 'vol_expansion'])
            ->merge($closed->keys())->filter()->unique()->values();

        $rows = $names->map(function ($s) use ($closed, $open, $fills, $pnlSeries) {
            $c = $closed->get($s);
            $trades = (int) ($c->trades ?? 0);
            $wins = (int) ($c->wins ?? 0);
            $net = round((float) ($c->net_pnl ?? 0), 2);
            $gp = (float) ($c->gross_profit ?? 0);
            $gl = (float) ($c->gross_loss ?? 0);
            $pf = $gl > 0 ? round($gp / $gl, 2) : ($gp > 0 ? 99.0 : 0.0);
            $winRate = $trades ? round($wins / $trades * 100, 1) : 0.0;
            $expectancy = $trades ? round($net / $trades, 2) : 0.0;

            // Verdict needs a minimum sample so we don't crown/condemn on noise.
            $verdict = 'insufficient';
            if ($trades >= 10) {
                if ($net > 0 && $pf >= 1.1) {
                    $verdict = 'winning';
                } elseif ($net < 0 || $pf < 0.9) {
                    $verdict = 'losing';
                } else {
                    $verdict = 'breakeven';
                }
            }

            $rm = $this->riskMetrics($pnlSeries[$s] ?? []);

            return [
                'strategy' => $s,
                'label' => ucwords(str_replace('_', ' ', $s)),
                'trades' => $trades,
                'wins' => $wins,
                'losses' => (int) ($c->losses ?? 0),
                'win_rate' => $winRate,
                'net_pnl' => $net,
                'profit_factor' => $pf,
                'expectancy' => $expectancy,
                'avg_win' => round((float) ($c->avg_win ?? 0), 2),
                'avg_loss' => round((float) ($c->avg_loss ?? 0), 2),
                'sharpe' => $rm['sharpe'],
                'sortino' => $rm['sortino'],
                'max_dd' => $rm['max_dd'],
                'open_positions' => (int) (optional($open->get($s))->c ?? 0),
                'fills_24h' => (int) ($fills[$s] ?? 0),
                'verdict' => $verdict,
            ];
        })->values();

        // Data-driven recommendation: best risk-adjusted winner with enough sample.
        $best = $rows->filter(fn ($r) => $r['verdict'] === 'winning')
            ->sortByDesc('profit_factor')->first();

        // Strategy-research run history (so the running/last research is visible here).
        $runs = rescue(fn () => $this->db()->table('strategy_research_runs')
            ->orderByDesc('id')->limit(10)->get()->map(fn ($r) => [
                'id' => $r->id, 'source' => $r->source, 'status' => $r->status,
                'recommended' => $r->recommended, 'summary' => $r->summary,
                'started_at' => $r->started_at, 'completed_at' => $r->completed_at,
            ])->values(), collect());

        $defaultStrategy = rescue(fn () => $this->db()->table('platform_strategy')
            ->where('id', 1)->value('default_strategy'), null);

        $backtests = rescue(fn () => $this->db()->table('strategy_backtests')
            ->orderByDesc('id')->limit(12)->get()->map(fn ($b) => [
                'id' => $b->id, 'strategy' => $b->strategy, 'symbol' => $b->symbol,
                'timeframe' => $b->timeframe, 'bars' => $b->bars,
                'metrics' => json_decode($b->metrics, true) ?: [],
                'created_at' => $b->created_at,
            ])->values(), collect());

        return Inertia::render('Admin/Strategies', [
            'strategies' => $rows,
            'recommended' => $best['strategy'] ?? null,
            'defaultStrategy' => $defaultStrategy,
            'totalClosed' => (int) $rows->sum('trades'),
            'since' => $since,
            'researchRuns' => $runs,
            'backtests' => $backtests,
            'forwardTest' => [
                'active' => (bool) $since,
                'since' => $since,
                'closed_trades' => (int) $rows->sum('trades'),
                'best' => $best['strategy'] ?? null,
            ],
        ]);
    }
}
