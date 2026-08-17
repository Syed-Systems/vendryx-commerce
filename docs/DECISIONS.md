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
| [ADR-009](#adr-009) | Plan tiers by entitlements on single-tenant deployments, not multi-tenancy | Accepted |
| [ADR-010](#adr-010) | `Id<T>` carries its Medusa prefix as a compile-time constant | Accepted |
| [ADR-011](#adr-011) | `FilterSpec` stays single-entity; joins belong to the query engine | Accepted |

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

**Status:** Accepted · **Date:** 2026-08-14

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
