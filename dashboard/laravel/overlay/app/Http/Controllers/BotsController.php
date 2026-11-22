<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Validation\Rule;
use Inertia\Inertia;

/**
 * Bots = a tenant's strategy instances. All metrics are computed live from the
 * tenant's real orders/positions (RLS-scoped) — nothing is static. Creating a
 * bot enables its strategy so the engine begins trading it; pausing disables it.
 */
class BotsController extends Controller
{
    /** Strategy types the engine can run (see signals crate `strategies::build`). */
    public const STRATEGIES = [
        'trend_following' => 'Trend Following',
        'momentum' => 'Momentum',
        'mean_reversion' => 'Mean Reversion',
        'breakout' => 'Breakout',
        'vol_expansion' => 'Volatility Expansion',
    ];
    public const SYMBOLS = [
        'BTCUSDT', 'ETHUSDT', 'SOLUSDT', 'BNBUSDT', 'XRPUSDT', 'DOGEUSDT',
        'AVAXUSDT', 'LINKUSDT', 'ADAUSDT', 'DOTUSDT', 'LTCUSDT', 'ATOMUSDT',
    ];
    /** Market-selection modes (one strategy can monitor many markets). */
    public const MODES = [
        'ai_auto' => 'AI Auto-Discovery — scan all markets',
        'custom_watchlist' => 'Custom Watchlist — markets you choose',
        'single_symbol' => 'Single Symbol — one market only',
    ];
    private const GRADS = ['grad-violet', 'grad-blue', 'grad-emerald', 'grad-amber', 'grad-rose', 'grad-pink'];

    public function index()
    {
        $rows = rescue(fn () => DB::table('bots')->orderBy('created_at')->get(), collect());

        // Brokers this tenant can run a bot on: Paper sim + any connected broker.
        $connected = rescue(fn () => DB::table('exchange_keys')->where('safe', true)
            ->whereNotNull('broker_id')->get(['broker_id', 'label']), collect());
        $brokers = collect([['id' => 'paper', 'label' => 'Paper (simulated)', 'live' => false]])
            ->concat($connected->map(fn ($c) => [
                'id' => $c->broker_id,
                'label' => $c->label ?: ucfirst($c->broker_id),
                'live' => true,
            ]))->unique('id')->values();
        $brokerLabel = $brokers->keyBy('id');

        // Live 24h metrics, grouped once per strategy (no per-bot N+1).
        // "Trades" = orders that actually executed (filled) — NOT every attempt.
        // Counting all attempts inflates the number with skipped/rejected orders.
        $trades = rescue(fn () => DB::table('orders')
            ->where('status', 'filled')
            ->whereRaw("created_at >= now() - interval '24 hours'")
            ->select('strategy', DB::raw('count(*) c'))->groupBy('strategy')->pluck('c', 'strategy'), collect());
        $pnl = rescue(fn () => DB::table('positions')->where('status', 'closed')
            ->whereRaw("closed_at >= now() - interval '24 hours'")
            ->select('strategy', DB::raw('sum(realized_pnl) s'))->groupBy('strategy')->pluck('s', 'strategy'), collect());
        $wins = rescue(fn () => DB::table('positions')->where('status', 'closed')
            ->select('strategy', DB::raw('count(*) total'), DB::raw('count(*) filter (where realized_pnl > 0) won'))
            ->groupBy('strategy')->get()->keyBy('strategy'), collect());
        // Real capital deployed in open positions per strategy (the bot's used allocation).
        $deployed = rescue(fn () => DB::table('positions')->where('status', 'open')
            ->select('strategy', DB::raw('sum(qty * entry_price) d'))->groupBy('strategy')->pluck('d', 'strategy'), collect());

        $bots = $rows->map(function ($b) use ($trades, $pnl, $wins, $deployed, $brokerLabel) {
            $c = $this->config($b);
            $strat = $b->strategy;
            $w = $wins->get($strat);
            $winRate = $w && $w->total > 0 ? round($w->won / $w->total * 100, 1) : 0.0;
            $broker = $b->exchange ?: 'paper';
            $mode = $c['mode'] ?? 'ai_auto';
            $watch = $c['watchlist'] ?? self::SYMBOLS;
            $marketsLabel = $mode === 'ai_auto' ? 'All markets ('.count(self::SYMBOLS).')'
                : ($mode === 'single_symbol' ? ($c['symbol'] ?? ($watch[0] ?? '—')) : count($watch).' markets');
            return [
                'id' => $b->id, 'name' => $b->name, 'type' => $b->type,
                'strategy' => $c['strategy_label'] ?? (self::STRATEGIES[$strat] ?? $strat),
                'broker' => $broker,
                'broker_label' => $brokerLabel[$broker]['label'] ?? ($broker === 'paper' ? 'Paper sim' : ucfirst($broker)),
                'is_live' => $broker !== 'paper',
                'mode' => $mode,
                'markets' => is_array($watch) ? count($watch) : 0,
                'markets_label' => $marketsLabel,
                // raw fields for the edit form
                'watchlist' => is_array($watch) ? array_values($watch) : [],
                'symbol' => $c['symbol'] ?? ($watch[0] ?? 'BTCUSDT'),
                'allocation' => (float) ($c['alloc_total'] ?? 0),
                'min_pct' => (float) ($c['min_pct'] ?? 25),
                'max_pct' => (float) ($c['max_pct'] ?? 100),
                'strategy_key' => $strat,
                'exchange' => $b->exchange, 'status' => $b->status,
                'pair' => $c['pair'] ?? null, 'coin' => $c['coin'] ?? null,
                'trades_24h' => (int) ($trades[$strat] ?? 0),
                'profit_24h' => round((float) ($pnl[$strat] ?? 0), 2),
                'win_rate' => $winRate,
                'alloc_used' => round((float) ($deployed[$strat] ?? 0), 2),
                'alloc_total' => (float) ($c['alloc_total'] ?? 0),
                'grad' => $c['grad'] ?? 'grad-violet',
            ];
        })->values();

        $running = $bots->where('status', 'running')->count();
        return Inertia::render('Bots', [
            'bots' => $bots,
            'options' => ['strategies' => self::STRATEGIES, 'symbols' => self::SYMBOLS, 'brokers' => $brokers, 'modes' => self::MODES],
            'kpi' => [
                'active' => $running,
                'total_bots' => $bots->count(),
                'trades_24h' => (int) $bots->sum('trades_24h'),
                'profit_24h' => round((float) $bots->sum('profit_24h'), 2),
                'win_rate' => $bots->count() ? round((float) $bots->avg('win_rate'), 1) : 0,
            ],
        ]);
    }

    public function store(Request $request)
    {
        $tenant = $request->user()->tenant_id;
        // Valid brokers = "paper" + this tenant's connected, safe broker ids.
        $connectedIds = rescue(fn () => DB::table('exchange_keys')->where('safe', true)
            ->whereNotNull('broker_id')->pluck('broker_id')->all(), []);
        $validBrokers = array_values(array_unique(array_merge(['paper'], $connectedIds)));

        $data = $request->validate([
            'name' => ['required', 'string', 'max:80'],
            'strategy' => ['required', Rule::in(array_keys(self::STRATEGIES))],
            'mode' => ['required', Rule::in(array_keys(self::MODES))],
            'symbol' => ['required_if:mode,single_symbol', Rule::in(self::SYMBOLS)],
            'watchlist' => ['required_if:mode,custom_watchlist', 'array'],
            'watchlist.*' => [Rule::in(self::SYMBOLS)],
            'allocation' => ['required', 'numeric', 'min:10', 'max:1000000'],
            'min_pct' => ['nullable', 'numeric', 'min:1', 'max:100'],
            'max_pct' => ['nullable', 'numeric', 'min:1', 'max:100'],
            'broker' => ['required', Rule::in($validBrokers)],
        ]);

        // Resolve which markets this bot monitors from its mode.
        $watchlist = match ($data['mode']) {
            'single_symbol' => [$data['symbol']],
            'custom_watchlist' => array_values(array_unique($data['watchlist'])),
            default => self::SYMBOLS, // ai_auto: scan the whole universe
        };
        $repr = $watchlist[0] ?? 'BTCUSDT';
        $coin = preg_replace('/USDT$/', '', $repr);
        $config = [
            'mode' => $data['mode'],
            'watchlist' => $watchlist,
            'symbol' => $repr,
            'pair' => $coin . '/USDT', 'coin' => $coin,
            'strategy_label' => self::STRATEGIES[$data['strategy']],
            'alloc_used' => 0, 'alloc_total' => (float) $data['allocation'],
            'min_pct' => $this->bounds($data)[0], 'max_pct' => $this->bounds($data)[1],
            'grad' => self::GRADS[crc32($data['name']) % count(self::GRADS)],
        ];

        DB::transaction(function () use ($data, $tenant, $config) {
            // The bot's broker is stored on `exchange`; the engine routes its
            // orders to that broker (or the paper simulator when "paper").
            DB::table('bots')->insert([
                'tenant_id' => $tenant, 'name' => $data['name'], 'type' => $data['strategy'],
                'strategy' => $data['strategy'], 'exchange' => $data['broker'], 'status' => 'running',
                'config' => json_encode($config), 'created_at' => now(),
            ]);
            // Enable the strategy so the engine starts trading it for this tenant.
            DB::table('strategies')->updateOrInsert(
                ['tenant_id' => $tenant, 'name' => $data['strategy']],
                ['kind' => $data['strategy'], 'enabled' => true, 'updated_at' => now()]
            );
        });

        $where = $data['broker'] === 'paper' ? 'paper mode' : 'your '.ucfirst($data['broker']).' account';
        return back()->with('success', "Bot “{$data['name']}” created — trading on {$where}.");
    }

    public function update(Request $request, string $id)
    {
        $bot = rescue(fn () => DB::table('bots')->where('id', $id)->first(), null);
        if (! $bot) {
            return back()->with('error', 'Bot not found.');
        }
        $tenant = $request->user()->tenant_id;
        $connectedIds = rescue(fn () => DB::table('exchange_keys')->where('safe', true)
            ->whereNotNull('broker_id')->pluck('broker_id')->all(), []);
        $validBrokers = array_values(array_unique(array_merge(['paper'], $connectedIds)));

        $data = $request->validate([
            'name' => ['required', 'string', 'max:80'],
            'strategy' => ['required', Rule::in(array_keys(self::STRATEGIES))],
            'mode' => ['required', Rule::in(array_keys(self::MODES))],
            'symbol' => ['required_if:mode,single_symbol', Rule::in(self::SYMBOLS)],
            'watchlist' => ['required_if:mode,custom_watchlist', 'array'],
            'watchlist.*' => [Rule::in(self::SYMBOLS)],
            'allocation' => ['required', 'numeric', 'min:10', 'max:1000000'],
            'min_pct' => ['nullable', 'numeric', 'min:1', 'max:100'],
            'max_pct' => ['nullable', 'numeric', 'min:1', 'max:100'],
            'broker' => ['required', Rule::in($validBrokers)],
        ]);

        $watchlist = match ($data['mode']) {
            'single_symbol' => [$data['symbol']],
            'custom_watchlist' => array_values(array_unique($data['watchlist'])),
            default => self::SYMBOLS,
        };
        $repr = $watchlist[0] ?? 'BTCUSDT';
        $coin = preg_replace('/USDT$/', '', $repr);
        $old = $this->config($bot);
        $config = array_merge($old, [
            'mode' => $data['mode'], 'watchlist' => $watchlist, 'symbol' => $repr,
            'pair' => $coin . '/USDT', 'coin' => $coin,
            'strategy_label' => self::STRATEGIES[$data['strategy']],
            'alloc_total' => (float) $data['allocation'],
            'min_pct' => $this->bounds($data)[0], 'max_pct' => $this->bounds($data)[1],
        ]);

        DB::transaction(function () use ($id, $data, $tenant, $config, $bot) {
            DB::table('bots')->where('id', $id)->update([
                'name' => $data['name'], 'type' => $data['strategy'], 'strategy' => $data['strategy'],
                'exchange' => $data['broker'], 'config' => json_encode($config),
            ]);
            // Enable the (possibly new) strategy.
            DB::table('strategies')->updateOrInsert(
                ['tenant_id' => $tenant, 'name' => $data['strategy']],
                ['kind' => $data['strategy'], 'enabled' => true, 'updated_at' => now()]
            );
            // If the strategy changed, disable the old one when no other running bot uses it.
            if ($bot->strategy !== $data['strategy']) {
                $stillUsed = DB::table('bots')->where('strategy', $bot->strategy)
                    ->where('status', 'running')->where('id', '!=', $id)->exists();
                DB::table('strategies')->where('tenant_id', $tenant)->where('name', $bot->strategy)
                    ->update(['enabled' => $stillUsed, 'updated_at' => now()]);
            }
        });

        return back()->with('success', "Bot “{$data['name']}” updated.");
    }

    public function toggle(Request $request, string $id)
    {
        $bot = rescue(fn () => DB::table('bots')->where('id', $id)->first(), null);
        if (! $bot) {
            return back()->with('error', 'Bot not found.');
        }
        $next = $bot->status === 'running' ? 'paused' : 'running';
        $tenant = $request->user()->tenant_id;

        DB::transaction(function () use ($id, $next, $bot, $tenant) {
            DB::table('bots')->where('id', $id)->update(['status' => $next]);
            // Keep the strategy enabled only while at least one bot on it is running.
            $stillRunning = DB::table('bots')->where('strategy', $bot->strategy)
                ->where('status', 'running')->where('id', '!=', $id)->exists() || $next === 'running';
            DB::table('strategies')->where('tenant_id', $tenant)->where('name', $bot->strategy)
                ->update(['enabled' => $stillRunning, 'updated_at' => now()]);
        });

        return back()->with('success', "Bot {$next}.");
    }

    public function destroy(Request $request, string $id)
    {
        $bot = rescue(fn () => DB::table('bots')->where('id', $id)->first(), null);
        if ($bot) {
            DB::transaction(function () use ($id, $bot, $request) {
                DB::table('bots')->where('id', $id)->delete();
                $stillRunning = DB::table('bots')->where('strategy', $bot->strategy)->where('status', 'running')->exists();
                DB::table('strategies')->where('tenant_id', $request->user()->tenant_id)
                    ->where('name', $bot->strategy)->update(['enabled' => $stillRunning, 'updated_at' => now()]);
            });
        }
        return back()->with('success', 'Bot deleted.');
    }

    private function config($b): array
    {
        return is_string($b->config) ? (json_decode($b->config, true) ?: []) : (array) $b->config;
    }

    /** Clamp + order the per-trade budget bounds (% of the per-market allocation). */
    private function bounds(array $data): array
    {
        $min = max(1.0, min(100.0, (float) ($data['min_pct'] ?? 25)));
        $max = max($min, min(100.0, (float) ($data['max_pct'] ?? 100)));
        return [$min, $max];
    }
}
