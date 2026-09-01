# Site Map — Storefront and Admin Dashboard

**Storefront is Next.js (forked from Medusa's starter). Admin is React + Vite (forked from Medusa's dashboard).** Route groups in parentheses `(…)` do not appear in the URL.
Admin routes are Medusa's 38 dashboard route groups at `b733400`, inherited as-is via the fork
([ADR-002](DECISIONS.md#adr-002)). They are listed, not redesigned.

Legend: **RSC** = React Server Component (server-rendered) · **CC** = Client Component · **ISR** = incrementally regenerated

---

## Part 0 — Official Stheno SaaS website (`apps/marketing`)

The marketing site is the merchant-facing entry point. It is separate from the storefront and has
no documentation navigation. Its public routes are:

| Route | Purpose |
|---|---|
| `/` | Product story and spatial 3D launch experience |
| `/features` | Focused merchant service capabilities |
| `/pricing` | Free 14-day store-request trial and Pro commercial terms |
| `/login` | Merchant Google or email/password authentication |
| `/signup` | Merchant signup with resumable optional business draft |
| `/forgot-password` | Password reset request |
| `/welcome` | Account-level empty state, create-store request, and launch status |

`/welcome` is intentionally not an admin tenant route. Email accounts verify before a store request
may be submitted. Once submitted, the customer-visible status is `Submitted`, `In review`,
`Provisioning`, `Online`, or `Needs changes`; the tenant dashboard appears only after `Online`.

---

## Part 1 — Storefront (`apps/storefront`)

Public commerce surface. Locale and country are path segments, matching Medusa's starter so the
compatibility claim in PRD G2 is testable against a real storefront.

### 1.1 URL structure

```
/[countryCode]/[locale]/…      e.g. /us/en/products/merino-crew
```

`countryCode` resolves the region (currency, tax, payment providers). `locale` resolves translations.
Both are validated at the layout level; an unknown value 404s rather than silently falling back.

### 1.2 Route tree

```
app/
└── [countryCode]/
    └── [locale]/
        ├── layout.tsx                     RSC  · region + locale resolution, nav, cart badge
        ├── page.tsx                       RSC  · home
        │
        ├── (catalog)/
        │   ├── store/page.tsx             RSC  · all products, faceted
        │   ├── categories/[...handle]/    RSC  · nested category, arbitrary depth
        │   ├── collections/[handle]/      RSC  · collection listing
        │   ├── products/[handle]/         RSC  · product detail (ISR 60s)
        │   └── search/page.tsx            CC   · search results
        │
        ├── (checkout)/
        │   ├── cart/page.tsx              CC   · cart review & line items
        │   └── checkout/
        │       ├── page.tsx               CC   · address → delivery → payment → review
        │       │                                 payment shows COD only unless Pro
        │       └── confirmed/[id]/        RSC  · order confirmation
        │
        │   — product page also offers SOCIAL CHECKOUT (PRODUCT §5.2):
        │     WhatsApp → wa.me?text= pre-fill
        │     Messenger/Instagram → copy to clipboard, then open app
        │
        │   NOTE: (account)/* is NOT MOUNTED for this product — buyers have no
        │   accounts (PRODUCT §2). Routes retained in the engine, unused here.
        ├── (account)/account/
        │   ├── layout.tsx                 RSC  · auth guard, account nav
        │   ├── page.tsx                   RSC  · overview
        │   ├── profile/page.tsx           CC   · name, email, phone, password
        │   ├── addresses/page.tsx         CC   · address book CRUD
        │   ├── orders/page.tsx            RSC  · order history
        │   ├── orders/[id]/page.tsx       RSC  · order detail + timeline
        │   ├── orders/[id]/returns/new/   CC   · request a return
        │   └── orders/[id]/transfer/      CC   · accept/decline an order transfer
        │
        ├── (auth)/
        │   ├── login/                     CC   · customer email/password login
        │   ├── register/                  CC   · customer registration
        │   ├── forgot-password/           CC   · password reset request
        │   ├── reset-password/[token]/    CC   · password reset confirmation
        │   └── auth/callback/[provider]/  RSC  · OAuth/OIDC landing
        │
        └── (content)/
            ├── [handle]/page.tsx          RSC  · CMS-agnostic static page slot
            └── not-found.tsx              RSC  · 404 page
```

### 1.3 Page data contracts

| Page | Endpoints | Method & Parameters | Notes |
|---|---|---|---|
| **Home** | `GET /store/collections`<br>`GET /store/products` | `limit=8&fields=*variants.calculated_price` | ISR 300 s |
| **Store listing** | `GET /store/products` | `fields=*variants.calculated_price&limit=20&offset=0&order=-created_at` | Facets from `product-tags`, `product-types`, `collections` |
| **Category** | `GET /store/product-categories`<br>`GET /store/products` | `handle=:handle`<br>`category_id=:id` | Full ancestor chain for breadcrumbs |
| **Product detail** | `GET /store/products` | `handle=:handle&fields=*variants.calculated_price,*variants.options,*images` | ISR 60 s; `generateStaticParams` over published handles |
| **Search** | `GET /store/products` | `q=:query` | Backed by the search provider (`stheno-search`) |
| **Cart** | `GET /store/carts/:id`<br>`POST /store/carts/:id/line-items`<br>`POST /store/carts/:id/promotions` | Line item CRUD, coupon codes | Cart ID stored in httpOnly cookie |
| **Checkout — delivery** | `GET /store/shipping-options`<br>`POST /store/shipping-options/:id/calculate` | `cart_id=:id` | Shipping option selection |
| **Checkout — payment** | `GET /store/payment-providers`<br>`POST /store/payment-collections`<br>`POST /store/payment-collections/:id/payment-sessions` | `region_id=:id`<br>`cart_id=:id` | Provider SDK mounts client-side in iframe |
| **Checkout — complete** | `POST /store/carts/:id/complete` | Header: `Idempotency-Key` | Idempotent saga execution |
| **Order confirmation** | `GET /store/orders/:id` | `id=:id` | Order receipt & fulfillment status |
| **Account orders** | `GET /store/orders` | `limit=10&offset=0` | Requires customer JWT |
| **Return request** | `GET /store/return-reasons`<br>`POST /store/returns` | Line items, quantities, return reasons | Return flow submission |
| **Customer Auth** | `POST /auth/customer/emailpass`<br>`POST /auth/customer/emailpass/register`<br>`POST /auth/customer/emailpass/reset-password` | Email, password, credentials | Session issuance & token rotation |

### 1.4 Storefront non-functional gates

- **Lighthouse Score**: ≥ 95 on Performance, Accessibility, Best Practices, and SEO for product detail pages.
- **Core Web Vitals**: LCP < 2.0 s, INP < 150 ms, CLS < 0.05 at p75.
- **No-JS Catalog Browsing**: Catalog browsing and product pages remain completely functional with JavaScript disabled.
- **Image Optimization**: `next/image` with explicit aspect ratios; zero layout shift on product media.
- **Structured Data**: JSON-LD `Product`, `BreadcrumbList`, and `Organization` embedded on all relevant pages.
- **SEO Assets**: Dynamic `sitemap.xml` and `robots.txt` generated per country/locale pair.

---

## Part 2 — Admin dashboard (`apps/admin`) — forked

**Not built. Forked** from Medusa's `packages/admin/dashboard` at a pinned version
([ADR-002](DECISIONS.md#adr-002)). React 18 + Vite + React Router + TanStack Query/Table — **not**
Next.js, and not server-rendered.

The route inventory below is therefore a description of what we inherit, not a design.

### 2.1 Curated merchant workspace

The Medusa fork remains the operational foundation, but Stheno mounts only the merchant workflows
below. Any inherited page outside this set is not mounted for a merchant and resolves as 404. Plan
access is also a mount-point decision: Trial and Free receive core operations and brand kit;
Pro additionally mounts gateways, couriers, domains, and bespoke-theme status.

| Workspace area | Route family |
|---|---|
| Home and onboarding | `/`, `/onboarding` |
| Orders | `/orders`, `/orders/social` |
| Catalog | `/products`, `/collections`, `/categories`, `/inventory` |
| Analytics | `/analytics` |
| Merchant settings | `/settings/store`, `/settings/users`, `/settings/notifications`, `/settings/billing`, `/profile` |
| Brand kit | `/settings/brand-kit` |
| Pro operations | `/settings/gateways`, `/settings/couriers`, `/settings/domains`, `/settings/bespoke-theme` |

### 2.2 Inherited route inventory (not automatically exposed)

Exactly Medusa's, verified against `packages/admin/dashboard/src/routes` at `b733400`:

| Route Group | Primary URL | Description | Domain Module |
|---|---|---|---|
| `home` | `/` | Store summary dashboard and quick actions | `stheno-store` |
| `orders` | `/orders`, `/orders/:id` | Order processing, fulfillment, and refunds | `stheno-order`, `stheno-fulfillment` |
| `draft-orders` | `/orders/drafts` | Draft order creation and customer send | `stheno-order` |
| `products` | `/products`, `/products/:id` | Catalog management, media, options | `stheno-product` |
| `product-variants` | `/products/:id/variants` | Variant inventory and pricing matrix | `stheno-product`, `stheno-pricing` |
| `product-options` | `/products/:id/options` | Option attributes (Size, Color, Material) | `stheno-product` |
| `product-tags` | `/products/tags` | Product classification tags | `stheno-product` |
| `product-types` | `/products/types` | Product category types | `stheno-product` |
| `categories` | `/categories` | Hierarchical product taxonomy tree | `stheno-product` |
| `collections` | `/collections` | Marketing collections | `stheno-product` |
| `inventory` | `/inventory` | Multi-location inventory item tracking | `stheno-inventory` |
| `reservations` | `/reservations` | Stock reservations against orders/carts | `stheno-inventory` |
| `customers` | `/customers` | Customer profiles and metadata | `stheno-customer` |
| `customer-groups` | `/customer-groups` | Customer tiers and VIP segmentation | `stheno-customer` |
| `promotions` | `/promotions` | Standard and code-based promotion rules | `stheno-promotion` |
| `campaigns` | `/campaigns` | Multi-promotion budget campaigns | `stheno-promotion` |
| `price-lists` | `/price-lists` | Custom price lists and overrides | `stheno-pricing` |
| `regions` | `/settings/regions` | Regional currencies, taxes, payment methods | `stheno-region` |
| `tax-regions` | `/settings/tax-regions` | Regional and postal tax rules | `stheno-tax` |
| `sales-channels` | `/settings/sales-channels` | Channel-based catalog scoping | `stheno-sales-channel` |
| `locations` | `/settings/locations` | Warehouses and stock locations | `stheno-stock-location` |
| `shipping-profiles` | `/settings/shipping-profiles` | Default and custom shipping profiles | `stheno-fulfillment` |
| `shipping-option-types` | `/settings/shipping-option-types` | Flat rate and calculated delivery types | `stheno-fulfillment` |
| `return-reasons` | `/settings/return-reasons` | Merchant-defined return reasons | `stheno-order` |
| `refund-reasons` | `/settings/refund-reasons` | Financial refund classification reasons | `stheno-payment` |
| `property-labels` | `/settings/property-labels` | Custom attribute display names | `stheno-settings` |
| `translations` | `/settings/translations` | Multilingual locale translation overrides | `stheno-translation` |
| `store` | `/settings/store` | Store profile, default currencies, links | `stheno-store` |
| `users` | `/settings/users` | Admin staff accounts and roles | `stheno-user` |
| `invite` | `/settings/users/invites` | Team member email invitations | `stheno-user` |
| `profile` | `/profile` | Current staff profile and password change | `stheno-user` |
| `settings` | `/settings` | Global administration hub | `stheno-settings` |
| `api-key-management` | `/settings/api-keys` | Publishable and Secret API tokens | `stheno-api-key` |
| `workflow-executions` | `/settings/workflows` | Saga step inspection and retry console | `stheno-workflow` |
| `policies` ‡ | `/settings/policies` | Clean-room RBAC policy editor | `stheno-rbac` |
| `roles` ‡ | `/settings/roles` | Clean-room RBAC role and permission mapping | `stheno-rbac` |
| `login` | `/login` | Admin authentication with MFA prompt | `stheno-auth` |
| `reset-password` | `/reset-password` | Admin password recovery | `stheno-auth` |

`‡` = **Enterprise. Removed from the fork and rebuilt clean-room** ([ADR-008](DECISIONS.md#adr-008)).

### 2.3 New product routes added by the fork

Medusa's screens cover products, orders, and inventory. These are our dedicated product workflows
([PRODUCT.md](PRODUCT.md)):

| Route | Purpose | Backend Endpoints |
|---|---|---|
| `/onboarding` | 5-minute initial setup: Google sign-in → store name + WhatsApp → spreadsheet upload → column mapping → 5-row preview | `POST /admin/products/batch`, `POST /admin/stores` |
| `/orders/social` | The Social queue — draft orders from chat handoff (WhatsApp / Messenger / Instagram), matched by `Ref` code | `GET /admin/orders?status=draft&is_social=true` |
| `/analytics` | Real-time analytics (Revenue, Orders, Conversion Rate, AOV); horizon bounded by plan tier | `GET /admin/reports/dashboard` |
| `/settings/billing` | Subscription tier, trial countdown, quota usage progress bars (seats, products, monthly orders), upgrade button | `GET /admin/platform/usage`, `POST /admin/platform/upgrade` |
| `/settings/notifications` | Web push subscription management (VAPID public keys and event trigger subscriptions) | `GET/POST /admin/notifications/subscriptions` |

### 2.4 What the fork changes

Everything else is adopted as-is. This list plus §2.1b is the complete scope of admin UI work:

| # | Change | Implementation Details |
|---|---|---|
| 1 | **Strip Enterprise paths** | Remove `routes/policies/`, `routes/roles/`, `hooks/api/rbac-policies.tsx`, `hooks/api/rbac-roles.tsx`, `hooks/use-require-rbac-feature.tsx`, `components/common/required-permissions-section/`, `lib/permissions/`, `providers/permissions-provider/`, `login/components/sso-login.tsx`. |
| 2 | **Rebuild those screens clean-room** | Rebuild roles list/detail/create, policy editor, permissions provider, required-permissions sections, SSO login without any reference to proprietary code. |
| 3 | **Point at Stheno** | Configure base URL and API client interceptor to send `X-Stheno-CSRF` on state-mutating requests ([SECURITY §6](SECURITY.md)). |
| 4 | **Rebrand** | Update branding tokens, logo SVGs, and typography to Stheno design system. |
| 5 | **Deepen workflow-executions screen** | Step timeline with inputs, outputs, compensation history, and retry-from-step capability (FR-PLUS-1). |
| 6 | **Fix WCAG gaps** | Enforce WCAG 2.2 Level AA compliance (ARIA landmarks, focus management, color contrast). |

### 2.5 The admin extension system comes along

Because the dashboard is forked rather than rebuilt, Medusa's `admin-vite-plugin` extension system
comes with it — build-time discovery of `src/admin/{widgets,routes,custom-fields,layouts,i18n}`.

### 2.6 Non-functional gates

| Gate | Status | Details |
|---|---|---|
| **WCAG 2.2 AA** | **Required** | European Accessibility Act (EAA) compliance across all forms and navigation. |
| **Functional Parity** | **Required** | 100% test coverage against Medusa v2 dashboard behavior. |
| **URL State Encoding** | **Inherited** | Filter, sort, pagination, and active drawer states fully serialized to query parameters. |
| **Command Palette** | **Inherited** | Global keyboard shortcut (`Cmd+K` / `Ctrl+K`) for rapid entity search and navigation. |

### 2.7 Upstream policy

- Fork point is a **pinned Medusa version**, recorded in `apps/admin/UPSTREAM.md`.
- Upstream merges are **scheduled, deliberate tasks** — never automatic, never tracking `develop`.
- Local changes stay minimal and are documented, so merges stay tractable.
- Enterprise paths are re-checked on every upstream merge; the carve-out list can grow.

---

## Part 3 — Public API surface map

Full technical specification is documented in [API_SPEC.md](API_SPEC.md).

```
/store/*          Storefront Commerce APIs (Products, Collections, Carts, Orders, Payments, Shipping)
/admin/*          Administration & Operational APIs (Catalog, Inventory, Pricing, Customers, Settings)
/auth/*           Authentication Handlers (Email/Password, Google, GitHub, OIDC)
/hooks/*          Provider Webhook Handlers (Stripe, SendGrid)
/health/live      Liveness Probe (Worker status)
/health/ready     Readiness Probe (DB & Redis connection status)
/metrics          Prometheus Telemetry Metrics
/openapi.json     OpenAPI v3 Specification Endpoint
```
