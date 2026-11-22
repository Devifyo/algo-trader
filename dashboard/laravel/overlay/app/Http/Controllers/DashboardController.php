<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Redis;
use Inertia\Inertia;

class DashboardController extends Controller
{
    public function index()
    {
        return Inertia::render('Dashboard', [
            'widgets' => [
                'portfolio' => $this->portfolioSummary(),
                'bots' => $this->activeBots(),
                'market' => $this->marketConditions(),
                'aiConfidence' => $this->aiConfidence(),
                'dailyPnl' => $this->dailyPnl(),
                'riskScore' => $this->riskScore(),
                'killSwitch' => $this->killSwitchEngaged(),
                'notifications' => $this->notifications(),
                'positions' => $this->positions(),
                'allocation' => $this->allocation(),
            ],
        ]);
    }

    private function positions(): array
    {
        return rescue(fn () => DB::table('positions')->where('status', 'open')
            ->selectRaw('symbol, side, qty, entry_price, leverage, strategy, (qty*entry_price) as notional')
            ->orderByDesc(DB::raw('qty*entry_price'))->limit(12)->get()->toArray(), []);
    }

    private function allocation(): array
    {
        return rescue(fn () => DB::table('positions')->where('status', 'open')
            ->selectRaw('symbol as label, sum(qty*entry_price) as value')
            ->groupBy('symbol')->orderByDesc(DB::raw('sum(qty*entry_price)'))
            ->limit(8)->get()->toArray(), []);
    }

    private function portfolioSummary(): array
    {
        $row = rescue(fn () => DB::table('pnl_snapshots')->orderByDesc('time')->first(), null);
        $open = rescue(fn () => DB::table('positions')->where('status', 'open')->count(), 0);
        return [
            'equity_usd' => $row->equity_usd ?? 0,
            'unrealized_usd' => $row->unrealized_usd ?? 0,
            'realized_usd' => $row->realized_usd ?? 0,
            'open_positions' => $open,
        ];
    }

    private function activeBots(): array
    {
        return rescue(fn () => DB::table('bots')
            ->whereIn('status', ['paper', 'forward', 'live'])
            ->select('id', 'name', 'type', 'status', 'exchange')
            ->limit(8)->get()->toArray(), []);
    }

    private function marketConditions(): array
    {
        // Latest per-asset news sentiment as a quick market-mood read.
        $sent = rescue(fn () => DB::table('sentiment_history')
            ->select('asset', 'sentiment_score', 'confidence')
            ->orderByDesc('time')->limit(5)->get()->toArray(), []);
        return ['sentiment' => $sent];
    }

    private function aiConfidence(): float
    {
        return (float) rescue(fn () => DB::table('ai_decisions')
            ->orderByDesc('created_at')->value('confidence'), 0);
    }

    private function dailyPnl(): float
    {
        return (float) rescue(fn () => DB::table('positions')
            ->where('status', 'closed')
            ->whereRaw("closed_at >= date_trunc('day', now())")
            ->sum('realized_pnl'), 0);
    }

    private function riskScore(): float
    {
        return (float) rescue(fn () => DB::table('ai_decisions')
            ->orderByDesc('created_at')->value('risk_score'), 0);
    }

    private function killSwitchEngaged(): bool
    {
        return rescue(fn () => in_array((string) Redis::get('kill_switch'), ['1', 'on', 'true'], true), false);
    }

    private function notifications(): array
    {
        return rescue(fn () => DB::table('market_events')
            ->select('event_type', 'severity', 'asset', 'headline', 'detected_at')
            ->orderByDesc('detected_at')->limit(6)->get()->toArray(), []);
    }
}
