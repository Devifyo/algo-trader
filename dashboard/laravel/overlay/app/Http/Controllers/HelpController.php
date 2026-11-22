<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Inertia\Inertia;

class HelpController extends Controller
{
    public function index(Request $request)
    {
        $q = trim((string) $request->query('q', ''));
        $articles = rescue(function () use ($q) {
            $query = DB::table('help_articles')->select('slug', 'title', 'category', 'audience', 'body');
            if ($q !== '') {
                $query->where(fn ($w) => $w->where('title', 'ilike', "%$q%")
                    ->orWhere('body', 'ilike', "%$q%"));
            }
            return $query->orderBy('category')->get()->toArray();
        }, []);

        return Inertia::render('Help', ['articles' => $articles, 'q' => $q]);
    }
}
