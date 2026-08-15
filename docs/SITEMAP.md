# Site Map — Storefront and Admin Dashboard

**Storefront is Next.js (forked from Medusa's starter). Admin is React + Vite (forked from Medusa's dashboard).** Route groups in parentheses `(…)` do not appear in the URL.
Admin routes are Medusa's 38 dashboard route groups at `b733400`, inherited as-is via the fork
([ADR-002](DECISIONS.md#adr-002)). They are listed, not redesigned.

Legend: **RSC** = React Server Component (server-rendered) · **CC** = Client Component · **ISR** = incrementally regenerated

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
        │   ├── cart/page.tsx              CC   · cart review
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
        │   ├── login/  register/  forgot-password/  reset-password/[token]/
        │   └── auth/callback/[provider]/  RSC  · OAuth/OIDC landing
        │
        └── (content)/
            ├── [handle]/page.tsx          RSC  · CMS-agnostic static page slot
            └── not-found.tsx
```

### 1.3 Page data contracts

| Page | Endpoints | Notes |
|---|---|---|
| Home | `GET /store/collections`, `GET /store/products?limit=8` | ISR 300 s |
| Store listing | `GET /store/products?fields=*variants.calculated_price&limit&offset&order` | Facets from `product-tags`, `product-types`, `collections` |
| Category | `GET /store/product-categories?handle=`, then products by `category_id` | Full ancestor chain for breadcrumbs |
| Product detail | `GET /store/products?handle=&fields=*variants.calculated_price,*variants.options,*images` | ISR 60 s; `generateStaticParams` over published handles |
| Search | `GET /store/products?q=` | Backed by the search provider |
| Cart | `GET/POST /store/carts/:id`, line-item CRUD, `POST …/promotions` | Cart id in an httpOnly cookie |
| Checkout — delivery | `GET /store/shipping-options?cart_id=`, `POST …/calculate` for calculated options | — |
| Checkout — payment | `GET /store/payment-providers?region_id=`, `POST /store/payment-collections`, `POST …/payment-sessions` | Provider SDK mounts client-side |
| Checkout — complete | `POST /store/carts/:id/complete` | Sends an idempotency key; safe to retry |
| Order confirmation | `GET /store/orders/:id` | — |
| Account orders | `GET /store/orders` | Requires customer JWT |
| Return request | `GET /store/return-reasons`, `POST /store/returns` | — |
| Auth | `POST /auth/customer/emailpass`, `/register`, `/reset-password`, `POST /store/customers` | — |

### 1.4 Storefront non-functional gates

- Lighthouse ≥ 95 (performance, a11y, SEO) on product detail.
- LCP < 2.0 s, INP < 150 ms, CLS < 0.05 at p75.
- Catalog browsing functional with JavaScript disabled.
- `next/image` everywhere; no layout shift on product media.
- JSON-LD `Product`, `BreadcrumbList`, `Organization` on the relevant pages.
- `sitemap.xml` and `robots.txt` generated per country/locale pair.

---

## Part 2 — Admin dashboard (`apps/admin`) — forked

**Not built. Forked** from Medusa's `packages/admin/dashboard` at a pinned version
([ADR-002](DECISIONS.md#adr-002)). React 18 + Vite + React Router + TanStack Query/Table — **not**
Next.js, and not server-rendered.

The route inventory below is therefore a description of what we inherit, not a design.

### 2.1 Inherited route groups (38)

Exactly Medusa's, verified against `packages/admin/dashboard/src/routes` at `b733400`:

```
home                orders              draft-orders*       products
product-variants    product-options     product-tags        product-types
categories          collections         inventory           reservations
customers           customer-groups     promotions          campaigns
price-lists         regions             tax-regions         sales-channels
locations           shipping-profiles   shipping-option-types
return-reasons      refund-reasons      property-labels     translations
store               users               invite              profile
settings            api-key-management  workflow-executions
policies ‡          roles ‡             login               reset-password
no-match
```

`‡` = **Enterprise. Removed from the fork and rebuilt clean-room** ([ADR-008](DECISIONS.md#adr-008)).

*(`draft-orders` is reached through the orders routes in Medusa's tree rather than as a sibling
group; listed here because it is a distinct operator surface.)*

### 2.1b New product routes added by the fork

Medusa's screens cover products, orders, and inventory. These are ours
([PRODUCT.md](PRODUCT.md)):

| Route | Purpose |
|---|---|
| `/onboarding` | Google sign-in → store name + WhatsApp → spreadsheet upload → column mapping → 5-row preview |
| `/orders/social` | The Social queue — draft orders from chat handoff, matched by `Ref` code |
| `/analytics` | Four aggregates; horizon bounded by the tier's window gate |
| `/settings/billing` | Tier, trial countdown, quota usage, upgrade |
| `/settings/notifications` | Web push subscription management (VAPID) |

### 2.2 What the fork changes

Everything else is adopted as-is. This list plus §2.1b is the complete scope of admin UI work
in 1.0.

| # | Change | Effort |
|---|---|---|
| 1 | **Strip Enterprise paths** — `routes/policies/`, `routes/roles/`, `hooks/api/rbac-policies.tsx`, `hooks/api/rbac-roles.tsx`, `hooks/use-require-rbac-feature.tsx`, `components/common/required-permissions-section/`, `lib/permissions/`, `providers/permissions-provider/`, `login/components/sso-login.tsx` | Mechanical |
| 2 | **Rebuild those screens clean-room** — roles list/detail/create, policy editor, permissions provider, required-permissions sections, SSO login. No reference to the removed code | **The real work** |
| 3 | **Point at Stheno** — base URL, and patch the API client to send `X-Stheno-CSRF` on mutations ([SECURITY §6](SECURITY.md)) | One interceptor |
| 4 | **Rebrand** — name, logo, colour tokens | Small |
| 5 | **Deepen the workflow-executions screen** — step timeline with inputs, outputs, compensation history, replay-from-step (FR-PLUS-1) | Medium |
| 6 | **Fix WCAG gaps** found in the P9 audit | Unknown until audited |

**Nothing else is redesigned in 1.0.** No new navigation model, no new list-view patterns, no new
IA. Those were specified against a rebuild that is no longer happening, and re-adding them would
forfeit the reason for forking.

### 2.3 The admin extension system comes along

Because the dashboard is forked rather than rebuilt, Medusa's `admin-vite-plugin` extension system
comes with it — build-time discovery of `src/admin/{widgets,routes,custom-fields,layouts,i18n}`.
This was previously listed as a deliberate exclusion; it is now **inherited, and should be verified
working in P9** rather than assumed.

### 2.4 Non-functional gates

Revised, because a client-rendered SPA cannot meet the targets written for a server-rendered app:

| Gate | Status |
|---|---|
| WCAG 2.2 AA | **Required** — EAA obligation is ours regardless of who wrote the code |
| Functional parity with Medusa's dashboard | **Required**, verified by use |
| Every list state encoded in the URL | Inherited |
| Command palette | Inherited |
| ~~Server-rendered first paint~~ | **Withdrawn** — SPA |
| ~~≤ 200 KB initial JS~~ | **Withdrawn** — SPA |
| ~~Interactive < 1.5 s on 4G~~ | **Withdrawn** — measure and record, do not gate |

### 2.5 Upstream policy

- Fork point is a **pinned Medusa version**, recorded in `apps/admin/UPSTREAM.md`.
- Upstream merges are **scheduled, deliberate tasks** — never automatic, never tracking `develop`.
- Local changes stay minimal and are documented, so merges stay tractable.
- Enterprise paths are re-checked on every upstream merge; the carve-out list can grow.

---

## Part 3 — Public API surface map

Not pages, but part of the site map. Full detail in [API_SPEC.md](API_SPEC.md).

```
/store/*      46 route files   Medusa v2 wire-compatible  — FROZEN
/admin/*     ~278 route files  Medusa v2 wire-compatible — FROZEN
/auth/*       actor_type × provider  Medusa-compatible shape
/hooks/*      provider webhooks
/health/live  /health/ready  /metrics  /openapi.json
```
