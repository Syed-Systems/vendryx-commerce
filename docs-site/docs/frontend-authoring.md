# Frontend Theme Authoring Guide

> **Rule 0 — Checkout is core at every tier (ADR-013 Decision 2):**
> Checkout flow, steps, state machine, validation, and the payment script manifest live in
> `packages/storefront-core` and are **not** theme-editable. Themes skin checkout through
> CSS custom properties and theme tokens, but never rewrite or replace checkout components.
> Any merchant requirement that breaks this rule removes the tenant from shared compliance scope.

---

## 1. Architecture: The Unified Runtime (ADR-031)

Stheno utilizes a single, unified Next.js storefront runtime that serves all multi-tenant branded storefronts.
We **do not** scaffold separate `frontend-*` packages for different tenants. The battle-tested, mature `frontend-default` DOM foundation serves as the canonical chassis for the store.

- **`storefront-core`** owns: Routing, URL paths, SEO meta tags, data fetching (`src/lib/data/`), region resolution, cart/order state management, and the checkout funnel.
- **`frontend-default`** owns: Presentation components, layout templates, typography, color tokens, and animation.

All visual differentiation across tenants is achieved via:
1. Dynamic Runtime Design Token Injection (CSS custom properties)
2. Component Visual Variant Modifiers (`data-*` attributes)

---

## 2. Dynamic Runtime Theming

Visual variations must be structured through parameterized tokens and CSS custom properties rather than arbitrary freeform JSX forks.

### The Token Layer (`--brand-*`)
Next.js `middleware.ts` resolves the tenant identity from the `Host` header. Then, `storefront-core/src/app/layout.tsx` dynamically injects tenant-specific design tokens into `<style id="stheno-brand-tokens">:root { ... }</style>` on SSR.

Tokens govern:
- **Color System**: `--brand-primary`, `--brand-surface`, `--brand-background`, `--brand-text`, `--brand-hairline`
- **Typography**: `--brand-font-family`
- **Corner Radii**: `--brand-radius-card`, `--brand-radius-button`, `--brand-radius-input`
- **Materials**: frosted glass blur effects

### Visual Variant Modifiers (`data-*`)
To support structural layout adaptivity without mutating the DOM, mature components in `frontend-default` accept semantic data attributes:
- `data-card-style="apple-minimal"`
- `data-image-aspect="portrait"`
- `data-buy-box="sticky"`

This enables presentation variations (e.g., Apple sticky 2-column detail page vs inline, segmented option pills vs standard chips) without modifying core data pipelines.

---

## 3. Aesthetic Baseline: Apple HIG

The default aesthetic baseline strictly adopts Apple Human Interface Guidelines (HIG):
- 8pt spatial grid and generous whitespace
- High-contrast typography with tabular pricing numerals
- Smooth 16px squircle corner radii
- 9999px full-pill buttons
- Translucent frosted materials (`backdrop-filter`)
- Fluid spring transitions (`cubic-bezier(0.16, 1, 0.3, 1)`)

---

## 4. Real-Time Admin Preview

When merchants customize their theme in the Admin settings, changes are previewed live via `postMessage` (`STHENO_PREVIEW_TOKENS`). This provides instant feedback without build cycles or server redeployments, ensuring a zero-deploy, real-time styling experience.

---

## 5. Verification Checklist

Before opening a PR affecting frontend components or tokens:
1. `npm --prefix apps/storefront run typecheck`
2. `npm --prefix apps/storefront run test:lighthouse` (Ensures zero render-path round trips for tokens)
3. `./scripts/load-suite.sh --multi-tenant` (Ensures 100% tenant isolation and sub-50ms TTFB)
