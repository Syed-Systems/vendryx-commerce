# Vendryx Unified Storefront

> High-performance, multi-tenant e-commerce storefront for Vendryx Commerce, built on Next.js 15, React 19, Tailwind CSS, and TypeScript. Governed by [ADR-031](../../docs/DECISIONS.md#adr-031) and [AGENTS.md §12](../../AGENTS.md#12-frontends-visual-theming).

---

## 1. Overview & Architecture

Vendryx utilizes a **single, unified Next.js storefront runtime** to serve all multi-tenant merchant stores. Rather than spinning up or deploying separate container fleets or forking JSX packages for each brand theme, all visual differentiation is handled dynamically through **runtime design token injection** and **config-driven visual variant modifiers**.

### Key Architectural Pillars

1. **Unified Multi-Tenant Engine ([ADR-031](../../docs/DECISIONS.md#adr-031)):**
   One Next.js instance serves thousands of distinct branded merchant storefronts. Tenant identity is resolved from the HTTP `Host` header in `<1ms` in `middleware.ts`.
2. **Canonical Chassis (`frontend-default`):**
   The accessible, battle-tested DOM foundation handles responsive edge cases, mobile drawer navigation, multi-currency pricing, line item operations, and the checkout seam. We never fork JSX for styling.
3. **Dynamic SSR Design Token Injection:**
   On server-side rendering, `src/app/layout.tsx` injects tenant-specific design tokens into `<style id="stheno-brand-tokens">:root { ... }</style>`:
   - Colors: `--brand-primary`, `--brand-surface`, `--brand-background`, `--brand-text`, `--brand-hairline`
   - Typography: `--brand-font-family` (e.g., Plus Jakarta Sans, Inter, Geist)
   - Shapes: `--brand-radius-card`, `--brand-radius-button`, `--brand-radius-input`
   - Materials: Frosted glass backdrops (`backdrop-blur-md`)
4. **Config-Driven Visual Variant Modifiers:**
   Semantic data attributes allow layout shifts without mutating underlying logic:
   - `data-card-style="apple-minimal"` vs classic product card
   - `data-image-aspect="square"` (1:1) vs `data-image-aspect="portrait"` (4:5)
   - `data-buy-box="sticky"` vs inline buy box
5. **Apple Human Interface Guidelines (HIG) Baseline:**
   Standard spatial grid based on 8pt increments, tabular pricing figures, 16px squircles, 9999px full-pill action buttons, and fluid spring transitions (`cubic-bezier(0.16, 1, 0.3, 1)`).
6. **Instant Zero-Deploy Admin Live Preview:**
   When merchants customize their brand in the Admin Settings, changes are previewed live via window `postMessage` (`STHENO_PREVIEW_TOKENS`) without build cycles or redeployments.
7. **Strict PCI SAQ-A Checkout Security ([docs/PCI_SAQ_A.md](../../docs/PCI_SAQ_A.md)):**
   All sensitive payment processing occurs inside hosted iframes (Stripe Elements) or hosted redirect gateways (bKash, Nagad, SSLCommerz). Checkout routes strictly enforce checkout-specific Content Security Policies ([docs/payment-page-scripts.md](../../docs/payment-page-scripts.md)).

---

## 2. Directory Structure

```
apps/storefront/
├── middleware.ts                         # Multi-tenant host header resolver
├── package.json                          # Package scripts and dependencies
├── tailwind.config.js                    # Dynamic Tailwind token integration
├── tsconfig.json                         # TypeScript configuration
├── packages/
│   ├── storefront-core/                  # App Router, API clients, SSR layout & token injector
│   │   └── src/
│   │       ├── app/                      # Next.js App Router ([countryCode], cart, checkout)
│   │       ├── lib/                      # Commerce data access, Medusa client SDK
│   │       └── styles/                   # Base CSS & token layers
│   └── frontend-default/                 # Canonical UI components & templates
│       └── src/modules/
│           ├── cart/                     # Cart drawer, line items, summary
│           ├── checkout/                 # Addresses, delivery options, payment step
│           ├── common/                   # Buttons, inputs, modals, icons
│           ├── home/                     # Hero, category grid, product rails
│           ├── layout/                   # Apple HIG navbar, footer, side menu
│           └── products/                 # Product detail templates, image gallery
```

---

## 3. Environment Variables

Create `.env.local` in `apps/storefront/`:

```bash
# Backend Engine URL (default local Rust engine)
NEXT_PUBLIC_MEDUSA_BACKEND_URL="http://127.0.0.1:9000"

# Public storefront base URL
NEXT_PUBLIC_BASE_URL="http://localhost:8000"

# Default country code for localized catalog routes
NEXT_PUBLIC_DEFAULT_REGION="us"

# Optional: Default publishing channel / sales channel
NEXT_PUBLIC_SALES_CHANNEL_ID=""
```

---

## 4. Development & Build Commands

All commands are run from the `apps/storefront/` directory:

| Command | Action |
|---|---|
| `npm run dev` | Starts local Next.js dev server on `http://localhost:8000` |
| `npm run build` | Compiles the production Next.js bundle |
| `npm run start` | Starts the production server on port `8000` |
| `npm run typecheck` | Validates TypeScript types across all storefront packages (`tsc --noEmit`) |
| `npm run lint` | Runs Next.js ESLint rules |
| `npm test` | Runs storefront unit & integration tests |
| `npm run test:e2e` | Runs Playwright end-to-end checkout & navigation tests |

---

## 5. Accessibility & Quality Standards (WCAG 2.2 Level AA)

The storefront strictly follows accessibility standards:
- **Minimum Target Size:** All buttons, selectors, and interactive links meet the `24×24px` minimum CSS target size (WCAG 2.2 SC 2.5.8).
- **Focus Indicators:** High-contrast visible focus rings (`2px` solid with `2px` offset) on all keyboard-navigable elements.
- **Modal Focus Management:** Side menus, cart drawers, and modal dialogs contain keyboard focus when open and cleanly restore focus to the trigger upon closing.
- **Accessible Forms:** All inputs carry explicit `<label>` bindings or `aria-label` descriptors, with descriptive inline error announcements.
- **No Div-Buttons:** Semantic HTML `<button>` and `<a>` elements are used exclusively for actions and navigation.
