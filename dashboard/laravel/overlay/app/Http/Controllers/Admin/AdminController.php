<?php

namespace App\Http\Controllers\Admin;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Redis;
use Inertia\Inertia;

/**
 * Platform admin. Uses the OWNER database connection (`pgsql_admin`) so it can
 * read across every tenant (RLS bypass) for global analytics + management.
 */
class AdminController extends Controller
{
    private function db()
    {
        return DB::connection('pgsql_admin');
    }

    public function overview()
    {
        $db = $this->db();
        return Inertia::render('Admin/Overview', [
            'stats' => [
                'tenants' => (int) rescue(fn () => $db->table('tenants')->count(), 0),
                'users' => (int) rescue(fn () => $db->table('users')->count(), 0),
                'bots' => (int) rescue(fn () => $db->table('bots')->count(), 0),
                'open_positions' => (int) rescue(fn () => $db->table('positions')->where('status', 'open')->count(), 0),
                'ai_cost_24h' => round((float) rescue(fn () => $db->table('ai_requests')
                    ->whereRaw("time >= now() - interval '24 hours'")->sum('cost_usd'), 0), 2),
                'kill_switch' => rescue(fn () => in_array((string) Redis::get('kill_switch'), ['1', 'on', 'true'], true), false),
            ],
            'providers' => rescue(fn () => $db->table('ai_providers')
                ->select('name', 'enabled', 'priority')->orderBy('priority')->get(), []),
            'recentAudit' => rescue(fn () => $db->table('audit_log')
                ->select('actor', 'action', 'target', 'at')->orderByDesc('at')->limit(10)->get(), []),
        ]);
    }

    public function tenants()
    {
        $db = $this->db();
        $tenants = rescue(fn () => $db->table('tenants as t')
            ->leftJoin('users as u', 'u.tenant_id', '=', 't.id')
            ->select('t.id', 't.name', 't.region', 't.regime', 't.plan', 't.status',
                't.owner_email', 't.created_at', DB::raw('count(u.id) as users'))
            ->groupBy('t.id', 't.name', 't.region', 't.regime', 't.plan', 't.status', 't.owner_email', 't.created_at')
            ->orderByDesc('t.created_at')->get(), []);

        return Inertia::render('Admin/Tenants', ['tenants' => $tenants]);
    }

    public function suspendTenant(Request $request, string $id)
    {
        $status = $request->input('status', 'suspended');
        rescue(fn () => $this->db()->table('tenants')->where('id', $id)
            ->update(['status' => $status, 'updated_at' => now()]));
        return back()->with('success', "Tenant {$status}.");
    }

    /**
     * Impersonate a tenant: sign in as that tenant's owner so the admin sees
     * exactly what the tenant sees (RLS scopes everything to that tenant). The
     * original admin id is stashed in the session so they can return.
     */
    public function impersonate(Request $request, string $tenantId)
    {
        $target = rescue(fn () => $this->db()->table('users')
            ->where('tenant_id', $tenantId)->where('role', '!=', 'admin')
            ->orderBy('id')->first(), null);

        if (! $target) {
            return back()->with('error', 'This tenant has no user account to impersonate.');
        }

        $request->session()->put('impersonator_id', Auth::id());
        Auth::loginUsingId($target->id);

        return redirect('/')->with('success', "You are now viewing as {$target->email}.");
    }

    public function featureFlags()
    {
        $flags = rescue(fn () => $this->db()->table('feature_flags')
            ->orderBy('flag')->get(), []);
        return Inertia::render('Admin/FeatureFlags', ['flags' => $flags]);
    }

    public function toggleFlag(Request $request, int $id)
    {
        rescue(fn () => $this->db()->table('feature_flags')->where('id', $id)
            ->update(['enabled' => $request->boolean('enabled')]));
        return back()->with('success', 'Feature flag updated.');
    }
}
