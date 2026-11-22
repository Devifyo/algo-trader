<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class CopyTradingController extends Controller
{
    public function index()
    {
        $traders = rescue(fn () => DB::table('copy_traders')
            ->orderByDesc('verified')->orderByDesc('followers')
            ->limit(40)->get()->toArray(), []);
        $following = rescue(fn () => DB::table('copy_followers as f')
            ->join('copy_traders as t', 't.id', '=', 'f.trader_id')
            ->select('f.id', 't.display_name', 'f.copy_mode', 'f.amount_usd',
                'f.multiplier', 'f.max_drawdown_stop', 'f.status')
            ->orderByDesc('f.started_at')->get()->toArray(), []);

        return Inertia::render('CopyTrading', compact('traders', 'following'));
    }
}
