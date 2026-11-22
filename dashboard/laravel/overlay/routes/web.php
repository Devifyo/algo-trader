<?php

use App\Http\Controllers\Admin\AdminController;
use App\Http\Controllers\Admin\StrategyController;
use App\Http\Controllers\AiDecisionsController;
use App\Http\Controllers\AiProviderController;
use App\Http\Controllers\Auth\AuthController;
use App\Http\Controllers\BotsController;
use App\Http\Controllers\BrokerController;
use App\Http\Controllers\CopyTradingController;
use App\Http\Controllers\ExecutionLogsController;
use App\Http\Controllers\DashboardController;
use App\Http\Controllers\ExchangeKeyController;
use App\Http\Controllers\HelpController;
use App\Http\Controllers\KillSwitchController;
use App\Http\Controllers\MarketplaceController;
use App\Http\Controllers\MarketsController;
use App\Http\Controllers\NewsController;
use App\Http\Controllers\NotificationsController;
use App\Http\Controllers\OnboardingController;
use App\Http\Controllers\OpportunityController;
use App\Http\Controllers\PortfolioController;
use App\Http\Controllers\PublicController;
use Illuminate\Support\Facades\Route;

// ── Public marketing + legal (no auth; reachable by everyone) ─────────────
Route::get('/', [PublicController::class, 'home'])->name('home');
Route::get('/terms', [PublicController::class, 'terms'])->name('terms');
Route::get('/privacy', [PublicController::class, 'privacy'])->name('privacy');
Route::get('/risk-disclosure', [PublicController::class, 'risk'])->name('risk');
Route::get('/about', [PublicController::class, 'about'])->name('about');
Route::get('/contact', [PublicController::class, 'contact'])->name('contact');
Route::post('/contact', [PublicController::class, 'contactSubmit'])->middleware('throttle:5,1')->name('contact.submit');

// ── Guest (SaaS sign-up + login) ─────────────────────────────────────────
Route::middleware('guest')->group(function () {
    Route::get('/login', [AuthController::class, 'showLogin'])->name('login');
    Route::post('/login', [AuthController::class, 'login'])->middleware('throttle:6,1');
    Route::get('/register', [AuthController::class, 'showRegister'])->name('register');
    Route::post('/register', [AuthController::class, 'register'])->middleware('throttle:4,1');
});
Route::post('/logout', [AuthController::class, 'logout'])->middleware('auth')->name('logout');

// ── Tenant user workspace (authenticated) ────────────────────────────────
Route::middleware('auth')->group(function () {
    Route::get('/dashboard', [DashboardController::class, 'index'])->name('dashboard');
    Route::get('/portfolio', [PortfolioController::class, 'index'])->name('portfolio');
    Route::get('/markets', [MarketsController::class, 'index'])->name('markets');
    Route::get('/opportunities', [OpportunityController::class, 'index'])->name('opportunities');
    Route::get('/opportunities/scan', [OpportunityController::class, 'scan'])->name('opportunities.scan');
    Route::get('/notifications', [NotificationsController::class, 'index'])->name('notifications');
    Route::get('/news/feed', [NewsController::class, 'feed'])->name('news.feed');
    Route::get('/ai-decisions', [AiDecisionsController::class, 'index'])->name('ai-decisions');
    Route::get('/bots', [BotsController::class, 'index'])->name('bots');
    Route::post('/bots', [BotsController::class, 'store'])->name('bots.store');
    Route::patch('/bots/{id}', [BotsController::class, 'update'])->name('bots.update');
    Route::post('/bots/{id}/toggle', [BotsController::class, 'toggle'])->name('bots.toggle');
    Route::delete('/bots/{id}', [BotsController::class, 'destroy'])->name('bots.destroy');
    Route::get('/logs', [ExecutionLogsController::class, 'index'])->name('logs');
    Route::get('/news', [NewsController::class, 'index'])->name('news');
    Route::get('/marketplace', [MarketplaceController::class, 'index'])->name('marketplace');
    Route::get('/copy-trading', [CopyTradingController::class, 'index'])->name('copy');
    Route::get('/help', [HelpController::class, 'index'])->name('help');
    Route::get('/onboarding', [OnboardingController::class, 'index'])->name('onboarding');
    Route::post('/onboarding/complete', [OnboardingController::class, 'complete'])->name('onboarding.complete');

    // Broker catalog + connections (replaces the old crypto-only exchange keys).
    Route::get('/brokers', [BrokerController::class, 'index'])->name('brokers');
    Route::post('/brokers/{brokerId}/connect', [BrokerController::class, 'connect'])->name('brokers.connect');
    Route::post('/broker-connections/{id}/disconnect', [BrokerController::class, 'disconnect'])->name('brokers.disconnect');

    Route::get('/exchange-keys', [ExchangeKeyController::class, 'index'])->name('exchange.keys');
    Route::post('/exchange-keys/test', [ExchangeKeyController::class, 'test'])->name('exchange.keys.test');
    Route::post('/exchange-keys', [ExchangeKeyController::class, 'store'])->name('exchange.keys.store');

    Route::post('/kill-switch', [KillSwitchController::class, 'toggle'])->name('kill.switch');

    // Available while impersonating (the session, not the admin role, authorizes it).
    Route::post('/impersonate/stop', [AuthController::class, 'stopImpersonating'])->name('impersonate.stop');
});

// ── Platform admin (role=admin) ──────────────────────────────────────────
Route::middleware(['auth', 'admin'])->prefix('admin')->name('admin.')->group(function () {
    Route::get('/', [AdminController::class, 'overview'])->name('overview');
    Route::get('/tenants', [AdminController::class, 'tenants'])->name('tenants');
    Route::get('/strategies', [StrategyController::class, 'index'])->name('strategies');
    Route::post('/strategies/research', [StrategyController::class, 'rerunResearch'])->name('strategies.research');
    Route::post('/strategies/default', [StrategyController::class, 'setDefault'])->name('strategies.default');
    Route::post('/strategies/backtest', [StrategyController::class, 'runBacktest'])->name('strategies.backtest');
    Route::post('/strategies/nautilus-backtest', [StrategyController::class, 'nautilusBacktest'])->name('strategies.nautilus');
    Route::post('/tenants/{id}/status', [AdminController::class, 'suspendTenant'])->name('tenants.status');
    Route::post('/tenants/{id}/impersonate', [AdminController::class, 'impersonate'])->name('tenants.impersonate');
    Route::get('/feature-flags', [AdminController::class, 'featureFlags'])->name('flags');
    Route::post('/feature-flags/{id}/toggle', [AdminController::class, 'toggleFlag'])->name('flags.toggle');

    Route::get('/ai-providers', [AiProviderController::class, 'index'])->name('ai.providers');
    Route::post('/ai-providers/{id}/toggle', [AiProviderController::class, 'toggleProvider'])->name('ai.provider.toggle');
    Route::post('/ai-providers/{id}/priority', [AiProviderController::class, 'setPriority'])->name('ai.provider.priority');
    Route::post('/ai-providers/settings', [AiProviderController::class, 'saveSettings'])->name('ai.settings.save');
    Route::post('/ai-providers/{id}/keys', [AiProviderController::class, 'addKey'])->name('ai.key.add');
    Route::post('/ai-providers/keys/{id}/toggle', [AiProviderController::class, 'toggleKey'])->name('ai.key.toggle');
    Route::post('/ai-providers/keys/{id}/delete', [AiProviderController::class, 'deleteKey'])->name('ai.key.delete');
    Route::get('/ai-providers/keys/{id}/requests', [AiProviderController::class, 'keyRequests'])->name('ai.key.requests');
});
