# Architecture Decision Records

Each record states what was decided, what it costs, and what would reverse it. A decision without
a stated cost is a preference wearing a suit.

| # | Decision | Status |
|---|---|---|
| [ADR-001](#adr-001) | Extensibility is compile-time; no runtime plugins | Accepted |
| [ADR-002](#adr-002) | Full wire compatibility, Store **and** Admin. Fork the dashboard | Accepted (revised) |
| [ADR-003](#adr-003) | Durable workflows run on a custom Postgres SAGA engine | Accepted |
| [ADR-004](#adr-004) | Diesel + diesel-async for persistence | Accepted |
| [ADR-005](#adr-005) | Postgres only; no other database backends | Accepted |
| [ADR-006](#adr-006) | Money is decimal, never float or integer minor units | Accepted |
| [ADR-007](#adr-007) | Modular monolith with build-enforced boundaries | Accepted (expanded) |
| [ADR-008](#adr-008) | RBAC and SSO are clean-room, not ported — they are Medusa Enterprise | Accepted |
| [ADR-009](#adr-009) | Plan tiers by entitlements on single-tenant deployments, not multi-tenancy | **Superseded by [ADR-012](#adr-012)** |
| [ADR-010](#adr-010) | `Id<T>` carries its Medusa prefix as a compile-time constant | Accepted |
| [ADR-011](#adr-011) | `FilterSpec` stays single-entity; joins belong to the query engine | Accepted |
| [ADR-012](#adr-012) | One database, `tenant_id` on every row, isolation enforced by Postgres RLS | Accepted |
| [ADR-013](#adr-013) | Storefront presentation is swappable; commerce logic is not | Accepted |
| [ADR-014](#adr-014) | A line item's price comes from the catalogue, never from the request | Accepted |
| [ADR-015](#adr-015) | Wire-shape expansion lives in the API layer, not in module DTOs | Accepted |

---

<a id="adr-001"></a>

## ADR-001 — Extensibility is compile-time

**Status:** Accepted · **Date:** 2026-08-14

### Context

Medusa's extensibility is dynamic. A plugin is an npm package; modules resolve through an Awilix
container by string key; providers load at runtime from config. This is why `medusa-plugin-*`
exists as an ecosystem.

Rust compiles. Reproducing dynamic loading needs either a WASM component host (wasmtime) with a
stable host ABI, or out-of-process providers over gRPC.

### Options considered

| Option | For | Against |
|---|---|---|
| **Compile-time crates** (chosen) | Zero dispatch cost, full type safety, simplest architecture, smallest attack surface | No drop-in plugins; customisation means recompiling |
| WASM component plugins | Third-party plugins without recompiling; sandboxed | A host ABI is a permanent compatibility burden; every host type crosses a serialisation boundary; substantial design cost |
| gRPC sidecar providers | Plugins in any language | N processes to operate; network latency inside the checkout path; hardest operational story of the three |

### Decision

Modules and providers are Rust crates, wired into `AppContext` at startup. Customisation means
depending on Stheno as a library and recompiling.

**The one exception:** *which* provider handles a given payment session is a database value, so
`ProviderMap<dyn PaymentProvider>` resolves by string at runtime. The **set** of available
providers is fixed at compile time; the **choice** among them is not. That is the only dynamic
dispatch in the system, and it is in the right place.

### Consequences

**Good.** No ABI to maintain. No serialisation at module boundaries. Provider trait changes are
compile errors, not runtime surprises. `AppContext` is a struct field access where Medusa does a
container lookup and builds a proxy.

**Bad, and worth naming plainly.** No plugin marketplace — this is PRD risk R4 and it is an
adoption risk, not a technical one. Every deployment that needs a custom provider builds from
source. Merchants who want to install a shipping integration from a registry cannot.

**Mitigation:** 16 first-party providers in core, a `stheno new-module` scaffold, and a
provider-authoring guide, so most users never write one.

### Reversal condition

If adoption data shows recompilation is the primary blocker, a WASM host can be added *behind* the
existing provider traits without changing module code — the traits were designed to make that
possible. It is additive, not a rewrite.

---

<a id="adr-002"></a>

## ADR-002 — Full wire compatibility with Medusa v2, Store and Admin

**Status:** Accepted · **Date:** 2026-08-14 · **Supersedes:** the original "Admin v3 redesign" decision

### Context

The first version of this ADR kept the Store API compatible and redesigned the Admin API — cursor
pagination, RFC 9457 errors, uniform envelopes. That was decided on the assumption we would build
our own admin dashboard, so Medusa's admin API shape bought us nothing.

**That assumption was wrong.** Medusa's admin dashboard is mature, MIT-licensed (outside the
Enterprise carve-out in [ADR-008](#adr-008)), and represents years of accumulated UI work — 38 route
groups, order-edit flows, rule builders, data tables. Rebuilding it in the five weeks the plan
allotted was not realistic.

Reusing it requires the Admin API to speak Medusa's dialect, because the dashboard reads
`{ products, count, offset, limit }` and would receive `{ data, page }`.

### Decision

**Both surfaces are wire-compatible with Medusa v2.** Store and Admin alike emit Medusa's exact
JSON shapes, key names, pagination style, and error format.

The admin dashboard is **forked** from `packages/admin/dashboard`, with all Enterprise paths
excluded and replaced by clean-room equivalents ([ADR-008](#adr-008)). The storefront is forked
from Medusa's Next.js starter.

Improvements are **additive only** — new optional query parameters, new endpoints, new headers.
Never a changed response shape on an existing endpoint.

### Rationale

The backend language is irrelevant to this choice, and that is worth stating because it caused
confusion: **Rust emits either JSON shape with identical effort.** The engine, the workflow
orchestrator, the query planner, and every performance target are unaffected. This decision is
purely about which dialect we speak on the wire.

Given that, the trade is: Medusa's admin API design warts, against a working mature admin on day
one and a genuine drop-in story — swap the binary, keep your storefront *and* your admin.

### Consequences

**Good.**
- A true drop-in replacement. The most persuasive demonstration this project can make, and testable
  rather than assertable.
- The largest risk reduction in the plan: P9 goes from "build a mature admin" to "fork and adapt."
- The parity harness now covers both surfaces, so compatibility is continuously verified.

**Bad, and these are real losses.**
- **G5 "a better admin" is gone.** The forked dashboard is a React + Vite SPA, not Next.js. The
  server-rendered admin, the ≤200 KB JS budget, cursor pagination, and RFC 9457 field-level errors
  are all withdrawn. [PRD §6.1](PRD.md) is rewritten accordingly.
- **Offset pagination on admin lists** can skip or duplicate rows while data changes underneath.
  Inherited, not fixed.
- **Permanent downstream dependency.** Every Medusa admin API change becomes a migration for us.
  Mitigated by pinning a target Medusa version rather than tracking `develop`, and re-recording the
  parity corpus on our schedule.
- Enterprise RBAC and SSO admin screens must be **rebuilt**, not forked — real clean-room work.

### Reversal condition

An additive `/admin/v3/*` surface could be introduced later without breaking the forked dashboard,
and the dashboard migrated onto it gradually. That option stays open precisely because
improvements are additive. It was explicitly deferred rather than rejected.

---

<a id="adr-003"></a>

## ADR-003 — Custom Postgres SAGA engine

**Status:** Accepted · **Date:** 2026-08-14

### Context

249 workflows need durable execution with compensation, retries, timeouts, async suspension, and
crash recovery. Medusa implements its own orchestrator; the alternative is delegating to Temporal
or Restate.

### Options considered

| Option | For | Against |
|---|---|---|
| **Custom on Postgres** (chosen) | No extra infrastructure; exact semantic parity with Medusa's step options; full control; `SKIP LOCKED` is genuinely sufficient | We own the correctness of a distributed state machine |
| Temporal | Proven durability, excellent UI, replay determinism | A mandatory cluster in every deployment, including a one-merchant VPS. Its execution model is replay-based, not compensation-stack-based, so all 249 workflows would need semantic mapping rather than porting |
| Pluggable driver | Flexibility | Two engines to test and keep semantically identical. The abstraction costs more than either engine |

### Decision

A custom orchestrator in `stheno-workflow`, checkpointing to `workflow_execution` in Postgres,
claiming work with `FOR UPDATE SKIP LOCKED`.

### Rationale

The deciding factor is [PRD G4](PRD.md) — operational simplicity. Requiring a Temporal cluster to
run a small store contradicts the reason this project exists. Postgres is already a hard
dependency, and `SKIP LOCKED` plus expiring locks is a complete solution for this workload.

The second factor is semantic parity. Medusa's step options (`skip_on_permanent_failure` taking a
step name to resume at, `background_execution`, `compensate_async`) map directly onto a
compensation-stack orchestrator and awkwardly onto a replay engine. Porting is cheaper and more
faithful than mapping.

### Consequences

**Good.** Zero added infrastructure. Semantics port 1:1. Workflow state is queryable with ordinary
SQL, which is what makes FR-PLUS-1 cheap to build.

**Bad.** We own the correctness of a distributed state machine, which is genuinely hard. One
checkpoint write per step is the throughput ceiling. There is no replay-based debugging.

**Mitigation:** the five engine test gates in [WORKFLOWS §1.9](WORKFLOWS.md) — particularly
kill-and-resume at every step boundary — are non-negotiable P1 exit criteria.

### Reversal condition

If checkpoint write throughput becomes the system bottleneck at scale, the engine is one crate
behind a narrow interface, so a Temporal driver could be introduced then — with real numbers
justifying it, rather than speculatively now.

---

<a id="adr-004"></a>

## ADR-004 — Diesel and diesel-async

**Status:** Accepted · **Date:** 2026-08-14

### Context

127 entities and 22 link tables across 25 modules. The data layer choice affects every module,
every day, and is expensive to reverse.

### Options considered

| Option | For | Against |
|---|---|---|
| **Diesel + diesel-async** (chosen) | Strongest compile-time schema guarantees available; mature; fast; `schema.rs` diffing catches migration drift at build time | Dynamic filtering needs `into_boxed()` and a macro layer; async is a bolt-on rather than native |
| SQLx | Compile-time-checked raw SQL; async-native; full control on hot paths | Verification needs a live database at build time; no schema-drift detection; hand-written SQL for 127 entities of CRUD |
| SeaORM | Fast CRUD scaffolding; ergonomic dynamic queries | Weaker compile-time guarantees; you drop to raw SQL for the hard pricing and promotion queries anyway |

### Decision

Diesel 2.2 with diesel-async 0.6 and deadpool. `diesel::sql_query` with `QueryableByName` for the
pricing resolution and promotion evaluation hot paths.

### Rationale

The dominant risk across 149 tables is **schema drift** — a migration changes a column, a query
still compiles against a stale assumption, and it fails at runtime under load. Diesel's checked-in
`schema.rs` plus `check_for_backend(Pg)` turns that class of bug into a build failure. Over a
multi-year codebase, that is worth more than SQLx's ergonomics.

### Consequences

**Good.** A migration that breaks a query fails `cargo build` (FR-PLUS-6). CRUD is generated by
the service derive macro rather than hand-written 127 times.

**Bad.** Dynamic filtering — roughly 200 filterable fields with nested paths and `$and`/`$or`
groups — does not fall out of Diesel naturally. It requires the `FilterSpec` macro
([TRD §5.3](TRD.md)), which is PRD risk R3 and net new code we own.

**Mitigation:** the macro is built and validated in P0 against the 20 hardest list endpoints,
**before** the rest of the codebase depends on it. If it fails that validation, fall back to
per-module hand-written builders — a decision taken in P0, not discovered in P5.

### Reversal condition

Reversal is expensive and unlikely after P2. The realistic escape valve is per-query, not
wholesale: any endpoint Diesel serves badly drops to `sql_query` while everything else is
unaffected.

---

<a id="adr-005"></a>

## ADR-005 — Postgres only

**Status:** Accepted · **Date:** 2026-08-14

### Decision

Postgres 15+ is the only supported database. No MySQL, no SQLite, no abstraction layer to allow
them later.

### Rationale

The engine depends on Postgres-specific features that have no portable equivalent:
`FOR UPDATE SKIP LOCKED` (the workflow scheduler), lateral joins and CTE ranking (price
resolution), `jsonb` with GIN indexes (the read index and workflow context), partial indexes (soft
deletes), and `numeric` with exact arithmetic (money).

A database abstraction would either forfeit these or degrade to the weakest common denominator,
which is precisely how ORMs make commerce engines slow.

### Consequences

Forecloses MySQL and SQLite deployments permanently, including SQLite-backed local development.
Accepted: `docker compose up postgres` is not a meaningful barrier, and the performance targets in
[PRD §7.1](PRD.md) are not reachable through an abstraction layer.

---

<a id="adr-006"></a>

## ADR-006 — Money is decimal

**Status:** Accepted · **Date:** 2026-08-14

### Decision

`Money { amount: Decimal, currency: CurrencyCode }`, stored as `numeric(20,6)` plus a currency
column. No `f64`. No integer minor units.

### Rationale

Floats are disqualified outright — `0.1 + 0.2 != 0.3` is not acceptable in a payment path.

Integer minor units (Stripe's model) are exact but fail on: currencies with non-2 decimal places
(JPY 0, KWD 3), per-unit prices requiring sub-minor precision, and percentage discount
intermediates. Every one of those appears in this system, and each would need a special case.

Decimal is exact, handles all precisions uniformly, and matches Postgres `numeric` with no
conversion.

### Consequences

Slightly slower than integer arithmetic — irrelevant next to a database round trip. Requires
discipline: no `From<f64>` on `Money`, clippy denies `float_arithmetic` in domain crates, and
allocation across lines uses largest-remainder so split amounts sum **exactly** to the original.
Property-tested at every ISO 4217 precision, because PRD risk R6 is a silent financial error.

Medusa's `raw_amount` jsonb columns are written on export for compatibility and never read as
truth.

### Amendment (2026-08-20) — amounts serialize as JSON strings, deliberately

`rust_decimal` serializes to a JSON **string** (`"120"`, `"25.000000"`), where Medusa returns a
JSON number. That difference is repeatedly mistaken for a wire-compatibility defect, so it is
recorded here as a decision rather than left to be rediscovered.

Emitting a number costs one of two things, and neither is worth it:

- **Via `f64`** (`rust_decimal/serde-float`): reintroduces the exact representation this ADR
  exists to forbid. A total that round-trips as `0.30000000000000004` is not a formatting
  quirk, it is a wrong number in a payment path.
- **Via `serde_json/arbitrary_precision`**: exact, but it is a *global* feature. It changes
  `serde_json::Value::Number` semantics for every crate in the workspace, including code that
  reads `as_f64`/`as_i64`, for a cosmetic gain on the wire.

The string form is lossless and unambiguous, and no client is broken by it: JavaScript coerces
in arithmetic (`original - calculated`), and `Intl.NumberFormat().format("120.000000")` returns
`$120.00` — verified against the storefront's own `convertToLocale` and `getPercentageDiff`.

What is worth fixing is *inconsistency*: the cart returned `"25.000000"` where a product
returned `"25"`, because `PricingService` normalizes scale on the way out and the cart did not.
Scale is a property of the column, not of the price. Compare amounts by value, never by their
JSON token — the parity suites do.

---

<a id="adr-007"></a>

## ADR-007 — Modular monolith with build-enforced boundaries

**Status:** Accepted · **Date:** 2026-08-14

### Decision

Stheno is a **modular monolith**: one binary, one database, one deployment — with module boundaries
enforced by CI rather than by convention.

1. A crate in `crates/modules/` may not depend on another crate in `crates/modules/`.
2. Cross-module interaction happens through exactly **four channels**: workflows (writes), query
   (reads), events (notification), links (association). Nothing else.
3. **Event subscribers live in `crates/subscribers/`, never inside a module** — a subscriber inside
   a module would need the emitter's crate for the event type, defeating rule 1 through the back
   door.
4. Each module owns **one Postgres schema**; raw SQL may not reference another module's schema.
5. `stheno-types` is a starved shared kernel: zero dependencies, primitives only, no domain types.
6. The composition layers — `flows/` and `api/` — are **split by domain**, never single crates.

Enforced by `check-module-isolation.sh`, `check-sql-boundaries.sh`, `check-shared-kernel.sh`, and
`check-module-contracts.sh`. Each has a negative test proving it fails on a planted violation.

### Rationale

This is Medusa's most valuable architectural property and the easiest to lose. In TypeScript it is
maintained by convention and review; in Rust it can be maintained by the build, so it should be.

Without it, a 149-table schema becomes a distributed monolith: every table reachable from
everywhere, and changing a column means auditing the codebase. With it, a module owns its tables
outright and the blast radius of a schema change is one crate — verified by the compiler.

Three refinements came from auditing the first draft, and each closed a hole that would have
hollowed out the boundary while leaving it nominally intact:

- **`flows` and `api` were single crates depending on all 34 modules.** A crate that already
  depends on everything is where boundaries erode, and every module change recompiled 249 workflows
  or 278 routes. Split by domain.
- **Events were an unguarded back door.** Rule 3 closes it.
- **Raw SQL bypassed the crate graph entirely.** The pricing hot path uses `sql_query` by design
  ([ADR-004](#adr-004)), so this was live, not theoretical. Per-module schemas plus a scanner close it.

### Consequences

**Good.** Schema changes are locally reasoned. Modules are independently testable. The dependency
graph stays flat, which materially helps compile times (PRD risk R5). Ownership is legible from
`MODULE.toml` rather than inferred.

**Bad.** Cross-module operations are more verbose — you write a workflow instead of calling a
method. Schema-qualified table references are an ergonomic tax on every query. Newcomers hit both
immediately and experience them as friction.

That friction is the feature: it is the moment the architecture asks whether this really is a
cross-module operation. Usually it is.

### What this decision does not promise

**Extraction to microservices is a non-goal.** Leaf modules with no cross-module transactions
(`notification`, `file`, `analytics`, `search`) could be lifted out without redesign. `order`,
`cart`, `payment`, `inventory`, and `pricing` **could not** — they share Postgres transactions
through the workflow engine, and `complete-cart` spans several in one transaction. Extracting any
of those converts local compensation into distributed compensation, which is a different system.

The boundaries exist to make *this* codebase safe to change, not to enable a migration nobody has
scheduled. Judge the structure on that.

---

<a id="adr-008"></a>

## ADR-008 — RBAC and SSO are clean-room, not ported

**Status:** Accepted · **Date:** 2026-08-14

### Context

Medusa's repository is MIT **except** for an enumerated set of Enterprise Materials in
`ENTERPRISE-LICENSE.md`, which are proprietary and require a commercial agreement with MedusaJS,
Inc. Possession of the source grants no license.

The carve-out covers **RBAC** and **SSO/OIDC** in full:

| Area | Paths |
|---|---|
| RBAC backend | `modules/rbac/`, `core-flows/src/rbac/`, `api/admin/rbac/`, `types/src/rbac/`, `framework/src/policies/`, `check-permissions.ts`, `rbac-field-filter.ts`, `define-policies.ts`, `policy-to-types.ts`, rbac feature flags, both rbac link definitions |
| RBAC user/invite | role-assignment steps and workflows, `api/admin/users/[id]/roles/` |
| RBAC admin UI | `dashboard/src/routes/policies/`, `routes/roles/`, rbac hooks, `lib/permissions/`, `providers/permissions-provider/`, `required-permissions-section/` |
| SSO / OIDC | `providers/auth-oidc/`, `modules/auth-oidc.ts`, `types/src/auth/providers/oidc.ts`, `api/auth/[auth_provider]/user/route.ts`, `sso-login.tsx` |
| Tests | all rbac integration tests |

Compiled or transformed versions are also covered, "even if emitted to another path."

Explicitly **still MIT**: generic authentication, generic auth-provider registration and discovery,
generic user and invite management, shared HTTP and workflow infrastructure, and integration points
that merely register or display the availability of an Enterprise feature.

### Decision

1. **RBAC ships in Stheno, implemented clean-room.** Roles, policies, inheritance, and
   deny-beats-allow are ordinary access-control concepts, not Medusa inventions. They are designed
   from requirements and from public RBAC literature (NIST RBAC), **not** by reading Medusa's
   implementation.
2. **OIDC/SSO ships, implemented clean-room** from the OpenID Connect specification. `emailpass`,
   Google, and GitHub providers are outside the carve-out and may be ported normally.
3. **No Enterprise path is copied, adapted, translated, or used as a reference** — including for
   naming, schema shape, or test fixtures.
4. **If the admin dashboard is forked** ([ADR-002](#adr-002) permitting), the Enterprise UI paths
   are excluded from the fork and replaced with our own.
5. Parity claims are made against Medusa's **MIT surface**. The parity matrix marks RBAC and SSO as
   *clean-room equivalent*, never *ported*.

### Consequences

**Good.** No licensing exposure. The concepts are unencumbered and RBAC is well-trodden ground.

**Bad.** RBAC is 5 entities and 9 workflows of real work with no reference implementation to check
against, and it cannot use Medusa's fixtures for parity testing. Estimate P7 accordingly — this is
now genuine design work, not translation.

**Note on earlier versions.** The notice states it applies prospectively and does not withdraw
rights already granted under MIT for earlier versions. Relying on that to source an older MIT-era
RBAC implementation is a legal question, not an engineering one. **Do not do it without written
advice from counsel.** The clean-room route costs weeks; getting this wrong costs the project.

### Reversal condition

A commercial agreement with MedusaJS, Inc. would permit direct use. That is a procurement decision,
not a technical one.

---

<a id="adr-009"></a>

## ADR-009 — Plan tiers by entitlements, on single-tenant deployments

**Status:** **Superseded by [ADR-012](#adr-012)** (2026-08-19) · Originally accepted 2026-08-14

> **What survives:** decisions 2, 3, 3b, 5, 6 and 7 below — the tier states, the cost-line
> rule, the three kinds of gate, atomic quotas, never deleting data on downgrade, and
> deferred licence signing — are all still in force.
>
> **What is superseded:** decision 1 (one binary and one database per store) and
> decision 4 (a gate is a *mount-point* decision). ADR-012 replaces the isolation model,
> and one process serving tenants on different tiers makes boot-time route mounting
> unable to express per-tenant gating. See ADR-012 for the replacement and its cost.

### Context

Stheno is sold as a service with pricing tiers, so features must be switchable per customer. The
modular monolith ([ADR-007](#adr-007)) was initially assumed to provide this. **It does not** —
module boundaries are a build-time property and give nothing at runtime. They make gating *clean*;
they are not the gate.

Two questions had to be answered together: how customers are isolated, and how finely tiers cut.

### Options considered

| Option | For | Against |
|---|---|---|
| **One deployment per store** (chosen) | Physical data isolation — a gating bug leaks a feature, never another customer's orders. Entitlements are simple config. **Affordable precisely because the binary is small** (§ below) | N deployments to operate and upgrade |
| Multi-tenant, one deployment | Cheapest per customer at scale, one thing to operate | All 127 entities need `tenant_id`; every query needs scoping; +12 weeks minimum touching all 252 tasks; a cross-tenant leak is company-ending |
| Single-tenant now, multi-tenant later | Ships on schedule, door open | Pays discipline cost for a maybe; the retrofit stays expensive regardless |

### Decision

1. **One binary and one database per store.** [PRD NG2](PRD.md) stands and is now a commercial
   decision, not only a scoping one.
2. **Three account states — Trial (30 days) → Free (indefinite) → Pro.** Trial and Free are states
   of one account, not separate products. There is no Enterprise tier. Defined in
   `config/tiers.toml`, selected by `STHENO_TIER`, with per-deployment env overrides.
3. **Gating is decided by one rule:** *the free tier does nothing that costs marginal money; the
   paid tier is exactly the set of features that cost money to run.* A cost line, not a greed line
   — which is what makes an indefinite free tier survivable. Core commerce is never gateable.
3b. **Three kinds of gate:** feature (route group unmounted), quota (atomic counter), and **window**
   (query horizon — e.g. 30-day analytics on Free, full history on Pro). Window gates are the
   cheapest tier lever and were the one kind the first draft missed.
4. **A gate is a mount-point decision, never an `if` inside domain logic.** Gated routes are not
   mounted; gated workflows are not registered. `crates/modules/` may not reference `Entitlements`
   at all, CI-enforced.
5. **Quotas are enforced atomically** via a counter updated in the insert transaction, the same
   pattern as campaign budgets.
6. **Downgrade never deletes or hides data.** A merchant over quota keeps everything —
   grandfathered, visible, sellable — and is blocked only from creating more. Trial expiry and paid
   downgrade run the **same code path**. Removing a seller's live products is an unrecoverable trust
   failure, and the architecture must make it impossible rather than merely discouraged.
7. **Licence signing is deferred.** Config is sufficient while every deployment is ours; an Ed25519
   signed licence is added when the first self-hosted deal is real.

### Why this is affordable here — and would not have been on Node

A prior plan for this same product, built on Medusa, was **forced into multi-tenancy by
resource cost**: a Node commerce runtime needs 600 MB–1.5 GB per instance, which on a small
infrastructure budget means one process serving every merchant, with a sales channel as the tenant
boundary and every storefront route audited for scoping gaps.

Stheno's budget is **≤ 128 MB resident at idle and a ~40 MB static binary**
([PRD §7.2](PRD.md)). At that footprint one process per store is not extravagant — a single modest
VPS hosts many, each with its own database on a shared Postgres server.

**This inverts the usual justification for the rewrite.** The Rust engine is not primarily about
p95 latency; it is what makes physical per-merchant isolation cheap enough to choose. Multi-tenancy
was never rejected on principle — it was rejected because the runtime no longer forces it.

### Small-store deployment profile

[PRD §7.1](PRD.md)'s 4 vCPU / 8 GB figures are the **benchmark rig**, not a per-store requirement.
A real small merchant — 20–300 products, tens of orders a day — needs far less:

| Resource | Per store |
|---|---|
| App process | 128–256 MB RAM, fractional vCPU |
| Database | one database on a **shared** Postgres server |
| Storage | media on shared object storage, prefixed per store |

Provisioning must be automated before roughly the tenth merchant. Manual `docker run` per customer
is fine for a pilot and becomes the bottleneck immediately after.

### Consequences

**Good.** Data isolation is physical rather than a query predicate — the highest-value property for
a business selling to many merchants. Medusa's `GET /admin/feature-flags` already exists in the
shape the forked dashboard reads, so disabled screens hide for free. Rule 4 keeps domain code
entirely unaware of pricing, which means tier changes never risk commerce correctness.

**Bad.** Ops cost scales linearly with customers; provisioning must be automated before roughly the
tenth. Quota counters can drift and need a reconciliation job. Sub-feature gating (returns without
exchanges) is achievable only because those capabilities happen to live in separable route groups —
a future feature that does not will force a choice between coarser tiers and breaking rule 4.
**Break the tier, not the rule.**

### Reversal condition

Moving to multi-tenancy later means adding `tenant_id` to 127 entities and scoping every query. It
is a rewrite of the data layer, not a refactor. Treat this decision as effectively permanent, and
if per-customer infrastructure cost ever becomes the binding constraint, revisit it as a new
product rather than a migration.

---

<a id="adr-010"></a>

## ADR-010 — `Id<T>` carries its prefix as a compile-time constant

**Status:** Accepted · **Date:** 2026-08-16

### Context

`Id<T>` was introduced to model Medusa's prefixed identifiers (`prod_01J...`, `cart_01J...`).
It did not. `Id::from_string` split on the first `_`, **discarded the prefix**, and kept only the
ULID; `Display` and `Serialize` then emitted the bare ULID.

The prefix was therefore destroyed at the first parse and never reconstructed. Two failures
followed, both already in the tree:

1. **Wire incompatibility.** A DTO built via `Id::from_string(&product.id)` serialised
   `01J...` where Medusa serialises `prod_01J...`. Parity was preserved only where the model
   kept a raw `String` id and bypassed `Id<T>` altogether — which is why id representation is
   currently split between `String` and `Id<T>` across modules.
2. **Suffix matching.** Because a caller's `prod_01J...` never equalled a stored `01J...`,
   services compensated with `id.ends_with(caller_id)` — **55 sites** across cart, payment,
   pricing, product, demo, and three API crates.

Against a `HashMap` that is merely wrong-ish. Against an indexed Postgres column,
`WHERE id LIKE '%' || $1` is a guaranteed full table scan on **every retrieve**, on every one of
the 127 entities. Phase 2 migrates 25 modules onto real tables, so this must be settled before
those migrations encode the workaround (PHASE2_PLAN.md P11 risk 2).

### Options considered

**A. Store the prefix in the value** — `Id { prefix: String, inner: Ulid }`. Round-trips
correctly and needs no trait bounds, but costs a heap allocation per identifier on a struct
that appears 127 times across every row of every list response, and `Id::new()` still cannot
know which prefix to mint.

**B. Drop `Id<T>` and use `String` everywhere.** Honest and cheap, and matches what the product
module already does. Discards all type safety: nothing then stops an `Id<Cart>` being passed
where an `Id<Order>` is expected, which is the entire reason the type exists.

**C. Compile-time constant per entity.** A `Prefixed` trait carrying
`const PREFIX: &'static str`, implemented once per entity type.

### Decision

**Option C.** `Id<T>` requires `T: Prefixed`, and:

- `Display`/`Serialize` emit `{PREFIX}_{ULID}` — the Medusa wire form, always.
- `FromStr`/`Deserialize`/`from_string` accept **only** `{PREFIX}_{ULID}` and reject any other
  prefix.
- The ULID remains the stored primary key representation; the prefix is derived, never a column.

Cross-type rejection is deliberate. An id arriving on an HTTP path is untrusted input, and
`GET /admin/products/cart_01J...` should be a 400, not a lookup that happens to miss. This
costs one string comparison at the parse boundary and is the only place the check can be made
once for all 127 entities.

### Consequences

**Good.** The prefix survives end to end, so the 55 `ends_with` scans delete. Retrieval becomes
an equality match on an indexed column, which is what P12's index work assumes. Passing a
`cart_` id to a product endpoint is now an error at the boundary rather than a silent miss.

**Bad.** Every entity type needs a `Prefixed` impl — 127 of them. They are declared as a table
via the `prefixed_id!` macro rather than hand-written, but the table must be **correct**: a
wrong prefix here is a wire-compatibility bug that the parity suite catches only for endpoints
it covers. The prefixes are therefore taken from Medusa's own model definitions, and
`stheno-types` owns the table so no module can disagree with another about its own prefix.

Modules currently holding raw `String` ids (product and its taxonomy) are **not** converted by
this ADR — they already carry the correct prefixed form and are wire-correct today. They move
to `Id<T>` as each is migrated in P11-19 and P12, not in a separate sweep.

### Reversal condition

If a future entity needs a prefix that is not known at compile time — a customer-defined
extension type, say — Option A becomes necessary for that type alone. Nothing in this decision
prevents a second id type existing alongside; it prevents 127 heap allocations to serve a case
that does not currently exist.

---

<a id="adr-011"></a>

## ADR-011 — `FilterSpec` stays single-entity; joins belong to the query engine

**Status:** Accepted · **Date:** 2026-08-17

### Context

PRD risk R3 and PHASE2_PLAN P11 both name this as the largest schedule risk:
`FilterSpec` was specified in P0 and **never executed** — `InMemoryCrudStore::list`
ignores it entirely — so nothing was known about whether it could express the real list
endpoints. P11-12 required running it against the 20 hardest, with a budgeted fallback of
four weeks for hand-written per-module query builders if it could not.

### Findings

The spike ran all 20 against Postgres (`crates/core/stheno-db/tests/filter_hard_cases.rs`).

**18 of 20 are expressible and correct.** The operator set — equality, comparison, null,
set, prefix-`LIKE`, ordering, offset pagination — covers every single-entity filter in
API_SPEC, including date ranges, price bands, multi-key sorts and deep pagination.

**One real defect was found and fixed.** `FilterSpec.predicates` was a
`HashMap<String, FilterOp>`, keyed by column. A range needs two operators on one column
(`created_at >= x AND created_at <= y`), so the second bound silently replaced the first
and every range filter returned a half-open set. `from_json` already parsed both bounds,
which means the loss was invisible at the boundary. `predicates` is now an ordered
`Vec<(String, FilterOp)>`.

**Two cases are deliberately out of scope**, and both are recorded as tests rather than
hidden: cross-module filters (`products?price_list_id=`), and infix free-text search.

### Decision

**`FilterSpec` is not extended with joins, and the hand-written fallback is not taken.**

- Cross-module filtering is a **query engine** concern (P12-18, P14-20). Reaching from
  `product` into `pricing` inside a filter would be exactly the cross-schema join ADR-007
  forbids modules from writing.
- Infix search stays refused. `Like` rejects a leading wildcard so every list remains
  index-backed; free-text belongs to the search module and its own index, not to a
  sequential scan wearing a filter's clothes.

Values are bound through `json_populate_record(NULL::<table>, $1)`, which coerces each
value to the column's own type using the table's row type. This is what keeps one generic
translation for all 127 entities without a per-entity column-type table, and it keeps
comparisons in the column's type — comparing as text would make `quantity > '9'` false
for a quantity of 10.

### Consequences

**Good.** P12–P14 are execution rather than redesign, and the four-week fallback is not
spent. One translation layer serves every entity.

**Bad.** Column names cannot be parameterised by Postgres, so they are concatenated into
SQL after a strict identifier check — that check, not a bind, is what prevents injection,
and it is therefore load-bearing security code with its own tests. Any future operator
must preserve it.

### Reversal condition

If a single-entity filter is found that the operator set cannot express, add the operator.
If the *join* cases multiply beyond what the query engine absorbs, that is a signal to
revisit the query engine's design — not to grow `FilterSpec` into a second one.

---

<a id="adr-012"></a>

## ADR-012 — One database, `tenant_id` on every row, isolation enforced by Postgres RLS

**Status:** Accepted · **Date:** 2026-08-19 · **Supersedes:** [ADR-009](#adr-009) decisions 1 and 4

### Context

[ADR-009](#adr-009) chose one binary and one database per store, and [PRD NG2](PRD.md)
recorded multi-tenancy as a non-goal. Both rested on an assumption the PRD itself flagged
as open — **Q1: "Is a hosted/cloud offering planned? It changes multi-tenancy and metering
assumptions"**, whose stated default was "No. Self-host only, per NG2."

That question now has the opposite answer. Stheno is being sold as a hosted service, and
operating one database per merchant is the cost that model cannot carry: N databases to
provision, migrate, back up, and monitor, growing linearly with customers.

ADR-009 considered multi-tenancy and rejected it as *"All 127 entities need `tenant_id`;
every query needs scoping; +12 weeks minimum touching all 252 tasks; a cross-tenant leak
is company-ending."* Two facts about the codebase as built make that estimate pessimistic:

1. `PgCrudRepository<T>` is a **single generic implementation** serving all ~127 entities,
   and `with_conn` is the one function through which every read and write acquires its
   connection. There is one place to enforce scoping, not 500.
2. **Postgres Row-Level Security** moves enforcement out of application code entirely. The
   "every query needs scoping" cost is paid once, in the database, by a policy — not per
   call site in Rust.

The "company-ending" warning is not dismissed. It is the reason this ADR mandates an audit
test (below) rather than trusting review.

### Options considered

| Option | For | Against |
|---|---|---|
| **One DB + `tenant_id` + RLS** (chosen) | One database to operate. Enforcement in the database, so a missed filter returns zero rows rather than another tenant's. Entity structs unchanged | Every table grows a column and a policy; advisory locks are not covered; boot-time route gating stops working |
| One DB + application-level scoping | No database features required | This *is* ADR-009's rejected option. A forgotten filter is a silent leak with nothing beneath it to catch the mistake |
| Keep one DB per store, add a control plane above | No engine change at all | Does not solve the problem that prompted the change — N databases remain |
| Schema-per-tenant | Strong isolation, no row filtering | 31 schemas × N tenants; migrations become O(tenants); connection `search_path` juggling |

### Decision

1. **One database. Every tenant-owned row carries `tenant_id`.** Reference data that no
   two tenants could disagree about — ISO currency and country tables, migration
   bookkeeping — stays global. The exclusion list is the leak surface: it stays under five
   entries and every entry is argued in a comment.
2. **Isolation is enforced by RLS, not by application code.** One policy per table:
   `USING (tenant_id = current_setting('app.tenant_id', true))`. `missing_ok = true` is
   deliberate — an unset setting yields `NULL`, the predicate is `NULL`, and the row is
   filtered. **An unscoped query returns zero rows; it never returns everything.**
3. **`FORCE ROW LEVEL SECURITY` on every scoped table.** The application connects as the
   table owner, and Postgres exempts owners from RLS unless forced. Without this the
   policies are inert and every test passes while proving nothing.
4. **The tenant travels in a task-local**, set once per request by middleware
   (`stheno_db::tenant`). Services are built once at boot holding a pool, so a parameter
   would mean changing every repository and service signature — exactly ADR-009's
   12 weeks. `tenant::current()` fails closed.
5. **`SET LOCAL` inside a transaction, never session-level state.** A pooled connection
   must not carry a tenant identity between requests. This costs round trips on the
   non-transactional path and is accepted: the two designs fail in opposite directions,
   and only one of them fails safe.
6. **ADR-009 decision 4 is replaced.** Route groups mount unconditionally and a
   per-request layer returns 404 for an unentitled feature. `crates/modules/` still may
   not reference `Entitlements`, so the CI rule behind that decision is unchanged.
7. **A permanent audit test**, not a one-time review: every table in every application
   schema, minus the exclusion list, must have `tenant_id`, `relrowsecurity`,
   `relforcerowsecurity`, and a policy. It also asserts `is_superuser = 'off'`, so a
   superuser connection string fails loudly instead of passing with RLS inert.

### Consequences

**Good.** One database to operate, migrate and back up. The 127 entity structs are
untouched. `FilterSpec` and the query engine learn nothing about tenancy. The existing 407
test suites begin exercising isolation for free, because they connect as the owner and
`FORCE` applies to them.

**Bad, and worth stating plainly:**

- **Anti-probing is weakened.** ADR-009 chose 404-by-not-mounting specifically so a
  response could not confirm a feature exists. A per-request layer returns the same status
  but is distinguishable by timing. This is a real regression in that property, accepted
  because per-tenant tiers cannot be expressed any other way in one process.
- **Advisory locks are not covered by RLS.** `pg_advisory_lock` operates on a global
  64-bit key space; tenant keys must be namespaced in application code. There is no
  database backstop for this one, which makes it the sharpest edge in the design.
- **Ambient context.** A repository call's correctness depends on a task-local that does
  not cross `tokio::spawn`. Detached work must re-enter a scope explicitly.
- **Round trips.** Wrapping the non-transactional path in a transaction adds two.

### Reversal condition

If cross-tenant isolation is ever breached in a way RLS could not have prevented, the
answer is not to add application-level filtering on top — that is paying twice while
leaving the policy untested. It is to move the affected data to schema-per-tenant.
ADR-009's model remains valid for self-hosted single-merchant deployments, which is why
`STHENO_TIER` survives as a fallback when no tenant is resolvable.

---

<a id="adr-013"></a>

## ADR-013 — Storefront presentation is swappable; commerce logic is not

**Status:** Accepted · **Date:** 2026-08-19 · **Implemented by:** [Phase 3](PHASE3_PLAN.md)

### Context

[ADR-012](#adr-012) committed to a hosted, multi-tenant product. That creates two pressures at
once, and they are the same question wearing two hats.

**Merchants want storefronts that look like theirs.** A hosted platform where every store is
visibly the same store does not sell. The obvious answer — fork the storefront per merchant — is
the expensive one, because the storefront is not mostly design. `apps/storefront/src/lib/data/`
holds fifteen files of cart, payment, fulfillment, region and order logic. Forking for a new look
also forks that, and five merchants become five checkouts, five region-routing bugs, and five ways
to lose a cart on a payment redirect.

**Merchants are not all in the same country.** Stripe Elements does not sell in Dhaka. bKash,
Nagad, SSLCommerz, Pathao, Steadfast and RedX do, and each new market adds more. The engine
already anticipated this: `PaymentProvider` and `FulfillmentProvider` are complete traits with
three implementations between them. What was never decided is **who may enable what, and when that
choice is made** — build time or run time.

Both pressures reduce to one question: *what varies per merchant, and what must not?*

### Options considered

| Option | For | Against |
|---|---|---|
| Fork the storefront per merchant | Trivial to start. Total freedom per store | N checkouts, N PCI surfaces, N sets of the same bug. Every engine change becomes an N-way migration. The cost arrives at merchant three, not merchant one |
| Runtime theme registry — components resolved per tenant per request | One deploy. A merchant switches design instantly | Dynamic imports throughout, no tree-shaking, and a component API that can never change once three merchants depend on it. It is [ADR-001](#adr-001)'s rejected runtime-plugin model, rewritten in TypeScript |
| **Core/theme split — branding at runtime, themes at build time, commerce logic fixed** (chosen) | One checkout, one compliance surface, one set of commerce bugs. The tenth theme costs what the second did | Themes must be versioned against core. A bespoke design is a deploy, not a toggle |
| JSON-composed page-builder blocks | Merchants self-serve their own layouts | A block system is a framework, and nobody has asked for one. [AGENTS.md §1](../AGENTS.md) rung 1: this does not need to exist yet |

### Decision

1. **Three tiers of variation, and only three.**
   - **Base** — branding *data*: store name, logo, favicon, a font chosen from a curated
     self-hosted set, and colour tokens. Resolved per request in the shared runtime. No build, no
     deploy, no engineering per merchant.
   - **Pro** — a **theme package**: own components, own layout, own animation, compiled in and
     deployed to its own target.
   - **Custom** — a services engagement with our engineers. Not a tier a merchant can buy by
     clicking. Governed by decision 8.

2. **Checkout is core at every tier — but it is never unbranded.** Flow, steps, validation, state
   machine and the script manifest are not editable. Everything above that line is:
   - **Base tier**: the tenant's branding record applies on checkout routes exactly as it does
     everywhere else — store name, logo, favicon, font, colour tokens. A merchant paying nothing
     still does not hand their customer to a page that looks like someone else's shop at the
     moment they enter a payment.
   - **Pro tier**: the theme additionally skins checkout — its own components, type and buttons —
     while the flow underneath is the same one every other store runs.

   The rule is **skinnable, never rewritable**. Presentation is free at every tier; behaviour is
   fixed at every tier.

2a. **Branding assets on checkout obey the script manifest.** Logos and CSS custom properties are
   not scripts and are unaffected. **Fonts must be self-hosted** — the curated set exists partly
   for this reason. A font pulled from a third-party CDN on a checkout route is a
   [payment-page-scripts.md](payment-page-scripts.md) violation and fails
   `check-checkout-scripts.sh`, which is the correct outcome and the reason merchants choose a
   font key rather than upload a URL.

3. **The reason for decision 2 is not only modularity.** SAQ-A eligibility rests on
   [payment-page-scripts.md](payment-page-scripts.md): a two-entry allowlist on `/checkout/*`,
   enforced by `check-checkout-scripts.ps1`. Merchant-specific code on checkout routes is
   therefore a **compliance** boundary, not a taste boundary, and it already fails the build.

4. **Routes, data access, region resolution and SEO belong to core.** A theme supplies components
   against a typed contract and receives props. It does not fetch, and it does not route. URL
   structure is not a design decision.

5. **Themes are build-time; branding is run-time.** No runtime component registry. This is
   [ADR-001](#adr-001) applied to the frontend, for the identical reason: a plugin API is a
   promise you cannot withdraw. Base-tier branding is exempt because it is *data* — CSS custom
   properties and strings — never code.

6. **Payment and fulfillment providers are compile-time; enablement is tenant data.** One binary
   carries every provider. Which ones a merchant may use is rows in
   `links.link_region_payment_provider`, tenant-owned and RLS-covered. A merchant turning on bKash
   is a toggle; adding a gateway that has never existed is a release.

7. **Redirect or iframe gateways only.** A gateway that requires a PAN to be posted from our own
   page is **refused, not integrated** — it would move the platform from SAQ-A to SAQ-A-EP, which
   is a different compliance posture and a different company. `check-provider-pci.sh` enforces
   this at build time.

8. **A Custom-tier engagement that modifies checkout removes that merchant from the shared
   compliance scope**, and requires its own attestation. This is the one exception to decision 2
   and it is expensive on purpose.

9. **Enforcement is mechanical, never review.** `check-storefront-boundaries.sh`,
   `check-theme-purity.sh`, the theme conformance e2e pack, `check-provider-conformance.sh` and
   `check-provider-pci.sh`. A theme or provider that passes review and fails a gate is not
   shipped. This mirrors [ADR-007](#adr-007): boundaries that are asked for are boundaries that
   erode at 2am before a launch.

### Rationale

The split is drawn where the cost is. Design changes often and breaks nothing when it is wrong.
Checkout changes rarely and loses money when it is wrong. Putting them behind the same seam prices
every design change at the risk of the checkout.

It also matches what is actually being sold. A merchant buying Pro is buying a look, not a
different commerce engine — and the thing that makes their look affordable to build is that the
cart, region and payment logic underneath was written once and has been in production across every
other store.

### Consequences

**Good.** One checkout, one PCI surface, one set of commerce bugs to fix. The tenth theme costs
what the second did. A new market is one provider crate and one registration line, not a fork.
Base-tier branding costs zero engineering per merchant, which is what makes a free tier viable at
all.

**Bad, and worth stating plainly:**

- **Themes must be versioned against core.** Additive-within-a-major is a discipline, not a
  mechanism — nothing in the type system enforces it. A core major bump is an N-theme migration,
  and that cost grows with every theme sold.
- **A Pro merchant's redesign is a deploy, not a setting.** Accepted: designs change twice a year,
  not twice a day. It will still be argued about by someone in a sales call.
- **The Custom exception is a real hole in decision 2**, and decision 8 prices it rather than
  closing it. If Custom engagements become common, the shared compliance scope stops meaning much.
- **A merchant wanting a genuinely different purchase flow** — subscription, quote-to-order,
  wholesale approval — is not served by any of the three tiers and will push on this ADR. That is
  a core feature request, not a theme, and answering it with a theme would be the failure this
  record exists to prevent.

### Reversal condition

If the conformance pack cannot express a theme's checkout without theme-specific branches, the
**contract** is wrong, not the pack — fix the contract. If more than a third of Pro merchants
require changes to `storefront-core` to build their theme, the seam is drawn in the wrong place;
the answer then is to move routing into themes and ship a heavier core SDK instead, accepting the
loss of URL and SEO control that implies.

---

<a id="adr-014"></a>

## ADR-014 — A line item's price comes from the catalogue, never from the request

**Status:** Accepted (2026-08-19)

### Context

`POST /store/carts/{id}/line-items` read `unit_price` from the request body and defaulted it
to `Decimal::ZERO` when absent. `/store/*` is unauthenticated by design — a shopper has no
token — so this made the price of a sale a client-supplied value. Sending
`{"unit_price": 0.01}` bought anything for a penny; omitting the field bought it for nothing.

It survived review because nothing could contradict it. `GET /store/products` returned a flat
product with no `variants` and no prices at all, so there was no catalogue price for a cart to
disagree with, and the parity tests asserted the endpoint returned 200 — which it faithfully
did, at whatever price it was told.

The pricing module was complete throughout: `calculate_prices_batch` implements Medusa's full
precedence (override list > sale list > rules > base). The link `product_variant_price_set` was
declared in `stheno-links` and never written or read by anything. Nothing joined the two.

### Decision

A line item is priced from the variant, server-side, on every add:

- `unit_price` in the request body is **ignored**, not validated. Rejecting a mismatched price
  would leak the real one and still invite probing; there is no case where a client's price is
  the right answer.
- `update_line_item` ignores it too — changing quantity must not be a path to changing price.
- `variant_id` becomes **required**. It is the only thing that has a price.
- A variant with no price in the cart's currency is a **400**, not a zero. "Unpriced" and
  "free" must never be the same value.

The currency comes from the cart's own `currency_code`, falling back to its region — not from
the request either, for the same reason.

### Consequences

- Custom-priced line items (a negotiated discount, a manual draft order) are no longer
  expressible through the store API. That is correct — it is an admin action, and the draft
  order flow already owns it.
- Three parity tests had to seed a priced variant instead of asserting on a fabricated price.
  Each of them had been documenting the bug as expected behaviour.
- The catalogue endpoints now return `variants[]` with `calculated_price`, `options[]` and
  `inventory_quantity`, because a storefront that cannot see a price cannot show one — the read
  and write halves of this had to land together.

---

<a id="adr-015"></a>

## ADR-015 — Wire-shape expansion lives in the API layer, not in module DTOs

**Status:** Accepted (2026-08-20)

### Context

The storefront is a fork of Medusa's Next.js starter, so it reads Medusa's response shapes
structurally. Ours were the same data at a different depth or behind a link, and every
mismatch failed *silently* — the component rendered, with zeros or nothing, and no request
errored:

| Mismatch | What the shopper saw |
|---|---|
| `variants` / `options` / prices absent from `GET /store/products` | Catalogue with no prices; add-to-cart never enabled |
| `inventory_quantity` absent | Every product "Out of stock" |
| Cart totals nested under `totals` | Every figure in checkout `$0.00` |
| `cart.region` not expanded | No "Proceed to Checkout" button at all |
| `cart.payment_collection` not expanded (behind a link table) | Review step never appeared |
| Order totals only in the accounting `summary` | Confirmation page `$0.00` after paying |

Each was individually invisible and collectively made the checkout impossible to complete.

### Decision

Expansion belongs in `crates/api/stheno-api-store`, as response types that wrap a module
DTO — `StoreProduct`, `StoreCart`, `StoreOrder` — never by widening the module DTO itself.

Two exceptions were made deliberately, both where the field already existed and only its
*depth* was wrong: `CartDTO::totals` and the line-item/shipping-method totals now carry
`#[serde(flatten)]`. That changes one attribute rather than duplicating twelve fields
through a wrapper, and it is a struct field in Rust either way.

### Consequences

- `ProductDTO` and the other ~127 entity structs stay free of presentation concerns, and
  the admin API is unaffected by storefront shape changes.
- Expansion costs queries, so it is batched where it counts: one price evaluation covers
  every variant on a page (`calculate_for_linked`), and one availability lookup covers
  every variant. Per-product option queries remain and are bounded by the page limit.
- **A route that returns 200 proves nothing about whether a client can use the response.**
  Every one of these gaps sat behind a passing parity test that asserted a status code and
  a couple of fields. The e2e suites added alongside this ADR drive the real journeys,
  which is the only thing that would have caught them.

---

## ADR-016 — Plan definitions are data the control plane owns, not config baked into the image

**Status:** Accepted · **Date:** 2026-08-20 · **Refines:** [ADR-012](#adr-012) · [ENTITLEMENTS.md 5b](ENTITLEMENTS.md)

### Context

[ENTITLEMENTS.md 5b](ENTITLEMENTS.md) put tier definitions in `config/tiers.toml` and made
a promise about them: *"Adding or retuning a tier must never require a release."* That
promise held under [ADR-009](#adr-009), where a deployment was one merchant's binary on
one merchant's machine and an operator could edit the file in place.

[ADR-012](#adr-012) changed the deployment shape and quietly broke it. One process now
serves every merchant, that process runs from a container image, and `config/tiers.toml` is
inside the image. Retuning a plan meant a rebuild and a redeploy — which is a release, and
which is the thing 5b said would never be required.

The gap was wider than the file. `platform.subscription` already carried
`quota_overrides` and `window_overrides`, but **nothing wrote them**: there was no API and
no UI, so the only lever the control plane actually had was moving a merchant between three
fixed tiers. Features had no per-tenant lever at all, so "sell the payment gateway to this
one merchant" had no answer short of putting them on Pro and giving away the rest of it.

### Options considered

| Option | For | Against |
|---|---|---|
| **DB row overrides the shipped TOML** (chosen) | The TOML stays the shipped default, so an untouched deployment behaves exactly as before. Editable from the control plane. One source of truth per tier at any moment | A tier the operator edits stops tracking the defaults a future release ships — stated in the UI as `edited` rather than left to be discovered |
| Move tier definitions into the database entirely | One place to look | An empty or unreachable table means every plan is undefined. The failure mode of the storage layer becomes "nobody has any features" |
| Merge the row into the TOML per key | Smaller writes | Needs an answer for *"the row omits `max_products` — unlimited, or unspecified?"*, and every answer to that is a plan silently granting more than the operator believes |
| Mount the config file as a volume | No code | Moves the problem to whoever operates the volume, and gives the control-plane UI nothing to write to |
| Let merchants edit their own plan from `/admin/*` | Self-serve | A merchant who can define what Pro includes is a merchant on Pro for free |

### Decision

1. **`platform.tier_definition` replaces a tier's definition wholesale when a row exists.**
   No row means the shipped TOML governs. Replacement, not merge, for the reason above.
2. **`platform.subscription.feature_overrides`** completes the per-tenant axis alongside
   the quota and window overrides that already existed: `{"payment_gateway": true}` grants,
   an explicit `false` revokes, an absent key defers to the tier.
3. **Overrides are validated at the write boundary**, not at the next boot. ENTITLEMENTS.md
   5b makes an unknown key abort startup; a write API can reject the request instead, so
   the bad key never reaches the table. A stored typo would make every subsequent request
   for the affected merchants fall back to the least-privileged entitlements — which reads
   as an outage, days after the typo was made.
4. **Definitions are read per request, not cached in the process.** That costs one small
   query on top of the subscription read the tenant layer already does, and it buys an edit
   that is live on every node immediately with no invalidation to get wrong. A cached node
   would be serving a plan nobody is on.
5. **The whole surface lives on `/platform/*`, behind the operator credential.** Nothing
   about it is reachable from `/admin/*`, where the merchant's view stays read-only.

### Consequences

- ENTITLEMENTS.md 5b's promise is true again, and for hosted deployments for the first
  time.
- §6.1 still holds: overrides are applied during *resolution*, before the router reads the
  value, so a gate remains a mount-point decision and no domain code learns that
  entitlements exist. `scripts/check-no-domain-gates.sh` still passes.
- The `0`-means-unlimited sentinel now spans three layers — TOML, tier row, and per-tenant
  override — and means the same thing in all of them. The control-plane UI says
  "unlimited" under any field holding 0 rather than letting the operator guess.
- A tier marked `edited` no longer tracks the shipped defaults. `DELETE /platform/tiers/{tier}`
  drops the row and returns it to them; that is the only way back, and it is a button.
- The operator token now defines what every plan includes, not just which one a merchant is
  on. It is compared in constant time, and it remains a placeholder for the
  `platform_owner` actor type that belongs with the RBAC work.
