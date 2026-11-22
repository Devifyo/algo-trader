<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class MarketplaceController extends Controller
{
    public function index()
    {
        $strategies = rescue(fn () => DB::table('marketplace_strategies')
            ->where('status', 'published')
            ->orderByDesc('rating')->orderByDesc('subscribers')
            ->limit(40)->get()->toArray(), []);

        return Inertia::render('Marketplace', ['strategies' => $strategies]);
    }
}
