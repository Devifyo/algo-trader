<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

/**
 * Public, unauthenticated marketing + legal pages. Served at the site root and
 * standalone legal URLs so they're reachable by guests and crawlers. Authed
 * users hitting "/" are bounced to their app dashboard.
 */
class PublicController extends Controller
{
    public function home()
    {
        if (Auth::check()) {
            return redirect(Auth::user()->isAdmin() ? '/admin' : '/dashboard');
        }
        return Inertia::render('Public/Landing');
    }

    public function terms()
    {
        return Inertia::render('Public/Terms', ['updated' => 'June 25, 2026']);
    }

    public function privacy()
    {
        return Inertia::render('Public/Privacy', ['updated' => 'June 25, 2026']);
    }

    public function risk()
    {
        return Inertia::render('Public/RiskDisclosure', ['updated' => 'June 25, 2026']);
    }

    public function about()
    {
        return Inertia::render('Public/About');
    }

    public function contact()
    {
        return Inertia::render('Public/Contact');
    }

    public function contactSubmit(Request $request)
    {
        $data = $request->validate([
            'name' => ['required', 'string', 'max:120'],
            'email' => ['required', 'email', 'max:160'],
            'subject' => ['required', 'string', 'max:160'],
            'message' => ['required', 'string', 'max:4000'],
        ]);

        // Capture the message (owner connection bypasses tenant RLS for this public table).
        rescue(fn () => DB::connection('pgsql_admin')->table('contact_messages')->insert([
            'name' => $data['name'],
            'email' => $data['email'],
            'subject' => $data['subject'],
            'message' => $data['message'],
            'created_at' => now(),
        ]));

        return back()->with('success', 'Thanks — your message has been received. We’ll get back to you by email.');
    }
}
