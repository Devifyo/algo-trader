<!DOCTYPE html>
<html>
<body style="margin:0;background:#0a0a0f;font-family:Arial,Helvetica,sans-serif;color:#e8e8ee;padding:24px">
    <div style="max-width:520px;margin:0 auto;background:#15151f;border:1px solid #26263a;border-radius:14px;overflow:hidden">
        <div style="height:4px;background:linear-gradient(90deg,#6E56F8,#9b8cff)"></div>
        <div style="padding:28px">
            <p style="margin:0 0 4px;font-size:12px;letter-spacing:1px;text-transform:uppercase;color:#8a8a9a">
                {{ ucfirst($severity) }} · AI Trading Platform
            </p>
            <h1 style="margin:0 0 14px;font-size:20px;color:#fff">{{ $title }}</h1>
            <p style="margin:0 0 22px;font-size:14px;line-height:1.6;color:#c7c7d2">{{ $body }}</p>
            <a href="{{ rtrim(config('app.url'), '/') }}/bots"
               style="display:inline-block;background:#6E56F8;color:#fff;text-decoration:none;padding:11px 20px;border-radius:10px;font-size:14px;font-weight:bold">
                Review your bots
            </a>
            <p style="margin:22px 0 0;font-size:11px;color:#5a5a6a">
                You're receiving this because you have an active account. Manage alerts from your dashboard.
            </p>
        </div>
    </div>
</body>
</html>
