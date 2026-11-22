<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Redis;

/**
 * Emergency controls, honored directly by the Rust risk + execution services on
 * their next loop (independent of this dashboard staying up). Admin toggles the
 * GLOBAL kill switch; a tenant user halts only their own tenant.
 */
class KillSwitchController extends Controller
{
    public function toggle(Request $request)
    {
        $on = $request->boolean('on');
        $user = $request->user();
        $tenant = $user->tenant_id ?? config('app.default_tenant');

        if ($user && $user->isAdmin()) {
            rescue(fn () => Redis::set('kill_switch', $on ? '1' : '0'));
            $action = $on ? 'kill_switch_on' : 'kill_switch_off';
        } else {
            // Per-tenant halt (engine reads halt:{tenant}).
            rescue(fn () => $on ? Redis::set("halt:{$tenant}", '1') : Redis::del("halt:{$tenant}"));
            $action = $on ? 'tenant_halt_on' : 'tenant_halt_off';
        }

        rescue(fn () => DB::table('audit_log')->insert([
            'tenant_id' => $tenant,
            'actor' => $user->email ?? 'admin',
            'action' => $action,
            'target' => $user && $user->isAdmin() ? 'global' : 'tenant',
        ]));

        return back()->with($on ? 'error' : 'success',
            $on ? 'Trading halted. New entries are blocked.' : 'Halt cleared — trading may resume.');
    }
}
