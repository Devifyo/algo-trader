# ── Stage 1: PHP base (Laravel 12 via Composer) ───────────────────────────
FROM composer:2 AS php-base
WORKDIR /app
# Create the canonical Laravel 12 skeleton so all framework boilerplate is
# correct, then add Inertia server adapter + Ziggy.
RUN composer create-project laravel/laravel:^12.0 . --no-interaction --prefer-dist \
 && composer require inertiajs/inertia-laravel:^2.0 tightenco/ziggy:^2.0 predis/predis:^2.2 --no-interaction
# Overlay our authored domain code (controllers, models, routes, middleware,
# Inertia views, bootstrap wiring). Overwrites the scaffold defaults.
COPY overlay/app ./app
COPY overlay/routes ./routes
COPY overlay/bootstrap ./bootstrap
COPY overlay/resources/views ./resources/views
# Self-hosted static assets (broker logos) merged into Laravel's public/.
COPY overlay/public/ ./public/
RUN composer dump-autoload -o

# ── Stage 2: Frontend build (Vue 3 + Inertia + Tailwind via Vite) ─────────
FROM node:22-slim AS frontend
WORKDIR /app
COPY --from=php-base /app /app
COPY overlay/package.json overlay/vite.config.js overlay/tailwind.config.js overlay/postcss.config.js ./
COPY overlay/resources/js ./resources/js
COPY overlay/resources/css ./resources/css
RUN npm install && npm run build

# ── Stage 3: Runtime (php 8.4 + apache) ───────────────────────────────────
FROM php:8.4-apache AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
      libpq-dev libzip-dev unzip \
 && docker-php-ext-install pdo pdo_pgsql zip opcache \
 && a2enmod rewrite \
 && rm -rf /var/lib/apt/lists/*

# Serve Laravel's public/ directory.
ENV APACHE_DOCUMENT_ROOT=/var/www/html/public
RUN sed -ri -e 's!/var/www/html!${APACHE_DOCUMENT_ROOT}!g' /etc/apache2/sites-available/*.conf \
 && sed -ri -e 's!/var/www/!${APACHE_DOCUMENT_ROOT}!g' /etc/apache2/apache2.conf /etc/apache2/conf-available/*.conf

WORKDIR /var/www/html
COPY --from=frontend /app /var/www/html
RUN chown -R www-data:www-data storage bootstrap/cache \
 && cp -n .env.example .env || true

COPY overlay/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 80
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["apache2-foreground"]
