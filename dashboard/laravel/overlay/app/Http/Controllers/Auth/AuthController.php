<?php

namespace App\Http\Controllers\Auth;

use App\Http\Controllers\Controller;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Str;
use Inertia\Inertia;

class AuthController extends Controller
{
    public function showLogin()
    {
        return Inertia::render('Auth/Login');
    }

    public function login(Request $request)
    {
        $data = $request->validate([
            'email' => ['required', 'email'],
            'password' => ['required'],
        ]);

        if (! Auth::attempt($data, $request->boolean('remember'))) {
            return back()->with('error', 'Those credentials do not match our records.');
        }
        $request->session()->regenerate();

        return redirect()->intended(Auth::user()->isAdmin() ? '/admin' : '/dashboard');
    }

    public function showRegister()
    {
        return Inertia::render('Auth/Register');
    }

    public function register(Request $request)
    {
        $data = $request->validate([
            'name' => ['required', 'string', 'max:120'],
            'org' => ['required', 'string', 'max:120'],
            'email' => ['required', 'email', 'unique:users,email'],
            'password' => ['required', 'string', 'min:8', 'confirmed'],
            'risk_profile' => ['nullable', 'in:conservative,balanced,aggressive'],
        ]);

        // Tenant creation + the first user bypass RLS via the owner connection.
        $admin = DB::connection('pgsql_admin');
        $tenantId = (string) Str::uuid();
        $admin->table('tenants')->insert([
            'id' => $tenantId,
            'name' => $data['org'],
            'region' => env('REGION', 'us-east-1'),
            'regime' => env('REGION', 'us-east-1') === 'ap-south-1' ? 'sebi' : 'finra',
            'plan' => 'starter',
            'owner_email' => $data['email'],
        ]);
        $userId = $admin->table('users')->insertGetId([
            'tenant_id' => $tenantId,
            'name' => $data['name'],
            'email' => $data['email'],
            'password' => Hash::make($data['password']),
            'role' => 'owner',
            'risk_profile' => $data['risk_profile'] ?? 'balanced',
        ]);

        Auth::loginUsingId($userId);
        $request->session()->regenerate();

        return redirect('/onboarding')->with('success', 'Welcome — your workspace is ready.');
    }

    public function stopImpersonating(Request $request)
    {
        $originalId = $request->session()->pull('impersonator_id');
        if ($originalId) {
            Auth::loginUsingId($originalId);
            return redirect('/admin')->with('success', 'Returned to your admin account.');
        }
        return redirect('/dashboard');
    }

    public function logout(Request $request)
    {
        Auth::logout();
        $request->session()->invalidate();
        $request->session()->regenerateToken();
        return redirect('/login');
    }
}
