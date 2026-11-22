<?php

namespace App\Providers;

use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        // Default tenant for the bundled single-tenant paper stack.
        config(['app.default_tenant' => env('DEFAULT_TENANT', '00000000-0000-0000-0000-000000000001')]);

        // A second connection as the DB OWNER (RLS bypass) used only by the admin
        // area + signup (cross-tenant reads, tenant creation). Everything else
        // uses the restricted `pgsql` (RLS-enforced) connection.
        $base = config('database.connections.pgsql', []);
        config(['database.connections.pgsql_admin' => array_merge($base, [
            'username' => env('DB_OWNER_USERNAME', 'algotrader'),
            'password' => env('DB_OWNER_PASSWORD', 'algotrader'),
        ])]);
    }

    public function boot(): void
    {
        //
    }
}
