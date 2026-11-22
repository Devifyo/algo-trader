<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class OnboardingController extends Controller
{
    private array $steps = [
        ['key' => 'create_account', 'title' => 'Create your account', 'desc' => 'You are here. Welcome.'],
        ['key' => 'choose_risk_profile', 'title' => 'Choose a risk profile', 'desc' => 'Conservative, balanced, or aggressive — you can change this anytime.'],
        ['key' => 'connect_exchange', 'title' => 'Connect an exchange', 'desc' => 'Add a trade-only API key. We refuse keys that can withdraw funds.'],
        ['key' => 'start_paper_trading', 'title' => 'Start paper trading', 'desc' => 'Run risk-free with a virtual balance and real market data.'],
        ['key' => 'run_first_strategy', 'title' => 'Run your first strategy', 'desc' => 'Pick a strategy and let a bot trade it on paper.'],
        ['key' => 'enable_live_trading', 'title' => 'Go live (when ready)', 'desc' => 'Only after 30 days / 100+ paper trades that meet the quality bar.'],
    ];

    public function index()
    {
        $done = rescue(fn () => DB::table('onboarding_progress')
            ->where('completed', true)->pluck('step')->toArray(), []);

        $steps = array_map(fn ($s) => $s + ['completed' => in_array($s['key'], $done, true)], $this->steps);

        return Inertia::render('Onboarding', [
            'steps' => $steps,
            'progress' => count($done),
            'total' => count($this->steps),
        ]);
    }

    public function complete(Request $request)
    {
        $step = $request->input('step');
        rescue(fn () => DB::table('onboarding_progress')->updateOrInsert(
            ['tenant_id' => config('app.default_tenant'), 'user_email' => optional($request->user())->email, 'step' => $step],
            ['completed' => true, 'completed_at' => now()]
        ));
        return back()->with('success', 'Step completed.');
    }
}
