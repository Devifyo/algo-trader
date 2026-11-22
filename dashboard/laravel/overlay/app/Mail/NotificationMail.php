<?php

namespace App\Mail;

use Illuminate\Bus\Queueable;
use Illuminate\Mail\Mailable;
use Illuminate\Queue\SerializesModels;

class NotificationMail extends Mailable
{
    use Queueable, SerializesModels;

    public function __construct(
        public string $title,
        public string $body,
        public string $severity = 'info',
    ) {}

    public function build()
    {
        return $this->subject('['.config('app.name', 'AI Trading Platform').'] '.$this->title)
            ->view('emails.notification');
    }
}
