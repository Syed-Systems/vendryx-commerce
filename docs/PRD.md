# Product Requirements Document — Stheno Commerce Engine

**Status:** Draft v1.1 · **Owner:** Syed Systems · **Last updated:** 2026-09-02
**Baseline:** `medusajs/medusa@b733400` (`develop` branch)

> **Scope of this document.** This is the **engine** PRD: what the commerce runtime must do
> and how fast. The *product* built on it — who buys it, what it costs, how a merchant is
> onboarded — is [PRODUCT.md](PRODUCT.md) and [ENTITLEMENTS.md](ENTITLEMENTS.md).
>
> Written on 2026-08-14, when the plan was a self-hosted single-tenant engine. That changed:
> Stheno is a hosted multi-tenant service ([ADR-012](DECISIONS.md#adr-012)) sold as a
> subscription ([ADR-017](DECISIONS.md#adr-017)). The engine requirements below survive that
> change unaltered — the deployment and distribution claims did not, and are corrected inline.

---

## 1. Summary

Stheno is a headless commerce engine written in Rust, with dedicated frontends for the storefront and the merchant dashboard, integrating with **Space** (`D:\Syed Systems\Space`) as the standalone company operator control plane ([ADR-031](DECISIONS.md#adr-030)). It delivers a complete commerce feature set — catalog, pricing, promotions, carts, orders, returns/claims/exchanges, multi-region, multi-channel, inventory, fulfillment, tax, payments, RBAC, i18n — on a compiled runtime with an order-of-magnitude smaller operational footprint than the Node stack it is measured against.

**One sentence:** a full commerce domain in a single 40 MB binary that starts in under a second, holds p95 latency under 25 ms, and serves every merchant from one process.

The Medusa v2 baseline is a **measuring stick and an internal wire contract, not a product claim** ([ADR-027](DECISIONS.md#adr-027)). It tells us the domain is complete and lets the admin dashboard be forked rather than rebuilt. Nothing a customer sees mentions it.

---

## 2. Problem statement

Medusa is the best-architected open-source commerce engine available. Its module system, durable workflow orchestration, and link-based composition are genuinely good ideas. Its runtime is the weak link.

Concretely, teams running Medusa v2 in production hit five recurring problems:

| # | Problem | Consequence |
|---|---|---|
| P1 | Node.js memory footprint (600 MB–1.5 GB per instance under load) | Horizontal scaling is expensive; small VPS deployments are impractical |
| P2 | Cold start of 5–12 s including module loading and migration checks | Serverless and autoscaling are effectively closed off; rolling deploys are slow |
| P3 | Latency of composite reads — product list with `calculated_price`, cart totals recomputation | p95 in the 80–250 ms range on realistic catalogs; degrades sharply past ~10k variants |
| P4 | Type-safety stops at the TypeScript boundary | Runtime errors on malformed module data; migrations drift from types silently |
| P5 | Dependency surface — a fresh install pulls 1,200+ npm packages | Supply-chain exposure; audit noise; slow CI |

None of these are design flaws in Medusa's *model*. They are properties of its *runtime*. Stheno keeps the model and replaces the runtime.

### 2.1 Why not just optimise Medusa

Considered and rejected. The dominant costs are structural — GC pressure from per-request object graphs in the module container, JSON serialisation of `BigNumber` money values, and the DataLoader-based remote-query hydration path. Fixing these inside Node means rewriting the same components this project rewrites, while keeping the runtime that caused them.

---

## 3. Goals and non-goals

### 3.1 Goals

- **G1 — Functional parity.** Every capability in [PARITY_MATRIX.md](PARITY_MATRIX.md) reaches *Complete*. Parity is measured by golden-file tests against a live Medusa instance, not by reading feature lists.
- **G2 — Admin wire compatibility, as an internal economy.** The forked Medusa dashboard runs against Stheno's Admin API unmodified, which is what makes G5 affordable. **Amended by [ADR-027](DECISIONS.md#adr-027):** this is an engineering goal, never a marketed one. The former goal — "swap the binary, keep your frontends", addressed to Medusa operators — is withdrawn, because we sell a hosted service and have no such audience.
- **G3 — Performance.** Meet every budget in §7 at the stated load, on the stated hardware.
- **G4 — Operational simplicity.** One statically linked binary. Postgres required, Redis optional. No Node runtime, no build step at deploy, no separate workflow service.
- **G5 — A working admin on day one.** Medusa's dashboard is forked, not rebuilt: Enterprise RBAC and SSO screens replaced clean-room, the rest adopted as-is. Improvements are additive and post-1.0. *(Previously "a better admin" — see [ADR-002](DECISIONS.md#adr-002) for what was traded and why.)*
- **G6 — Migration path.** A supported one-command import from a Medusa v2 Postgres database that preserves IDs, order history, and customer password hashes.

### 3.2 Non-goals

- **NG1 — A plugin marketplace.** Extensibility is compile-time (ADR-001). There is no runtime plugin installer. Deliberate.
- **NG2 — ~~Multi-tenancy in the core.~~ Reversed by [ADR-012](DECISIONS.md#adr-012) (2026-08-19).** Multi-tenancy *is* the core: one database, `tenant_id` on every row, isolation enforced by Postgres RLS with `FORCE`, tier resolved per request. The original non-goal — one deployment per merchant with its own database — was affordable per-store and unaffordable per-fleet: it made every engine update N updates. Kept visible rather than deleted, because it is the assumption most of this document was written under.
  Multiple stores are multiple deployments. This is a **deliberate commercial choice, not just a
  scoping one**: when Stheno is sold as a service, physical separation means a gating bug leaks a
  *feature*, never another customer's orders. Plan tiers are delivered by
  [entitlements](ENTITLEMENTS.md), not by tenancy.
- **NG3 — A CMS.** Product content, translations, and media are in scope. Landing pages, blogs, and page builders are not.
- **NG3b — Buyer accounts.** Buyers never register, log in, or see order history. All carts are guest carts; repeat buyers are recognised by phone number in the dashboard only. A product-wide decision, never a tier gate ([PRODUCT §2](PRODUCT.md)).
- **NG3c — Per-merchant theming.** One storefront design for everyone. Branding is name, logo, and colour — not layout.
- **NG3d — A mobile app.** Web push covers the notification need at zero marginal cost.
- **NG4 — Runtime JavaScript execution.** No embedded JS engine for hooks or custom logic.
- **NG5 — Improving Medusa's Admin API shape in 1.0.** Cursor pagination and RFC 9457 errors were considered and deferred; both would break the forked dashboard. Additive-only until a v3 surface is justified. See [ADR-002](DECISIONS.md#adr-002).
- **NG6 — Databases other than Postgres.** The pricing and query engines use Postgres-specific features (lateral joins, `SKIP LOCKED`, jsonb, partial indexes). Not portable, by choice.

---

## 4. Users and jobs to be done

### 4.1 Merchant operator — "Priya, ops lead at a 40-person DTC brand"

Lives in the admin dashboard 6 hours a day. Their day is order exceptions, not product creation.

| Job | Requirement |
|---|---|
| Find one order among 200k in under 3 seconds | Full-text + faceted order search; p95 < 150 ms |
| Edit a paid order — swap a variant, add an item, refund the difference | Order edit flow with a preview of the payment delta before commit |
| Process a return, decide restock vs. write-off, issue partial refund | Return workflow with per-item damage/restock disposition |
| Run a BOGO promo on one channel for one region for one weekend | Promotion rules with channel/region/date scoping and a campaign budget cap |
| Never lose work to a half-applied change | Every mutation is a durable workflow; interrupted operations resume or roll back cleanly |

### 4.2 Storefront developer — "Marco, agency contractor"

Building a headless storefront on a 6-week timeline. Cares about API predictability above all.

| Job | Requirement |
|---|---|
| Fetch a product with prices for a region and channel in one call | `GET /store/products?fields=*variants.calculated_price&region_id=&sales_channel_id=` |
| Build cart → checkout → payment → order without reading source code | Complete OpenAPI 3.1 spec; typed SDK generated from it |
| Reuse an existing Medusa storefront | Store API wire compatibility (G2) |
| Get a straight answer when a request is wrong | Structured errors: stable `code`, human `message`, offending field path |

### 4.3 Platform engineer — "Dana, runs infra for 12 merchant deployments"

| Job | Requirement |
|---|---|
| Deploy without a Node toolchain | Single static binary; distroless container under 60 MB |
| Run a small merchant on a 1 GB VPS | Resident memory under 128 MB at idle, under 400 MB at 500 rps |
| Diagnose a slow checkout | OpenTelemetry traces spanning HTTP → workflow → step → SQL |
| Survive a Redis outage | Redis is optional; in-process fallbacks for cache, lock, and event bus |
| Know a deploy is safe | Migrations are versioned, forward-only, and verified in CI against a schema snapshot |

### 4.4 Extension developer — "Sam, building a custom fulfillment integration"

| Job | Requirement |
|---|---|
| Add a carrier integration | Implement `FulfillmentProvider`, register in the workspace, recompile |
| Add a domain module | `stheno new-module` scaffolds crate, migrations, service, tests |
| Add a checkout step | Compose a workflow from existing steps; register in the flow registry |
| Trust it still compiles after upgrade | Provider traits are semver-stable within a major version |

---

## 5. Functional requirements

Requirements are grouped by module. Each maps to a parity-matrix row and one or more phases in the implementation plan. **MUST** = required for 1.0. **SHOULD** = required for 1.1.

### 5.1 Catalog

- **FR-CAT-1 (MUST)** Products with variants, options, option values, images, tags, types, collections, and hierarchical categories (self-referential, ranked, arbitrary depth).
- **FR-CAT-2 (MUST)** Variant ⇄ option-value matrix; a variant is uniquely identified by its option-value combination within a product.
- **FR-CAT-3 (MUST)** Product status lifecycle: `draft → proposed → published → rejected`.
- **FR-CAT-4 (MUST)** Per-variant image assignment distinct from the product gallery.
- **FR-CAT-5 (MUST)** Batch create/update/delete for products, variants, and images in one transactional workflow with per-row error reporting.
- **FR-CAT-6 (MUST)** Products scoped to sales channels; the store API filters by the channel resolved from the publishable API key.
- **FR-CAT-7 (SHOULD)** CSV/JSON import and export with a dry-run diff preview.

### 5.2 Pricing

- **FR-PRC-1 (MUST)** Price sets with multiple prices per currency, attached to variants and shipping options via link tables.
- **FR-PRC-2 (MUST)** Price rules on arbitrary attributes (`region_id`, `customer_group_id`, `sales_channel_id`, custom). Resolution is by rule specificity: most matched rules wins; ties break on lowest amount.
- **FR-PRC-3 (MUST)** Quantity tiers via `min_quantity` / `max_quantity` on prices.
- **FR-PRC-4 (MUST)** Price lists — `sale` and `override` types, with start/end dates, customer-group rules, and per-list price entries.
- **FR-PRC-5 (MUST)** Price preferences controlling tax-inclusive vs tax-exclusive display per region or currency.
- **FR-PRC-6 (MUST)** `calculated_price` on variants, returning calculated amount, original amount, currency, the price-list id if applied, and the full rule set that matched.
- **FR-PRC-7 (MUST)** Exact decimal arithmetic. No binary floating point anywhere in a money path.

### 5.3 Inventory and stock locations

- **FR-INV-1 (MUST)** Inventory items linked to variants (many-to-many; a variant may consume several inventory items with a required quantity each).
- **FR-INV-2 (MUST)** Inventory levels per stock location: `stocked_quantity`, `reserved_quantity`, `incoming_quantity`; `available = stocked − reserved`.
- **FR-INV-3 (MUST)** Reservations created at order placement, released on cancel or fulfillment.
- **FR-INV-4 (MUST)** `manage_inventory` and `allow_backorder` per variant.
- **FR-INV-5 (MUST)** Stock locations linked to sales channels and fulfillment sets.
- **FR-INV-6 (MUST)** Concurrency-safe decrements. Two simultaneous checkouts for the last unit: exactly one succeeds. Enforced by row-level locking, verified by a concurrency test.

### 5.4 Cart and checkout

- **FR-CRT-1 (MUST)** Cart with line items, shipping methods, addresses, region, currency, channel, and optional customer.
- **FR-CRT-2 (MUST)** Totals: item subtotal, discounts, tax, shipping, gift-card/credit lines, and grand total — recomputed on every mutation, never trusted from the client.
- **FR-CRT-3 (MUST)** Per-line and per-shipping-method adjustments and tax lines, stored not derived.
- **FR-CRT-4 (MUST)** Promotion application and removal with automatic recalculation.
- **FR-CRT-5 (MUST)** Shipping option listing filtered by cart contents, address, and channel; calculated shipping options priced through the fulfillment provider.
- **FR-CRT-6 (MUST)** Cart completion: inventory-reserving, payment-authorising, and order-creating in one compensating workflow.
- **FR-CRT-7 (MUST)** Completion is idempotent under retry — a duplicate `POST /carts/:id/complete` returns the same order and never double-charges. Enforced by an idempotency key on the workflow.

### 5.5 Promotions

- **FR-PRO-1 (MUST)** Promotion types `standard` and `buyget`.
- **FR-PRO-2 (MUST)** Application methods `fixed` and `percentage`, allocated `each` or `across`, targeting `items`, `shipping_methods`, or `order`.
- **FR-PRO-3 (MUST)** Rule engine over attributes with operators `eq, ne, in, nin, gt, gte, lt, lte` — evaluated against cart, item, and customer context.
- **FR-PRO-4 (MUST)** Buyget: buy-rules and target-rules with quantities and `apply_to_quantity`.
- **FR-PRO-5 (MUST)** Campaigns with a spend or usage budget; budget consumption is atomic and cannot be exceeded under concurrency.
- **FR-PRO-6 (MUST)** Automatic promotions (no code) and code-entered promotions.
- **FR-PRO-7 (MUST)** Deterministic stacking order for multiple applicable promotions, documented and covered by fixture tests.

### 5.6 Orders

- **FR-ORD-1 (MUST)** Order with items, shipping methods, addresses, transactions, credit lines, and a computed summary (paid, refunded, outstanding, difference).
- **FR-ORD-2 (MUST)** Order status set `pending, completed, draft, archived, canceled, requires_action`. Payment and fulfillment statuses tracked independently.
- **FR-ORD-3 (MUST)** Order changes as an event-sourced diff layer: `order_change` + `order_change_action`, supporting preview-before-commit for every mutation.
- **FR-ORD-4 (MUST)** Order edits — add, remove, or change quantity on a placed order, with a payment delta collected or refunded on confirm.
- **FR-ORD-5 (MUST)** Returns: request, approve, receive (with per-item received and damaged quantities), and refund. Partial receipt supported.
- **FR-ORD-6 (MUST)** Claims: `replace` and `refund` kinds, with inbound and outbound items and evidence images.
- **FR-ORD-7 (MUST)** Exchanges: inbound return plus outbound items with a computed difference.
- **FR-ORD-8 (MUST)** Draft orders — full authoring flow, edit before conversion, then convert to a live order.
- **FR-ORD-9 (MUST)** Order transfer between customers, with request/accept/decline/cancel.
- **FR-ORD-10 (MUST)** Complete, append-only order timeline reconstructable from change actions.

### 5.7 Fulfillment

- **FR-FUL-1 (MUST)** Fulfillment sets containing service zones, each with geo zones (country / province / city / zip).
- **FR-FUL-2 (MUST)** Shipping options with types, rules, price sets, and provider binding.
- **FR-FUL-3 (MUST)** Shipping profiles grouping products by fulfillment requirements.
- **FR-FUL-4 (MUST)** Fulfillment creation, label generation, cancellation, and per-item quantities.
- **FR-FUL-5 (MUST)** Pickup-type shipping options in addition to shipping.
- **FR-FUL-6 (MUST)** `manual` provider in core; the provider trait must be sufficient for real carriers.

### 5.8 Payments

- **FR-PAY-1 (MUST)** Payment collections holding sessions, payments, captures, and refunds.
- **FR-PAY-2 (MUST)** Session lifecycle `pending → authorized → captured`, plus `requires_more` for 3DS/SCA step-ups.
- **FR-PAY-3 (MUST)** Partial and multiple captures; partial and multiple refunds with reasons.
- **FR-PAY-4 (MUST)** Webhook ingestion with signature verification, replay protection, and idempotent application.
- **FR-PAY-5 (MUST)** Account holders — provider-side saved customers for stored payment methods.
- **FR-PAY-6 (MUST)** Stripe provider in core covering cards, wallets, and SCA.
- **FR-PAY-7 (MUST)** Payment provider availability scoped per region.
- **FR-PAY-8 (MUST)** Outbound idempotency on every mutating provider call — a deterministic key
  derived from the workflow transaction, plus an `unknown`-state resolution protocol so an
  ambiguous timeout is resolved by reading provider state, never by blind retry.
- **FR-PAY-9 (MUST)** Webhook and API response treated as two observations of one truth, converging
  through a single reducer with monotonic state transitions ordered by the provider's timestamp.
  Neither ordering may corrupt state.
- **FR-PAY-10 (MUST)** A single audited money boundary for minor-unit conversion, covering
  zero-decimal and three-decimal currencies including Stripe's round-up-to-nearest-ten rule.
- **FR-PAY-11 (MUST)** Ledger invariants enforced in the database: captures never exceed the
  payment, refunds never exceed captures, under concurrent writes.
- **FR-PAY-12 (MUST)** Nightly reconciliation against the provider, reporting drift without
  auto-correcting it.
- **FR-PAY-13 (MUST)** Authorization-expiry tracking with operator warning before funds are lost.
- **FR-PAY-14 (MUST)** Abandoned 3DS sessions expire and release their inventory reservations.
- **FR-PAY-15 (SHOULD)** Dispute and chargeback records, with refunds blocked on disputed payments.
- **FR-PAY-16 (MUST)** PCI SAQ-A posture — card data never reaches Stheno; tokens only.

Full specification, including the 34-case edge atlas: [PAYMENTS.md](PAYMENTS.md).

### 5.9 Tax

- **FR-TAX-1 (MUST)** Tax regions hierarchically — country with optional province children.
- **FR-TAX-2 (MUST)** Tax rates with rules targeting products, product types, or shipping options; one default rate per region plus overrides.
- **FR-TAX-3 (MUST)** Combinable rates (province stacks on country) and non-combinable overrides.
- **FR-TAX-4 (MUST)** Provider trait for external calculation (Avalara/TaxJar-class integrations).
- **FR-TAX-5 (MUST)** Tax-inclusive pricing with correct reverse computation, rounding to the currency's minor unit, and residual distribution across lines that sums exactly to the total.

### 5.10 Customers, auth, and access control

- **FR-AUTH-1 (MUST)** Actor types `user` (admin) and `customer` (store), extensible by crate.
- **FR-AUTH-2 (MUST)** Providers: emailpass, Google, GitHub (portable), and generic OIDC
  (**clean-room from the OIDC spec** — Medusa's is Enterprise, see [ADR-008](DECISIONS.md#adr-008)).
- **FR-AUTH-3 (MUST)** JWT access tokens carrying `actor_id`, `actor_type`, `auth_identity_id`, and `app_metadata`; refresh-token rotation with reuse detection.
- **FR-AUTH-4 (MUST)** MFA — TOTP factors, recovery codes, and a challenge/verify flow.
- **FR-AUTH-5 (MUST)** Password reset with single-use, expiring, hashed tokens.
- **FR-AUTH-6 (MUST)** API keys — `publishable` (store, scoped to sales channels) and `secret` (admin).
- **FR-AUTH-7 (MUST)** RBAC — policies, roles, role inheritance, and role assignment to users and
  invites. Deny beats allow; effective permissions computed over the inheritance closure.
  **Clean-room implementation** — Medusa's RBAC is Enterprise Edition and may not be ported,
  referenced, or adapted ([ADR-008](DECISIONS.md#adr-008)).
- **FR-AUTH-8 (MUST)** Customer groups with group-scoped pricing and promotions.
- **FR-AUTH-9 (MUST)** Registered and guest customers; guest-to-registered upgrade preserving orders.
- **FR-AUTH-10 (MUST)** Verify imported Medusa `scrypt-kdf` password hashes, and transparently re-hash to Argon2id on the user's next successful login.

### 5.11 Regions, channels, store

- **FR-REG-1 (MUST)** Regions with currency, countries, tax settings, and payment providers.
- **FR-REG-2 (MUST)** Multi-currency with per-currency decimal precision from ISO 4217.
- **FR-REG-3 (MUST)** Sales channels scoping products, stock locations, and API keys.
- **FR-REG-4 (MUST)** Store settings — name, default region, default currency, supported locales.

### 5.12 Platform services

- **FR-PLT-1 (MUST)** Event bus with typed events, subscriber registration, and at-least-once delivery. In-process and Redis backends.
- **FR-PLT-2 (MUST)** Scheduled jobs with cron expressions and a distributed lock so exactly one instance runs each tick.
- **FR-PLT-3 (MUST)** File storage — local and S3-compatible providers, presigned upload URLs.
- **FR-PLT-4 (MUST)** Notifications — templated, per-channel, with delivery-attempt records. Local and SendGrid providers.
- **FR-PLT-5 (MUST)** Caching with a provider trait; in-process and Redis backends.
- **FR-PLT-6 (MUST)** Distributed locking with Postgres and Redis backends.
- **FR-PLT-7 (MUST)** Search — provider trait plus a built-in Postgres full-text provider; index sync driven by domain events.
- **FR-PLT-8 (MUST)** Read index — denormalised cross-module projection enabling filter and sort across module boundaries without N+1 fan-out.
- **FR-PLT-9 (MUST)** Translations — locale-scoped field overrides for catalog content.
- **FR-PLT-10 (SHOULD)** Analytics event emission with local and PostHog providers.
- **FR-PLT-11 (MUST)** Admin settings persistence — view configurations, column layouts, property labels, and per-user preferences.

### 5.13 Entitlements — selling this as a service

Stheno is delivered as a hosted service with plan tiers, so it needs a runtime layer that decides
what a given deployment may do. Full spec: [ENTITLEMENTS.md](ENTITLEMENTS.md).

- **FR-ENT-1 (MUST)** A resolved, immutable `Entitlements` value at boot from tier config plus env
  overrides. Unknown keys or malformed quotas abort startup.
- **FR-ENT-2 (MUST)** Three account states — **Trial (30 days) → Free (indefinite) → Pro** — with
  gating decided by one rule: *the free tier does nothing that costs marginal money.* Core commerce
  is never gateable.
- **FR-ENT-3 (MUST)** Three kinds of gate: **feature** (route group unmounted), **quota** (atomic
  counter — products, staff seats), and **window** (query horizon — analytics history).
- **FR-ENT-3b (MUST)** Trial starts on **first product created**, never at signup; warnings at day
  23 and 29; a daily job evaluates state and audit-logs every transition.
- **FR-ENT-4 (MUST)** **Gates are mount-point decisions only.** Gated routes are not mounted, gated
  workflows not registered. No `Entitlements` reference may appear inside `crates/modules/` —
  CI-enforced. A disabled route returns 404, never 403.
- **FR-ENT-5 (MUST)** Quota enforcement is atomic — a counter updated in the same transaction as
  the insert, rejecting at the limit under concurrency.
- **FR-ENT-6 (MUST)** Downgrade never deletes or hides data. A merchant over quota keeps everything
  — **grandfathered, visible, sellable** — and is blocked only from creating more. Trial expiry and
  paid downgrade use the same code path. Removing a seller's live products is an unrecoverable trust
  failure.
- **FR-ENT-7 (MUST)** `GET /admin/feature-flags` in Medusa's shape, so the forked dashboard hides
  disabled screens with no changes; plus an additive `GET /admin/entitlements` exposing quotas,
  usage, and grace state.
- **FR-ENT-8 (SHOULD)** Signed licence file for self-hosted deployments. Deferred until the first
  self-hosted deal — config suffices while every deployment is ours.

### 5.14 Beyond parity — where Stheno exceeds Medusa

These are the "or better" items. Each is additive and none breaks Store API compatibility.

- **FR-PLUS-1 — Workflow observability.** Every execution is queryable: step timeline, inputs, outputs, compensation history, and a replay-from-step control in the admin.
- **FR-PLUS-2 — Idempotency keys on all mutating Store endpoints**, not just cart completion.
- ~~**FR-PLUS-3 — Cursor pagination**~~ and ~~**FR-PLUS-4 — RFC 9457 errors**~~ — **withdrawn.** Both would break the forked admin dashboard. Revisit only alongside an additive `/admin/v3/*` surface ([ADR-002](DECISIONS.md#adr-002)).
- **FR-PLUS-5 — Native OpenTelemetry.** Traces, metrics, and logs correlated by trace id out of the box, with no APM vendor package.
- **FR-PLUS-6 — Compile-time-verified queries.** A migration that breaks a query fails `cargo build`, not a 2 a.m. page.
- **FR-PLUS-7 — Deterministic totals engine.** Totals are computed by one pure function over a cart/order snapshot, unit-tested against a fixture corpus, identical on server and in preview.
- **FR-PLUS-8 — Bulk admin operations as first-class workflows** with progress, partial success, and cancellation — not fire-and-forget loops.
- **FR-PLUS-9 — Zero-downtime migration mode.** Expand/contract migration phases with an explicit compatibility window declared per migration.

---

## 6. User experience requirements

### 6.1 Admin dashboard — forked

The dashboard is Medusa's, forked from `packages/admin/dashboard` (React 18 + Vite + React Router + TanStack Query/Table). It is **not** rebuilt and it is **not** Next.js. Requirements are therefore about the fork, not about a design:

- **Functional parity with Medusa's dashboard** across all 38 route groups, verified by using it.
- **Enterprise screens rebuilt clean-room** — roles, policies, the permissions provider, and SSO login ([ADR-008](DECISIONS.md#adr-008)). The only genuinely new UI in 1.0.
- **Rebranding only** beyond that: name, logo, colour tokens. No layout redesign in 1.0.
- **Workflow-execution timeline deepened** to expose step inputs, outputs, and replay-from-step (FR-PLUS-1). Medusa already has the route; we extend it.
- **WCAG 2.2 AA** — inherited where Medusa already conforms; gaps found in the P9 audit are fixed in the fork, because the EAA obligation is ours regardless of who wrote the code.

Admin performance targets are **withdrawn**. It is a client-rendered SPA; the 200 KB budget and server-rendered first paint are not achievable without the rewrite this decision avoids.

### 6.2 Storefront reference implementation

> [!NOTE]
> **ADR-038 (accepted 2026-11-27) replaces this storefront.** The engine will render storefronts from `minijinja` template themes (`themes/<theme-id>/` plus `theme.json`), and the Next.js storefront is deleted at task T4.7. What follows describes the storefront that serves until then. Plan: [STOREFRONT_THEME_ENGINE_PLAN.md](STOREFRONT_THEME_ENGINE_PLAN.md).

Forked from Medusa's Next.js starter, which already speaks the Store API we implement. Kept close to upstream so the compatibility claim stays testable.

- Next.js App Router, React Server Components for catalog, client components for cart.
- Lighthouse ≥ 95 on performance, accessibility, and SEO for product detail pages.
- Core Web Vitals at p75: LCP < 2.0 s, INP < 150 ms, CLS < 0.05.
- Catalog browsing works with JavaScript disabled.

Page inventory: [SITEMAP.md](SITEMAP.md). The merchant-facing product built on the engine —
onboarding, social checkout, notifications, reporting — is specified in [PRODUCT.md](PRODUCT.md).

---

## 7. Non-functional requirements

Reference hardware for all budgets: **4 vCPU / 8 GB** application node, **4 vCPU / 16 GB** Postgres 16, same region, catalog of **50,000 variants / 200,000 orders**.

### 7.1 Latency budgets (p95, excluding network)

| Endpoint class | Budget | Medusa v2 observed |
|---|---|---|
| `GET /store/products` (20 items, with `calculated_price`) | **25 ms** | 90–160 ms |
| `GET /store/products/:id` (full variant + price expansion) | **15 ms** | 60–110 ms |
| `POST /store/carts/:id/line-items` (totals recompute) | **30 ms** | 120–240 ms |
| `POST /store/carts/:id/complete` (full checkout workflow) | **250 ms** | 600–1500 ms |
| `GET /admin/orders` (50 items, filtered) | **40 ms** | 150–400 ms |
| `GET /admin/orders/:id` (full expansion) | **35 ms** | 130–300 ms |
| Any single workflow step | **10 ms** | — |

Medusa figures are indicative, drawn from public benchmarks and community reports. **Phase 0 must establish measured baselines on identical hardware** — the claim is only worth what the harness proves. See [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).

### 7.2 Throughput and resource use

| Metric | Requirement |
|---|---|
| Sustained catalog read throughput | ≥ 2,000 rps per node at p95 budget |
| Sustained cart mutation throughput | ≥ 400 rps per node at p95 budget |
| Checkout completions | ≥ 80/s per node |
| Resident memory, idle | ≤ 128 MB |
| Resident memory, 500 rps sustained | ≤ 400 MB |
| Cold start to first served request | ≤ 1.0 s |
| Container image size | ≤ 60 MB (distroless) |
| Postgres connections per node | ≤ 20 (pooled) |

### 7.3 Reliability

- **Availability target 99.9%** for the API tier, excluding planned migration windows.
- **No lost writes.** Every state transition is either committed or fully compensated. A crash mid-workflow resumes on the next instance to acquire the execution lock.
- **RPO 5 minutes / RTO 30 minutes**, via Postgres PITR.
- **Graceful shutdown** — in-flight requests drain up to 30 s; running workflows checkpoint and release their locks.
- **Redis is optional.** Losing Redis degrades to in-process cache, Postgres locks, and in-process events. It never causes an outage on a single-node deployment.

### 7.4 Security and compliance

Full specification, threat model, and the 29-finding audit of this package:
[SECURITY.md](SECURITY.md). Summary of the binding obligations:

| Standard | Effect |
|---|---|
| **PCI DSS v4.0.1** | SAQ A. Note the March 2025 eligibility change: an embedded payment form now requires confirming the page is not susceptible to script attacks |
| **GDPR / UK GDPR / CPRA** | Erasure by pseudonymisation, retention limits, export, 72-hour breach notice |
| **European Accessibility Act** | Enforceable since 28 June 2025 for EU e-commerce. EN 301 549 / WCAG 2.1 AA minimum; we target 2.2 AA |
| **PSD2 / SCA** | 3DS step-up, already specified |

- OWASP ASVS Level 2.
- **CSRF defence on all cookie-authenticated requests** — double-submit token, Origin validation,
  and a custom-header requirement, applied in middleware.
- **Ownership enforced by type** (`Owned<T, Scope>` extractor), not by handler discipline. 404 on
  ownership failure, never 403.
- **Request DTOs are never entity types** — mass assignment is structurally impossible.
- **Data erasure by pseudonymisation**, preserving financial records while removing identity, with
  a cascade across the read index, search index, workflow context, and webhook store.
- **Append-only audit log** for authentication, authorization denials, money movement, price and
  permission changes, and exports.
- Security headers on every response; nonce-based CSP with no `unsafe-inline`.
- Egress guard on all admin-configurable URLs (SSRF, including cloud metadata).
- Upload allowlist with content sniffing and re-encoding; **SVG rejected**.
- Abuse controls: card-testing, promo-code enumeration, cart flooding, user enumeration.
- Argon2id password hashing (`m=19456, t=2, p=1`), with scrypt verification retained for imports.
- Access tokens ≤ 15 min TTL; refresh tokens rotating with reuse detection.
- Rate limiting per IP and per actor on auth endpoints; exponential-backoff lockout.
- All input validated at the HTTP boundary before reaching a service. Rejection is the default.
- No secrets in logs. Structured logging with an explicit redaction allowlist for money, PII, and tokens.
- SQL injection structurally prevented by Diesel's query builder; raw `sql_query` sites require bound parameters and carry a review marker.
- Dependency audit (`cargo audit`, `cargo deny`) gating CI.

### 7.5 Compatibility

- Postgres **15+**. Redis **7+** where enabled.
- Linux `x86_64` and `aarch64`, gnu and musl targets.
- Rust stable, MSRV pinned per minor release and bumped only in minor releases.

---

## 8. Success metrics

| Metric | Target | When measured |
|---|---|---|
| Parity matrix completion | 100% of MUST rows | 1.0 release gate |
| Store API compatibility suite | 100% of recorded Medusa request/response pairs pass | Every CI run from P2 |
| p95 latency budgets met | All 7 classes in §7.1 | 1.0 release gate, then nightly |
| Memory ceiling | ≤ 400 MB at 500 rps | Nightly load test |
| Test coverage — domain crates | ≥ 85% line, 100% of money and totals paths | Every CI run |
| Unmodified Medusa storefront runs against Stheno | Yes, zero code changes | P10 gate |
| Migration import fidelity | 100% of entities, IDs preserved, totals identical | P10 gate |
| Time for a new developer to add a module | ≤ 1 day using the scaffold | Post-1.0 survey |

---

## 9. Release plan

| Release | Contents | Gate |
|---|---|---|
| **0.1 Foundations** | Workspace, DB layer, HTTP, workflow engine, query engine | Engine executes a 3-step compensating workflow durably across a crash |
| **0.2 Catalog** | Product, category, collection, inventory, stock location | Store product endpoints pass the compatibility suite |
| **0.3 Commerce core** | Pricing, tax, region, currency, cart, promotion, payment | A cart reaches a correct total with promo + tax across 3 regions |
| **0.4 Orders** | Order, fulfillment, sales channel, full checkout | Guest checkout end-to-end with Stripe test mode |
| **0.5 Order lifecycle** | Edits, returns, claims, exchanges, draft orders | Every order mutation previewable and committable |
| **0.6 Identity** | Auth, user, customer, API keys, RBAC | RBAC denies correctly across the inheritance closure |
| **0.7 Platform** | File, notification, search, index, cache, lock, i18n, settings | Search index converges after a bulk product update |
| **0.8 Admin** | Admin API (Medusa dialect) + forked dashboard | Unmodified Medusa dashboard runs; then Priya's five jobs complete unaided |
| **0.9 Storefront + migration** | Storefront, importer, docs | A Medusa DB imports; storefront passes Lighthouse gates |
| **1.0 GA** | Hardening, perf, security review | All §8 metrics green |

---

## 10. Risks

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| R1 | The order-change model (edits/claims/exchanges/returns) is the most intricate part of Medusa and is under-documented | High — a wrong model here is a rewrite | Port it first as a standalone crate in P1 with a fixture corpus extracted from Medusa's own integration tests, before any UI depends on it |
| R2 | Store API parity drifts as Medusa's `develop` moves | Medium | Pin the baseline commit. Re-record the compatibility corpus quarterly; treat drift as scheduled work, not an emergency |
| R3 | Diesel's dynamic-filter ergonomics across ~200 filterable fields | Medium | Build a `FilterSpec → BoxedSelectStatement` macro in P0 and validate it against the 20 hardest endpoints before committing to it |
| R4 | Compile-time extensibility deters adopters who expect npm plugins | Medium — adoption risk, not technical | Ship the scaffold CLI, a provider-authoring guide, and 16 first-party providers so most users never need to write one |
| R5 | Workspace compile times at 60+ crates | Medium — developer velocity | Enforce a flat dependency graph, `cargo-hakari` for feature unification, sccache in CI, and a compile-time budget gate at every phase |
| R6 | Money and rounding bugs, especially tax-inclusive reverse computation | High — silent financial error | One `Money` type, decimal-only, no `From<f64>`. Property tests asserting line sums equal order totals exactly, at every currency precision |
| R7 | Team unfamiliarity with Rust async and Diesel | Medium | P0 is deliberately a learning phase with the smallest domain surface; pair the first module implementation |
| R8 | Scope creep from "or better" | Medium | §5.13 is a closed list. Anything else is 1.1 |
| R10 | Compliance gaps discovered late — erasure, audit log, and PCI script controls are cheap to design in and expensive to retrofit | **High** — an EU launch blocker, not a technical debt item | [SECURITY.md](SECURITY.md) assigns each control to a phase; F1–F3 are launch blockers tracked from P0 |
| R9 | Payment correctness — ambiguous timeouts, webhook races, minor-unit conversion. A bug here charges customers twice or loses money silently | **High** — the only failure mode that is both invisible and financial | [PAYMENTS.md](PAYMENTS.md) specifies the idempotency protocol, convergence reducer, DB-enforced ledger invariants, and reconciliation. Its 34-case edge atlas is the P4 test suite, not a checklist |

---

## 11. Open questions

| # | Question | Needed by | Default if unanswered |
|---|---|---|---|
| Q1 | ~~Is a hosted/cloud offering planned?~~ **Answered 2026-08-19: yes, and it is the only offering.** Self-hosting is not sold. See [ADR-012](DECISIONS.md#adr-012), [ADR-017](DECISIONS.md#adr-017), [ENTITLEMENTS.md](ENTITLEMENTS.md) | — | — |
| Q2 | Which payment providers beyond Stripe are required for 1.0? | P4 | Stripe only; the trait proven with a second stub |
| Q3 | Is B2B (company accounts, quotes, spending limits) in 1.0 scope? | P5 | No. 1.1, as a module |
| Q4 | Subscriptions and recurring billing? | P5 | No. Out of scope, matching Medusa |
| Q5 | Which locales must ship translated in the admin at 1.0? | P8 | English only; i18n plumbing complete |
| Q6 | Target license — MIT to match Medusa, or AGPL? | 1.0 | MIT, to keep the storefront ecosystem reusable |
