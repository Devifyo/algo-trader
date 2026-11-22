<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;

/**
 * Sets the request-scoped tenant id on the Postgres session so Row-Level
 * Security filters every query to the current tenant. The hub connects as the
 * restricted `algotrader_app` role (RLS enforced); the engine connects as the
 * owner (RLS bypass). See infrastructure/docker/postgres/init/03_multitenancy.sql.
 */
class SetTenantContext
{
    public function handle(Request $request, Closure $next)
    {
        // Resolve tenant from the authenticated user; default tenant zero for
        // the bundled single-tenant paper stack.
        $tenantId = optional($request->user())->tenant_id
            ?? config('app.default_tenant', '00000000-0000-0000-0000-000000000001');

        // set_config() is the bind-parameter-safe way to set a GUC (plain SET
        // does not accept placeholders). false = session scope.
        DB::statement("select set_config('app.tenant_id', ?, false)", [$tenantId]);

        return $next($request);
    }
}
