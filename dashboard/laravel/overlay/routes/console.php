<?php

use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Hash;

// Ensure a platform admin exists (idempotent). Run on container start.
Artisan::command('admin:ensure', function () {
    $email = env('ADMIN_EMAIL', 'admin@example.com');
    if (DB::table('users')->where('email', $email)->exists()) {
        $this->info("admin already exists: {$email}");
        return;
    }
    DB::table('users')->insert([
        'tenant_id' => config('app.default_tenant'),
        'name' => 'Platform Admin',
        'email' => $email,
        'password' => Hash::make(env('ADMIN_PASSWORD', 'admin12345!')),
        'role' => 'admin',
    ]);
    $this->info("admin created: {$email}");
})->purpose('Create the platform admin user if missing');
