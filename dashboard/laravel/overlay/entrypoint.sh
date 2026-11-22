#!/bin/sh
set -e
cd /var/www/html

# Ensure an app key exists (only generate if not supplied via env).
if ! grep -q '^APP_KEY=base64' .env 2>/dev/null && [ -z "${APP_KEY}" ]; then
    php artisan key:generate --force || true
fi

# Don't cache config: we rely on runtime env from docker-compose.
php artisan config:clear || true
php artisan route:cache || true

# Storage symlink + permissions.
php artisan storage:link 2>/dev/null || true
chown -R www-data:www-data storage bootstrap/cache || true

# Ensure the platform admin user exists (waits briefly for the DB).
for i in 1 2 3 4 5 6 7 8 9 10; do
    if php artisan admin:ensure 2>/dev/null; then break; fi
    echo "waiting for database… ($i)"; sleep 3
done

# Background notification email sender (best-effort). Mail driver defaults to
# 'log' (writes to storage/logs) until SMTP MAIL_* env is configured.
( while true; do php artisan notifications:send >/dev/null 2>&1 || true; sleep 60; done ) &

exec "$@"
