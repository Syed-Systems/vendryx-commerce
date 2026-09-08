# Frontend Theme Authoring Guide

> **Rule 0 — Checkout is core at every tier (ADR-013 Decision 2):**
> Checkout flow, steps, state machine, validation, and the payment script manifest live in
> `packages/storefront-core` and are **not** theme-editable. Themes skin checkout through
> CSS custom properties and theme tokens, but never rewrite or replace checkout components.
> Any merchant requirement that breaks this rule removes the tenant from shared compliance scope.

---

## 1. Architecture: The Core / Theme Split

Stheno separates presentation from commerce logic per [ADR-013](file:///D:/Syed%20Systems/Store/docs/DECISIONS.md#adr-013):
- **`storefront-core`** owns: Routing, URL paths, SEO meta tags, data fetching (`src/lib/data/`), region resolution, cart/order state management, and the checkout funnel.
- **`frontend-*`** owns: Presentation components, layout templates, typography, color tokens, and animation.

A frontend **presents data passed to it as props**. It never fetches data directly, never invokes server actions, and never reaches into the database or SDK.

---

## 2. The 45-Slot Frontend Contract

Every frontend must satisfy [`FrontendContract`](file:///D:/Syed%20Systems/Store/apps/storefront/packages/storefront-core/src/frontend/contract.ts).
Compile-time conformance is strictly enforced at build and typecheck time via `conformance.ts`:

### Layout & Navigation
- `Nav`: Primary store header and navigation bar.
- `Footer`: Multi-column site footer with category/collection links.
- `Hero`: Homepage promotional hero section.
- `BrandFeatures`: Three-pillar editorial value proposition section.
- `NewsletterSection`: Email subscription input block.
- `LookbookStory`: Brand story or editorial feature.
- `CartMismatchBanner`: Currency/region mismatch alert.

### Catalog & Product
- `StoreTemplate`: Product listing page (PLP) grid with sorting and filters.
- `CategoryTemplate`: Category-specific product listing view.
- `CategoryGrid`: Category card grid for homepage or hub views.
- `CollectionTemplate`: Collection-specific product listing view.
- `ProductTemplate`: Product detail page (PDP) with gallery, description, and options.

### Cart & Pricing
- `CartTemplate`: Full cart page view.
- `ItemsPreviewTemplate`: Cart drawer and preview item list.
- `CartTotals`: Subtotal, shipping, tax, and order total summary.
- `DiscountCode`: Promotion / coupon input form.
- `FreeShippingPriceNudge`: Progress bar toward free shipping threshold.

### Common Primitives & Form Controls
- `Input`: Semantic text input with label and error state.
- `Checkbox`: Square toggle checkbox.
- `RadioIndicator`: Radio selection indicator for shipping/payment options.
- `CountrySelect`: Localized region/country picker dropdown.
- `SubmitButton`: Primary interactive action button with pending state.
- `InteractiveLink`: Animated navigation link.
- `Divider`: Visual separation rule.
- `ErrorMessage`: Accessible error callout.
- `ChevronDown`: Collapsible section icon.
- `Spinner`: Loading indicator.

### Customer Account & Orders
- `AccountLayout`: Inset container for logged-in customer account views.
- `Overview`: Customer account dashboard overview.
- `ProfileName`, `ProfileEmail`, `ProfilePhone`, `ProfilePassword`, `ProfileBillingAddress`: Account attribute editors.
- `AddressBook`: Shipping and billing address manager.
- `OrderOverview`: Order history listing.
- `OrderDetailsTemplate`: Deep order review view.
- `OrderCompletedTemplate`: Post-checkout order confirmed receipt.
- `TransferActions`, `TransferImage`, `TransferRequestForm`: Order ownership transfer flows.
- `LoginTemplate`: Authentication / login / registration view.

### Skeletons (Loading States)
- `SkeletonCardDetails`: Payment method skeleton.
- `SkeletonCartPage`: Full cart loading skeleton.
- `SkeletonOrderConfirmed`: Receipt loading skeleton.

---

## 3. ADAPT vs SKIN

Per [AGENTS.md §12](file:///D:/Syed%20Systems/Store/AGENTS.md#12-frontends-adapting-a-design):

| Paradigm | Instruction | Scope | Verification |
|---|---|---|---|
| **ADAPT** | `ADAPT <design> into packages/frontend-<name>` | **New DOM, new components, independent styling.** Never inherit `frontend-default` markup. Every slot marked `ported` or `derived`. | `./scripts/check-frontend-not-forked.sh` asserts 0 forked files and 0 `@medusajs/ui` imports. |
| **SKIN** | `SKIN frontend-default to <palette>` | **Tokens only.** `tokens.css`, typography, spacing, radii. Zero JSX edits. | DOM and test IDs remain 100% identical. |

---

## 4. The Token Layer (`tokens.css`)

Every frontend must export `src/modules/styles/tokens.css`. This file is loaded after `globals.css` in RootLayout:

```css
:root {
  --bg-primary: #050505;
  --fg-primary: #f5f5f5;
  --accent-gold: #c5a880;
  --border-subtle: 1px solid rgba(255, 255, 255, 0.1);
  --font-mono: 'Space Mono', monospace;
  --font-heading: 'Plus Jakarta Sans', sans-serif;
}
```

This token layer styles both the frontend's own templates and the core-owned checkout components.

---

## 5. Time-to-New-Theme Benchmark

Historical adaptation velocity measured across the Stheno platform:

| Theme Package | Type | File Count | Effort (Clock) | Notes |
|---|---|---|---|---|
| `frontend-matrix` (Initial) | `SKIN` | 127 files | 3.5 hours | Palette, tokens, and identity rewrite. |
| `frontend-lustre` | `ADAPT` | 125 files | 8.0 hours | 100% independent cold-luxury watch DOM; 0 forked files; 0 `@medusajs/ui`. |
| Incremental contract port | Extension | 1–3 slots | < 30 mins | Added to existing independent theme. |

---

## 6. Verification Checklist

Before opening a PR for a new frontend:
1. `npm --prefix apps/storefront run typecheck` (and per-theme tsconfig)
2. `npm --prefix apps/storefront run test:conformance -- --frontend=<name>`
3. `./scripts/check-storefront-boundaries.sh`
4. `./scripts/check-frontend-purity.sh`
5. `./scripts/check-frontend-pin.sh`
6. `./scripts/check-frontend-resolution.sh`
7. `./scripts/check-frontend-switch.sh`
8. `./scripts/check-frontend-not-forked.sh` (if `stheno.independent: true`)
