# Parity Matrix — Stheno vs Medusa v2

**Baseline:** `medusajs/medusa@b733400` (`develop`)
**Purpose:** the checklist that decides whether [PRD G1](PRD.md) is met. Every **MUST** row must
reach ✅ before 1.0.

### Status legend

| | Meaning |
|---|---|
| ⬜ | Not started |
| 🟡 | In progress |
| ✅ | Complete — implemented, tested, **and passing its parity fixtures** |
| ➕ | Beyond Medusa — Stheno only |
| ➖ | Deliberately excluded, with a reason |

**A row is only ✅ when its parity fixtures pass.** "Implemented" is not "complete" — that
distinction is the entire point of this document.

---

## 1. Modules — 34 packages

| Module | Entities | Workflows | Priority | Phase | Status |
|---|---:|---:|---|---|---|
| product | 12 | 27 | MUST | P2 | ✅ |
| pricing | 6 | 10 | MUST | P3 | ✅ |
| inventory | 3 | 4 | MUST | P2 | ✅ |
| stock-location | 2 | 3 | MUST | P2 | ✅ |
| sales-channel | 1 | 3 | MUST | P2 | ✅ |
| cart | 9 | 15 | MUST | P4 | ✅ |
| promotion | 7 | 11 | MUST | P4 | ✅ |
| payment | 8 | 10 | MUST | P4 | ✅ |
| tax | 4 | 9 | MUST | P3 | ✅ |
| region | 2 | 3 | MUST | P3 | ✅ |
| currency | 1 | — | MUST | P3 | ✅ |
| store | 3 | 3 | MUST | P3 | ✅ |
| order | 23 | 68 | MUST | P5–P6 | ✅ |
| fulfillment | 12 | 8 | MUST | P5 | ✅ |
| customer | 4 | 11 | MUST | P7 | ✅ |
| auth | 6 | 2 | MUST | P7 | ✅ |
| user | 2 | 12 | MUST | P7 | ✅ |
| api-key | 1 | 4 | MUST | P7 | ✅ |
| rbac ‡ | 5 | 9 | MUST | P7 | ✅ |
| file | — | 2 | MUST | P8 | ✅ |
| notification | 2 | — | MUST | P8 | ✅ |
| search | 2 | — | MUST | P8 | ✅ |
| index | 4 | — | MUST | P8 | ✅ |
| caching | — | — | MUST | P8 | ✅ |
| locking | — | — | MUST | P8 | ✅ |
| translation | 3 | 5 | MUST | P8 | ✅ |
| settings | 4 | 8 | MUST | P8 | ✅ |
| analytics | — | — | SHOULD | P8 | ✅ |
| event-bus (local, redis) | — | — | MUST | P1 | ✅ |
| cache (inmemory, redis) | — | — | MUST | P8 | ✅ |
| workflow-engine (inmemory, redis) | 1 | — | MUST | P1 | ✅ |
| link-modules | 22 tables | — | MUST | P1 | ✅ |

---

## 2. Capabilities

### 2.1 Catalog

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Products with variants, options, option values | MUST | P2 | ⬜ |
| Variant ⇄ option-value identity matrix | MUST | P2 | ⬜ |
| Product status lifecycle (draft/proposed/published/rejected) | MUST | P2 | ⬜ |
| Hierarchical categories, arbitrary depth, ranked | MUST | P2 | ⬜ |
| Collections, tags, types | MUST | P2 | ⬜ |
| Product and per-variant images | MUST | P2 | ⬜ |
| Batch product/variant/image operations | MUST | P2 | ⬜ |
| Sales-channel scoping | MUST | P2 | ⬜ |
| CSV/JSON import and export | SHOULD | P10 | ⬜ |
| Import dry-run diff preview | ➕ | P10 | ⬜ |

### 2.2 Pricing

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Price sets with multi-currency prices | MUST | P3 | ⬜ |
| Price rules on arbitrary attributes | MUST | P3 | ⬜ |
| Specificity-based resolution (`rules_count DESC`) | MUST | P3 | ⬜ |
| Quantity tiers (min/max) | MUST | P3 | ⬜ |
| Price lists — sale and override | MUST | P3 | ⬜ |
| Price list rules and date windows | MUST | P3 | ⬜ |
| Price preferences (tax-inclusive display) | MUST | P3 | ⬜ |
| `calculated_price` with original and rule set | MUST | P3 | ⬜ |
| Single-query resolution for N variants | ➕ | P3 | ⬜ |

### 2.3 Inventory

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Inventory items, many-to-many with variants | MUST | P2 | ⬜ |
| Levels per location (stocked/reserved/incoming) | MUST | P2 | ⬜ |
| Reservations with release on cancel/fulfil | MUST | P2 | ⬜ |
| `manage_inventory`, `allow_backorder` | MUST | P2 | ⬜ |
| Concurrency-safe decrement, no oversell | MUST | P2 | ⬜ |

### 2.4 Cart and checkout

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Cart with items, methods, addresses, region, channel | MUST | P4 | ⬜ |
| Totals engine (subtotal, discount, tax, shipping, credit) | MUST | P4 | ⬜ |
| Line and shipping adjustments + tax lines | MUST | P4 | ⬜ |
| Promotion apply/remove with recalculation | MUST | P4 | ⬜ |
| Eligible shipping options by cart and address | MUST | P5 | ⬜ |
| Calculated shipping via provider | MUST | P5 | ⬜ |
| Cart completion as a compensating workflow | MUST | P5 | ⬜ |
| Idempotent completion — no double charge | MUST | P5 | ⬜ |
| Credit lines (gift card / store credit) | MUST | P4 | ⬜ |

### 2.5 Promotions

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Standard and buyget types | MUST | P4 | ⬜ |
| Fixed and percentage methods | MUST | P4 | ⬜ |
| Allocation each / across | MUST | P4 | ⬜ |
| Targets: items, shipping methods, order | MUST | P4 | ⬜ |
| Rule engine, 8 operators | MUST | P4 | ⬜ |
| Buy-rules and target-rules with quantities | MUST | P4 | ⬜ |
| Campaigns with spend/usage budgets | MUST | P4 | ⬜ |
| Atomic budget consumption under concurrency | MUST | P4 | ⬜ |
| Automatic and code-entered promotions | MUST | P4 | ⬜ |
| Deterministic, documented stacking order | MUST | P4 | ⬜ |

### 2.6 Orders

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Order with items, methods, transactions, summary | MUST | P5 | ⬜ |
| Independent payment and fulfillment statuses | MUST | P5 | ⬜ |
| Versioned order items | MUST | P5 | ⬜ |
| `order_change` / `order_change_action` model | MUST | P1 | ⬜ |
| **Preview before commit, zero writes** | MUST | P6 | ⬜ |
| Order edits with payment delta | MUST | P6 | ⬜ |
| Returns: request, approve, receive, refund | MUST | P6 | ⬜ |
| Partial receipt with damaged quantities | MUST | P6 | ⬜ |
| Claims: replace and refund, with evidence images | MUST | P6 | ⬜ |
| Exchanges with computed difference | MUST | P6 | ⬜ |
| Draft orders and conversion | MUST | P6 | ⬜ |
| Order transfer between customers | MUST | P6 | ⬜ |
| Append-only timeline from change actions | MUST | P6 | ⬜ |
| Sequential, gap-free `display_id` | MUST | P5 | ⬜ |

### 2.7 Fulfillment

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Fulfillment sets, service zones, geo zones | MUST | P5 | ⬜ |
| Geo zone types: country/province/city/zip | MUST | P5 | ⬜ |
| Shipping options with rules and types | MUST | P5 | ⬜ |
| Shipping profiles | MUST | P5 | ⬜ |
| Fulfillment creation, labels, cancellation | MUST | P5 | ⬜ |
| Pickup options | MUST | P5 | ⬜ |
| Manual provider | MUST | P5 | ⬜ |
| Provider trait sufficient for real carriers | MUST | P5 | ⬜ |

### 2.8 Payments

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Payment collections, sessions, payments | MUST | P4 | ⬜ |
| Lifecycle incl. `requires_more` (3DS/SCA) | MUST | P4 | ⬜ |
| Partial and multiple captures | MUST | P4 | ⬜ |
| Partial and multiple refunds with reasons | MUST | P4 | ⬜ |
| Webhooks: signature, replay protection, idempotent | MUST | P4 | ⬜ |
| Account holders and saved methods | MUST | P4 | ⬜ |
| Stripe provider | MUST | P4 | ⬜ |
| Region-scoped provider availability | MUST | P4 | ⬜ |
| 18-method provider interface (11 required + 7 vault) | MUST | P4 | ⬜ |
| Minor-unit conversion: zero- and three-decimal currencies | MUST | P4 | ⬜ |
| Three-decimal round-up-to-ten rule | MUST | P4 | ⬜ |
| 7 session states, 8 webhook actions | MUST | P4 | ⬜ |
| Async-settlement methods (OXXO, boleto, bank transfer) | MUST | P4 | ⬜ |
| Outbound idempotency keys on all provider calls | ➕ | P4 | ⬜ |
| Ambiguous-timeout resolution protocol | ➕ | P4 | ⬜ |
| Webhook/response convergence reducer | ➕ | P4 | ⬜ |
| Monotonic transitions, provider-timestamp ordering | ➕ | P4 | ⬜ |
| DB-enforced capture/refund ledger invariants | ➕ | P4 | ⬜ |
| Nightly reconciliation with drift reporting | ➕ | P8 | ⬜ |
| Authorization-expiry warning job | ➕ | P4 | ⬜ |
| Abandoned-3DS sweep releasing reservations | ➕ | P4 | ⬜ |
| Provider capability probe | ➕ | P4 | ⬜ |
| Disputes and chargebacks | ➕ | P8 | ⬜ |
| 34-case edge atlas covered by tests | ➕ | P4 | ⬜ |

### 2.9 Tax

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Hierarchical tax regions (country → province) | MUST | P3 | ⬜ |
| Rates with product/type/shipping rules | MUST | P3 | ⬜ |
| Combinable and override rates | MUST | P3 | ⬜ |
| Provider trait for external calculation | MUST | P3 | ⬜ |
| Tax-inclusive reverse computation, exact | MUST | P3 | ⬜ |

### 2.10 Identity and access

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Actor types user/customer, extensible | MUST | P7 | ⬜ |
| emailpass, Google, GitHub | MUST | P7 | ⬜ |
| OIDC / SSO ‡ | MUST | P7 | ⬜ |
| JWT with actor context | MUST | P7 | ⬜ |
| Refresh rotation with reuse detection | ➕ | P7 | ⬜ |
| MFA — TOTP + recovery codes | MUST | P7 | ⬜ |
| Password reset, single-use expiring tokens | MUST | P7 | ⬜ |
| Publishable and secret API keys | MUST | P7 | ⬜ |
| Publishable key → sales-channel scoping | MUST | P7 | ⬜ |
| RBAC: policies, roles, inheritance ‡ | MUST | P7 | ⬜ |
| Deny-beats-allow over the closure | MUST | P7 | ⬜ |
| Customer groups | MUST | P7 | ⬜ |
| Guest → registered upgrade preserving orders | MUST | P7 | ⬜ |
| scrypt import + transparent Argon2id re-hash | ➕ | P7 | ⬜ |

### 2.11 Platform

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Event bus, in-process and Redis | MUST | P1 | ⬜ |
| Transactional event buffering | ➕ | P1 | ⬜ |
| Subscribers with retry and dead-letter | MUST | P1 | ⬜ |
| Scheduled jobs with distributed lock | MUST | P1 | ⬜ |
| File providers: local, S3 | MUST | P8 | ⬜ |
| Notification providers: local, SendGrid | MUST | P8 | ⬜ |
| Cache providers: in-process, Redis | MUST | P8 | ⬜ |
| Lock providers: Postgres, Redis | MUST | P8 | ⬜ |
| Search provider + Postgres full-text | MUST | P8 | ⬜ |
| Read index with incremental sync | MUST | P8 | ⬜ |
| Translations, locale-scoped | MUST | P8 | ⬜ |
| Admin settings persistence | MUST | P8 | ⬜ |
| Analytics: local, PostHog | SHOULD | P8 | ⬜ |
| Redis-optional degradation | ➕ | P8 | ⬜ |

### 2.12 Workflow engine

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Sequential and parallel steps | MUST | P1 | ⬜ |
| Compensation per step | MUST | P1 | ⬜ |
| Retries with interval and cap | MUST | P1 | ⬜ |
| Step and workflow timeouts | MUST | P1 | ⬜ |
| Async steps with external completion | MUST | P1 | ⬜ |
| `background_execution` | MUST | P1 | ⬜ |
| `continue_on_permanent_failure` | MUST | P1 | ⬜ |
| `skip_on_permanent_failure` (bool or step name) | MUST | P1 | ⬜ |
| `no_compensation`, `compensate_async`, `no_wait` | MUST | P1 | ⬜ |
| Nested sub-transactions | MUST | P1 | ⬜ |
| Durable state, crash resumption | MUST | P1 | ⬜ |
| Idempotency by transaction id | MUST | P1 | ⬜ |
| Compile-time type checking between steps | ➕ | P1 | ⬜ |
| Execution timeline API + replay-from-step | ➕ | P9 | ⬜ |

### 2.13 API

| Capability | Priority | Phase | Status |
|---|---|---|---|
| ~46 Store endpoints, wire-compatible | MUST | P2–P6 | ⬜ |
| ~278 Admin endpoints, **wire-compatible** | MUST | P9 | ⬜ |
| Unmodified Medusa dashboard runs against Stheno | MUST | P9 | ⬜ |
| Auth endpoints, Medusa-compatible shape | MUST | P7 | ⬜ |
| Field selection with `*` expansion | MUST | P1 | ⬜ |
| Filter grammar, 8+ operators | MUST | P0 | ⬜ |
| Offset pagination (Store) | MUST | P0 | ⬜ |
| ~~Cursor pagination (Admin)~~ | ➖ | — | withdrawn, ADR-002 |
| ~~RFC 9457 errors (Admin)~~ | ➖ | — | withdrawn, ADR-002 |
| Idempotency keys on all mutations | ➕ | P4 | ⬜ |
| OpenAPI 3.1 generated from handlers | ➕ | P9 | ⬜ |
| `@medusajs/js-sdk` works unmodified on both surfaces | MUST | P9 | ⬜ |
| Rate limiting | MUST | P0 | ⬜ |

### 2.14 Admin dashboard

| Capability | Priority | Phase | Status |
|---|---|---|---|
| All 38 Medusa route groups — **forked, not rebuilt** | MUST | P9 | ⬜ |
| Enterprise RBAC + SSO screens rebuilt clean-room ‡ | MUST | P9 | ⬜ |
| Order list/detail with full expansion | MUST | P9 | ⬜ |
| Order edit with delta preview | MUST | P9 | ⬜ |
| Return receive with per-item disposition | MUST | P9 | ⬜ |
| Product create wizard and price matrix | MUST | P9 | ⬜ |
| Promotion and price-list rule builders | MUST | P9 | ⬜ |
| RBAC role and policy editors | MUST | P9 | ⬜ |
| Saved views and column configuration | MUST | P9 | ⬜ |
| ~~Server-rendered lists, ≤ 200 KB JS~~ | ➖ | — | withdrawn, ADR-002 — forked SPA |
| Command palette (`⌘K`) | inherited | P9 | ⬜ |
| WCAG 2.2 AA | ➕ | P9 | ⬜ |
| Workflow execution viewer | ➕ | P9 | ⬜ |
| Top-level returns/claims/exchanges queues | ➕ | P9 | ⬜ |

### 2.15 Security and compliance

Almost entirely ➕ — Medusa provides the primitives, not the hardened posture. Findings map to
[SECURITY §2](SECURITY.md).

| Capability | Priority | Phase | Status |
|---|---|---|---|
| CSRF defence on cookie auth (F1) | **MUST — blocker** | P0 | ⬜ |
| Data erasure by pseudonymisation (F2) | **MUST — blocker** | P8 | ⬜ |
| Append-only audit log (F3) | **MUST — blocker** | P1 | ⬜ |
| DTO/entity separation, CI-enforced (F4) | MUST | P0 | ⬜ |
| `Owned<T, Scope>` ownership extractor (F5) | MUST | P0 | ⬜ |
| Cart secret binding and TTL (F6, F22) | MUST | P4 | ⬜ |
| Security headers + nonce CSP (F7) | MUST | P0 | ⬜ |
| Upload allowlist, sniffing, re-encode, no SVG (F8) | MUST | P2 | ⬜ |
| Egress guard against SSRF and metadata (F9) | MUST | P2 | ⬜ |
| Promo-code and card-testing abuse limits (F10) | MUST | P4 | ⬜ |
| Filter value/pattern/depth limits (F11) | MUST | P0 | ⬜ |
| Webhook pre-signature rate limit (F12) | MUST | P4 | ⬜ |
| `token_version` invalidation on credential change (F13) | MUST | P7 | ⬜ |
| Anti-enumeration on auth endpoints (F14) | MUST | P7 | ⬜ |
| `metadata` size, depth, and key limits (F15) | MUST | P0 | ⬜ |
| Webhook and workflow-context retention (F16, F17) | MUST | P8 | ⬜ |
| Trusted-proxy IP resolution (F19) | MUST | P0 | ⬜ |
| Scoped secret API keys (F20) | MUST | P7 | ⬜ |
| Encryption at rest for sensitive columns (F23) | MUST | P7 | ⬜ |
| Encrypted, restore-tested backups (F24) | MUST | P10 | ⬜ |
| PCI SAQ A + script-attack criterion (F25) | **MUST** | P10 | ⬜ |
| EAA / WCAG 2.2 AA conformance (F26) | **MUST** | P9 | ⬜ |
| Customer data export endpoint | MUST | P8 | ⬜ |
| Breached-password check, MFA on sensitive permissions | MUST | P7 | ⬜ |
| Authorization matrix test, every role × endpoint | MUST | P7 | ⬜ |
| Checkout script manifest enforced in CI | MUST | P4 | ⬜ |
| Disclosure policy and `security.txt` (F29) | SHOULD | P10 | ⬜ |
| Sanctions screening hook (F28) | SHOULD | 1.1 | ⬜ |

### 2.16 Entitlements — service delivery

Entirely ➕. Medusa has a 7-flag feature system; this is the commercial layer built on it.
Spec: [ENTITLEMENTS.md](ENTITLEMENTS.md).

| Capability | Priority | Phase | Status |
|---|---|---|---|
| `Entitlements` resolved at boot, fail-closed validation | ➕ MUST | P0 | ⬜ |
| Trial → Free → Pro state machine | ➕ MUST | P2 | ⬜ |
| Trial starts on first product created | ➕ MUST | P2 | ⬜ |
| Day-23 / day-29 warnings, daily evaluation job | ➕ MUST | P2 | ⬜ |
| Quota gates: products, staff seats | ➕ MUST | P2 | ⬜ |
| **Window gates**: analytics history horizon | ➕ MUST | P2 | ⬜ |
| Feature gates: payment gateway, courier, fraud score | ➕ MUST | P9 | ⬜ |
| **Grandfathering** — over-quota data stays live | ➕ MUST | P2 | ⬜ |
| COD / offline payment provider | ➕ MUST | P4 | ⬜ |
| Web push notification provider (VAPID) | ➕ MUST | P8 | ⬜ |
| Reporting — 4 SQL aggregates, window-gated | ➕ MUST | P8 | ⬜ |
| Onboarding + spreadsheet column mapping | ➕ MUST | P9 | ⬜ |
| Social checkout (WhatsApp pre-fill, clipboard fallback) | ➕ MUST | P9 | ⬜ |
| Social draft queue + convert-to-order | ➕ MUST | P9 | ⬜ |
| Web push on order.placed, skipping social conversions | ➕ MUST | P8 | ⬜ |
| Gates are mount-point only; no `Entitlements` in modules | ➕ MUST | P0 | ⬜ |
| Disabled routes 404, never 403 | ➕ MUST | P9 | ⬜ |
| Downgrade grace state — no data deletion | ➕ MUST | P9 | ⬜ |
| `GET /admin/feature-flags` in Medusa's shape | MUST | P9 | ⬜ |
| Additive `GET /admin/entitlements` | ➕ MUST | P9 | ⬜ |
| Tier matrix test | ➕ MUST | P9 | ⬜ |
| Signed licence file (self-hosted) | ➕ SHOULD | 1.1 | ⬜ |

### 2.17 Operations

| Capability | Priority | Phase | Status |
|---|---|---|---|
| Single binary, no runtime dependency | ➕ | P0 | ⬜ |
| ≤ 60 MB distroless image | ➕ | P10 | ⬜ |
| ≤ 1 s cold start | ➕ | P10 | ⬜ |
| ≤ 128 MB idle / ≤ 400 MB at 500 rps | ➕ | P10 | ⬜ |
| Embedded, advisory-locked migrations | MUST | P0 | ⬜ |
| Expand/contract migration phasing | ➕ | P0 | ⬜ |
| OpenTelemetry traces/metrics/logs | ➕ | P0 | ⬜ |
| Graceful shutdown with workflow checkpoint | MUST | P1 | ⬜ |
| Medusa v2 database importer | ➕ | P10 | ⬜ |
| Import dry-run with fidelity report | ➕ | P10 | ⬜ |

---

**‡ Clean-room required.** RBAC and SSO/OIDC are Medusa **Enterprise Edition** — proprietary, not
MIT. These rows are satisfied by independent implementation from requirements and public specs.
They may not be ported, adapted, or referenced, and Medusa fixtures cannot be used to test them.
See [ADR-008](DECISIONS.md#adr-008).

---

## 3. Deliberate exclusions

| Item | Reason |
|---|---|
| ➖ Runtime plugin loading | [ADR-001](DECISIONS.md#adr-001) — compile-time extensibility |
| ➖ Admin API *improvements* in 1.0 (cursor pagination, RFC 9457) | [ADR-002](DECISIONS.md#adr-002) — would break the forked dashboard. Deferred to an additive v3 surface |
| ➖ Non-Postgres databases | [ADR-005](DECISIONS.md#adr-005) |
| ➖ Embedded JS execution for hooks | Compiled runtime; use a workflow step |
| ⬜ Medusa admin extension API (widgets, routes, custom fields, layouts, i18n) | **No longer excluded.** Since the dashboard is now forked rather than rebuilt ([ADR-002](DECISIONS.md#adr-002)), the `admin-vite-plugin` extension system comes with it. Verify it works against Stheno in P9 |
| ➖ Multi-tenancy in core | [PRD NG2](PRD.md) — matches Medusa |
| ➖ B2B, subscriptions | Out of 1.0 scope; matches Medusa core |

Two of these are worth separating, because they are not the same kind of exclusion:

- **Runtime plugins are structural.** A compiled binary has no `require()`, and Rust has no stable
  ABI, so there is no mechanism to add. Reversing this means adopting a WASM host ([ADR-001](DECISIONS.md#adr-001)).
- **Admin extensions are scope, not physics.** Medusa's widget system is build-time codegen, and an
  equivalent could be built for the Next.js admin. It is excluded from 1.0 because it is a
  multi-week frontend project, not because it is impossible.

Both are what an existing Medusa user will actually feel. Say so early in an adoption conversation
rather than letting it be discovered during migration.

---

## 4. Gate for 1.0

- [ ] Every **MUST** row is ✅
- [ ] Store parity suite: 100% of recorded fixtures pass
- [ ] All 7 latency budgets met on reference hardware
- [ ] Memory ≤ 400 MB at 500 rps for 1 hour
- [ ] Coverage ≥ 85%; money and totals paths at 100%
- [ ] An unmodified Medusa storefront runs against Stheno
- [ ] A production-shaped Medusa database imports with exact financial totals
- [ ] Penetration test: no high or critical findings
- [ ] **F1, F2, F3 closed** — CSRF, erasure, audit log
- [ ] PCI SAQ A complete, including the script-attack eligibility criterion
- [ ] EAA / WCAG 2.2 AA conformance verified on storefront and admin
- [ ] Authorization matrix test green across every role, endpoint, and ownership case
