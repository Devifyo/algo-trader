<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class PortfolioController extends Controller
{
    public function index()
    {
        $bots = rescue(fn () => DB::table('bots')->orderByDesc('created_at')->get()->toArray(), []);
        $allocations = rescue(fn () => DB::table('capital_allocations')
            ->orderByDesc('created_at')->limit(20)->get()->toArray(), []);
        $health = rescue(fn () => DB::table('strategy_health')
            ->orderByDesc('evaluated_at')->limit(20)->get()->toArray(), []);
        $decisions = rescue(fn () => DB::table('ai_decisions')
            ->orderByDesc('created_at')->limit(15)->get()->toArray(), []);

        return Inertia::render('Portfolio', compact('bots', 'allocations', 'health', 'decisions'));
    }
}
