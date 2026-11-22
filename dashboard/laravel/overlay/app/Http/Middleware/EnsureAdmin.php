<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;

/** Gate the platform admin area. Only role=admin may pass. */
class EnsureAdmin
{
    public function handle(Request $request, Closure $next)
    {
        if (! $request->user() || ! $request->user()->isAdmin()) {
            abort(403, 'Admins only.');
        }
        return $next($request);
    }
}
