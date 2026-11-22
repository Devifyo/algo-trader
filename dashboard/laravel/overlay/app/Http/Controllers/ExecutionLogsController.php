<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class ExecutionLogsController extends Controller
{
    public function index(Request $request)
    {
        // All filters are server-side so they apply across the FULL history, not
        // just the current page (old logs stay findable, never "vanish").
        $f = [
            'tab' => $request->query('tab', 'all'),        // all | trades | errors  (quick preset)
            'status' => $request->query('status', 'all'),  // all | filled | submitted | rejected | canceled
            'side' => $request->query('side', 'all'),      // all | buy | sell
            'broker' => $request->query('broker', 'all'),  // all | alpaca | paper | ...
            'symbol' => $request->query('symbol', 'all'),  // all | BTCUSDT | ...
            'from' => trim((string) $request->query('from', '')), // YYYY-MM-DD
            'to' => trim((string) $request->query('to', '')),     // YYYY-MM-DD
            'q' => trim((string) $request->query('q', '')),       // free text (id / pair / strategy)
        ];
        $perPage = 50;

        // Build the filtered query once (RLS scopes to the current tenant).
        $apply = function ($qb) use ($f) {
            if ($f['tab'] === 'trades') {
                $qb->where('status', '!=', 'rejected');
            } elseif ($f['tab'] === 'errors') {
                $qb->where('status', 'rejected');
            }
            if ($f['status'] !== 'all') {
                $qb->where('status', $f['status']);
            }
            if ($f['side'] !== 'all') {
                $qb->where('side', strtolower($f['side']));
            }
            if ($f['broker'] !== 'all') {
                $qb->where('broker', $f['broker']);
            }
            if ($f['symbol'] !== 'all') {
                $qb->where('symbol', $f['symbol']);
            }
            if ($f['from'] !== '') {
                $qb->whereDate('created_at', '>=', $f['from']);
            }
            if ($f['to'] !== '') {
                $qb->whereDate('created_at', '<=', $f['to']);
            }
            if ($f['q'] !== '') {
                $like = '%' . str_replace('%', '\\%', $f['q']) . '%';
                $qb->where(function ($w) use ($like) {
                    $w->where('symbol', 'ilike', $like)
                        ->orWhere('strategy', 'ilike', $like)
                        ->orWhereRaw('id::text ilike ?', [$like]);
                });
            }
            return $qb;
        };

        $page = rescue(fn () => $apply(DB::table('orders'))
            ->select('id', 'client_id', 'exchange', 'broker', 'symbol', 'strategy', 'side', 'type',
                'qty', 'price', 'stop_price', 'status', 'exchange_oid', 'reason', 'created_at')
            ->orderByDesc('created_at')
            ->paginate($perPage)
            ->withQueryString(), null);

        $logs = collect($page ? $page->items() : [])->map(function ($o) {
            return [
                'time' => $o->created_at,
                'log_id' => 'LOG-' . substr(str_replace('-', '', (string) $o->id), 0, 6),
                'bot' => ucfirst($o->strategy ?? 'bot') . ' Bot',
                'kind' => $o->status === 'rejected' ? 'error' : 'trade',
                'pair' => $o->symbol,
                'side' => strtoupper((string) $o->side),
                'qty' => (float) $o->qty,
                'price' => (float) $o->price,
                'status' => $o->status,
                'exchange' => $o->exchange,
                'strategy' => $o->strategy,
                'type' => $o->type,
                'broker' => $o->broker ?: $o->exchange,
                'stop_price' => $o->stop_price !== null ? (float) $o->stop_price : null,
                'exchange_oid' => $o->exchange_oid,
                'reason' => $o->reason,
                'client_id' => $o->client_id,
            ];
        })->values();

        // Dropdown option lists (distinct, tenant-scoped).
        $symbols = rescue(fn () => DB::table('orders')->select('symbol')->distinct()->orderBy('symbol')->pluck('symbol')->all(), []);
        $brokers = rescue(fn () => DB::table('orders')->whereNotNull('broker')->select('broker')->distinct()->orderBy('broker')->pluck('broker')->all(), []);

        // KPIs aggregate over ALL the tenant's orders (full history). Volume counts
        // FILLED orders only — canceled/rejected never traded.
        $agg = rescue(fn () => DB::table('orders')->selectRaw(
            "count(*) AS total,
             count(*) FILTER (WHERE status = 'filled') AS filled,
             count(*) FILTER (WHERE status = 'rejected') AS failed,
             count(*) FILTER (WHERE status != 'rejected') AS trades,
             COALESCE(sum(qty * price) FILTER (WHERE status = 'filled'), 0) AS volume"
        )->first(), null);

        $total = (int) ($agg->total ?? 0);
        $filled = (int) ($agg->filled ?? 0);
        $failed = (int) ($agg->failed ?? 0);

        return Inertia::render('ExecutionLogs', [
            'logs' => $logs,
            'filters' => $f,
            'options' => [
                'symbols' => array_values($symbols),
                'brokers' => array_values($brokers),
                'statuses' => ['filled', 'submitted', 'rejected', 'canceled'],
                'sides' => ['buy', 'sell'],
            ],
            'pagination' => [
                'current_page' => $page ? $page->currentPage() : 1,
                'last_page' => $page ? $page->lastPage() : 1,
                'per_page' => $perPage,
                'total' => $page ? $page->total() : 0,
                'from' => $page ? $page->firstItem() : 0,
                'to' => $page ? $page->lastItem() : 0,
            ],
            'kpi' => [
                'total' => $total,
                'success_rate' => $total ? round($filled / $total * 100, 1) : 0,
                'failed' => $failed,
                'volume' => round((float) ($agg->volume ?? 0), 2),
            ],
            'counts' => [
                'all' => $total,
                'trades' => (int) ($agg->trades ?? 0),
                'errors' => $failed,
            ],
        ]);
    }
}
