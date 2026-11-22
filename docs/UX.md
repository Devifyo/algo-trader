# UX & safety principles

The hub is built for both beginners and experts: TradingView-grade clarity,
Stripe/Linear/Notion-level simplicity. Never present black-box behavior.

## Design system

- **Dark-first**, calm fintech palette via CSS variables (`resources/css/app.css`);
  light is a refined inversion. Theme persists in `localStorage`, applied before
  paint to avoid flashes.
- **Type:** display *Sora*, body *Manrope*, mono *JetBrains Mono* (deliberately
  not Inter/Roboto).
- **Mobile-first & responsive:** sidebar collapses to a drawer; grids reflow.
- **Accessible:** semantic landmarks, `aria-label`s, focus-visible states,
  `prefers-reduced-motion` honored, tabular numerals for figures.

## Every screen has

A title, a one-line description, a contextual help button, tooltips on jargon,
and explicit **empty / loading / error / success** states. Complex data is shown
through cards, risk badges, health-score rings, sentiment badges, and progress
indicators — not walls of numbers.

## The confusion audit

Before shipping a page we answer five questions in the UI itself:

1. **What is this feature?**
2. **Why does it matter?**
3. **What happens if I enable it?**
4. **What are the risks?**
5. **What's the recommended setting?**

If the answer isn't obvious from the screen, the screen is improved. Example: the
AI Providers and Exchange Keys pages lead with a plain-language "What / Why /
Recommended" panel; the exchange-key flow shows a permission breakdown, warnings,
and fix steps, and refuses withdrawal-enabled keys.

## Safety surfaces

- **Emergency kill switch** in the top bar (and `scripts/kill-switch.sh`) — sets
  the Redis flag the engine honors directly.
- **Risk badges & health scores** make risk legible at a glance.
- **Onboarding** walks new users create-account → risk-profile → connect-exchange
  → paper-trade → first-strategy → go-live, with progress tracking.
- **Help center** has plain-language articles for every major feature.

## Frontend tooling

Built with the installed `frontend-design` / web-design-guidelines skills as
reference. Components live in `dashboard/laravel/overlay/resources/js/Components`.
