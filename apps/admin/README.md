# Vendryx Merchant Admin Dashboard

> Production-grade merchant operations dashboard for Vendryx Commerce, built with React 18, TypeScript, Vite, TanStack Query & Table, and Tailwind CSS. Governed by [ADR-029](../../docs/DECISIONS.md#adr-029), [ADR-028](../../docs/DECISIONS.md#adr-028), and [AGENTS.md](../../AGENTS.md).

---

## 1. Overview & Architectural Principles

The Vendryx Merchant Admin Dashboard provides merchants with complete operational control over their catalog, orders, customers, delivery couriers, payment gateways, and store appearance.

### Core Architecture Pillars

1. **Truth Before Beauty Contract ([ADR-029](../../docs/DECISIONS.md#adr-029)):**
   - **Zero Mock Storage:** `localStorage` is strictly forbidden across `src/routes/` and enforced by automated CI (`scripts/check-no-mock-storage.ps1`).
   - **Zero Fake Timers:** No `setTimeout` simulating network calls or operations.
   - Every card, toggle, list, and form binds directly to a verified Rust backend API endpoint.
2. **Apple Human Interface Guidelines (HIG) Information Architecture:**
   - Redesigned `/settings` hub featuring **7 organized categories**:
     1. *Store & Operations* (Store details, regions, sales channels, business policies)
     2. *Product Catalog* (Categories, tags, attributes, inventory locations)
     3. *Sales & Customers* (Customer groups, promotion rules, tax configurations)
     4. *Delivery & Fulfillment* (Shipping options, courier providers: Pathao, Steadfast, RedX)
     5. *Payments & Finance* (Gateways: Stripe, bKash, Nagad, SSLCommerz, COD reconciliation)
     6. *Security & Access* (Users & RBAC permissions, Clerk SSO, audit trail)
     7. *Developer & Integrations* (Publishable API keys, webhooks, custom domains)
   - Real-time synonym search (e.g. searching "vat" surfaces Tax Regions; "cash" surfaces Payment Gateways).
   - Component primitives: `SettingsRow`, `SettingsSection`, `StatusAccessory`, and `AsyncState` offering 44px minimum touch targets and progressive disclosure.
3. **Backend Wire Parity with Medusa v2 ([ADR-002](../../docs/DECISIONS.md#adr-002), [ADR-027](../../docs/DECISIONS.md#adr-027)):**
   - Communicates with the Rust engine on `http://127.0.0.1:9000/admin/*`.
   - Full compatibility with Medusa v2 Admin DTOs ensures that the forked dashboard receives authoritative data without custom wrapper friction.
4. **Multi-Tenant Identity & Authentication:**
   - Integrated with Clerk React (`@clerk/clerk-react`) for secure merchant staff authentication, session management, and tenant scoping.
5. **Real-Time Storefront Theme Styler:**
   - Allows merchants to configure primary colors, surface materials, typography, and card radii with instant zero-deploy preview via postMessage (`STHENO_PREVIEW_TOKENS`).

---

## 2. Directory Structure

```
apps/admin/
├── index.html                            # Dashboard entry point (Vendryx branding per ADR-028)
├── package.json                          # Package scripts and dependencies
├── vite.config.ts                        # Vite configuration and API proxy rules
├── tsconfig.json                         # TypeScript configuration
├── src/
│   ├── components/                       # Shared Apple HIG primitives (SettingsRow, DataTables)
│   ├── extensions/                       # Custom dashboard widget injections
│   ├── hooks/                            # TanStack Query data hooks and mutation wrappers
│   ├── lib/                              # Client SDK, API client, date/money formatting
│   ├── providers/                        # Clerk Auth, QueryClient, Theme, i18n providers
│   └── routes/                           # File-based dashboard routing
│       ├── orders/                       # Order fulfillment, social checkout orders, COD settlement
│       ├── products/                     # Product catalog, variants, pricing, inventory
│       ├── inventory/                    # Multi-warehouse stock locations & reservations
│       ├── customers/                    # Customer records & customer groups
│       ├── promotions/                   # Campaign rules, promo codes, percentage discounts
│       ├── payment-providers/            # Tenant credentials (Stripe, bKash, Nagad, SSLCommerz)
│       ├── couriers/                     # Courier dispatch & tracking (Pathao, Steadfast, RedX)
│       └── settings/                     # Apple HIG 7-category Settings Hub
```

---

## 3. Environment Variables

Create `.env.local` in `apps/admin/`:

```bash
# Backend Admin API base URL (defaults to proxy at /admin)
VITE_MEDUSA_BACKEND_URL="http://127.0.0.1:9000"

# Clerk Publishable Key for authentication
VITE_CLERK_PUBLISHABLE_KEY="pk_test_..."

# Optional: Storefront preview URL for live iframe theme styling
VITE_STOREFRONT_PREVIEW_URL="http://localhost:8000"
```

---

## 4. Development & Build Commands

All commands are run from the `apps/admin/` directory:

| Command | Action |
|---|---|
| `npm run dev` | Starts local Vite dev server on `http://127.0.0.1:5173` |
| `npm run build` | Builds the production dashboard bundle to `dist/` |
| `npm run preview` | Previews the production build locally |
| `npm run typecheck` | Validates TypeScript types across the dashboard |
| `npm test` | Runs Vitest unit and component tests |
| `npm run test:e2e` | Runs Playwright end-to-end admin tests |

---

## 5. Security & Data Integrity Rules

- **No Token/Credential Logging:** API keys, webhook secrets, and payment provider credentials must never be written to browser storage or console output.
- **Strict Role-Based Access Control:** Routes enforce staff permissions via `GET /admin/rbac/me/permissions`. Disabled actions render informative read-only accessories rather than breaking navigation.
