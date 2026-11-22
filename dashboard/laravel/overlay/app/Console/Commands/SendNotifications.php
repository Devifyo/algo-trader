<?php

namespace App\Console\Commands;

use App\Mail\NotificationMail;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Mail;

/**
 * Emails un-sent notifications (one row per engine alert) to the tenant's users,
 * then stamps emailed_at so each fires exactly once. Uses the owner connection
 * to read across tenants. Mail driver is 'log' until SMTP is configured.
 */
class SendNotifications extends Command
{
    protected $signature = 'notifications:send';
    protected $description = 'Email pending notifications to tenant users';

    public function handle(): int
    {
        $db = DB::connection('pgsql_admin');
        $pending = $db->table('notifications')->whereNull('emailed_at')
            ->orderBy('created_at')->limit(100)->get();
        if ($pending->isEmpty()) {
            return self::SUCCESS;
        }

        $emailsByTenant = [];
        foreach ($pending->pluck('tenant_id')->unique() as $tid) {
            $emailsByTenant[$tid] = $db->table('users')->where('tenant_id', $tid)
                ->whereNotNull('email')->pluck('email')->all();
        }

        foreach ($pending as $n) {
            foreach ($emailsByTenant[$n->tenant_id] ?? [] as $email) {
                try {
                    Mail::to($email)->send(new NotificationMail($n->title, $n->body, $n->severity));
                } catch (\Throwable $e) {
                    $this->error("mail failed for {$email}: {$e->getMessage()}");
                }
            }
            $db->table('notifications')->where('id', $n->id)->update(['emailed_at' => now()]);
        }
        $this->info('Sent '.$pending->count().' notification(s).');
        return self::SUCCESS;
    }
}
