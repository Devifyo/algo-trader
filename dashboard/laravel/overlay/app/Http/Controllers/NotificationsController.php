<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class NotificationsController extends Controller
{
    public function index()
    {
        $rows = rescue(fn () => DB::table('notifications')->orderByDesc('created_at')->limit(60)->get(), collect());
        // Opening the page marks everything read.
        rescue(fn () => DB::table('notifications')->whereNull('read_at')->update(['read_at' => now()]));

        return Inertia::render('Notifications', [
            'notifications' => $rows->map(fn ($n) => [
                'id' => $n->id,
                'kind' => $n->kind,
                'title' => $n->title,
                'body' => $n->body,
                'severity' => $n->severity,
                'read' => $n->read_at !== null,
                'created_at' => $n->created_at,
            ])->values(),
        ]);
    }
}
