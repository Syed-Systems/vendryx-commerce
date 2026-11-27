# Technical Requirements Document — Stheno Commerce Engine

**Status:** Draft v1.1 · **Baseline:** `medusajs/medusa@b733400` · **Last updated:** 2026-09-02
**Companion docs:** [DATA_MODEL.md](DATA_MODEL.md) · [API_SPEC.md](API_SPEC.md) · [WORKFLOWS.md](WORKFLOWS.md) · [PAYMENTS.md](PAYMENTS.md) · [ENTITLEMENTS.md](ENTITLEMENTS.md) · [SECURITY.md](SECURITY.md) · [DECISIONS.md](DECISIONS.md)

> **ADR currency.** §1–17 were written on 2026-08-15 against ADR-001–008 and describe the engine
> accurately. Nineteen ADRs have landed since. The one that changes this document materially is
> [ADR-012](DECISIONS.md#adr-012) — multi-tenancy — which §1.2 now covers; §13 carries the
> deployment consequences. Where any section below still implies one deployment per merchant,
> [DECISIONS.md](DECISIONS.md) wins.

---

## 1. Architectural overview

Stheno keeps Medusa's four load-bearing architectural ideas and re-expresses each in Rust:

| Medusa concept | What it does | Stheno equivalent |
|---|---|---|
| **Modules** — isolated domains, no cross-module FKs | Bound the blast radius of a schema change | One crate per module. A module crate may not depend on another module crate. Enforced in CI |
| **Module links** — join tables owned by neither side | Compose modules without coupling them | `stheno-links` owns all 22 link tables plus 14 read-only link definitions |
| **Workflows** — durable SAGA with compensation | Make multi-module mutations atomic-ish | `stheno-workflow`: a Postgres-checkpointed orchestrator, same state machine |
| **Query / RemoteQuery** — graph reads across modules | One call fetches a cross-module object graph | `stheno-query`: a planner that splits a field selection into per-module batched fetches, joined in memory |

Everything else — HTTP, validation, DI, ORM — is replaced outright, because the Rust idiom is better than a transliteration of the TypeScript one.

### 1.1 System context

```
                        ┌──────────────────────────┐
   Browser ────────────►│  apps/storefront (Next)  │──┐
                        └──────────────────────────┘  │
                        ┌──────────────────────────┐  │  HTTP/JSON
   Operator ───────────►│  apps/admin (Next)       │──┤
                        └──────────────────────────┘  │
                                                      ▼
   Carrier / PSP ──webhooks──────────────► ┌────────────────────────┐
                                           │   stheno-server        │
                                           │   (single binary)      │
                                           │                        │
                                           │  axum router           │
                                           │   ├ api-store  (v2 compat)
                                           │   ├ api-admin  (v3)    │
                                           │   └ api-auth           │
                                           │  ──────────────────    │
                                           │  flows (249 workflows) │
                                           │  workflow engine       │
                                           │  query engine          │
                                           │  34 module services    │
                                           │  16 providers          │
                                           │  scheduler │ subscribers│
                                           └───────┬────────┬───────┘
                                                   │        │
                                            ┌──────▼──┐  ┌──▼──────┐
                                            │Postgres │  │ Redis   │
                                            │(required)│  │(optional)│
                                            └─────────┘  └─────────┘
```

One process contains the API, the workflow engine, the scheduler, and the subscribers. Horizontal scaling runs N identical processes; Postgres advisory locks and `SKIP LOCKED` queues arbitrate who does background work. There is no separate worker deployment, and no leader election service.

**Optional worker mode.** `stheno-server --role=api` and `--role=worker` split the responsibilities for large deployments. Default is `--role=all`, and small deployments should stay there.


### 1.2 Tenancy — the shape of the whole system

Added 2026-09-02, recording [ADR-012](DECISIONS.md#adr-012) (Accepted, superseding
[ADR-009](DECISIONS.md#adr-009)). This section did not exist while the engine was single-tenant
and its absence made the rest of this document read as if it still is.

**One database. `tenant_id` on every tenant-owned row. Isolation enforced by Postgres, not by
the application.**

```
shop-a.example.com ─┐
shop-b.example.com ─┼─→ one stheno process ─→ one postgres, RLS by tenant_id
shop-c.example.com ─┘      tier resolved per request, never at boot
```

| Layer | Mechanism |
|---|---|
| Identify the tenant | `Host` header → `stheno_platform::service::tenant_for_domain`. An unknown host **404s**, so the surface cannot be probed to enumerate merchants |
| Scope the connection | `tenant_layer` sets the tenant GUC once per checkout, in the same round trip that opens the transaction |
| Enforce the scope | Every tenant-owned table carries `FORCE ROW LEVEL SECURITY`. `FORCE` matters: without it the table owner bypasses the policy, and the migration role is the owner |
| Resolve entitlements | Per request, from `entitlement_state` — one process serves Trial, Free and Pro simultaneously, so boot-time route mounting cannot express a gate (§6.1 of [ENTITLEMENTS.md](ENTITLEMENTS.md)) |

**Why the database and not the application.** The "every query needs scoping" cost is paid once,
in Postgres, rather than 127 times in Rust and again in every future query. A missed `WHERE
tenant_id = $1` in one handler is then a query that returns nothing, not a query that returns
another merchant's orders.

**What it costs, stated plainly:**

- **Isolation is a policy, not a boundary.** There is no physical separation to fall back on. An
  RLS policy that is missing, or a table that is `ENABLE` without `FORCE`, is a cross-tenant leak.
  `./scripts/rls_audit` exists for exactly this and is not optional.
- **Anything outside the tenant layer must be deliberate.** The legacy in-repo control plane (`/platform/*`)
  is mounted outside it on purpose and therefore sees every tenant; that is why it has its own credentials and its
  own append-only audit table ([ADR-016](DECISIONS.md#adr-016), [ADR-018](DECISIONS.md#adr-018)). The permanent
  company control plane is **Space** (`D:\Syed Systems\Space`), which integrates externally via signed product connectors ([ADR-030](DECISIONS.md#adr-030)).
- **Caches are a tenancy surface.** Any cache keyed without the tenant serves one merchant's data
  to another. This has been a real defect twice — the storefront region cache and the Next.js
  Data Cache both had to be partitioned by host.
- **Noisy-neighbour effects are now possible** and were not before. One merchant's import can
  contend for the same pool. Bounded by pool sizing and per-tenant quotas, not eliminated.

**What it buys.** One binary deploy and one migration run moves every merchant. Engine updates do
not scale with customer count — the property ADR-009 could not offer, and the reason it was
superseded.

**The N-cost did not vanish; it moved to the storefront and has not been paid.** `STHENO_FRONTEND`
binds a theme at *build* time with its own `distDir`, so each theme is a separate Next build and
nothing yet maps a merchant's host to the right one. [ADR-023](DECISIONS.md#adr-023) (**Proposed**)
answers this with one deployed artifact per theme routed by host, making deploy count O(themes)
and never O(merchants). Until it is accepted, this is a manual release checklist.

---

## 2. Technology stack

| Concern | Choice | Why this one |
|---|---|---|
| Runtime | `tokio` (multi-thread) | The only mature async runtime with the ecosystem this needs |
| HTTP | `axum` 0.8 + `tower` + `tower-http` | Typed extractors, composable middleware, hyper 1.x |
| Database | Postgres 15+ | Lateral joins, `SKIP LOCKED`, jsonb, partial indexes, `NOTIFY` |
| ORM | `diesel` 2.2 + `diesel-async` 0.6 + `deadpool` | Compile-time schema verification across 149 tables (ADR-004) |
| Migrations | `diesel_migrations`, embedded in the binary | No separate migration tool at deploy |
| Money | `rust_decimal` | Exact base-10 arithmetic; no float ever touches a money path |
| IDs | `ulid` with a type prefix | Wire-identical to Medusa (`prod_01J…`); lexically sortable |
| Serialisation | `serde` / `serde_json` | — |
| Validation | `garde` | Derive-based, async-capable, good error paths for the field-level errors the API must return |
| OpenAPI | `utoipa` + `utoipa-axum` | Spec generated from the handlers, so it cannot drift |
| Auth | `jsonwebtoken`, `argon2`, `scrypt`, `totp-rs` | scrypt retained solely to verify imported Medusa hashes |
| Observability | `tracing` + `tracing-opentelemetry` + `opentelemetry-otlp` | Trace, metric, log correlation with no vendor lock |
| Redis | `fred` | Native async, cluster support, good reconnection semantics |
| Cron | `croner` | Cron parsing only; scheduling is ours |
| HTTP client | `reqwest` (rustls) | Provider integrations |
| Testing | `cargo test`, `testcontainers`, `insta`, `proptest`, `wiremock` | Snapshot tests for API parity, property tests for money |
| Load testing | `oha` + `k6` | Budget enforcement in CI |
| Frontend | Next.js 15 (App Router), React 19, TypeScript, Tailwind 4, TanStack Query/Table | — |
| Storefront rendering (ADR-038, in progress) | `minijinja` 3 templates rendered by the engine; themes in `themes/<theme-id>/` | Replaces the Next.js storefront at task T4.14 |

**Deliberately excluded:** `sqlx` (ADR-004), `wasmtime` (ADR-001), Temporal (ADR-003), GraphQL, and any dynamic scripting engine.

**Frontend note:** the Next.js/Tailwind row above describes the *storefront*. The admin is a forked React + Vite SPA ([ADR-002](DECISIONS.md#adr-002)), not a Next.js app.

---

## 3. Workspace layout — a modular monolith

**One binary, one database, hard internal boundaries.** Modules are compile-time-isolated crates
that cannot reach into each other; composition happens in exactly four declared channels. This is
the property that makes 149 tables and 34 domains survivable, and it is enforced by CI rather than
by review.

```
stheno/
├── Cargo.toml                      # workspace root, shared deps via [workspace.dependencies]
├── crates/
│   ├── stheno-server/              # bin: THE composition root. Only crate that sees everything
│   ├── stheno-cli/                 # bin: migrate, seed, new-module, import-medusa
│   │
│   ├── core/                       # infrastructure. Knows no domain.
│   │   ├── stheno-types/           # shared kernel: Money, Id<T>, Error. Depends on NOTHING
│   │   ├── stheno-db/              # pool, tx manager, FilterSpec macro, pagination
│   │   ├── stheno-events/          # EventBus trait + impls. No event types
│   │   ├── stheno-workflow/        # SAGA engine
│   │   ├── stheno-links/           # 22 link tables + 36 definitions. IDs only, no module types
│   │   ├── stheno-query/           # cross-module read planner
│   │   ├── stheno-http/            # extractors, error rendering, middleware
│   │   └── stheno-audit/           # append-only audit log
│   │
│   ├── modules/                    # 34 LEAF crates. Zero lateral edges. May depend on core/* only
│   │   ├── stheno-product/  stheno-pricing/  stheno-cart/  stheno-order/
│   │   ├── stheno-inventory/ stheno-fulfillment/ stheno-payment/ stheno-promotion/
│   │   ├── stheno-tax/ stheno-region/ stheno-customer/ stheno-auth/ stheno-user/
│   │   ├── stheno-rbac/ stheno-api-key/ stheno-sales-channel/ stheno-stock-location/
│   │   ├── stheno-store/ stheno-currency/ stheno-file/ stheno-notification/
│   │   ├── stheno-search/ stheno-index/ stheno-caching/ stheno-locking/
│   │   ├── stheno-analytics/ stheno-translation/ stheno-settings/ …
│   │   └── stheno-ai/              # Tenant-owned static social commerce knowledge (ADR-032)
│   │
│   ├── providers/                  # 25 crates. Each depends on ONE module (its trait owner)
│   │   ├── stheno-payment-stripe/  stheno-payment-{bkash,nagad,sslcommerz,offline}/
│   │   ├── stheno-fulfillment-{manual,pathao,redx,steadfast}/
│   │   ├── stheno-file-{s3,local}/  stheno-caching-redis/  stheno-search-local/
│   │   ├── stheno-notification-{local,resend,sendgrid,webpush}/
│   │   ├── stheno-auth-{emailpass,github,google,oidc}/
│   │   ├── stheno-ai-openai-compatible/  stheno-cdn-cloudflare/
│   │   └── stheno-meta-insights/  stheno-search-console/
│   │
│   ├── flows/                      # COMPOSITION LAYER for writes. Split by domain, not one crate
│   │   ├── stheno-flows-common/    # shared steps: query, links, events, locking, transform
│   │   ├── stheno-flows-catalog/   # product, category, inventory, stock-location, channel
│   │   ├── stheno-flows-pricing/   # pricing, price-list, tax, region, store, currency
│   │   ├── stheno-flows-cart/      # cart, promotion, line-item
│   │   ├── stheno-flows-payment/   # payment, payment-collection
│   │   ├── stheno-flows-order/     # order (68), fulfillment, draft-order
│   │   ├── stheno-flows-identity/  # auth, user, customer, invite, api-key, rbac
│   │   └── stheno-flows-platform/  # file, notification, settings, translation
│   │
│   ├── subscribers/                # ALL event handlers. The only place that reacts to events
│   │   └── stheno-subscribers/
│   │
│   └── api/                        # COMPOSITION LAYER for HTTP. Split by domain
│       ├── stheno-api-core/        # shared DTO helpers, response envelopes
│       ├── store/  stheno-api-store-{catalog,cart,order}/
│       ├── admin/  stheno-api-admin-{catalog,orders,pricing,identity,config,platform}/
│       ├── stheno-api-auth/
│       └── stheno-api-router/      # mounts every router. Depends on all api-* and nothing else
│
├── migrations/<module>/<timestamp>_<name>/{up.sql,down.sql}
├── apps/
│   ├── storefront/                 # forked Next.js starter
│   └── admin/                      # forked React+Vite dashboard
└── tests/{parity,integration,load}/
```

**Why `flows` and `api` are split.** In the first draft both were single crates depending on all 34
modules. That is a monolith inside the modular monolith: every module change recompiles a crate
holding 249 workflows or 278 routes, and a crate that already depends on everything is exactly
where boundaries quietly erode. Splitting them by domain bounds the recompile blast radius and
makes each composition unit state its own dependencies honestly.

### 3.1 The four channels — and nothing else

Modules are leaves. They never talk to each other directly. Cross-module interaction happens
through exactly four declared channels:

| # | Channel | Direction | Lives in |
|---|---|---|---|
| **1** | **Workflows** — orchestrated writes with compensation | write | `flows/*` |
| **2** | **Query** — graph reads across link definitions | read | `stheno-query` |
| **3** | **Events** — asynchronous reaction | notify | emitted by the module, handled **only** in `subscribers/` |
| **4** | **Links** — ID-to-ID association tables | join | `stheno-links` |

**Forbidden, and CI-enforced:**

- A `modules/*` crate depending on another `modules/*` crate.
- SQL — including `sql_query` — touching a table outside the module's own schema (§3.4).
- A module handling another module's event. **Subscribers live in `subscribers/`, never in a
  module.** This closes a hole the first draft left open: if `stheno-search` subscribed to
  `ProductCreated`, it would need `stheno-product` in its dependencies, and the isolation rule
  would be defeated by the event system.
- Shared mutable state between modules. `AppContext` is assembled in `stheno-server` and handed
  down; modules never reach back up into it.

### 3.2 The dependency graph

```
stheno-types ◄──────────────────────────────── everything

core/*       ◄── modules, providers, flows, subscribers, api

modules/*    ◄── flows/*  (many-to-many, declared per flow crate)
             ◄── subscribers
             ◄── api/*    (read paths and DTO mapping only)
             ◄── providers (exactly one module each)

flows/*      ◄── api/*, subscribers
api/*        ◄── stheno-api-router
everything   ◄── stheno-server
```

Enforced by `scripts/check-module-isolation.sh`, which parses `cargo metadata` and fails on:

1. any `modules/* → modules/*` edge;
2. any `modules/* → flows/*` or `modules/* → api/*` edge (upward dependency);
3. any crate outside `stheno-server` depending on more than one *layer* above its own;
4. `stheno-types` gaining any dependency at all.

A violation is a **build failure**, not a review comment.

### 3.3 The shared kernel is deliberately starved

`stheno-types` is depended on by every crate, which makes it the classic rot vector: one domain
type lands there "temporarily," and within a year it is a second god-crate that couples everything.

Rules, CI-checked:

- **Zero dependencies.** Not `diesel`, not `axum`, not any module.
- **Primitives only:** `Money`, `CurrencyCode`, `Id<T>`, `Error`, `Metadata`, timestamps.
- **No domain types.** No `Product`, no `Order`, no `CartTotals`. If two modules need the same
  domain concept, that is a signal the boundary is wrong — fix the boundary, do not widen the kernel.
- **No business logic** beyond the arithmetic of the primitives themselves.

`scripts/check-shared-kernel.sh` fails the build if `stheno-types` gains a dependency or exceeds a
declared type budget.

### 3.4 The boundary is physical, not just structural

The crate graph stops Rust code crossing a module boundary. It does **not** stop SQL. A single
`sql_query` joining `product` to `price` would defeat the entire scheme while compiling cleanly —
and the pricing hot path uses `sql_query` by design ([ADR-004](DECISIONS.md#adr-004)), so this is a
live risk rather than a theoretical one.

So the boundary is enforced in Postgres too: **one schema per module.**

```sql
CREATE SCHEMA product;   CREATE SCHEMA pricing;   CREATE SCHEMA "order";   -- …
```

- Diesel `table!` declarations are schema-qualified; a module's `schema.rs` contains only its own.
- Link tables live in a `links` schema, owned by `stheno-links`.
- `scripts/check-sql-boundaries.sh` scans every `sql_query` literal and fails on a schema prefix
  the module does not own.
- The query engine reads across schemas legitimately — that is its job, and it is the only crate
  permitted to.

**Cost, stated honestly:** every table reference carries a schema prefix, migrations create schemas,
and the import tool maps Medusa's single-schema layout onto ours. That is a real ergonomic tax. It
buys a boundary that raw SQL cannot silently cross, which the crate graph alone cannot give you.

### 3.5 What the boundaries buy commercially

Stheno is sold as a service with plan tiers, and the module boundary is what makes a capability
*unmountable*. A feature that lives in one route group, one flow crate, and one module can be
switched off by not mounting it. A feature smeared across the codebase cannot be switched off at
all without branching inside domain logic.

So the boundaries have a second job beyond maintainability: they are the seams along which the
product is sold. The gating layer itself is specified in [ENTITLEMENTS.md](ENTITLEMENTS.md), and its
central rule — **a gate is a mount-point decision, never an `if` inside a service** — is only
enforceable because the boundaries exist.

`crates/modules/` may not reference `Entitlements` at all. CI-enforced by
`scripts/check-no-domain-gates.sh`.

### 3.6 Extraction posture — what "modular monolith" does not promise

Modular monoliths are often sold as "microservices later." That claim is mostly false, and
pretending otherwise leads to designs that are worse at both.

**What is true here:** the module boundary is real, so a *leaf* module with few links —
`stheno-notification`, `stheno-file`, `stheno-analytics`, `stheno-search` — could be lifted into a
service without redesign, because nothing joins to it and it participates in no cross-module
transaction.

**What is not true:** `order`, `cart`, `payment`, `inventory`, and `pricing` **cannot** be
extracted cheaply. They share transactions through the workflow engine, and `complete-cart` depends
on a single Postgres transaction spanning several of them. Extracting any one of those turns
compensation from a local rollback into a distributed one, which is a different and much harder
system.

**Extraction is therefore a non-goal.** The boundaries exist to make *this* codebase maintainable —
bounded blast radius, independent testing, comprehensible ownership — not to enable a migration
nobody has scheduled. Judge the structure on whether it makes changes safe today.

---

## 4. Module architecture

### 4.1 Anatomy of a module crate

```
stheno-product/
├── MODULE.toml          # the module contract — machine-checked, see 4.1.1
├── src/
│   ├── lib.rs           # the public surface: service, DTOs, events. Nothing else is `pub`
│   ├── models.rs        # Diesel structs. pub(crate) — never leaves the module
│   ├── schema.rs        # diesel table! macros, schema-qualified `product.*` (generated, committed)
│   ├── service.rs       # ProductService — the only public write surface
│   ├── repo.rs          # query construction. private
│   ├── filters.rs       # FilterSpec impls
│   ├── events.rs        # event *types* it emits. It subscribes to nothing
│   ├── dto.rs           # what crosses the boundary. Distinct from models.rs
│   └── error.rs         # ProductError → stheno_types::Error
└── tests/               # integration tests against a testcontainer
```

**`models.rs` is `pub(crate)`.** Diesel entities never escape the module — that is what stops a
consumer binding to the storage shape and silently coupling to the schema. What crosses the
boundary is `dto.rs`, which the module owns and versions.

#### 4.1.1 The module contract

Every module declares what it owns and publishes. Negative rules (no lateral edges) are not enough;
a boundary you cannot read is a boundary nobody respects.

```toml
# stheno-product/MODULE.toml
name        = "product"
schema      = "product"                      # its Postgres schema. Owns these tables exclusively
owns        = ["product", "product_variant", "product_option", "product_option_value",
               "product_product_option", "product_product_option_value", "product_image",
               "product_variant_product_image", "product_category", "product_collection",
               "product_tag", "product_type"]
publishes   = ["product.created", "product.updated", "product.deleted",
               "product_variant.created", "product_variant.updated"]
links       = ["product_sales_channel", "product_variant_price_set",
               "product_variant_inventory_item", "product_shipping_profile"]
service     = "ProductService"
providers   = []
```

`scripts/check-module-contracts.sh` verifies that the declared tables match the migrations, the
declared events match what the code emits, and that no table is claimed by two modules.

### 4.2 The service pattern

Medusa generates a service from a `MedusaService` factory with `list/retrieve/create/update/delete` per model. Stheno does this with a derive macro, which is the same idea with compile-time output:

```rust
#[derive(StthenoService)]
#[service(models(Product, ProductVariant, ProductOption, ProductTag))]
pub struct ProductService {
    db: DbPool,
    events: Arc<dyn EventBus>,
}
```

The macro generates, per model: `list`, `list_and_count`, `retrieve`, `create`, `create_many`, `update`, `upsert`, `delete`, `soft_delete`, `restore`. Hand-written methods on the same `impl` block override generated ones for domain logic.

**Transaction handling.** Every service method takes an explicit connection:

```rust
pub async fn create(&self, conn: &mut Conn<'_>, input: CreateProduct) -> Result<Product>
```

The caller owns the transaction. This is the opposite of Medusa's ambient transaction context, and it is deliberate: a workflow step that spans two modules needs one transaction across both, and an ambient context makes that invisible. Explicit connections make transaction boundaries a compiler-visible fact.

### 4.3 The registry — compile-time DI

There is no container, no string-keyed resolution, no `resolve("productModuleService")`. `AppContext` is a plain struct built once at startup:

```rust
pub struct AppContext {
    pub product: ProductService,
    pub pricing: PricingService,
    pub cart: CartService,
    // … 34 fields
    pub payment_providers: ProviderMap<dyn PaymentProvider>,
    pub workflow: WorkflowEngine,
    pub query: QueryEngine,
    pub events: Arc<dyn EventBus>,
}
```

Handlers receive `State<Arc<AppContext>>`. Cost: a pointer deref. Medusa's equivalent is a hash lookup and a proxy object per request.

**Provider selection stays dynamic**, because it must be — which payment provider handles a session is a database value, not a compile-time fact. `ProviderMap<T>` is a `HashMap<String, Arc<dyn T>>` populated at startup from config, with the set of *available* providers fixed at compile time and the *chosen* one resolved at runtime. That is the correct place for the only dynamic dispatch in the system.

### 4.4 Adding a module

`stheno new-module discount-codes` generates the crate, a migration directory, a service skeleton, an events file, a test file, and the registry entry. Wiring is one line in `stheno-server`. Documented in [AGENTS.md](../AGENTS.md).

---

## 5. Data layer

### 5.1 Conventions

Every table carries:

| Column | Type | Note |
|---|---|---|
| `id` | `text` PK | `<prefix>_<ULID>` — wire-identical to Medusa |
| `created_at` | `timestamptz` | default `now()` |
| `updated_at` | `timestamptz` | maintained by trigger |
| `deleted_at` | `timestamptz` NULL | soft delete; every default query filters `IS NULL` |
| `metadata` | `jsonb` NULL | user-extensible bag, present on all customer-facing entities |

Cross-module references are `text` columns with **no foreign key constraint** — the module boundary is the reason. Intra-module references are real FKs with `ON DELETE` semantics.

### 5.2 Money

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money { amount: Decimal, currency: CurrencyCode }
```

Rules, enforced by the type:

- No `From<f64>`. No `as f64`. Not in tests either.
- Stored as `numeric(20,6)` plus a `currency_code` column. Never `float8`, never integer minor units.
- Arithmetic across differing currencies fails to compile where static, returns `Err` where dynamic.
- Rounding happens **once**, at presentation, to the currency's ISO 4217 minor unit, using banker's rounding.
- Allocation across lines uses largest-remainder distribution so that split amounts sum **exactly** to the original.

Medusa stores a `raw_amount` jsonb alongside each numeric column to preserve `BigNumber` precision. Stheno writes that column on export for compatibility but treats `numeric` as the source of truth.

### 5.3 Dynamic filtering

Roughly 200 fields are filterable across list endpoints, with operators (`$eq $ne $in $nin $gt $gte $lt $lte $like $ilike`), nested field paths, and `$and`/`$or` groups. Diesel's typed builder does not do this natively, so P0 builds it:

```rust
#[derive(FilterSpec)]
#[filter(table = products)]
pub struct ProductFilter {
    #[filter(ops(eq, in, like))]      pub title: Option<StringFilter>,
    #[filter(ops(eq, in))]            pub status: Option<EnumFilter<ProductStatus>>,
    #[filter(ops(gt, gte, lt, lte))]  pub created_at: Option<DateFilter>,
    #[filter(relation = "variants")]  pub variants: Option<Box<VariantFilter>>,
}
```

The macro emits a function that folds a parsed filter tree onto a `BoxedSelectStatement`. Only declared field/operator pairs compile, so an unknown filter is a 400 with a precise message rather than a silent full scan.

**This macro is on the critical path and is R3 in the PRD risk register.** P0 must validate it against the 20 hardest list endpoints — products with variant/price/category filters, orders with item/customer/region filters — before the rest of the codebase commits to it. If it cannot express them cleanly, fall back to a hand-written query builder per module. Decide by end of P0, not in P5.

### 5.4 Connection management

- `deadpool` pool, default max 20 per instance.
- Read-only endpoints may use a replica pool when `DATABASE_REPLICA_URL` is set.
- Transaction isolation: `READ COMMITTED` by default; `SERIALIZABLE` for inventory reservation and promotion-budget consumption, with a bounded retry on serialisation failure.
- Statement timeout 5 s for API requests, 60 s for workflow steps, unlimited for migrations.

### 5.5 Migrations

Forward-only, one directory per module, embedded via `diesel_migrations::embed_migrations!`. `stheno migrate` runs them under an advisory lock, so concurrent boots are safe.

**Zero-downtime mode (FR-PLUS-9).** Any migration that is not backward-compatible must be split into `expand` → `backfill` → `contract`, each a separate migration, with a declared compatibility window in a header comment. CI rejects a destructive statement (`DROP COLUMN`, `ALTER TYPE`, `NOT NULL` without default) that is not marked `-- phase: contract` and paired with an earlier expand.

---

## 6. The workflow engine

Full specification in [WORKFLOWS.md](WORKFLOWS.md). Architectural summary here.

### 6.1 What it must reproduce

Medusa's orchestrator supports, and Stheno's must too:

- Sequential and parallel steps, with a compensation function per step
- `maxRetries`, `retryInterval`, `autoRetry`, `retryIntervalAwaiting`, `maxAwaitingRetries`
- `timeout` per step and per workflow
- `async` steps that suspend until `set_step_success` / `set_step_failure` arrives externally
- `backgroundExecution` — async steps that complete themselves
- `continueOnPermanentFailure`, `skipOnPermanentFailure` (bool or a step name to resume at)
- `noCompensation`, `compensateAsync`, `noWait`, `nested` sub-transactions
- Durable state: `not_started → invoking → waiting_to_compensate → compensating → done | reverted | failed`

### 6.2 Rust shape

Steps are types, not closures — this keeps input/output types checkable and lets the engine store them:

```rust
#[async_trait]
pub trait Step: Send + Sync {
    type Input:  Serialize + DeserializeOwned + Send;
    type Output: Serialize + DeserializeOwned + Send;

    const NAME: &'static str;
    const CONFIG: StepConfig = StepConfig::DEFAULT;

    async fn invoke(&self, ctx: &StepContext, input: Self::Input) -> Result<Self::Output>;
    async fn compensate(&self, ctx: &StepContext, output: Self::Output) -> Result<()> { Ok(()) }
}
```

A workflow is a declarative DAG built by a macro that preserves type-checking between steps:

```rust
workflow! {
    name: "complete-cart",
    input: CompleteCartInput,
    steps: {
        validate    = ValidateCartStep(input.cart_id),
        reserve     = ReserveInventoryStep(validate.items)              [retries: 2, timeout: 10s],
        authorize   = AuthorizePaymentStep(validate.payment_collection) [timeout: 30s],
        create      = CreateOrderStep { cart: validate.cart, reservations: reserve },
        finalize    = [ EmitOrderPlacedStep(create.order),
                        MarkCartCompletedStep(validate.cart_id) ],      // parallel
    },
    output: create.order,
}
```

The macro emits a `StepDefinition` tree plus a typed executor. Type mismatches between a producer and a consumer step are compile errors.

### 6.3 Durability

State lives in `workflow_execution`:

| Column | Purpose |
|---|---|
| `id`, `workflow_id`, `transaction_id` | identity; `transaction_id` is the idempotency key |
| `state` | the transaction state enum |
| `context` | `jsonb` — step inputs, outputs, and the compensation stack |
| `attempts`, `errors` | retry accounting |
| `run_at` | next scheduled attempt (retry backoff, awaiting steps) |
| `locked_by`, `locked_until` | instance ownership |

Every checkpoint is one `UPDATE`. Workers claim executable work with:

```sql
SELECT … FROM workflow_execution
WHERE state IN ('invoking','compensating') AND run_at <= now()
  AND (locked_until IS NULL OR locked_until < now())
ORDER BY run_at
FOR UPDATE SKIP LOCKED LIMIT 10
```

That single query is why no Redis and no Temporal are required. A crashed instance's lock expires and another instance resumes from the last checkpoint.

**Checkpoint cost.** One write per step is the price of durability, and it is the engine's dominant cost. Workflows marked `store: false` (pure-read flows) skip persistence entirely; the checkout workflow does not, because losing it costs money.

### 6.4 Idempotency

`transaction_id` is unique per workflow. Starting a workflow with an existing `transaction_id` returns the existing execution rather than starting a second one. This is what makes FR-CRT-7 (no double-charge on retried checkout) structural rather than defensive.

---

## 7. The query engine

### 7.1 The problem

`GET /store/products?fields=*variants.calculated_price,*categories` must return products (product module), their variants (product module), the calculated price of each variant (pricing module, via the `product_variant_price_set` link), and categories — as one JSON graph, with no module importing another.

### 7.2 Design

```
field selection string
   ↓ parse
FieldTree
   ↓ plan against the link registry
QueryPlan := FetchNode{ module, entity, fields, filters, children[] }
   ↓ execute level by level
per-level: tokio::try_join! across sibling modules; one batched query per node
   ↓ join
hash-join on link keys, in memory
   ↓ shape
serde_json::Value matching the requested tree
```

Guarantees:

- **One query per node per level.** Never per parent row. N+1 is structurally impossible; the planner batches parent ids into a single `WHERE id = ANY($1)`.
- **Depth limit 5, node limit 30**, configurable. Over the limit is a 400, not a slow query.
- **Field allowlist per entity.** An undeclared field is a 400.

### 7.3 The read index

The planner cannot efficiently answer filters that span modules — "products in sales channel X with a price under 50 in region Y, sorted by price" needs a join the module boundary forbids. Medusa solves this with the `index` module; Stheno does the same.

`index_data` holds a denormalised, flattened projection of configured entities. Domain events drive incremental updates; `index_sync` tracks per-entity watermarks and supports full rebuild. Cross-module filter and sort resolve against `index_data` first, returning ids, which the planner then hydrates.

Staleness is bounded and observable: index lag is a published metric, and admin list views that depend on it show a freshness indicator rather than pretending to be live.

---

### 7.4 The storefront read path

Recorded because "Postgres will be slow for visitors, let us add MongoDB for the storefront" is a
recurring instinct, and the answer should not have to be re-derived.

**It is one database with four layers in front of it, not two databases.**

```
CDN / ISR                       most storefront traffic never reaches the app
  ↓ miss
Cache            stheno-caching · in-process or Redis
  ↓ miss
Read index       stheno-index   · denormalised, jsonb + GIN (§7.3)
  ↓ miss
Read replica     DATABASE_REPLICA_URL, when configured
  ↓
Primary tables   source of truth
```

**`stheno-index` is the document store.** A flattened projection built for fast filtering across
module boundaries — the job a document database would be brought in to do — except it updates in
the same transaction as the write, so there is no dual-write consistency problem and no second
system to operate.

#### Why not a separate NoSQL store for the storefront

| Claim | Reality |
|---|---|
| "Postgres is slow for catalog reads" | A 300-product catalog is a few MB and lives in shared buffers. The query is microseconds. The real storefront bottleneck is images, JS, TLS, and network — the database is not in the top four |
| "Document reads are faster" | The expensive query is *product + variants + calculated price for region/channel/group* — a join with specificity ranking (§2.2 of DATA_MODEL). One CTE in Postgres; `$lookup` or mass denormalisation in a document store, both worse |
| "It scales better" | Dual writes create a distributed-systems problem the moment one side fails after the other commits |
| Money | Never in a document store. [ADR-006](DECISIONS.md#adr-006) |

#### Cache invalidation

Event-driven, never TTL-guessed for catalog data. A subscriber on `product.*`, `price.*`, and
`inventory_level.*` evicts by key prefix. TTL is a backstop, not the mechanism — a merchant who
edits a price and cannot see it on their own storefront concludes the software is broken.

#### When a second store *is* justified

On measurement, never on speculation:

| Trigger | Add | Already possible? |
|---|---|---|
| Full-text search degrades past ~50k products | Meilisearch / Typesense | **Yes** — the `SearchProvider` trait exists for exactly this |
| Analytics outgrows aggregate queries at millions of orders | ClickHouse | Additive; nothing to redesign |
| Cache or session pressure | Redis | Already an optional dependency |

All three are additive. None require the data model to change, which is the property that makes
deferring them safe.

---

## 8. HTTP layer

### 8.1 Routing

Three routers mounted on one axum app:

| Mount | Crate | Contract |
|---|---|---|
| `/store/*` | `api/store/*` | **Frozen.** Medusa v2 wire-compatible |
| `/admin/*` | `api/admin/*` | **Frozen.** Medusa v2 wire-compatible — the forked dashboard depends on it. Additions must be additive |
| `/auth/*` | `stheno-api-auth` | Medusa-compatible shape (`/auth/:actor_type/:provider`) |
| `/hooks/*` | `stheno-api-store-order` | Provider webhooks |
| `/health`, `/metrics` | server | Liveness, readiness, Prometheus |

### 8.2 Middleware order

```
TraceLayer → RequestId → Cors → BodyLimit(2MB) → RateLimit
  → Authenticate → ResolveScope (publishable key → sales channels)
  → Authorize (RBAC, admin only) → Idempotency → Handler
```

`ResolveScope` resolves a publishable API key to its sales channels and filters every store query by them. It is applied by the router, not by handlers, so a new endpoint cannot forget it.

**A publishable key is not authentication.** It ships in browser JavaScript and is public by
construction. It narrows *which channel's* data is visible; it never establishes *who* is asking.
Store endpoints are public by design, and anything customer-specific requires the customer JWT and
the `Owned<T>` ownership extractor ([SECURITY §7](SECURITY.md)). Treating the key as a credential
is the mistake this note exists to prevent.

### 8.3 Error model

One internal error enum maps to both response formats:

```rust
pub enum Error {
    NotFound { entity: &'static str, id: String },
    Invalid { code: &'static str, message: String, field: Option<String> },
    Unauthorized, Forbidden { permission: &'static str },
    Conflict { code: &'static str, message: String },
    ProviderFailed { provider: String, source: BoxError },
    Internal(BoxError),
}
```

**Both surfaces** render Medusa's shape: `{ "type": "not_found", "message": "…" }`, with matching status codes. The RFC 9457 renderer was withdrawn with [ADR-002](DECISIONS.md#adr-002) — it would break the forked dashboard's error handling.

Field-level validation detail is still produced internally and rendered into the `message` string in Medusa's format, so no diagnostic information is lost even though the envelope is flatter.

`Internal` never leaks its source to the client; it logs with the trace id and returns a correlation id.

### 8.4 Validation

`garde` derives on request DTOs, run by an extractor before the handler. A validation failure produces field-level errors, so the admin UI can attach messages to inputs — the thing Medusa's Zod errors do well and which the redesign must not regress.

---

## 9. Events, jobs, subscribers

**Event bus.** `trait EventBus { async fn emit(&self, events: Vec<Event>) -> Result<()>; }` with in-process (tokio broadcast) and Redis Streams implementations. Events emitted inside a transaction buffer until commit — no event fires for a rolled-back write. At-least-once delivery; subscribers must be idempotent, and that requirement is in AGENTS.md.

**Subscribers.** Registered at compile time in `stheno-server`. A failing subscriber retries with backoff and lands in a dead-letter table after exhaustion. A subscriber never blocks the request that emitted the event.

**Scheduled jobs.** Cron expressions parsed by `croner`; a Postgres advisory lock per job name ensures single execution across instances. Jobs are workflows, so they get durability and observability for free.

Jobs that must exist at 1.0, because each closes a silent-failure hole: payment reconciliation
(nightly, [PAYMENTS §9](PAYMENTS.md)), authorization-expiry warning (T−48h), abandoned-3DS session
sweep (30 min), search and read-index sync, and workflow-execution retention pruning.

> **Status 2026-09-02 — the scheduler exists as of today; three of the five jobs above do not.**
> Until this afternoon `crates/core/` contained no scheduler crate and nothing invoked periodic
> work at all. It now lives in `crates/stheno-server/src/scheduler.rs`: cron schedules via
> `croner`, single execution across instances via a Postgres advisory lock keyed on the job
> name and taken with zero wait, a panicking job isolated on its own task, and an unparseable
> schedule refused at boot rather than logged and skipped.
>
> Two jobs are registered — `payment-reconcile` (every 15 minutes) and `subscription-dunning`
> (daily 03:00). Both iterate tenants explicitly, because the methods they call read
> tenant-owned tables under RLS and an unscoped sweep reports a clean pass over an empty
> result. The remaining three jobs above, and the trial → Free transition from
> [ENTITLEMENTS §5](ENTITLEMENTS.md), have no domain method to call yet.
>
> What was inert before it existed:
>
> | Logic | Where it lives | What its absence means |
> |---|---|---|
> | `reconcile_unsettled_requests` | `stheno-payment` | A payment left `in_flight` by a crash converges only if the same idempotency key is retried. After a crash nothing retries, so the row sits forever |
> | Dunning ladder advance | `stheno-platform` (`dunning_ladder.rs`, `dunning_is_sole_writer.rs`) | Invoices generate and the ladder is correct, but no clock advances it. Non-payment is never noticed and no subscription is ever suspended for it |
> | Trial → Free transition | [ENTITLEMENTS §5](ENTITLEMENTS.md) specifies a daily job | A 14-day trial does not end on its own |
>
> The first two now run. The third still needs its domain method. It was one missing
> component, not three bugs — which is why one module closed two of them.
> **A convergence path that never runs is not a convergence path.**

---

## 10. Providers

Each pluggable concern is a trait owned by its module. Providers implement it in their own crate and are registered at startup.

```rust
#[async_trait]
pub trait FulfillmentProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOption>>;
    async fn validate_option(&self, data: &Json) -> Result<bool>;
    async fn calculate_price(&self, ctx: ShippingContext) -> Result<Money>;
    async fn create_fulfillment(&self, ctx: FulfillmentContext) -> Result<FulfillmentResponse>;
    async fn cancel_fulfillment(&self, data: &Json) -> Result<()>;
    async fn create_return(&self, ctx: ReturnContext) -> Result<FulfillmentResponse>;
    async fn get_documents(&self, data: &Json) -> Result<Vec<Document>>;
}
```

**Payments are the exception and get their own specification.** `PaymentProvider` is 11 required
methods plus a 7-method optional `PaymentVault` trait — matching Medusa's 18-method interface —
and the trait is the least of it. Outbound idempotency, webhook/response convergence, the
minor-unit money boundary, and ledger reconciliation are all load-bearing and none of them are
visible in a trait signature. See **[PAYMENTS.md](PAYMENTS.md)**.

Traits to ship: `PaymentProvider` + `PaymentVault`, `FulfillmentProvider`, `TaxProvider`, `NotificationProvider`, `FileProvider`, `AuthProvider`, `SearchProvider`, `CacheProvider`, `LockProvider`, `AnalyticsProvider`, `EventBus`.

Every provider trait also exposes a `capabilities()` probe so the module can branch on what a
provider actually supports (partial capture, async settlement, authorization TTL) instead of
attempting an operation and interpreting the failure.

Provider traits are **semver-stable within a major version** (FR, §4.4). Adding a method is a breaking change unless it has a default implementation.

---

## 11. Frontend architecture

### 11.1 Admin (`apps/admin`) — forked, not built

Medusa's dashboard, forked from `packages/admin/dashboard`: React 18, Vite, React Router, TanStack Query and Table. It talks to `/admin/*`, which speaks Medusa's dialect exactly, so it works unmodified.

What changes in the fork:

1. **Enterprise paths removed and rebuilt clean-room** ([ADR-008](DECISIONS.md#adr-008)): `routes/policies/`, `routes/roles/`, `hooks/api/rbac-*`, `use-require-rbac-feature`, `components/common/required-permissions-section/`, `lib/permissions/`, `providers/permissions-provider/`, `login/components/sso-login.tsx`.
2. **Rebranding** — name, logo, colour tokens.
3. **Workflow-execution timeline deepened** for FR-PLUS-1.
4. **`@medusajs/js-sdk` retained as the client**, which proves compatibility by construction.

The fork point is a **pinned Medusa version**, never `develop`. Upstream is merged deliberately, on our schedule — that is how the permanent-chase risk in ADR-002 is bounded.

### 11.2 Storefront (`apps/storefront`)

Forked from Medusa's Next.js starter and kept close to upstream, so the compatibility claim stays testable. RSC for catalog with ISR, client components for cart and checkout, `@medusajs/js-sdk` as the client — proving G2 by construction rather than by assertion.

Page inventory and data contracts: [SITEMAP.md](SITEMAP.md).

---

## 12. Observability

- **Tracing.** One span per request, propagated through workflow → step → SQL. Baggage carries `actor_id`, `actor_type`, `transaction_id`.
- **Metrics.** RED per route; workflow executions by state; step duration histograms; pool utilisation; index lag; provider call latency and error rate.
- **Logs.** JSON, `tracing-subscriber`, correlated by trace id, with an explicit redaction allowlist.
- **Workflow observability (FR-PLUS-1).** `GET /admin/workflow-executions/:id` returns the full step timeline with inputs, outputs, errors, and compensation history — and the admin UI renders it. This is the single biggest operational advantage over Medusa, where a failed workflow is largely opaque.

---

## 13. Deployment

**Artifact.** `FROM gcr.io/distroless/cc-debian12`, statically linked where musl is used, ≤ 60 MB. Non-root. No shell.

**Configuration.** Environment variables only, parsed into a typed `Config` at boot; unknown or malformed values abort startup rather than defaulting silently.

| Variable | Required | Note |
|---|---|---|
| `DATABASE_URL` | yes | — |
| `DATABASE_REPLICA_URL` | no | enables read routing |
| `REDIS_URL` | no | absent ⇒ in-process cache, Postgres locks, in-process events |
| `JWT_SECRET`, `COOKIE_SECRET` | yes | boot fails if under 32 bytes |
| `STHENO_ROLE` | no | `all` (default) \| `api` \| `worker` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | no | absent ⇒ stdout logs only |
| `STORE_CORS`, `ADMIN_CORS`, `AUTH_CORS` | yes | — |

**Scaling.** API is stateless — scale on CPU. Worker concurrency is bounded by pool size. Postgres is the first bottleneck; add a read replica before adding app nodes.

**Health.** `/health/live` (process up) and `/health/ready` (pool reachable, migrations current). Readiness fails during migration, so rolling deploys hold traffic correctly.

### 13.1 Running it as a hosted service

Added 2026-09-02. The table above describes one process; this describes the fleet that actually
serves merchants.

**One deploy serves everyone.** Per [§1.2](#12-tenancy--the-shape-of-the-whole-system), a release
is one binary rollout and one migration run regardless of merchant count. A migration that is not
backward-compatible with the running binary takes every merchant down at once — expand/contract is
mandatory here in a way it was not under one-deployment-per-store.

**The transitional control plane is off unless credentialed.** `STHENO_PLATFORM_TOKEN` unset means the entire
in-repo `/platform/*` surface 404s. Fails closed, never open by default. Named `platform.operator`
credentials are the real identities; the environment token is a bootstrap and break-glass path
only, audited as `op_bootstrap`. Note that company control plane architecture has transitioned to
**Space** (`D:\Syed Systems\Space`, [ADR-030](DECISIONS.md#adr-030)).

**Connection pooling is transaction-mode** ([ADR-025](DECISIONS.md#adr-025), **Proposed**), with a
session-mode route reserved for the paths that carry session state. This interacts directly with
§1.2: the tenant GUC is set per transaction, so a transaction-mode pooler is safe only because the
scope is established inside the transaction that uses it, never on the connection.

**Compute sits next to the data; a CDN sits in front of the origin, never as the runtime**
([ADR-024](DECISIONS.md#adr-024), **Proposed**). The storefront is SSR on the **Node** runtime, not
edge — not a preference but a constraint: the theme binding at `apps/storefront/next.config.js` is
a webpack `NormalModuleReplacementPlugin` hook, and any bundler or adapter that does not read it
(Turbopack, `@cloudflare/next-on-pages`) will serve the wrong theme's markup while reporting a
successful build. `scripts/check-frontend-switch.sh` exists because that failure has happened.

**Docker Compose now, Kubernetes only at scale** ([ADR-026](DECISIONS.md#adr-026), **Proposed**).

Deployment manifests live in [deploy/](../deploy); the operator runbook is
[RUNBOOK_CONTROL_PLANE.md](RUNBOOK_CONTROL_PLANE.md) and [DEPLOYMENT.md](DEPLOYMENT.md).

---

## 14. Testing strategy

| Layer | Tool | Gate |
|---|---|---|
| Unit — domain logic, totals, rules | `cargo test` | 85% line coverage; 100% on money and totals |
| Property — money, allocation, tax rounding | `proptest` | Line sums equal totals exactly, every currency precision |
| Integration — per module against real Postgres | `testcontainers` | Every module, every phase |
| **Parity — Store API vs recorded Medusa** | `insta` snapshots | 100% pass, from P2 onward |
| Concurrency — inventory, promo budget, idempotency | custom harness | No oversell, no budget overrun, no double-charge, under 100-way concurrency |
| Workflow durability | kill-and-resume harness | Every compensating workflow survives a SIGKILL at every step boundary |
| Load — budget enforcement | `k6` | Every §7.1 budget, nightly |
| Frontend | Playwright | Priya's five jobs, end to end |

### 14.1 The parity corpus

The mechanism behind G2 and the top-line success metric. A recording proxy sits in front of a seeded Medusa v2 instance, captures request/response pairs for every Store endpoint across a scripted scenario set, and stores them as fixtures. Stheno replays them against the identical seed and diffs, with an explicit allowlist for legitimately variable fields (ids, timestamps).

Build this in **P0**, before there is anything to test. Retrofitting it after the API exists is how parity projects fail — you end up asserting what you built rather than what Medusa does.

---

## 15. Performance strategy

Where the budgets in PRD §7.1 actually come from:

1. **No per-request allocation of the module graph.** `Arc<AppContext>` is built once. Medusa constructs a scoped container per request.
2. **Batched, planned reads.** The query engine issues a bounded number of queries per request, known at plan time.
3. **Pricing in one query.** `calculated_price` for N variants resolves in a single CTE-based query with rule specificity ranked in SQL, not by fetching candidate prices and ranking in application code.
4. **No serialisation of intermediate representations.** The graph is assembled directly into `serde_json::Value`, once.
5. **Prepared statements.** Diesel prepares and caches per connection.
6. **Compiled totals.** The totals engine is a pure function over a snapshot struct — no I/O inside the calculation loop.

**Non-negotiable:** every phase gate includes running the load suite. A phase that meets its functional gate but misses its latency budget is not complete. Performance regressions are cheapest to fix in the phase that caused them.

---

## 16. Migration from Medusa v2

> **Status after [ADR-027](DECISIONS.md#adr-027) (Proposed).** This importer is an **engine
> capability and an onboarding tool for an individual merchant arriving from Medusa**. It is no
> longer a product promise: "swap the binary, keep your frontends" was addressed to self-hosting
> Medusa operators, and we do not sell to them. Build it when a real merchant needs it — not
> before. The requirement it came from is [PRD G6](PRD.md), which is amended by the same ADR.

`stheno import-medusa --source-url postgres://… --dry-run`

- Maps Medusa's schema to Stheno's table by table. **Preserves all IDs**, so external references, webhooks, and analytics keep working.
- Converts `raw_amount` jsonb + numeric pairs to canonical `numeric(20,6)`, asserting no precision loss and failing loudly if any row disagrees.
- Imports `scrypt-kdf` password hashes as-is. Login verifies against scrypt, then transparently re-hashes to Argon2id (FR-AUTH-10). No password reset email to your entire customer base.
- Rebuilds the read index and search index after import.
- `--dry-run` reports per-table counts, unmapped columns, and any value that would not round-trip, changing nothing.

**Verification gate:** for a given source database, `SELECT` totals for orders, payments, and refunds must match exactly between source and target. Import fails otherwise.

---

## 17. Known limitations

Stated plainly, because a TRD that lists only strengths is marketing.

1. **No runtime plugins.** Adding a provider means recompiling. This is the accepted cost of ADR-001 and it will lose some adopters.
2. **Compile times.** A cold workspace build is minutes, not seconds. Mitigated (R5), not eliminated.
3. **Postgres-only.** By design (NG6), but it forecloses MySQL and SQLite deployments permanently.
4. **Personal data lives in more places than the customer table** — the read index, search index,
   workflow context, and raw webhook store all hold projections of it. Erasure is a cascade, and
   the data inventory in [SECURITY §11.1](SECURITY.md) must be updated whenever a new jsonb column
   can hold request data.
5. **The read index is eventually consistent.** Cross-module filtered lists can lag writes by seconds under heavy load. Bounded and observable, but not zero.
6. **Tenant isolation has no physical fallback.** Under [§1.2](#12-tenancy--the-shape-of-the-whole-system) a missing RLS policy, a table left `ENABLE` without `FORCE`, or a cache key without the tenant in it is a cross-tenant data leak. The mitigations are `./scripts/rls_audit` and review discipline; there is no second wall behind them.
7. **One bad migration takes every merchant down together.** The flip side of "engine updates do not scale with customers". Expand/contract is not a nicety here.
8. **Three of the five scheduled jobs in §9 still have no domain method**, and so does the daily trial → Free transition. The scheduler itself now exists; what it can run is the limit.
9. **Storefront theme selection binds at build time.** Deploy count is currently O(themes) only by hand; nothing yet routes a merchant's host to the right build. [ADR-023](DECISIONS.md#adr-023) is **Proposed** and unimplemented.
6. **The admin inherits Medusa's API design.** Offset pagination can skip or duplicate rows under concurrent writes, and error envelopes are flat. Accepted in [ADR-002](DECISIONS.md#adr-002) in exchange for a working dashboard, and it makes us a permanent downstream consumer of Medusa's admin API decisions.
7. **Rust talent pool.** Smaller than Node's. This is a real staffing constraint on the plan, not a footnote.
