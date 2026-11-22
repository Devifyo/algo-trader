<?php

use App\Http\Middleware\HandleInertiaRequests;
use App\Http\Middleware\SetTenantContext;
use Illuminate\Foundation\Application;
use Illuminate\Foundation\Configuration\Exceptions;
use Illuminate\Foundation\Configuration\Middleware;

return Application::configure(basePath: dirname(__DIR__))
    ->withRouting(
        web: __DIR__ . '/../routes/web.php',
        commands: __DIR__ . '/../routes/console.php',
        health: '/up',
    )
    ->withMiddleware(function (Middleware $middleware) {
        // Trust the reverse proxy you'll add later (for the domain + HTTPS).
        // Harmless when reaching the app directly by IP:port.
        $middleware->trustProxies(at: '*');
        // Inertia shared props + per-request tenant context for Postgres RLS.
        $middleware->web(append: [
            HandleInertiaRequests::class,
            SetTenantContext::class,
        ]);
        $middleware->alias([
            'admin' => \App\Http\Middleware\EnsureAdmin::class,
        ]);
        // Send unauthenticated users to the login page.
        $middleware->redirectGuestsTo(fn () => '/login');
    })
    ->withExceptions(function (Exceptions $exceptions) {
        //
    })->create();
