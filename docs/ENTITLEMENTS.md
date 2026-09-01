# Entitlements — Feature Gating and Plan Tiers

**Status:** Draft v1.0 · **Model:** one deployment per store, config-driven entitlements
**Companion:** [TRD §3](TRD.md) (modular monolith) · [DECISIONS ADR-009](DECISIONS.md#adr-009)

This document exists because the modular monolith was mistaken for a feature-toggle mechanism. It
is not. Module boundaries are what make gating *clean*; this is the layer that actually gates.

---

## 1. The two layers

| Layer | What it is | When it acts | Gives you |
|---|---|---|---|
| **Modular monolith** ([TRD §3](TRD.md)) | Source organisation, CI-enforced boundaries | Build time | Safe change, bounded blast radius, *a place to put a gate* |
| **Entitlements** (this doc) | Runtime config declaring what a deployment may do | Startup + request time | Turning a feature off for a customer |

House wiring and breaker switches. You need both, and only the first one existed.

---

## 2. Deployment model

**One database, `tenant_id` on every row, isolation enforced by Postgres RLS.** One binary and
one process serve every tenant. [ADR-012](DECISIONS.md#adr-012) supersedes ADR-009, which is
what this section used to describe.

```
shop-a.example.com ─┐
shop-b.example.com ─┼─→ one stheno process → one postgres, RLS by tenant_id
shop-c.example.com ─┘     tier resolved per request, not at boot
```

Consequences, stated plainly:

- **Isolation is a database policy, not a deployment boundary.** Every tenant-owned row carries
  `tenant_id` and every table has `FORCE ROW LEVEL SECURITY`. The "every query needs scoping"
  cost is paid once, in the database, rather than in each of 127 entities.
- **Entitlements are per-tenant rows, resolved per request.** A single process serves Trial,
  Free and Pro simultaneously, so boot-time route mounting cannot express a gate. That is what
  `mount_gated_for_request` exists for (§6.1). `STHENO_TIER` survives only as a fallback when
  no tenant is resolvable.
- **Engine updates do not scale with customers.** One binary deploy and one migration run moves
  every tenant together. This is the cost model ADR-009 could not offer and the reason it was
  superseded.
- **The N-cost moved to the storefront, and has not been paid.** `STHENO_FRONTEND` binds at
  *build* time with its own `distDir`, so each bespoke theme is a separate Next build, and
  nothing yet maps a tenant's host to the right one. Invisible at two frontends; a manual
  release checklist at five. See `.orchestration/task-archetypes-and-platform-design/spec.md`
  §C2.

> `ponytail:` this section previously described one binary and one database per store, and said
> "N upgrades are still N upgrades". That was true under ADR-009 and false since ADR-012.
> Corrected rather than deleted, because the superseded model is the one people remember.

---

## 3. The governing rule

> **The free tier does nothing that costs marginal money.**
> **The paid tier is exactly the set of features that cost money to run.**

This is a cost line, not a greed line — which is what lets the free tier stay **indefinite**
without bleeding the business. Every tier question resolves against it: if a capability costs
nothing per merchant per month, it is free; if it burns gateway fees, courier API calls, compute,
or support, it is Pro.

Applied consistently it also produces the right *product* answer, not just the right commercial
one: web push is free because it is a browser standard with no vendor and no per-message cost, and
analytics history is gated by *window* rather than switched off because storage and query cost
scale with the window, not with the feature.

---

## 4. The three states

**Trial and Free are states of one account, not separate products.** There is no Enterprise tier.

| Capability | Trial (14 days) | Free (after trial) | Pro |
|---|---|---|---|
| Dashboard, order management | Full | Full | Full |
| Inventory tracking | Full | Full | Full |
| Products | Unlimited | **100** (existing grandfathered) | Unlimited |
| Staff seats | 10 | **Owner + 2** | 10 |
| Storefront | supported brand kit + subdomain | supported brand kit + subdomain | developer-built bespoke storefront + custom domain |
| Order placement + COD | Yes | Yes | Yes |
| Spreadsheet import / CSV export | Yes | Yes | Yes |
| Analytics | Basic | Basic, **30-day window** | **Full history** |
| Web push notifications | Yes | Yes | Yes |
| **Payment gateway** | No | No | **Yes** |
| **Courier dispatch** | No | No | **Yes** |
| **Buyer fraud / risk score** | No | No | **Yes** |
| **Brand kit** | Yes | Yes | Yes |
| **Bespoke storefront service** | No | No | **Yes, after payment** |

Pro is sold on a **6- or 12-month commitment**, never month-to-month. That is defensible only
because the free tier is indefinite: a merchant runs a real store and takes real orders before
being asked for anything. Commercial terms live in the business plan, not here — what the engine
needs from them is a **term end date** on the entitlement.

### 4.1 What is never gateable

Core commerce, per the rule in §3 — none of it costs marginal money:

`product` · `cart` · `order` · `payment` (offline/COD) · `tax` · `currency` · `store` · `customer` ·
`auth` · `inventory` · `fulfillment` (manual) · `sales-channel` · `region` · `file` ·
`notification` (web push, in-dashboard) · `event-bus` · `workflow-engine` · `locking` · `caching` ·
`index` · `query` · `links`

### 4.2 Three kinds of gate

The first two were already specified. **The third came from this tier plan and is new.**

| Kind | Mechanism | Used by |
|---|---|---|
| **Feature gate** | Route group not mounted | `payment_gateway`, `courier_dispatch`, `fraud_score`, `custom_domain` |
| **Quota gate** | Atomic counter, reject at limit | `max_products` (100), `max_staff_seats` (3) |
| **Window gate** | Query bounded by a time horizon | `analytics_window_days` (30 vs unlimited) |

**Window gates** are the cheapest and most under-used tier lever: the feature works identically,
the query simply carries `WHERE created_at >= now() - $window`. Zero branching in domain logic, and
the cost genuinely scales with the window, so it satisfies §3 honestly.

### 4.3 Engine gates vs product gates

Not every row above is a Stheno capability. Stated plainly so nothing is silently assumed:

| Gate | Enforced where |
|---|---|
| `max_products`, `max_staff_seats` | **Engine** — quota counters |
| `analytics_window_days` | **Engine** — reporting queries |
| `payment_gateway` | **Engine** — provider registration; COD-only when off |
| `courier_dispatch` | **Engine** — fulfillment provider registration |
| `custom_domain` | **Platform** — routing/TLS layer above the engine |
| `fraud_score` | **Platform** — a scoring service the engine calls, not a module |

Platform gates read the same `Entitlements` value; they are simply enforced outside the binary.

---

## 5. Trial mechanics

Three details here are load-bearing, and getting any of them wrong damages trust in a way that is
hard to recover.

**The 14-day trial starts when an email-verified merchant submits a store request, not at account
signup or first product creation.** A draft is intentionally free to resume. The submitted request
is the point at which Stheno begins review and provisioning work, so it is the honest start point.

**At day 14 the account moves to Free. Nothing is deleted, nothing is hidden.** A merchant with 400
products keeps all 400 — they are grandfathered, visible, and sellable. They are simply blocked
from creating the 401st until they upgrade or delete. **Removing a seller's live products is an
unrecoverable trust failure.**

This is exactly the grace state already specified in §6.4: `used > limit` means reads and updates
continue and creates are rejected. The trial expiry path and the downgrade path are **the same
code**, which is the right outcome — one mechanism, tested once.

**Warnings at day 7 and day 13**, by web push and email. Both are zero marginal cost.

Trial state is evaluated by a **daily job**, not computed per request:

```
entitlement_state
  tier             text          -- 'trial' | 'free' | 'pro'
  trial_started_at timestamptz   -- set on store-request submission, once, never reset
  trial_ends_at    timestamptz   -- trial_started_at + 14 days
  term_ends_at     timestamptz   -- Pro commitment end, null otherwise
  state            text          -- 'active' | 'grace'
```

The daily job moves `trial → free` at expiry, emits the day-23 and day-29 warnings, and writes an
audit-log entry on every transition. A silent tier change is a support ticket nobody can
reconstruct.

**Pro term expiry** follows the same path: `pro → free`, grandfathered, nothing deleted.

---

## 5b. Configuration

Tiers are config, shipped with the binary and overridable per deployment. Adding or retuning a tier
must never require a release.

```toml
# config/tiers.toml

[trial]
duration_days = 14
starts_on     = "store_request_submitted"    # never on account signup
features      = ["spreadsheet_import", "csv_export", "web_push", "analytics_basic", "branding"]
quotas        = { max_products = 0, max_staff_seats = 10 }   # 0 = unlimited
windows       = { analytics_window_days = 0 }                # 0 = unlimited

[free]
features      = ["spreadsheet_import", "csv_export", "web_push", "analytics_basic", "branding"]
quotas        = { max_products = 100, max_staff_seats = 3 }  # owner + 2
windows       = { analytics_window_days = 30 }

[pro]
features      = ["spreadsheet_import", "csv_export", "web_push", "analytics_basic",
                 "payment_gateway", "courier_dispatch", "fraud_score", "custom_domain"]
quotas        = { max_products = 0, max_staff_seats = 10 }
windows       = { analytics_window_days = 0 }
term_months   = [6, 12]                       # no month-to-month
```

`0` means unlimited for both quotas and windows — one sentinel, so a missing value can never
accidentally mean "unrestricted".

```bash
STHENO_TIER=free
STHENO_QUOTA_MAX_PRODUCTS=250      # negotiated exception, additive
```

Resolution: **tier defaults → env overrides → daily-job state**. Resolved at boot and on each daily
evaluation into an immutable `Entitlements` value. An unknown key or malformed quota **aborts
startup** — a typo must never silently grant or deny a feature.

> **Superseded in part by [5d](#5d-the-writable-layer-adr-016).** Under [ADR-012](DECISIONS.md#adr-012)
> the TOML above ships inside a container image, so editing it *is* a release — the exact thing this
> section promised would never be needed. It remains the shipped default; the writable layer is below.

---

## 5c. What this tier plan needs that the engine does not have

Adopting these tiers commits to four capabilities that are **not in the Stheno spec today**. Named
here rather than assumed, because each is real work:

| Needed by | Capability | Status | Rough size |
|---|---|---|---|
| Free + Trial + Pro | **COD / offline payment provider** — order placed unpaid, fulfil *before* payment, manual "mark as paid", no gateway to refund against | Medusa ships a `system` no-op provider to build on | Small |
| All tiers | **Web push provider** (VAPID) — a new `NotificationProvider`. No vendor, no per-message cost, no app | Not specified | ~1 week |
| Free vs Pro split | **Reporting/analytics module** — the current `analytics` module *emits events*; it does not produce reports. There is no revenue, sales-over-time, or top-products query anywhere in the spec | **Missing entirely** | ~2–3 weeks |
| All tiers | **Spreadsheet import** (`.xlsx`/`.csv`) with dry-run preview | PRD FR-CAT-7, currently SHOULD | ~1 week |

**The reporting module is the significant one.** Analytics is the Free-vs-Pro differentiator in the
tier table, and the engine currently has no reporting capability at all. That gap must be closed
before the tier plan is sellable, and it is not a small addition.

`courier_dispatch` and `fraud_score` are Pro-only and can follow 1.0 — nothing in Free or Trial
depends on them.

---

## 5d. The writable layer (ADR-016)

`config/tiers.toml` says what a plan means **by default**. `platform.tier_definition` says what it
means **on this deployment**, and the control plane writes it. A tier with no row still resolves
entirely from the TOML, so an untouched deployment behaves exactly as it did before this table
existed.

```
platform.tier_definition
  tier        text PRIMARY KEY   -- 'trial' | 'free' | 'pro'
  features    jsonb              -- ["csv_export", "payment_gateway"]
  quotas      jsonb              -- {"max_products": 100}   0 = unlimited
  windows     jsonb              -- {"analytics_window_days": 30}
```

**A row replaces that tier wholesale.** Not a merge — a merge needs an answer for *"the row omits
`max_products`, is that unlimited or unspecified?"*, and every answer to that is a plan silently
granting more than the operator believes it does.

Per-tenant exceptions gain the axis they were missing. `quota_overrides` and `window_overrides`
already existed on `platform.subscription` (5b's env overrides, persisted); `feature_overrides`
completes the set:

```json
{ "payment_gateway": true, "fraud_score": false }
```

An explicit `true` grants a feature the plan does not include, an explicit `false` revokes one it
does, and an absent key leaves the tier's answer standing.

### Resolution order

```
config/tiers.toml
  → platform.tier_definition   (replaces the tier, if a row exists)
  → subscription quota/window overrides
  → subscription feature overrides
  → Entitlements
```

Read **per request**, alongside the subscription row the tenant layer already reads. That costs one
small query and buys an edit that is live on every node immediately, with no cache to invalidate —
a stale node would be serving a plan nobody is on.

### Where validation moved

5b aborts startup on an unknown key. The write API rejects the request instead, so the bad key never
reaches the table at all. Startup validation stays as the backstop for the TOML; it is no longer the
first line of defence for anything an operator types.

### Control-plane API

| Route | Does |
|---|---|
| `GET /platform/tiers` | Every plan, plus the **catalog** — every feature, quota and window key the engine can gate, so the editor renders from the engine rather than from a list in TypeScript |
| `PUT /platform/tiers/{tier}` | Replaces a plan definition. Unknown key → 400 naming it |
| `DELETE /platform/tiers/{tier}` | Drops the row; the plan means what the TOML says again |
| `GET /platform/tenants/{id}/subscription` | The stored row **and** what it resolves to, so an operator is never editing two JSON blobs and guessing at their product |
| `PATCH /platform/tenants/{id}/subscription` | Tier, status, trial and term dates, and all three override blobs. Absent field leaves it alone; explicit `null` clears a date |

All of it sits behind the operator credential on `/platform/*`. **None of it is reachable from
`/admin/*`**: a merchant who can define what Pro includes is a merchant on Pro for free. The
merchant's own `GET /admin/entitlements` (7.2) stays read-only.

---

## 6. Enforcement

### 6.1 The rule that keeps this clean

> **A gate is a mount-point decision, never an `if` inside domain logic.**

Gated routes are not mounted. Gated workflows are not registered. Gated subscribers are not
attached. Domain code has **no idea entitlements exist** — no `if entitlements.has(...)` inside a
service method, ever.

This is what buys sub-feature granularity (returns vs exchanges) without the combinatorial mess
that in-domain branching creates. It is also why the modular monolith matters: a capability that
lives in one route group and one flow crate can be unmounted; one smeared across the codebase
cannot.

```rust
// crates/api/stheno-api-router/src/lib.rs
let mut app = Router::new().merge(core_routes());

if ent.has(Feature::Promotions)     { app = app.merge(promotion_routes()); }
if ent.has(Feature::OrderReturns)   { app = app.merge(return_routes()); }
if ent.has(Feature::Rbac)           { app = app.merge(rbac_routes()); }
```

A request to an unmounted route returns Medusa's **404** shape. Not 403 — a 403 confirms the
feature exists and invites probing.

Enforced by `scripts/check-no-domain-gates.sh`, which fails the build if `Entitlements` is
referenced anywhere under `crates/modules/`.

### 6.2 Quota enforcement

Counting rows on every create is a full scan waiting to happen. Use a counter table, incremented in
the **same transaction** as the insert:

```sql
CREATE TABLE entitlement_usage (
    resource text PRIMARY KEY,        -- 'products', 'users', 'regions'
    used     bigint NOT NULL DEFAULT 0
);
```

```sql
UPDATE entitlement_usage
   SET used = used + 1
 WHERE resource = $1 AND ($2::bigint IS NULL OR used + 1 <= $2)
RETURNING used;
```

Zero rows returned means the quota is exhausted → reject. This is the same atomic pattern as
campaign budgets ([DATA_MODEL §2.5](DATA_MODEL.md)) and for the same reason: read-then-write allows
overrun under concurrency.

Deletes decrement. A nightly job reconciles counters against actual counts and alerts on drift —
counters lie eventually, and finding out from a customer is the wrong way.

Enforcement lives in a **workflow step** (`CheckQuotaStep`), not in the service, so it obeys §5.1.

### 6.3 Over-quota errors

```json
{ "type": "not_allowed",
  "message": "Product limit reached (100). Upgrade to Pro for unlimited products." }
```

Medusa's error shape, so the forked dashboard renders it without modification
([ADR-002](DECISIONS.md#adr-002)). The upsell lives in the message.

### 6.4 Downgrade and grandfathering

A customer on Pro with 4 regions downgrades to Basic, which allows 1. What happens?

**Never delete data.** Destructive, irreversible, and they may upgrade back next month.

The rule: **existing data stays readable and editable; new creates are blocked.**

| State | Behaviour |
|---|---|
| `used <= limit` | Normal |
| `used > limit` | **Grace.** Reads work, updates work, creates rejected. Admin shows a persistent banner naming the resource and the overage |
| Feature removed while data exists | Routes unmount; data stays in the database untouched, and returns intact on re-upgrade |

Grace state is visible in `GET /admin/entitlements` so the dashboard can render it, and it is
logged to the audit log on transition — a silent downgrade is a support ticket you will not be able
to reconstruct.

---

## 7. Discovery API

### 7.1 Medusa-compatible flags — already works

```
GET /admin/feature-flags   →  { "feature_flags": { "rbac": true, "promotions": false, … } }
```

Medusa's own endpoint and shape. **The forked dashboard already reads it** to show and hide screens
— that plumbing comes free with the fork, and it is why Medusa's Enterprise features
(`use-require-rbac-feature.tsx`) hide cleanly.

### 7.2 Additive entitlements endpoint

Quotas and grace state have no Medusa equivalent, so this is a new endpoint. Additive is safe — the
dashboard ignores what it does not read ([ADR-002](DECISIONS.md#adr-002)).

```
GET /admin/entitlements
{
  "tier": "free",
  "trial_ends_at": null,
  "term_ends_at": null,
  "features": ["spreadsheet_import", "csv_export", "web_push", "analytics_basic"],
  "quotas": [
    { "key": "max_products",    "limit": 100, "used": 412, "state": "grace" },
    { "key": "max_staff_seats", "limit": 3,   "used": 2,   "state": "ok" }
  ],
  "windows": [
    { "key": "analytics_window_days", "limit": 30 }
  ]
}

`max_products` in `grace` with `used > limit` is the grandfathered case from §5: all 412 products
stay live and sellable; only creation is blocked.
```

The dashboard fork adds a settings screen rendering this, plus the grace banner. That is the only
new admin UI entitlements require.

---

## 8. Testing

| What | Requirement |
|---|---|
| Every gateable feature | A test asserting its routes 404 when off and work when on |
| Every quota | A test asserting creation is rejected exactly at the limit, not at limit+1 |
| Quota concurrency | 100 parallel creates against a limit of 10 → exactly 10 succeed |
| Downgrade | Data survives, reads work, creates reject, banner state exposed |
| Tier matrix | Each tier boots, mounts exactly its declared routes, and no others |
| No domain gates | `scripts/check-no-domain-gates.sh` fails on a planted `Entitlements` reference in a module |
| Config validation | Unknown feature key or malformed quota aborts startup |
| Write validation (5d) | An unknown key in a plan definition or a per-tenant override is a 400 and is **not stored** |
| Plan edit reaches the merchant | Redefining a plan changes what a merchant on it resolves to, on the next request, without a restart |
| Overrides are per merchant | A grant to one merchant leaves every other merchant on that plan unchanged |
| Control-plane access | An absent or wrong operator token 404s — never 401, and never 200 |

The tier matrix test is the one that catches drift: as routes are added, it fails loudly when a new
route is mounted unconditionally that should have been gated.

The last four are one test, `crates/stheno-server/tests/subscription_journey.rs`, and deliberately
so: what they prove is that an edit made in the control plane arrives in a merchant's request path,
which no step can assert on its own.

---

## 9. What this deliberately does not include

- **No multi-tenancy.** One deployment per store ([PRD NG2](PRD.md)).
- **No usage-based metering.** Entitlements say what is *allowed*; billing systems say what is
  *owed*. Every quota here is a cap, not a meter, and nothing counts consumption for a price.
  **Amended by [ADR-017](DECISIONS.md#adr-017) (2026-08-21):** the original bullet also said
  *"integrate a billing provider; do not build one here"*, which was correct about the engine
  and was read as a decision about the product. No provider can collect BDT from bKash or
  Nagad, so subscription invoicing and collection are first-party and live in the `platform`
  schema. The warning still holds for metering, revenue recognition and multi-currency
  dunning — none of those is built.
- **No in-domain feature branching** (§5.1).
- **No licence server.** Config now, signed licence file when self-hosting becomes real (§4.3).
- **No per-user or per-role feature gates.** That is RBAC's job, and it is a separate axis.
