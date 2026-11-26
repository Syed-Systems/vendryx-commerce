# Architecture Decision Records

Each record states what was decided, what it costs, and what would reverse it. A decision without
a stated cost is a preference wearing a suit.

| # | Decision | Decision Status | Implementation State |
|---|---|---|---|
| [ADR-001](#adr-001) | Extensibility is compile-time; no runtime plugins | Accepted | **Implemented** (`crates/stheno-*`) |
| [ADR-002](#adr-002) | Full wire compatibility, Store **and** Admin. Fork the dashboard | Accepted · amended by [ADR-027](#adr-027) | **Implemented** (Core API contracts) |
| [ADR-003](#adr-003) | Durable workflows run on a custom Postgres SAGA engine | Accepted | **Implemented** (`crates/core/stheno-workflow`) |
| [ADR-004](#adr-004) | Diesel + diesel-async for persistence | Accepted | **Implemented** (`crates/core/stheno-db`) |
| [ADR-005](#adr-005) | Postgres only; no other database backends | Accepted | **Implemented** (PostgreSQL 16) |
| [ADR-006](#adr-006) | Money is decimal, never float or integer minor units | Accepted | **Implemented** (`stheno-types::Money`) |
| [ADR-007](#adr-007) | Modular monolith with build-enforced boundaries | Accepted | **Implemented** (34 module crates) |
| [ADR-008](#adr-008) | RBAC and SSO are clean-room, not ported — they are Medusa Enterprise | Accepted | **Implemented** (`stheno-rbac`, `stheno-auth`) |
| [ADR-009](#adr-009) | Plan tiers by entitlements on single-tenant deployments, not multi-tenancy | **Superseded by [ADR-012](#adr-012)** | **Superseded** |
| [ADR-010](#adr-010) | `Id<T>` carries its Medusa prefix as a compile-time constant | Accepted | **Implemented** (`stheno-types`) |
| [ADR-011](#adr-011) | `FilterSpec` stays single-entity; joins belong to the query engine | Accepted | **Implemented** (`stheno-query`) |
| [ADR-012](#adr-012) | One database, `tenant_id` on every row, isolation enforced by Postgres RLS | Accepted | **Implemented** (Postgres RLS) |
| [ADR-013](#adr-013) | Storefront presentation is swappable; commerce logic is not | **Superseded by [ADR-031](#adr-031)** | **Superseded** |
| [ADR-014](#adr-014) | A line item's price comes from the catalogue, never from the request | Accepted | **Implemented** (`stheno-cart`) |
| [ADR-015](#adr-015) | Wire-shape expansion lives in the API layer, not in module DTOs | Accepted | **Implemented** (API layer) |
| [ADR-016](#adr-016) | Plan definitions are data the control plane owns, not config baked into the image | Accepted · refined by [ADR-030](#adr-030) | **Implemented** (`platform.tier_definition`) |
| [ADR-017](#adr-017) | Subscription billing is first-party, collected on the gateways already integrated | Accepted | **Partially Implemented** (Manual proofs active; recurring in P30) |
| [ADR-018](#adr-018) | Support impersonation: time-boxed, scope-limited, fully audited, secret-opaque | Accepted | **Implemented** (`stheno-platform`) |
| [ADR-019](#adr-019) | Provider credentials are tenant-owned, encrypted, and resolved through factories | Accepted | **Implemented** (`stheno-payment`) |
| [ADR-020](#adr-020) | Content pages a design needs are core routes fed by a theme slot | **Superseded by [ADR-031](#adr-031)** | **Superseded** |
| [ADR-021](#adr-021) | AI-authored storefront content is data against a closed schema, never code | Accepted · amended by [ADR-038](#adr-038) | **Implemented** (`apps/storefront`) |
| [ADR-022](#adr-022) | Tenant identity is the Host header; the Store API is served same-origin | Accepted · amended by [ADR-038](#adr-038) | **Implemented** (`middleware.ts`) |
| [ADR-023](#adr-023) | One storefront build per theme, routed by host | **Superseded by [ADR-031](#adr-031)** | **Superseded** |
| [ADR-024](#adr-024) | Compute next to the data; delivery by CDN over an origin that speaks HTTP caching | Accepted · amended by [ADR-037](#adr-037) | **Implemented** (Edge CDN headers) |
| [ADR-025](#adr-025) | Connection pooling is transaction-mode, with a session-mode route | Accepted | **Implemented** (PgBouncer/Supavisor) |
| [ADR-026](#adr-026) | Docker Compose now, Kubernetes at scale | Accepted | **Implemented** (Deploy configs) |
| [ADR-027](#adr-027) | Medusa compatibility is an implementation detail, never a product claim | Accepted | **Implemented** (Clean room engine) |
| [ADR-028](#adr-028) | Customer-facing brand is Vendryx; internal identifiers remain `stheno-*` | Accepted | **Implemented** (Brand inventory) |
| [ADR-029](#adr-029) | Merchant Admin Truth Contract & Apple HIG Information Architecture | Accepted | **Implemented** (`apps/admin`) |
| [ADR-030](#adr-030) | Company Control Plane extracted to standalone 'Space' project | Accepted | **Implemented** (`D:/Syed Systems/Space`) |
| [ADR-031](#adr-031) | Dynamic multi-tenant visual theming on a unified default storefront runtime | **Superseded by [ADR-038](#adr-038)** (runtime and chassis; token principle carried forward) | **Implemented, being replaced** (Next.js runtime serves until the ADR-038 cut-over) |
| [ADR-032](#adr-032) | AI-powered social commerce automation: OKF architecture, tool calling & safe draft intake | Accepted · amended 2026-11-14 | **Partially Implemented** (amendment for knowledge prompting, conversation states, store modes and 30-day transcripts is not implemented, see [OKF_PLAN.md](OKF_PLAN.md); 32.1–32.4 implemented; 32.6 owner-authorized chat offer flow is wired but acceptance gates remain; 32.5 confirmation-time dispatch is wired, with focused build and acceptance gates still pending) |
| [ADR-033](#adr-033) | Commercial entitlements overhaul: Free & 7-day Pro trial packaging | Accepted | **Partially Implemented** (trial resolution/lifecycle, verified-owner day-3/day-6 reminders, durable idempotency-aware notification outbox, subscription/grace UI, and persisted-expiry trial banner are wired; web push, quota matrices, and billing journey acceptance remain) |
| [ADR-034](#adr-034) | Manual payment "in verification" truth contract & email/WhatsApp notification pipeline | Accepted | **Partially Implemented** (manual-transfer/COD split, proof ledgers/review and pending/decision emails, cancellation and fulfillment status notices, encrypted cart cookie, durable order/welcome/invite outbox, opt-in order confirmation, consent ledger/STOP revocation, central-vs-merchant WhatsApp routing, and atomic Free daily email/WhatsApp caps are wired; remaining transaction events, deployed WABA/template setup, and focused gates remain) |
| [ADR-035](#adr-035) | Architecture implementation distance & readiness audit | Accepted | **Partially Implemented** (readiness matrix reconciled to code; implementation and acceptance evidence remains open in ADR-032–036 plan) |
| [ADR-036](#adr-036) | Space provides dedicated build execution and shared build caches | Accepted · pairs with Space SPACE-ADR-016 | **Partially Implemented** (scoped caches and hosted CI selector are implemented; current Space SSH check reports rootful Docker, so isolated manual execution and full gate parity are not verified; trusted CI routing remains off) |
| [ADR-037](#adr-037) | Cloudflare is the edge layer only; vendor calls are confined to one provider crate and Space | Accepted · amends [ADR-024](#adr-024) | **Not Implemented** (spec at `.orchestration/task-cloudflare-edge/spec.md`) |
| [ADR-038](#adr-038) | The engine renders the storefront from tokenized template themes; no Node.js storefront | Accepted · supersedes [ADR-031](#adr-031) | **Not Implemented** (plan: [STOREFRONT_THEME_ENGINE_PLAN.md](STOREFRONT_THEME_ENGINE_PLAN.md)) |

Ordered phases, dependencies, edge cases, and acceptance evidence for ADRs 032–036 are tracked in
[ADR_IMPLEMENTATION_PLAN.md](ADR_IMPLEMENTATION_PLAN.md).

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
|---|---|---|---|
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
|---|---|---|---|
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
|---|---|---|---|
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
|---|---|---|---|
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
|---|---|---|---|
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

> **Vocabulary note (2026-08-22).** This record says "theme" throughout. The code no longer does:
> the packages are `frontend-*`, the contract type is `FrontendContract`, the alias is
> `@frontend/*`, and the gates are `check-frontend-*.sh`. Nothing about the decision changed —
> only the noun, because "theme" was read as *palette* three times running and produced three
> recolours of `frontend-default` instead of three frontends. The prose below is left as written;
> an accepted record is not edited to match later naming. See
> [AGENTS.md §12](../AGENTS.md#12-frontends-adapting-a-design).


## ADR-013 — Storefront presentation is swappable; commerce logic is not

> [!WARNING]
> **ARCHITECTURE DEPRECATION NOTICE**
> The core/theme package split defined in this ADR is completely superseded by [ADR-031](#adr-031) (Unified Default Storefront Runtime). The platform no longer supports scaffolding distinct `frontend-*` packages for visual themes. Visual branding is now accomplished strictly via CSS design tokens and component variant modifiers on a single Next.js unified runtime.

**Status:** Superseded by [ADR-031](#adr-031) · **Date:** 2026-08-19 · **Implemented by:** [Phase 3](archive/PHASE3_PLAN.md)

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
|---|---|---|---|
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
   `check-frontend-purity.sh`, the theme conformance e2e pack, `check-provider-conformance.sh` and
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

<a id="adr-016"></a>

## ADR-016 — Plan definitions are data the control plane owns, not config baked into the image

> [!NOTE]
> **REFINED BY ADR-030**
> The rules regarding data ownership described in this ADR remain fully valid. However, the application surface providing the control plane (`apps/platform`) has been extracted into a completely independent project named **Space** ([ADR-030](#adr-030)).

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
|---|---|---|---|
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

---

<a id="adr-017"></a>

## ADR-017 — Subscription billing is first-party, collected on the gateways already integrated

**Status:** Accepted · **Date:** 2026-08-21 · **Refines:** [ADR-012](#adr-012) · [ADR-016](#adr-016) · **Amends:** [ENTITLEMENTS.md §9](ENTITLEMENTS.md) · **Tasks:** [PHASE4_TASKS.md](PHASE4_TASKS.md) P30

### Context

[ENTITLEMENTS.md §9](ENTITLEMENTS.md) scoped billing out of the engine, in one sentence:

> *No usage-based metering or billing. Entitlements say what is allowed; billing systems say
> what is owed. Integrate a billing provider; do not build one here.*

That was the right call for the **engine**. It was read as a decision about the **product**,
and nothing picked the product side up. The result, found while completing P29 and recorded
in [CONTROL_PLANE.md](CONTROL_PLANE.md) §2 D3:

- An operator can move a merchant onto Pro. Nothing then invoices them.
- Nothing notices when a merchant does not pay.
- `platform.tenant.status` has a `suspended` value that **no code path writes**.
- Zero of nine billing capabilities exist.

The business has a working product, a plan editor, per-merchant negotiated exceptions — and
no way to charge anyone for any of it. There is no enforcement behind its own pricing.

The obvious reading of §9 is "integrate Stripe Billing or Chargebee and be done". That
reading does not survive contact with the market [PRODUCT.md](PRODUCT.md) §1 describes.

### The constraint that decides this

The customers are **Bangladeshi social-commerce sellers**. They pay in **BDT**, from
**bKash** and **Nagad** wallets, on **Android phones**. Most do not hold an international
card.

Every hosted subscription-billing vendor is built around card-on-file in a supported
currency. Stripe Billing does not process BDT. Paddle is a merchant-of-record for card and
PayPal. Chargebee orchestrates gateways it supports, and the BD wallets are not among them.

So the vendor would collect nothing. We would still write the bKash and SSLCommerz
collection path ourselves, and then pay a vendor to hold a copy of the invoice we already
had to generate in order to drive it. **The integration cost is not avoided by the vendor;
it is duplicated by it.**

We also already have the expensive half. P21–P24 build the outbound ledger, idempotent
provider calls, webhook replay guards and reconciliation — for merchants' own commerce.
Subscription collection is the same rails pointed at ourselves.

### Options considered

| Option | For | Against |
|---|---|---|---|
| **First-party, on the existing gateways** (chosen) | Collects in the currency and wallets the customers actually hold. Reuses the outbound ledger, idempotency and reconciliation already built. One invoice, in one database, next to the subscription it bills | We own dunning, proration and VAT correctness. This is real money-handling code and it is ours to get right |
| Hosted billing vendor (Stripe Billing, Chargebee, Paddle) | Dunning, proration, revenue reporting and tax as someone else's problem | **Cannot collect BDT from bKash or Nagad.** We write the collection path anyway and pay for a duplicate invoice record. Also puts every merchant's billing identity in a third party under [ADR-012](#adr-012) |
| Collect manually — invoice by hand, reconcile in a spreadsheet | Zero code today | Fails at the third merchant. It is also the state we are in, and it is why `suspended` is unwritten |
| Build metered / usage billing now | Future-proof | No plan is metered. Every quota in `tiers.toml` is a cap, not a meter. Speculative, and the largest thing on this list |

### Decision

1. **Billing is first-party and lives in the `platform` schema**, global and cross-tenant,
   alongside `tenant`, `subscription` and `tier_definition`. A merchant's subscription
   invoice is not one of that merchant's own commerce records and must not sit in their
   RLS-scoped tables — that would make our revenue readable and, worse, writable through
   the tenant's own surface.
2. **Collection uses the same outbound-ledger *pattern* as merchant payments, in its own
   table.** `platform.billing_attempt` carries an idempotency key, a request hash and the
   same `in_flight → succeeded | failed` states as `payment.payment_provider_request`, and
   a convergence sweep settles what a crash left ambiguous. An ambiguous timeout on a
   subscription charge is a double charge to a customer already reluctant to pay us.

   It is **not** the same table, and that is deliberate rather than duplication for its own
   sake. `payment.payment_provider_request` is tenant-scoped and RLS-forced: writing our
   own revenue collection into it would put it inside the merchant's row set, reachable
   through the merchant's own surface — which is exactly what decision 1 forbids. It is
   also unreachable: `stheno-platform` may not depend on `stheno-payment`
   ([ADR-007](#adr-007), `check-module-isolation.sh`), and reaching into `payment.*` from
   the `platform` schema fails `check-sql-boundaries.sh`. Both gates are pointing at the
   same fact: our revenue is not one of the merchant's payments.
3. **Money follows [ADR-006](#adr-006)** — `numeric(20,6)` plus a currency column. No minor
   units, and no exception for "it is only subscriptions". Proration produces sub-minor
   intermediates, which is precisely the case ADR-006 was written for.
4. **Dunning is a state machine and the sole writer of `subscription.status`.**
   `active → past_due → grace → suspended`, reversible on payment. An operator override is
   an explicit, audited exception rather than a second writer — two things writing that
   column is how a paying merchant gets suspended.
5. **Suspension is enforced at the tenant layer**, not in domain code. A suspended
   merchant's storefront stops serving; a merchant who is merely `past_due` does not. This
   keeps [ADR-013](#adr-013)'s rule intact: no domain code learns that billing exists.
6. **What is deliberately not built:** usage metering, revenue recognition, multi-currency
   dunning, a merchant-facing billing portal, and tax jurisdictions beyond Bangladesh VAT.
   The §9 warning stands for all six. If any becomes real, that is the moment to reconsider
   a vendor — and by then there is an invoice table to migrate *from*, which is a better
   position than today's.

### Consequences

- **ENTITLEMENTS.md §9 is amended, not overturned.** "Do not build a billing *engine*"
  still holds: what P30 builds is an invoice, a collection attempt, and a dunning ladder.
  Anything that starts to look like a metering pipeline is out of scope by this record.
- **The business gains enforcement.** `suspended` becomes a state something writes, which is
  what makes a price a price rather than a suggestion.
- **We own correctness in a money path.** Six P30 rows are marked `[BP]` for that reason:
  invoice idempotency, proration summing to the period, ledger-backed collection, the
  dunning ladder, its sole-writer property, and revenue computed from invoices rather than
  from tier assignment.
- **Revenue is computed from invoices, never from `subscription.tier`.** A merchant sitting
  on Pro who has not paid is not revenue, and an MRR figure that counts them is a number
  that makes the business feel solvent while it is not.
- **A second collection currency is a schema change, not a rewrite** — the currency column
  is there from the first migration even though BDT is its only value for now.
- **What would reverse this:** the customer base shifting to card-paying merchants in a
  currency a vendor supports, or the dunning and tax surface growing past what P30 scopes.
  The invoice table is the migration source in both cases.

---

<a id="adr-018"></a>

## ADR-018 — Support impersonation: time-boxed, scope-limited, and audited

**Status:** Accepted · **Date:** 2026-08-21 · **Tasks:** [PHASE4_TASKS.md](PHASE4_TASKS.md) P34

### Context

Platform support operators need to assist merchants by inspecting issues within a specific tenant's admin scope. Without a formal impersonation mechanism, operators resort to database-level access or asking merchants for passwords—both violating tenant isolation, opacity, and auditability rules.

### Decision

1. **Time-Boxed Tokens:** Impersonation tokens are ephemeral (max 60 minutes, server-side enforced).
2. **Strict Scope Limitations:** An impersonating operator is restricted from:
   - Reading or updating provider credentials (`payment` and `fulfillment` secrets).
   - Changing subscription plans or issuing operator credentials.
   - Accessing platform-wide settings or other tenants.
3. **Comprehensive Audit Trail:**
   - Impersonation session creation (`impersonation.start`), every mutating write made under the session, and explicit termination (`impersonation.end`) are recorded in `platform.operator_audit`.
4. **Merchant Notification:**
   - Active and past support impersonations are visible to the tenant in their own admin audit view so no shadow access occurs.

### Consequences

- Operators can troubleshoot merchant setups safely within defined boundaries.
- No secrets or credentials can be leaked through impersonation sessions.
- Reversal condition: If fine-grained RBAC per operator completely eliminates the need for tenant scope assumption.

---

<a id="adr-019"></a>

## ADR-019 — Provider credentials are tenant-owned, encrypted, and resolved through factories

**Status:** Accepted · **Date:** 2026-08-21 · **Refines:** [ADR-001](#adr-001) ·
[ADR-012](#adr-012) · [ADR-013](#adr-013) · **Tasks:** [PHASE3_TASKS.md](archive/PHASE3_TASKS.md)
P21-P24

### Context

[ADR-013](#adr-013) decided that payment and fulfillment providers are compile-time code, while
provider enablement is tenant data. That is only half of a hosted provider platform. A merchant
also brings their own gateway and courier accounts, and those credentials are tenant data too.

The current shape cannot safely express that:

- Admin and Store provider listing routes return hardcoded provider literals instead of reading
  the registry or the tenant's enabled regions.
- `PaymentService` holds a process-global provider map keyed by provider id.
- `PaymentService::get_provider` takes only the provider id, so a caller can forget tenancy.
- The Stripe provider is constructed once at boot from environment variables, giving the whole
  process one Stripe account.
- The payment webhook route is provider-addressed, not tenant-addressed, and inbound signature
  verification needs the tenant's own signing secret before it can run.
- No encryption primitive exists in the workspace, while credentials include API keys, webhook
  secrets, PEM keys, and courier portal passwords.

A credential table alone would not fix the system. If the provider registry remains a
process-global map of already-constructed providers, every tenant still shares the same provider
instance and therefore the same secret-bearing client.

### Options considered

| Option | For | Against |
|---|---|---|---|
| **Per-tenant provider instance built by a factory** (chosen) | Keeps credentials inside the tenant boundary; changes one resolution point; supports provider-specific credential shapes; cache keys can include credential version | Adds a factory trait and credential schema contract |
| Pass credential context into every provider method | Makes credentials explicit at every call site | Rewrites every provider trait method and every provider implementation for no extra isolation |
| Mutate one provider instance's credentials per call | Small diff | Shared mutable secret state is unsafe under concurrency and makes tenant leaks a timing bug |
| Keep process-global environment credentials | Already implemented | Single-tenant behavior in a multi-tenant product; one wrong handler affects every merchant |

### Decision

1. **Provider credentials are tenant-owned database rows.** Payment gateway credentials live in
   `payment.payment_provider_credential`; courier credentials use the same table shape, encryption
   model, versioning model, and factory pattern rather than a parallel design. Rows are RLS-forced,
   soft-deletable, unique per `(tenant_id, provider_id)`, and carry a globally unique opaque
   `webhook_slug`.
2. **Credentials are encrypted in application code before persistence.** The master key comes from
   `STHENO_CREDENTIAL_KEY`. Writes use AEAD with a fresh random nonce. Additional authenticated data
   binds the ciphertext to `tenant_id` and `provider_id`, so moving encrypted bytes between tenants
   or providers fails to decrypt.
3. **Use a direct RustCrypto AEAD dependency for credential encryption.** Postgres `pgcrypto` is
   rejected because the key would travel through SQL text and risk exposure in query logs or
   `pg_stat_statements`. A narrow `aes-gcm` dependency is accepted because no AEAD primitive exists
   in the workspace and credential encryption is a security boundary, not a convenience helper.
4. **Provider registration is by factory.** Each provider crate registers a
   `PaymentProviderFactory` or fulfillment equivalent. A factory exposes `provider_id()`,
   `credential_schema()`, and `build(credentials)`. Provider structs may still carry their native
   clients and secrets, but only after being built for one tenant's decrypted credential row.
5. **Credential schemas are the UI and validation contract.** `CredentialField`,
   `CredentialKind`, and `CredentialMap` describe provider-specific fields such as text secrets,
   PEM blocks, URLs, and booleans. PEM and URL validation happens at credential write time, not at
   first checkout or first webhook delivery.
6. **Provider resolution always includes tenant identity.** `get_provider(tenant_id, provider_id)`
   reads the tenant credential, decrypts it, builds the provider, and may cache the result by
   `(tenant_id, provider_id, credential_version)`. The no-tenant overload is removed. A caller that
   forgot tenancy must fail to compile.
7. **Credential rotation invalidates by version.** Updating a credential increments
   `credential_version`. Caches key on that version, so rotation does not need explicit eviction to
   stop using the old secret.
8. **Provider read DTOs are structurally opaque.** No returned credential DTO contains plaintext
   fields. Secret values are write-only; reads can expose schema, health, version, masked state, and
   webhook URL, but never decrypted credential material.
9. **Inbound webhooks route by opaque slug before signature verification.** The public payment
   webhook route is `/hooks/payment/{webhook_slug}` and fulfillment mirrors it. The route is mounted
   outside the tenant layer, resolves exactly one `(tenant_id, provider_id)` from the slug, then
   enters tenant scope and verifies the signature with that tenant's provider instance.
10. **Unknown webhook slugs fail closed.** An unknown slug returns 404 and writes no replay row, no
    audit row containing secrets, and no tenant-scoped event. A slug is random and not derivable
    from `tenant_id`, provider id, store domain, or region.
11. **Gateway entitlement is per request, not boot-time.** Provider code is compiled into the
    binary, but whether a tenant may configure or use a provider is resolved from that tenant's
    current entitlements and region enablement. Domain logic still does not know entitlements
    exist.
12. **What is deliberately not built:** merchant-supplied provider code, runtime provider plugins,
    credential sharing or inheritance between tenants, automated key rotation, and KMS integration.
    `credential_version` leaves room for rotation and KMS later without changing provider call
    sites.

### Consequences

- A hosted process can serve two tenants using the same provider id with different credentials,
  without provider instances or webhook secrets crossing tenant boundaries.
- A missing tenant at provider resolution becomes a compile-time error rather than a hidden
  single-tenant fallback.
- The admin UI gets one credential renderer from `credential_schema()` instead of one bespoke form
  per provider. This matters because Bangladeshi gateways and couriers have different credential
  shapes, including multi-line PEM keys and portal passwords.
- Webhook URLs become merchant-facing configuration values. A merchant copies the slug URL into the
  provider portal; gateways do not need to know Stheno tenant ids or storefront hostnames.
- Credential health is derived from real provider outcomes, not from the existence of a row. A row
  can be present and invalid.
- The system now carries long-lived third-party secrets. Logs, audit rows, error messages, tests,
  fixtures, and serialized DTOs are part of the credential boundary and must prove opacity.
- The `aes-gcm` dependency is a deliberate exception to the default "no new dependency" posture
  because credential encryption cannot be implemented safely with the current workspace primitives.
- Reversal condition: if per-tenant provider construction becomes a measured throughput bottleneck,
  the cache policy can change behind the factory interface. Reverting to process-global
  secret-bearing providers is not an acceptable reversal in a multi-tenant deployment.

---

<a id="adr-020"></a>

## ADR-020 — Content pages a design needs are core routes fed by a theme slot

> [!WARNING]
> **ARCHITECTURE DEPRECATION NOTICE**
> The "theme slot" paradigm is entirely eradicated by [ADR-031](#adr-031). Since there are no longer separate theme packages, any necessary content pages are strictly core Next.js routes styled via data variant modifiers.

**Status:** Superseded by [ADR-031](#adr-031) · **Date:** 2026-08-22 ·
**Raised by:** `ADAPT The Matrix Hours` ([FRONTEND_MATRIX_SLOTS.md](archive/FRONTEND_MATRIX_SLOTS.md))

### Context

[AGENTS.md §12.5](../AGENTS.md#12-frontends-adapting-a-design) requires a frontend that needs the
design's own pages to stop and get a decision rather than route for itself. This is that stop.

The Matrix Hours ships six routes. Four already have a core equivalent, at a different URL:

| Source route | Core route | Gap |
|---|---|---|---|
| `/` | `/[countryCode]` | none |
| `/shop` | `/[countryCode]/store` | URL shape only |
| `/product/[id]` | `/[countryCode]/products/[handle]` | URL shape only, and core's is handle-based |
| `/checkout` | `/[countryCode]/checkout` | none |
| `/about` | — | **no core route** |
| `/policies/[id]` | — | **no core route** |

[ADR-013](#adr-013) decision 4 puts every route and URL shape in core, and
`check-storefront-boundaries.sh` fails the build on a theme that routes. So the two missing pages
cannot be added where they naturally live — in the theme that wants them — and the URL-shape
differences cannot be closed by the theme either.

That is the whole decision. It is not "should the storefront have an about page"; it is **who owns
a page that exists only because of a design**, in an engine that sells themes.

### Options

**A. Core owns the routes; the theme fills a slot.** Core adds `/[countryCode]/about` and
`/[countryCode]/policies/[handle]`, resolves the content, and renders `ContentPage` — a new
optional contract slot. A theme with no such page exports nothing and the route 404s.

- Keeps ADR-013 decision 4 intact and the boundary gate meaningful.
- Costs a core minor per *kind* of page. The second design that wants `/lookbook` needs core again,
  and that is exactly the per-theme core change [P27-07](archive/PHASE3_TASKS.md) budgets against.

**B. One generic content route.** Core owns `/[countryCode]/[...slug]` backed by merchant-authored
content, and every design's static pages become data rather than routes.

- One core change covers every design, forever. Nothing in `PRODUCT.md` provides the content store
  it needs, so this is a feature, not a route.
- A catch-all segment competes with every existing route and is the kind of change that breaks
  404 behaviour quietly.

**C. Themes may declare routes.** Reverses ADR-013 decision 4.

- Honest about where the pages belong.
- Gives away the property that makes a theme cheap to review and safe to sell: today a theme
  cannot fetch, cannot route, and therefore cannot leak another tenant's data through a URL.
  Recorded for completeness; **not recommended.**

### Recommendation

**A now, B when a second design asks.** A is small, reversible, and keeps the gate honest. If the
third theme also needs static pages, that is the signal for B — and by then there will be a real
requirement for the content store instead of a guess at one.

### What this does not decide

The URL-shape differences (`/shop` vs `/store`, `/product/[id]` vs `/products/[handle]`) are
**not** in scope. Core's Store API is frozen ([AGENTS.md §2.11](../AGENTS.md)) and the storefront
URLs are covered by SEO and sitemap contracts. A design does not get to rename them; adopting a
design does not mean adopting its URL taxonomy.

### Cost of the recommendation

Every design with a bespoke page costs a core minor until B lands. That is stated here so the
count is visible, because [P27-07](archive/PHASE3_TASKS.md) budgets two core changes per theme and this
would be the third for frontend-matrix.

---

<a id="adr-021"></a>

## ADR-021 — AI-authored storefront content is data against a closed schema, never code

**Status:** Accepted · **Date:** 2026-09-02 (Finalized 2026-09-08) · **Relates to:** [ADR-013](#adr-013), [ADR-020](#adr-020)

### Context

Onboarding assigns a new merchant a theme and a look automatically, from a short brief. That
output has to land somewhere. [ADR-013](#adr-013) considered and rejected exactly this shape —
its option 4 reads *"A block system is a framework, and nobody has asked for one. Rung 1: this
does not need to exist yet."*

Somebody has now asked. This is a reversal on new information, not an oversight, so it is
recorded rather than absorbed quietly into ADR-013.

### Options considered

| Option | For | Against |
|---|---|---|---|
| **Branding record + typed section list** (chosen) | No build, no deploy; regenerating a store is an UPDATE. Reviewable by an operator in seconds | A schema per section type; core changes when a theme wants a new kind of section |
| Generated components per merchant | Maximum visual uniqueness | Forks the storefront per merchant with unreviewed code — the exact cost [ADR-013](#adr-013) rejected — and collides with the `/checkout/*` script allowlist |
| Theme and palette only, no content | Smallest surface; needs no new ADR | Every free store shares one silhouette; the brief's answers go unused |

### Decision

1. **The model's entire output is one validated JSON document**: `theme`, colour `tokens`,
   `font`, and `home` — an ordered list of typed sections.
2. **`font` is a key into the curated self-hosted set, never a URL.** ADR-013 decision 2a: a
   third-party font on a checkout route fails `check-checkout-scripts.sh`, which is a
   compliance boundary, not a style rule.
3. **`theme` must be a registered frontend key.** Unknown keys fall back to `default` and warn.
   A storefront never renders unthemed and never 500s on a bad generation.
4. **Section types are a closed enum, declared per theme.** A section the bound theme cannot
   render is rejected at submission, in front of an operator — never at request time in front
   of a buyer.
5. **The document may not contain JSX, CSS text, HTML, third-party URLs, or script
   references.** The validator rejects these structurally, not by scanning strings.
6. **Regeneration must never cost a build or a deploy.** A design that makes it cost one has
   reintroduced the per-merchant fork and is a spec violation, not an optimisation.

### Consequences

**Good.** A merchant's look is tenant data: instant to change, cheap to review, trivially
reversible. Human review stays in the Launch Requests queue that already exists.

**Bad.** This is a schema, and schemas grow. The discipline that keeps it from becoming the
framework ADR-013 refused is decision 4 — a *closed* enum per theme. The day it gains an
`html` or `custom` section type, this ADR has been abandoned in practice.

**Reverses if** merchants demand self-service layout editing, at which point the honest move is
to admit a page builder is being built and design one deliberately.

---

<a id="adr-022"></a>

## ADR-022 — Tenant identity is the Host header, and the Store API is served same-origin

**Status:** Proposed · **Date:** 2026-09-02 · **Relates to:** [ADR-012](#adr-012), [ADR-013](#adr-013)

### Context

`tenant_layer.rs` resolves the tenant from `Host` and says so plainly: *"`Host` is
client-supplied. This is only safe behind a proxy that overwrites it … that is the single
assumption the whole scheme rests on."* For `/store/*` there is no alternative — a shopper has
no token, so a JWT cannot be the source of tenant identity.

The storefront breaks that chain. Its server-side fetches address a single
`STHENO_BACKEND_URL`, so the Host the engine sees is the backend's, not the merchant's. One
storefront process therefore serves exactly one tenant, which is incompatible with
[ADR-023](#adr-023).

### Options considered

| Option | For | Against |
|---|---|---|---|
| **Serve the API same-origin at `merchant.<domain>/api/*`** (chosen) | Host is already correct for both browser and SSR. **No application code.** Makes CORS moot. `/api` is already excluded from the storefront route matcher | SSR hairpins through the ingress; the rewrite annotation must be right |
| Forward the buyer's host on a trusted internal header | Slightly faster for SSR | Needs a second mechanism for the browser path, which posts to the API directly. Adds a forgeable tenant path if an edge strip-rule is ever wrong |
| Publishable API key identifies the tenant | Already sent on every store request | The key is a build-time public env var, so this reimposes one build per merchant — see [ADR-023](#adr-023) |
| `x-tenant-id` header (the common community answer) | Simple | Its own guidance requires verifying an authenticated user owns the tenant. `/store/*` has no authenticated user, so the check can never be performed |

### Decision

1. **`Host` remains the sole source of tenant identity for `/store/*`.** No header, key, or
   parameter may override it.
2. **The engine is reachable at `/api/*` on each merchant's own hostname**, so the Host is
   correct by construction rather than by forwarding.
3. **No second tenant-identity path is introduced.** The security precondition in
   `tenant_layer.rs` stays exactly as written; nothing new becomes trusted.
4. **Per-merchant values are runtime data, never build inputs.** Backend URL, publishable key
   and social handles resolve through `/store/config`.

### Consequences

**Good.** One mechanism for browser and server. CORS stops applying to storefront traffic, so
the boot-time origin allowlist stops being a per-merchant operational burden. Same-origin also
removes a DNS lookup, a TLS handshake and a CORS preflight from every cross-origin POST — which
matters most on the high-latency mobile connections this product targets.

**Bad.** SSR requests hairpin through the ingress rather than addressing the service directly,
costing an in-cluster hop. Split-horizon DNS removes it if a measurement ever justifies the
extra moving part.

**Landmine.** `middleware.ts` caches the region map in a **module-level `Map` with no tenant
key**. It is harmless only while one process serves one tenant. This ADR makes one process
serve many, so **host-keying that cache is part of this change, not a follow-up.** Shipping the
routing alone converts a dormant bug into a live cross-tenant leak.

---

<a id="adr-023"></a>

## ADR-023 — One storefront build per theme, routed by host

> [!WARNING]
> **ARCHITECTURE DEPRECATION NOTICE**
> This decision to maintain N builds for themes is completely superseded by [ADR-031](#adr-031). All merchant storefronts run on a **single** Next.js unified runtime application cluster. There are no separate themed builds.

**Status:** Superseded by [ADR-031](#adr-031) · **Date:** 2026-09-02 (Finalized 2026-09-08) · **Relates to:** [ADR-013](#adr-013), [ADR-022](#adr-022)

### Context

`STHENO_FRONTEND` binds a frontend at build time and each gets its own `distDir`, so every
theme is a separate deployed artifact. Nothing maps a tenant's host to the right artifact.
With two themes this is invisible; it becomes a manual release checklist at five.

### Options considered

| Option | For | Against |
|---|---|---|---|
| **N builds, host → build routing table** (chosen) | Deploy count is O(themes). Ten thousand merchants, still N builds | A routing table to maintain; a tenant column to keep honest |
| One deploy per merchant | Total per-store freedom | O(merchants) projects and env sets; every engine change is an N-way redeploy. Rejected by [ADR-013](#adr-013) |
| Single build, themes resolved at runtime | One deploy | Dynamic imports throughout, no tree-shaking, and a component API that can never change. [ADR-001](#adr-001) in TypeScript |

### Decision

1. **A `frontend` column on the tenant**, `not null default 'default'`, RLS-covered like every
   other tenant column, exposed on `/store/config`.
2. **A host → build routing table** keyed on the same wildcard the API already resolves.
3. **A CI build matrix over every frontend**, running `check-frontend-switch.sh` per build.
   Non-negotiable: that grep exists because a matrix build once reported *"Compiled
   successfully"* while emitting another frontend's markup, and every other gate passed.
4. **Deployed artifact count must equal registered frontend count, asserted in CI.** It is the
   invariant the model rests on, and it will be violated by someone in a hurry.

### Consequences

**Good.** Theme count and merchant count are decoupled. Adding a merchant is a row, not a build.

**Bad.** A bespoke Pro theme is still a deploy, and themes must be versioned against core.
Budget them against an honestly-built theme, not against `frontend-matrix`, which was cheap
only because it began as a copy of `frontend-default`.

---

<a id="adr-024"></a>

## ADR-024 — Compute sits next to the data; delivery is a CDN over an origin that speaks HTTP caching

**Status:** Accepted · **Date:** 2026-09-02 · **Relates to:** [ADR-012](#adr-012), [ADR-022](#adr-022)

### Context

Two questions arrive together: where does the storefront render, and how is it delivered
cheaply? A global edge network makes both look like one answer — run everything at the edge —
and for this architecture that answer is wrong.

### Decision

1. **The renderer runs next to Postgres, never at the edge.** Cached responses are already
   served near the buyer by the CDN; edge compute only changes *uncached* requests, and those
   are precisely the ones that must reach the single database of [ADR-012](#adr-012). A query
   costs five round trips — ~2.5 ms beside the database, a full second from 200 ms away. Edge
   rendering moves compute 5 ms closer to the buyer and 200 ms further from its data.
2. **A CDN is configuration, never an integration.** No CDN-specific code, ever. The origin
   emits correct HTTP cache semantics and any compliant CDN then works with a DNS record and
   credentials, which is also what keeps the vendor choice reversible.
3. **"CDN-ready" is defined, so it is testable:**
   - `Cache-Control: public, s-maxage=<n>, stale-while-revalidate=<n>` on cacheable responses.
   - **No `Set-Cookie` on anything cacheable.** Every CDN refuses to cache such a response —
     correctly, since caching one would serve a visitor's cookie to every visitor.
   - `Vary: Accept-Encoding`, never `Vary: Cookie`, which fragments the cache per visitor.
   - Cache key is host + path. Host *is* the tenant ([ADR-022](#adr-022)), so tenant-correct
     caching needs no key-construction code and therefore has no key-construction bug.
   - Tag purges emit both `Surrogate-Key` and `Cache-Tag`; they are headers, and emitting both
     is cheaper than caring which vendor is in front.
4. **Storefront SSR targets the Node runtime.** This keeps every host open except edge-only
   adapters, and closing that one is deliberate: the frontend binding is a webpack plugin an
   edge adapter does not honour, and its failure mode is serving another merchant's theme.

### Consequences

**Good.** The bill scales with cache misses rather than page views. Vendor choice stays a DNS
change.

**Resolved blocker (2026-09-08).** `getCacheTag()` was refactored to check against public catalog tags
(`products`, `categories`, `collections`, `regions`, `locales`, `variants`, `shippingOptions`) and
omits `cookies()` for public cacheable requests, restoring Next.js Full Route Cache (ISR).
`middleware.ts` no longer writes `_stheno_cache_id` on the public route path, allowing CDNs to cache
responses directly without `Set-Cookie` invalidation. Origin emits `public_cache_headers` with
`Surrogate-Key` and `Cache-Tag`.

---

<a id="adr-025"></a>

## ADR-025 — Connection pooling is transaction-mode, with a session-mode route for session state

**Status:** Accepted · **Date:** 2026-09-02 · **Relates to:** [ADR-012](#adr-012)

### Context

The engine autoscales and every replica holds its own connection pool, so connections to
Postgres are `replicas x pool_size`. Without a pooler a traffic spike triggers a scale-out that
exhausts `max_connections` and takes down tenants that were already healthy. Postgres
connections, not CPU, are the first ceiling a multi-tenant deployment meets — and the small
binary that makes fine-grained autoscaling attractive is exactly what makes this worse.

### Decision

1. **A pooler sits in front of Postgres in every environment**, including local development.
   Transaction pooling changes runtime behaviour, not merely capacity; discovering that
   difference in production is the failure this prevents.
2. **`pool_mode = transaction`.** [ADR-012](#adr-012) decision 5 already requires `SET LOCAL`
   so a pooled connection cannot carry a tenant identity between requests, and that is exactly
   what transaction pooling needs. **`statement` mode breaks `SET LOCAL` outright and must
   never be set.** Getting this wrong hands the next client the previous tenant's identity, and
   RLS then faithfully serves someone else's rows — it reads as an RLS bug and is a pooling bug.
3. **`max_prepared_statements` must be set, on PgBouncer >= 1.21.** diesel-async runs on
   tokio-postgres, which uses protocol-level prepared statements; without tracking they fail
   under transaction pooling.
4. **Session-scoped state gets its own connection string** — a session-mode alias onto the same
   database. `PostgresAdvisoryLockProvider` takes a session-level `pg_try_advisory_lock` and
   unlocks on the same connection; under transaction pooling the lock leaks onto a server
   connection that then serves other tenants, presenting as an intermittent hang no test
   reproduces. Anything using this route gets no pooling, so it stays low-volume.
5. **Test connections address Postgres directly.** The suite creates and drops a database per
   test and a pooler's database list is static.

### Consequences

**Good.** Connection count stops growing with replica count, so autoscaling becomes safe. PgBouncer
is configured in `docker-compose.yml` on port 6432 in transaction mode with prepared statements tracked.

**Bad.** Another hop and another component. And transaction duration now caps concurrency —
throughput is `pool_size / transaction duration` — which makes the extra round trips on the
non-transactional path a *concurrency* cost rather than merely a latency one.

---

<a id="adr-026"></a>

## ADR-026 — Docker Compose now, Kubernetes at scale

**Status:** Accepted · **Date:** 2026-09-02

### Context

The Helm chart exists and is the eventual target. What runs today is Docker Compose. Recording
this stops the chart being mistaken for the current environment, and stops each decision being
implemented in only one of the two.

### Decision

1. **Everything is containerised. Docker Compose is the current runtime**; Kubernetes is
   adopted when scale requires it, not before.
2. **Every infrastructure decision lands in both**, or it is not done — the pooler as a Compose
   service and a chart template, same-origin routing as a reverse-proxy container and an
   Ingress.
3. **No platform primitive may carry a decision.** Every mechanism chosen is HTTP or Postgres,
   which is what keeps the eventual migration a translation rather than a redesign.

### Consequences

**Good.** The move to Kubernetes stays mechanical. Local development matches production
behaviour.

**Bad.** Two deployment descriptions to keep honest, and they drift silently — nothing fails
when only one is updated. Autoscaling has no Compose equivalent and is simply absent until the
Kubernetes phase.

---

<a id="adr-027"></a>

## ADR-027 — Medusa compatibility is an implementation detail, never a product claim

**Status:** Accepted · **Date:** 2026-09-02 · **Amends:** [ADR-002](#adr-002)

### Context

[ADR-002](#adr-002) chose full wire compatibility with Medusa v2 for two reasons. One was
engineering economy: fork the dashboard rather than rebuild it. The other was commercial —
*"Existing storefronts, `@medusajs/js-sdk`, and Medusa's own admin dashboard work unmodified"* —
a migration story for Medusa users switching to Stheno.

The second reason is withdrawn. Stheno is sold as its own service, not as a Medusa-compatible
one. The first reason stands and is the only one that ever paid: `apps/admin` is coupled to
Medusa at build level through `@medusajs/admin-shared` and `@medusajs/admin-vite-plugin`, and
keeping `/admin/*` shapes stable is what lets ~278 admin route contracts and 38 route groups
keep working without a rewrite.

### Decision

1. **The Medusa name appears nowhere a customer can see it** — not in marketing copy, product
   pages, dashboard links, cookie names, environment variables, or error messages.
2. **Admin wire compatibility is retained**, as an internal economy. It is never marketed, and
   it is not a promise to any third party.
3. **The Store wire is free to diverge.** Nothing external depends on it once the migration
   story is withdrawn, so `@medusajs/js-sdk` may be replaced by a thin fetch client. Doing so
   removes `x-medusa-locale` from the wire, the Node-only constraint on `lib/data/`, and the
   build-time publishable key together — see [ADR-022](#adr-022) decision 4.
4. **Engineering provenance stays.** ADRs, `UPSTREAM.md`, `PARITY_MATRIX.md` and source
   comments continue to name Medusa where it explains why something is the way it is. Removing
   that would make the codebase harder to reason about and fools nobody.
5. **`@medusajs/ui`, `@medusajs/icons` and `@medusajs/ui-preset` are a design system, not a
   brand tie.** They emit `ui-*` class names, so the name does not reach the DOM. Replacing
   them is cost without benefit.

### What changed on 2026-09-02

Cookies renamed to `_stheno_*`; every `MEDUSA_*` environment variable renamed to `STHENO_*`
across apps, gate scripts and CI; the dashboard's Documentation and Changelog links moved off
`medusajs.com` to configured URLs that hide when unset; Medusa Cloud's hosted-Stripe fallback
and demo image buckets deleted; and both marketing claims rewritten.

One `docs.medusajs.com` link remains in `edit-profile-form.tsx`, inside a JSX block comment
around a disabled usage-insights feature. It is unreachable and was left rather than edited.

### Consequences

**Good.** The product stands on its own name. The store wire stops constraining decisions that
were only ever made to preserve a promise no longer being kept.

**Bad.** Diverging the store wire means the forked storefront must be maintained against an API
only we define — the fork is now genuinely ours, with no upstream to pull from. That was
already true in practice; this records it.

**Cost of the exception.** Keeping `/admin/*` compatible means a Medusa upgrade can still, in
principle, force admin-side work. That is accepted: it is cheaper than owning 278 route
contracts against a UI we did not write.

---

<a id="adr-028"></a>

## ADR-028 — Customer-facing brand is Vendryx; internal identifiers remain `stheno-*`

**Status:** Accepted · **Date:** 2026-09-08

### Context

The commerce engine, APIs, and UI suites were originally built and documented under the internal
engineering codename `stheno`. As the product enters commercial operations for merchants, a distinct
and mature customer-facing brand identity — **Vendryx** — has been established by Syed Systems.

Renaming every identifier across a 40-crate modular Rust workspace, database schemas, migration
history, API headers, and deployment configurations simultaneously introduces immense operational
risk, schema drift, and build disruption for zero commercial value.

### Decision

1. **Customer-facing brand is Vendryx.** All public interfaces, marketing copy, website pages,
   login and onboarding flows, merchant dashboard UI, email templates, favicons, wordmarks,
   metadata, and public documentation adopt **Vendryx** (and *Vendryx by Syed Systems* where legal
   attribution is required).
2. **Internal identifiers remain `stheno-*`.** Rust crate names (`crates/modules/stheno-*`,
   `stheno-types`, etc.), Cargo package and binary names (`stheno-server`, `stheno-cli`), Postgres
   database schemas (`payment`, `fulfillment`, etc.) and migration history, environment variables
   (`STHENO_*`), compatibility headers (`X-Stheno-CSRF`), and internal TypeScript packages
   (`@stheno/*`) remain unchanged during this phase to preserve stability and wire compatibility.
3. **Machine-readable manifest is source of truth.** `brand/vendryx/brand.json` defines approved
   names, legal copy, asset paths, domains, and social handles. Automated verification scripts
   (`scripts/check-brand-surface.sh`) enforce that public surfaces display Vendryx and never the
   legacy customer-facing name.
4. **No blind find-and-replace.** Code identifiers, historical ADRs, test fixtures, and internal
   provenance are maintained explicitly.
5. **Provider contract.** Merchant UI displays only zero-code supported providers compiled into the
   server registry. Unimplemented or unverified providers remain hidden from merchants.

### Consequences

**Good.** Clear, clean separation between external branding and internal implementation stability.
Public users experience a unified, premium Vendryx product while CI, migrations, and crate
dependencies compile without regression.

**Bad.** Engineers must remember that `stheno-*` represents the internal engine powering Vendryx.
Public documentation includes a single standard note explaining this relationship.

**Cost of the exception.** None. A later, dedicated compatibility migration can rename internal
crates and variables if ever deemed necessary.

---

<a id="adr-029"></a>

## ADR-029 — Merchant Admin Truth Contract & Apple HIG Information Architecture

**Status:** Accepted · **Date:** 2026-09-08

### Context

The merchant admin dashboard contained several mock implementations, simulated state (e.g.
`localStorage` for sessions and domains, `setTimeout` pretending to verify domains or update passwords,
hardcoded dummy invoices and token strings), and lacked a cohesive information architecture in Settings.
Settings was a flat unorganized list with an instant redirect to `/settings/store`, hiding crucial
features and overwhelming merchants.

Furthermore, route collisions existed (such as `/orders/:id` capturing `/orders/social` before the
latter could resolve) and several unmounted routes were referenced in navigation.

### Decision

1. **Truth Before Beauty Contract.** Zero mock storage or fake timers in production admin code.
   Every UI card, switch, and list must bind to a real, tested backend endpoint or be unmounted from
   navigation until the corresponding backend contract is delivered.
2. **Backend-First API Parity.** Implement genuine backend services and mount endpoints before
   functionalizing UI:
   - `GET /admin/rbac/me/permissions` resolves staff permissions across assigned roles and policies.
   - Column metadata and view configurations for all admin data tables (`columns_for()`).
   - `GET/POST /admin/settings/{key}` and `GET /admin/settings?category=...` for tenant business policies
     (such as Cash-on-Delivery rules and store policies) with schema and size validations.
   - `GET/POST /admin/domains`, `POST /admin/domains/{domain}/verify`, and `DELETE /admin/domains/{domain}`
     with tenant scoping, DNS record instructions, and conflict checks.
   - `GET /admin/billing/subscription` and `GET /admin/billing/invoices` exposing live billing and usage.
3. **Apple Human Interface Guidelines (HIG) Architecture.**
   - Redesign `/settings` into an organized Settings Hub featuring 7 clear categories:
     Store & Operations, Product Catalog, Sales & Customers, Delivery & Fulfillment, Payments & Finance,
     Security & Access, and Developer & Integrations.
   - Interactive search with synonyms (e.g. searching "vat" surfaces Tax Regions; "cash" surfaces
     Payment Providers; "rbac" surfaces Users & Permissions).
   - Component primitives: `SettingsRow`, `SettingsSection`, `StatusAccessory`, and `AsyncState`
     providing minimum 44px touch targets, progressive disclosure, and responsive push navigation.
4. **Route Integrity.**
   - Ensure specific static subroutes (`/orders/social`) mount before parameterized paths (`/orders/:id`).
   - Route `/profile` cleanly redirects to `/settings/profile`.
   - Remove orphaned/unmounted routes from navigation until implemented.
5. **Automated Enforcement.**
   - CI enforces zero mock storage in admin routes via `scripts/check-no-mock-storage.ps1` (denying
     `localStorage` and simulation timers across `apps/admin/src/routes`).

### Consequences

**Good.** Eliminates deceptive merchant states, fake successes, and data loss from browser refreshes.
Merchants navigate a predictable, polished, Apple HIG-compliant configuration suite backed by robust Rust APIs.

**Bad.** Some incomplete auxiliary mock screens (like fake ticket simulators) are removed or simplified
to knowledge hubs until genuine service contracts exist.

**Cost of the exception.** Free-tier platform-funded notifications are capped at the daily allowance above; outbound provider charges above that allowance cannot be incurred on Free traffic.

---

<a id="adr-030"></a>

## ADR-030 — Company Control Plane extracted to standalone 'Space' project

**Status:** Accepted · **Date:** 2026-09-08 · **Refines:** [ADR-012](#adr-012), [ADR-016](#adr-016)

### Context

The platform control plane began as `apps/platform` and `stheno-api-platform` inside the Store
repository. Its scope originally grew to manage tenants, subscriptions, plan definitions, and
operator identity for the commerce engine.

However, Syed Systems operates and is expanding multiple B2B and SaaS products — including Vendryx
Commerce, ERP, LMS, and future offerings. Operating commercial relationships, organizations,
memberships, unified billing, cross-product catalog bundles, and provisioning inside a single product's
repository (Store) would:
1. Inappropriately couple unrelated enterprise products (ERP, LMS) to Vendryx commerce release cycles
   and schemas.
2. Violate the shared-kernel boundary principle by turning `stheno-platform` into an unwieldy
   company-wide dependency.
3. Obscure commercial and operational oversight across disparate domain systems.

### Decision

1. **Independent Standalone Project:** The company control plane is extracted into **Space**, an
   independently deployed and maintained project situated at `D:\Syed Systems\Space` (a standalone
   folder beside `Store`). Space has its own workspace, `apps/space` console, `crates/space-server`,
   and migration history.
2. **Product Connector Architecture:** Products remain authoritative for their own domain databases.
   Vendryx owns stores, orders, and products; ERP owns ledgers and payroll; LMS owns courses. Space
   does not read or join product databases directly.
3. **Integration Boundary:** Vendryx Commerce connects to Space through versioned connector APIs
   (`/space/integration/v1/*`) using signed delivery, service identities, idempotent commands, and
   replay protection.
4. **Phased Migration (SPACE-ADR-007):** In-repo platform components (`apps/platform`,
   `stheno-api-platform`) are maintained as the legacy surface during the transition (expand, shadow,
   reconcile, cutover, and contract phases). Existing verified money, dunning, and audit logic will
   be wrapped by the Vendryx Space connector.

### Consequences

**Good.**
- Clean separation between company-level SaaS management (Space) and commerce engine internals (Vendryx).
- Enables cross-product customer organizations, unified billing, and centralized entitlement resolution.
- Preserves module isolation and keeps `stheno-types` starved of company-level domain concepts.

**Bad.**
- Requires maintaining an interim transitional platform in `Store` until the connector cutover is
  complete.
- Two repositories to deploy and coordinate during migration.

**Cost of the exception.** None. This replaces an ad-hoc commerce-centric control plane with a true
multi-product architecture.

---

<a id="adr-031"></a>

## ADR-031 — Dynamic Multi-Tenant Visual Theming on a Unified Default Storefront Runtime

> [!WARNING]
> **Superseded by [ADR-038](#adr-038) on 2026-11-27.** The Next.js runtime and the single
> `frontend-default` chassis are replaced by engine-rendered template themes. The token principle,
> the Apple HIG baseline (as the Default theme) and instant admin preview carry forward. This record
> still describes the storefront that serves until the ADR-038 cut-over.

**Status:** Superseded by [ADR-038](#adr-038) · Originally accepted 2026-09-09 · **Date:** 2026-09-09 · **Relates to:** [ADR-012](#adr-012), [ADR-013](#adr-013), [ADR-022](#adr-022), [ADR-023](#adr-023)

### Context

Merchants require distinct, branded storefront experiences (color schemes, curated typography, card aspect ratios, squircle corner radii, and Apple Human Interface Guidelines aesthetics).

However, deploying separate Next.js servers per merchant or theme creates:
1. Operational sprawl: N running containers, complex build pipelines, DNS routing fragility, and high hosting bills.
2. Divergence and bugs: Rebuilding or copying the DOM creates divergence in form validation, cart state, accessibility (WCAG 2.2 Level AA), and checkout integration.
3. Slower merchant updates: Recompiling or redeploying for simple styling updates introduces unacceptable minutes of delay.

The reference package `frontend-default` contains a battle-tested, mature DOM structure that already correctly handles complex edge cases, responsive behaviors, accessibility, and secure checkout boundaries.

### Options considered

| Option | For | Against |
|---|---|---|---|
| Deploy separate Next.js servers per merchant/theme | Complete DOM isolation per store | Severe server proliferation, high infrastructure costs, complex deployment orchestrations, version drift |
| Runtime dynamic module loading / component plugins | Can swap React components per request | Requires dynamic imports, defeats tree-shaking, unstable component ABI. Rejected by [ADR-001](#adr-001) and [ADR-013](#adr-013) |
| **Unified default storefront with runtime design tokens & visual variant modifiers** (chosen) | One Next.js server serves all merchants. Battle-tested mature DOM retained. Instant zero-deploy styling changes | Visual variations must be expressed through parameterized tokens, CSS custom properties, and variant data attributes rather than arbitrary freeform JSX forks |

### Decision

1. **Unified Next.js Storefront Runtime:** All merchant storefronts run on a single Next.js application instance/cluster. No per-merchant or per-theme server deployment is required for visual differentiation.
2. **Preserve `frontend-default` as the Mature Structural Foundation:** The rock-solid, accessible DOM structure of `frontend-default` remains the canonical chassis for the store.
3. **Runtime Design Token Injection:**
   - Next.js `middleware.ts` resolves the tenant identity from the `Host` header in `<1ms`.
   - `storefront-core/src/app/layout.tsx` dynamically injects tenant-specific design tokens into `<style id="stheno-brand-tokens">:root { ... }</style>` on SSR.
   - Tokens govern: Color system (`--brand-primary`, `--brand-surface`, `--brand-background`, `--brand-text`, `--brand-hairline`), Typography (`--brand-font-family`), Corner Radii (`--brand-radius-card`, `--brand-radius-button`, `--brand-radius-input`), and Materials (frosted glass blur).
4. **Config-Driven Visual Variant Modifiers:**
   - Mature components support semantic data attributes and variant modifiers (e.g. `data-card-style="apple-minimal"`, `data-image-aspect="portrait"`, `data-buy-box="sticky"`).
   - Enables presentation variations (Apple sticky 2-column detail page vs inline, 1:1 square vs 4:5 portrait media, segmented option pills vs standard chips) without mutating core data pipelines or validation logic.
5. **Apple Human Interface Guidelines (HIG) as the Platform Baseline:**
   - The default aesthetic baseline adopts Apple HIG: 8pt spatial grid, generous whitespace, high-contrast typography with tabular pricing numerals, smooth 16px squircle corner radii, 9999px full-pill buttons, translucent frosted materials (`backdrop-filter`), and fluid spring transitions (`cubic-bezier(0.16, 1, 0.3, 1)`).
6. **Instant Zero-Deploy Real-Time Admin Preview:**
   - When merchants customize their theme in the Admin settings, changes are previewed live via `postMessage` (`STHENO_PREVIEW_TOKENS`), providing instant feedback without build cycles or server redeployments.

### Consequences

**Good.**
- One Next.js server serves thousands of distinct branded storefronts with zero deployment overhead.
- Preserves the battle-tested, accessible DOM structure and PCI SAQ-A checkout compliance across all stores.
- Platform improvements, bug fixes, and security patches apply instantly to all stores.
- Instant merchant theme updates on save (0 seconds delay).

**Bad.**
- Visual variations must be structured through tokens and modifier attributes rather than uncontrolled ad-hoc JSX rewriting.
- Merchants wanting fundamentally incompatible commerce paradigms (such as live auction bidding or multi-step quote builders) cannot be satisfied by CSS/variant tokens alone and require custom service engagements per ADR-013.

**Cost of the exception.** None. This replaces operational server proliferation with a scalable token-driven multi-tenant architecture.

---

<a id="adr-032"></a>

## ADR-032 — AI-Powered Social Commerce Automation: OKF Architecture, Tool Calling & Safe Draft Intake

**Status:** Accepted · **Date:** 2026-10-22 · **Amended:** 2026-11-14 (see Amendment below) · **Relates to:** [ADR-012](#adr-012), [ADR-014](#adr-014), [ADR-019](#adr-019), [ADR-021](#adr-021), [ADR-031](#adr-031)

### Context

Merchants operating on social channels (Facebook Messenger, WhatsApp, Instagram) lose substantial revenue when customer inquiries go unanswered during off-hours or peak volume. Introducing conversational AI into e-commerce operations creates severe failure modes if unconstrained:
1. **Model Hallucinations:** Unbounded language models invent non-existent products, hallucinate discounted prices, or promise delivery terms that contradict merchant policy.
2. **Vector Embedding Stale Data Trap:** Embedding live inventory counts or prices into vector stores causes critical race conditions where out-of-stock items are sold or outdated prices are quoted.
3. **Runaway Operational Inference Costs:** Using premium frontier models ($15–$60 per million tokens) for high-frequency e-commerce chatter burns platform margin on a $20/month subscription.
4. **Unauthorized Direct Commitments:** Permitting an autonomous agent to directly deduct inventory, charge credit cards, or dispatch couriers without human verification creates massive liability from malicious buyer inputs or prompt injection.

### Decision

1. **Proprietary Open Knowledge Framework (OKF) with Tenant Isolation:**
   - Each merchant owns a proprietary knowledge base (store policies, FAQs, sizing charts, returns, delivery timelines, business hours) stored in PostgreSQL with strict `tenant_id` Row-Level Security (RLS).
   - Zero multi-tenant leakage: Knowledge retrieval queries are scoped strictly to the current tenant. No merchant's proprietary customer data or business rules enter a shared general-purpose corpus.
2. **Separation of Static Policy Knowledge vs. Dynamic Live Commerce:**
   - Static knowledge (policies, FAQs, shipping guidelines) is retrieved via keyword/vector search within the tenant's OKF.
   - Dynamic live commerce data (product variants, live prices, stock levels) is **never vectorized or guessed**. The AI brain interacts with the catalog strictly through typed function tools executing against the Rust engine:
     - `search_catalog(query)`
     - `get_product_details(variant_id)`
     - `check_variant_stock(variant_id)`
     - `calculate_shipping_rates(zone)`
   - Server-side price authority ([ADR-014](#adr-014)): the AI model may never choose, calculate, negotiate, or invent a price. The live published price remains authoritative unless the merchant explicitly configures a chat-only offer in the admin dashboard. A server-side offer tool may return an exact authorized price only for its configured published variant, currency, quantity, and validity window; the model may quote that returned amount and terms exactly. Customer messages and OKF knowledge cannot create or change offers. If no offer qualifies, use the live published price and sell on verified fit and benefits without pressure or deception.
3. **Ultra-Low-Cost Brain Architecture:**
   - The platform standardizes on cost-effective, high-throughput LLMs (DeepSeek V3/R1, Qwen 2.5, or equivalent ultra-low-cost API endpoints) costing <$0.50 per million tokens.
   - Keeps inference costs under fractions of a cent per conversation, preserving the profitability of the $20/month Pro tier even under heavy chat volume.
4. **Safe Draft-Order Intake Boundary (Human-in-the-Loop):**
   - The AI assistant handles intake and information gathering; it **never executes final order commitments, captures payments, deducts warehouse inventory, or dispatches couriers**.
   - Upon gathering buyer details (Name, Phone, Delivery Address, Variant, Quantity, Payment Preference), the AI calls `create_social_draft_order`.
   - Generates a human-friendly short code (`Ref: #VX-XXXX`) avoiding ambiguous characters (`0/O`, `1/I`).
   - The draft order is deposited into the Merchant Admin **Social Queue** (`/admin/orders?status=draft&is_social=true`).
   - The merchant reviews and clicks "Confirm" in their dashboard, which revalidates price and availability, then triggers standard inventory reservation and courier dispatch. A chat offer must remain valid at confirmation or the order is held for staff price review; confirmation cannot silently charge above the shopper-visible quoted price.
5. **Pro Dedicated AI & Messaging Panel:**
   - Pro merchants configure their Meta Webhook credentials (WhatsApp Cloud API Phone ID & Token, Messenger Page ID & Token, Instagram Account ID) inside `/admin/ai`.
   - Credentials are encrypted at rest with tenant secret keys ([ADR-019](#adr-019)).
6. **Courier ambiguity is reconciled before any retry:**
   - A courier timeout or lost response remains `unknown`; neither staff nor a retry may infer failure from the absence of a local fulfillment row.
   - An authenticated staff member records a required courier evidence reference after checking the courier's authoritative record by the stable merchant order reference. `accepted` requires a courier consignment reference and tracking number; the server stores only those bounded canonical fields and uses them to finish the interrupted local fulfillment writes without calling the courier again.
   - `not_created` is allowed only after the courier's authoritative lookup confirms that no consignment exists. The same durable request row is claimed for the next attempt, so concurrent replays cannot dispatch twice. Every resolution records the actor, outcome, evidence reference, and timestamp. The route is tenant-scoped and never accepts credentials or arbitrary provider payloads.

### Consequences

**Good.**
- 24/7 sales intake on WhatsApp, Messenger, and Instagram without merchant fatigue.
- Zero price hallucinations and zero out-of-stock commitments via live tool calling.
- Strict tenant data isolation under Postgres RLS.
- Human confirmation eliminates risk of prompt injection causing unauthorized shipments.
- Predictable ultra-low operational inference costs.

**Bad.**
- Orders require merchant manual confirmation before dispatch rather than being 100% autonomous.
- Merchants must complete Meta Business API setup for white-labeled social inboxes.

**Cost of the exception.** None. This replaces risky autonomous agent spending with a safe, verified human-in-the-loop commerce pipeline.

### Amendment (2026-11-14)

Accepted by the owner on 2026-11-14. The design, acceptance criteria and order of work are in
[OKF_PLAN.md](OKF_PLAN.md). None of this is implemented yet.

1. **Static knowledge is sent whole, not retrieved.** This replaces the retrieval half of
   decision 2. All of a tenant's enabled knowledge entries are sent to the model on every call,
   in a stable order, up to a size cap. Keyword search remains only for a tenant above the cap.
   Reason: a merchant's policies are a few pages, and keyword search cannot match a Banglish
   question to a policy written in English or Bangla script. Live commerce data is unchanged:
   it still comes only from typed tools and is never placed in the knowledge prompt.
   Cost: more input tokens per call, so the per-conversation token budget must be raised and
   re-priced before this ships.
2. **Every conversation has one state, and a human takeover silences the assistant.** States
   are `assistant`, `suggesting`, `human` and `closed`. A buyer asking for a person, a
   non-text message, a provider failure, an exhausted budget, repeated fallback answers, a
   complaint, or any staff reply moves the conversation to `human`, where the assistant sends
   nothing and drafts nothing until staff release it. A reply generated before the state
   changed is discarded before sending.
3. **Each store runs in one mode: `off`, `suggest` or `autonomous`.** New stores start `off`.
   In `suggest` the assistant drafts and staff send. `autonomous` is offered only after the
   store has used suggestions. A platform operator can force any store, or all stores, to
   `off`; this is the kill switch and it takes effect on the next inbound message.
4. **Merchant-visible transcripts are kept for 30 days.** This replaces the 24-hour expiry for
   history the merchant can read; the model's working context stays a short recent window.
   Transcripts are encrypted, tenant-scoped under RLS, included in tenant export and removed by
   tenant erasure. Reason: a merchant must be able to check what the assistant told a buyer
   when a dispute arrives days later.
   Cost: buyer messages are held 30 times longer, which raises the stakes of any breach and
   must be stated in merchant and buyer terms.

Decision 4 (human-confirmed draft intake) and the server-side price authority in decision 2 are
not changed by this amendment.

Implementation sequencing and edge-case acceptance checks are tracked in
[ADR_IMPLEMENTATION_PLAN.md](ADR_IMPLEMENTATION_PLAN.md#adr-032--social-commerce-and-okf).

---

<a id="adr-033"></a>

## ADR-033 — Commercial Entitlements Overhaul: Free & 7-Day Pro Trial Packaging

**Status:** Accepted · **Date:** 2026-10-22 · **Relates to:** [ADR-009](#adr-009), [ADR-012](#adr-012), [ADR-016](#adr-016), [ADR-017](#adr-017), [ADR-031](#adr-031), [ADR-032](#adr-032)

### Context

The platform previously documented a 14-day trial, an indefinite Free tier, and a $20/month Pro tier, but required formal rationalization of feature allocations, trial lengths, anti-tamper mechanisms, and upgrade UX.

### Decision

1. **Streamlined Two-Tier Model with 7-Day Pro Trial:**
   - **Free Tier (Indefinite):** 100 products, 3 staff seats, 30-day analytics, standard Apple HIG chassis with automated logo/category styling, custom domain mapping (no Vendryx connection fee), Cash on Delivery (COD) & manual bKash/Nagad paywall, manual social chat links (WhatsApp wa.me link & Messenger/IG clipboard copy), Web Push & Mobile App Push notifications, plus platform-funded email/WhatsApp notifications capped at 200 emails and 300 WhatsApp messages per tenant per UTC day.
   - **Pro Tier ($20/month, with 7-Day Free Trial):** Unlimited products, 10 staff seats, full historical analytics, online automated payment gateways (Stripe, bKash automated tokenized, Nagad direct, SSLCommerz), automated courier dispatch (Pathao, Steadfast, RedX), 24/7 AI Sales Automation & OKF, AI predictive restock insights, cart analytics (guest checkout tracking by IP/session), bespoke theme selection & variant modifiers (`data-*`), and fraud risk signal scoring.
2. **Pro is a Strict Superset:**
   - Pro merchants inherit 100% of Free tier features. Pro never removes, restricts, or replaces any capability available on the Free tier.
3. **Database-Baked Entitlements & 7-Day Grace Period:**
   - Entitlements are resolved per-request from `platform.subscription` and `platform.tier_definition`.
   - On subscription expiry (`term_ends_at < now()`), status transitions to `grace` for 7 days.
   - Admin displays a persistent countdown warning banner.
   - At day 8, status shifts to `free`. Grandfathering rule: Existing products and store remain online; creates over 100 and Pro-only capabilities lock down.
4. **Anti-Tamper & Console Immunity:**
   - Server-side token emission on Next.js SSR ensures unentitled Pro theme CSS is never sent to the browser.
   - Rust backend API validates `ctx.entitlements.has(...)` on every write; local DevTools edits cannot save Pro tokens or trigger Pro webhooks.
5. **Engaging Pro UI Teaser:**
   - Free merchants see Pro features labeled with a glowing orange `[PRO]` pill.
   - Clicking opens an interactive blurred frosted-glass paywall modal with a $20/mo upgrade CTA.

### Consequences

**Good.**
- Clear commercial boundary: Free remains indefinite, and platform-funded email/WhatsApp costs have a hard per-tenant daily ceiling.
- Shortened 7-day trial accelerates paid conversion velocity.
- Impossible for free users to unlock paid themes or AI features via browser console tampering.
- Zero data loss or product deactivation on downgrade preserves merchant trust.

**Bad.**
- Merchants on Free cannot customize store layout positioning or use automated payment gateways.

**Cost of the exception.** The platform funds at most 200 Free-tier emails and 300 Free-tier WhatsApp messages per tenant per UTC day. Trial and Pro are uncapped by this allowance.

---

<a id="adr-034"></a>

## ADR-034 — Manual Payment "In Verification" Truth Contract & Email/WhatsApp Channel Pipeline

**Status:** Accepted · **Date:** 2026-10-22 · **Relates to:** [ADR-002](#adr-002), [ADR-013](#adr-013), [ADR-017](#adr-017), [ADR-028](#adr-028), [ADR-033](#adr-033)

### Context

Manual payment methods (bKash, Nagad, Rocket, Bank Transfer) represent the majority of retail commerce in South Asia. In manual checkout, a buyer transfers funds and submits their Transaction ID (TrxID).
Two major defects frequently plague commerce platforms handling this flow:
1. **Deceptive Verification UX:** Displaying "Payment Verified" immediately upon user submission of a TrxID confuses customers and causes disputes before a merchant has checked their bank/bKash statement.
2. **Telco SMS Friction & Aggregator Costs:** Outbound SMS requires expensive local aggregator contracts, masking fees, and regulatory overhead.

### Decision

1. **Zero SMS Policy — Email & WhatsApp Exclusively:**
   - All customer transaction updates, order confirmations, and verification notices are transmitted strictly via **Email** and **WhatsApp**. Outbound telco SMS aggregators are eliminated platform-wide.
2. **Manual Payment Paywall & Refresh Resilience:**
   - Free and Pro merchants can display manual mobile banking instructions ("Send ৳X to 017XXXXXXXX").
   - The cart session is persisted in an encrypted cookie (`stheno_cart_id`) and PostgreSQL. If the buyer refreshes or switches apps to check their banking SMS, the checkout reloads directly to the TrxID submission step without losing cart state or address.
3. **Truth Contract — "Payment In Verification":**
   - When the buyer submits their TrxID, the status is set to `pending_verification` (NEVER `paid` or `verified`).
   - Screen displays: *"Payment Under Verification — Your Transaction ID has been submitted and is currently being verified by our team. You will receive a confirmation message once verification is complete."*
4. **WhatsApp Sender Identity Split:**
   - **Free Tier:** Dispatched from Vendryx central verified WhatsApp Business account (`Vendryx Notifications on behalf of [Store Name]`).
   - **Pro Tier:** 100% white-labeled! Sent directly from the merchant's own connected WhatsApp Business Account (WABA) with their store name, logo avatar, and phone number.
5. **Free-Tier Platform-Funded Notification Allowance:**
   - Free merchants may send at most **300 WhatsApp messages and 200 emails per UTC calendar day** through platform-funded delivery. The allowance is shared across all dispatch paths and enforced atomically per tenant/channel, including concurrent workers. When an outbox reaches its cap, it remains queued for the next UTC day; direct sends fail with a stable quota error. Trial and Pro subscriptions are not subject to these Free-tier caps.

### Consequences

**Good.**
- Completely eliminates SMS aggregator fees and masking red tape.
- Eliminates buyer confusion over unverified payments.
- Free merchants get automated WhatsApp confirmations without setting up Meta developer accounts.
- Pro merchants get a fully white-labeled branded sender identity.

**Bad.**
- Buyers without email or WhatsApp must rely on storefront order status receipts.

**Cost of the exception.** None.

---

<a id="adr-035"></a>

## ADR-035 — Architecture Implementation Distance & Readiness Audit

**Status:** Accepted · **Date:** 2026-10-22 · **Relates to:** [ADR-031](#adr-031), [ADR-032](#adr-032), [ADR-033](#adr-033), [ADR-034](#adr-034)

### Context

Following the consolidation of ADR-031 through ADR-034, an authoritative accounting is required to record what is already completed versus the exact implementation distance remaining for production readiness.

### Decision

Record the authoritative platform readiness matrix and implementation delta:

#### 1. Completed Baseline (GA Ready):
- **Commerce Engine Core:** 34 modules, 249 workflows, Diesel async, single Postgres database with Row-Level Security (`tenant_id`), connection pooling (P0–P20 complete).
- **Phase 3 GA Hardening (95 / 95 Tasks Verified):**
  - Provider platform & Bangladesh payment gateways (P21–P22).
  - Courier integrations: Pathao, Steadfast, RedX (P23).
  - COD / Offline settlement provider (`stheno-payment-offline`) (P24).
  - Storefront Core/Theme separation & Unified Next.js runtime (ADR-031, P25).
  - Branding tier & `/store/branding` endpoint (P26).
  - Theme conformance suite & Apple HIG baseline (P27).
  - PCI SAQ-A & security audit gates (P28).
- **Database Schema:** `platform.subscription`, `platform.tier_definition`, `platform.invoice_payment_proof` migrations applied.

#### 2. Implementation Distance (reconciled 2026-10-24):

The matrix records source evidence, not test completion. No percentage or calendar estimate is
assigned; outstanding gates and implementation boundaries live in
[ADR_IMPLEMENTATION_PLAN.md](ADR_IMPLEMENTATION_PLAN.md).

| Workstream | Source evidence present | Remaining implementation or acceptance work |
|---|---|---|
| **A: AI Sales & OKF** | Tenant-isolated knowledge, bounded catalog/shipping tools, constrained LLM orchestration, Meta webhook ingress, social draft handoff, staff confirmation, owner-configured chat offers and fail-closed pricing checks are wired. | Phase edge matrices and isolated Space gates; complete any missing authorization/recovery cases recorded in phases 32.5–32.6. |
| **B: Manual payment** | Invoice and customer-order proof ledgers, encrypted cart cookie rotation, customer TrxID submit/reload UI, merchant review, truth-preserving pending/approved/rejected email notices, and a separately configured encrypted `pp_manual` provider with checkout instructions are wired. The `pp_system_default` COD provider is distinct and does not request a TrxID. | End-to-end refresh recovery and authorization/concurrency acceptance. |
| **C: Transaction notifications** | Durable outbox covers trial warnings, customer order confirmation, account welcome/invite, payment-proof lifecycle, successful payment capture/refund and admin-recorded payment-collection paid status, return receipt/cancellation, order cancellation, and fulfillment preparation/shipped/delivered/canceled notices. Fulfillment preparation is announced only after the order status write succeeds and says explicitly that shipment has not happened yet. Return receive/confirm/cancel-receipt handlers now execute the order service transition rather than returning a fabricated status; received notices use stable per-return/status keys and state that receipt remains under review and does not mean a refund was issued. Capture/refund notices verify the payment collection's unique order association before notifying. Store return creation and reads require customer authentication and verify order ownership. Return completion cannot create a synthetic USD refund ledger entry; positive refunds must use the payment service and actual payment currency. WhatsApp consent is tenant-scoped and rechecked at send; authenticated customers can revoke all consent linked to their customer id, including a prior phone number. Free uses the central sender, entitled Pro/trial uses the configured merchant sender, approved-template payloads are validated, merchant credentials are encrypted, and Meta ambiguity is dead-lettered. Atomic Free caps enforce 200 emails and 300 WhatsApp messages per tenant per UTC day across direct and queued sends. A tenant-scoped admin API and dashboard now show sent, reserved, remaining, and unlimited daily allowance states. | Additional lifecycle events identified by the acceptance matrix, WABA/template deployment configuration, and focused provider/quota gates. |
| **D: Entitlement/admin** | Seven-day trial lifecycle and warning scheduling, `/admin/ai`, subscription status/countdown/upgrade surfaces, chat-offer controls, server-side Free notification caps, and tenant-scoped notification usage reporting in the admin dashboard are wired. | Product quota and route matrices, billing/browser journeys, and acceptance gates for tier transitions and credential handling. |
| **E: Store intelligence** | ADR-031's unified runtime and dynamic theme-token path provide the rendering foundation. A deterministic, token-free first-party order trend endpoint and owner dashboard are wired for product/category units across consecutive 7-day windows. The dashboard generates bounded next-step candidates, retrieves up to three tenant-owned knowledge excerpts with source IDs and lookup status, and can explicitly request a citation-validated, no-tools ranking from a fixed loopback Ollama endpoint. Ranking is optional, locally hosted, concurrency-limited, and abstains on invalid evidence/output. Owner/admin review, dismiss, and revert decisions persist to tenant-RLS state with append-only actor-attributed audit events and an owner-only bounded history view; review never executes commerce changes. Branding settings can sample a bounded same-origin raster logo palette into an explicitly applied preview/save path. An owner/admin can configure and on-demand query a Google Search Console property using encrypted, merchant-authorized refresh credentials; its date-only aggregates remain separate from order-derived demand, report partial coverage, persist tenant-RLS metric observations keyed by source/property/window for a 400-day API window, and expose exact basis-point comparisons only across complete compatible adjacent windows. Connect/update/disable/disconnect events record the owner/admin actor without credentials, and disconnect clears credentials and observations atomically. Read-only Facebook Page and Instagram professional-account insights now fetch only account-level aggregates through owner/admin routes, store source-specific metric definitions, and expose bounded history with compatible adjacent-window comparisons. WhatsApp analytics requires a separate explicit opt-in ledger, distinct from transactional order-update consent; customer revocation and WhatsApp STOP withdraw both. The cross-schema query returns only counts for processed inbound events after the current analytics consent and marks the result partial; no phone number or sender hash leaves the query layer. | Read-only Facebook Page and Instagram professional-account insights adapters now fetch only account aggregates through owner/admin routes, retain source-specific definitions, and persist bounded observations; Meta permission failures and partial pagination fail closed. Cross-source count aggregation is limited to identical normalized definitions on complete disjoint partitions. Owner/admin Google OAuth consent now signs state against tenant, actor, and property, consumes a one-time ten-minute tenant-RLS nonce, exchanges the code using the configured exact callback, and persists the read-only refresh token encrypted with actor audit. Deployment credentials can be referenced from existing Kubernetes Secrets through the Helm chart; live callback/property acceptance remains unverified. Distinct Meta Insights credentials and owner/admin controls, plus the cross-schema export/delete workflow, are implemented. The server export/erasure and owner-role target and the platform purge regression compile on Space but have not yet run against PostgreSQL because the Space service-capacity preflight is below its required memory. Schema-drift, broader API authorization, dashboard journey, live Search Console/Meta scope acceptance, and production acceptance remain outstanding. WhatsApp analytics implementation and fixtures are present. The new cross-schema lifecycle acceptance is queued for Space PostgreSQL execution after the service-capacity preflight passes; schema-drift acceptance remains pending. Search Console provider fixtures and the `configuration_secrets` persistence/audit/disconnect target passed on Space; the dedicated Meta Insights configuration target also passed 2/2. The new cross-schema export/purge and owner-role target compiles on Space, but its database execution is pending the service-capacity preflight. Deployment configuration and live Google property/Meta token permission acceptance have not run. Unconnected sources must never be presented as measured data. |

#### Store Intelligence scope and trust rules

Store Intelligence is an evidence-backed decision-support system. Its initial source is the
tenant's own non-draft, non-canceled order snapshots. It compares the latest rolling 168 hours
to the preceding seven days and reports product/category units, direction, and a bounded,
deterministic evidence-volume label (low: fewer than 5 product-order observations, medium: 5–19,
high: 20+; these are not statistical confidence intervals). Aggregation is tenant-scoped, excludes
buyer identity and message content, uses integer arithmetic, and costs no model tokens. Missing
history yields an explicitly missing percentage baseline; it must not be reported as a measured
percentage increase.

Facebook, Instagram, WhatsApp, and public online-search trends may be added only through
authorized platform APIs, merchant-authorized exports, or other documented/licensed data sources.
Meta post-insight APIs can support account-level content engagement; WhatsApp can support the
merchant's own consented aggregate inquiry/conversion signals, not a claim about market-wide
product demand. Search Console-like sources describe the merchant's own search impressions, while
broad keyword demand requires a separately licensed source. These meanings must not be blended.
The system must record source, scope, observation window, ingestion time, and freshness; handle
revoked credentials, permission loss, rate limits, partial pages, duplicate events, delayed data,
deleted posts, platform outages, locale mismatch, and unavailable history; and show incomplete or
stale coverage instead of filling gaps with estimates. It must not scrape private content, infer
individual customer attributes, merge tenant identities, or treat message contents as market
signals. Platform signals and store conversion data remain separately attributable.

Cross-source aggregation is opt-in per normalized metric definition. Only complete additive
`count` observations with identical tenant, metric definition, unit, window, timezone, and explicitly
disjoint scope partitions may be summed. Overlapping account/content scopes, ratios, averages,
currency values, and incompatible attribution windows are not additive. Ratios must be recomputed
from compatible numerator and denominator observations; weighted averages require their original
weights. A provider must document the attribution window and prove partition disjointness before it
uses the shared exact-decimal aggregation helper. Until an adapter establishes that contract, its
observations remain source-specific and cannot be blended.

The OKF/RAG path may retrieve approved trend evidence and product facts. A configured model may
rank or explain candidate next actions only from that evidence and must return source references,
time window, uncertainty, and a safe abstention when evidence is insufficient. Model choice is
provider-neutral and may use an operator-hosted model so recommendations do not depend on a
metered API; when inference is unavailable, deterministic signals remain available. Ranking is
advisory: it cannot change price, publish content, contact customers, alter inventory, or make a
purchase commitment. Existing owner-authorized chat-price offers remain the only authority for
discounts, including a specific lower price configured by the owner; the model cannot invent,
extend, stack, or negotiate a discount. Logo-derived colors, category-to-archetype mapping, and
storefront styling are recommendations until the owner previews and applies them. Manual theme
tokens and logo settings take precedence, invalid or inaccessible logo assets fall back to the
current theme, and applying an archetype must not alter routes, content, product facts, price, or
accessibility behavior.

### Consequences

**Good.**
- The readiness record separates implemented source from unverified acceptance and names the remaining work without unsupported completion percentages or time estimates.
- Prevents architectural drift and keeps engineering focused on the exact delta.

**Bad.**
- Requires coordinating Meta Cloud API app reviews for the central Vendryx WhatsApp notification profile.

**Cost of the exception.** None.

---

<a id="adr-036"></a>

## ADR-036 — Space provides dedicated build execution and shared build caches

**Status:** Accepted · **Date:** 2026-10-22 · **Related:** [ADR-030](#adr-030), Space ADR-016 in the separate Space project

### Context

Store and Space both compile Rust workspaces and build JavaScript frontends. Local build output
consumes disk and repeated clean CI jobs duplicate expensive compilation. Space is already the
company-level operations project, but its production control-plane service must remain separate
from build workloads.

### Decision

1. Space will provide a dedicated build host for Store and Space build jobs. This is build
   infrastructure managed alongside Space; it is not the Space application server and does not
   become a product runtime.
2. Build jobs may use persistent, remote compiler and package caches. Cache contents are
   disposable performance data; they are never an authority for source, dependencies, test
   results, or release artifacts. Jobs must validate outputs through the repository's normal
   gates.
3. Cache namespaces are separated by repository, toolchain, target, and relevant build profile.
   Concurrent jobs must not mutate the same Cargo target directory. Use a cache service designed
   for sharing (such as `sccache`) rather than mounting one writable `target/` directory across
   machines.
4. Secrets and deployment credentials are not placed in caches. Untrusted pull-request jobs do
   not receive privileged runner access or write access to caches used by trusted release jobs.
5. Existing CI remains the required source of gate results. Migration to the dedicated host is
   staged; until the host and its access controls are provisioned and verified, current CI
   runners remain the fallback.

### Implementation plan

This ADR is implemented in phases. A phase advances only when its acceptance evidence is recorded
in `docs/WORK_LOG.md`; the hosted `CI` status remains the merge and release authority until Phase 4.

1. **Manual Store build on Space.** Run Store's `scripts/build-on-space.ps1`; it delegates to
   Space's `scripts/build-on-space.ps1` with `-Project store`. Package a source snapshot without
   credentials or generated output, mount source read-only, use the pinned Rust image, and run
   `cargo check --workspace --locked` with the Store target cache. Record cold and warm durations.
   This is an additional build check, not a replacement for Store's CI gates.
   **2026-10-22 evidence:** the cold Store check passed in 5m47s; warm Cargo checks passed in
   2m21s and 2m50s (5m00s and 5m14s end-to-end including source packaging and transfer). All used
   the pinned Rust 1.88 image and Store target volume. This validates compile-cache reuse only; no
   Store CI job has been routed to the host. These historical runs used the current shared host;
   they do not prove the required runner/daemon trust isolation. The builder now runs a fail-closed
   isolation preflight before packaging or transferring source, and the current host is expected to
   fail that preflight until a separate build account/daemon or dedicated host is provisioned. The
   preflight fixture covers a shared runner UID, rootful Docker, a data root outside the build
   account's home, and unavailable Docker. A real Store invocation was rejected because the active
   Actions listener and SSH account both use UID 1000; no source archive was made or transferred.
   **2026-10-23 cache isolation:** provisioned and label-verified Store-only Cargo registry/Git
   volumes and a target volume named for Store, Rust 1.88, `x86_64-unknown-linux-gnu`, and `dev`.
   The cold isolated-cache check passed in 4m36s (6m37s end-to-end). The warm check passed in
   5m33s (9m41s end-to-end) with no dependency downloads, but did not show a wall-time improvement:
   the shared host was under heavy concurrent load. Treat cache correctness as verified and cache
   performance as inconclusive until measured on a quiet host. Neither run includes clippy, tests,
   Postgres/Redis integration, frontend, or release gates.
   **Cold-cache recovery:** Space's `ensure-build-cache-volumes.sh` creates only absent Store/Space
   Cargo volumes with the exact ownership labels before a build. The mocked-Docker fixture passed
   cold creation and verified fail-closed behavior for foreign labels, Docker inspection/creation
   failures, unexpected volume names, and invalid project input. Corruption detection remains
   manual and cache-volume deletion requires an operator command. Ownership validation now
   requires the expected cache purpose, builder identity, and project labels; the fixture rejects
   same-named volumes carrying a foreign manager or project label.
   **2026-10-23 ownership check:** the extended mocked-Docker fixture passed cold creation and
   rejection of mismatched purpose, manager, and project labels, plus daemon and argument failure
   cases. The fixture does not touch the remote Docker daemon.
   **Scoped cache recovery:** `reset-build-cache-volume.sh` now provides an operator recovery path
   for suspected cache corruption. It defaults to dry-run, selects only one exact project cache,
   validates purpose/manager/project labels, and requires the shared build-host lock for deletion.
   Its fixture passed dry-run, missing-volume, lock contention, cross-project/manager, exact-volume,
   and invalid-input cases. This proves the recovery command's scope against a mocked Docker CLI;
   remote deletion and cold rebuild have not been exercised.
   **Web static parity slice:** the hosted workflow previously selected Node 20, which reached
   end-of-life on 2026-03-24. The three hosted Node jobs now pin Node 24.22.1, and Space's
   `-Mode web-static` uses the matching digest-pinned official Node image. The Store npm cache volume
   name and ownership label include a 16-character hash of both frontend lockfiles, so either
   lockfile change selects a cold cache. Admin and Storefront dependency installs and typechecks,
   per-theme typechecks, and frontend boundary checks passed locally under Node 24.22.1. The Space
   host run is intentionally blocked by the isolation preflight; this does not yet count as remote
   web-static parity. [Node's release table](https://nodejs.org/en/about/previous-releases) lists
   Node 20 as EOL; [Node 24.22.1](https://nodejs.org/en/blog/release/v24.22.1/) is the current LTS
   security release. **2026-10-23 local verification:** strict installs passed under Node 24.22.1 /
   npm 11.19.1 for both apps; the Admin typecheck and production build passed. The Admin unit suite
   ran 168 tests with 167 passing and one existing route-smoke assertion failing because the home
   page does not render the test's expected “Overview” heading. Storefront's root TypeScript check
   passed, but its production build cannot start because `apps/storefront/scripts/frontend.js` is
   absent, and the configured `tsconfig.*.json` theme checks have no matching files in the checkout.
   Space now has a separate `-Mode web-test` path matching the hosted Admin test/build and
   Storefront build commands, using the same Node image and lockfile-keyed npm cache as `web-static`.
   The Store wrapper accepts this mode. Its live invocation stopped at the separate-UID isolation
   preflight before source packaging or cache setup, so the mode has not run remotely. The known
   application failures remain blockers to full web parity and are not counted as passed ADR gates.
2. **Store runner isolation.** Before scheduling CI work, give the runner a repository-scoped
   identity or a runner group limited to Store and Space. The current organization-wide runner
   group is not an acceptable trust boundary. Do not allow pull-request or fork code onto a
   persistent runner. Builds receive no deployment credentials; use only ephemeral test-service
   credentials generated for that job. **Observed 2026-10-23:** the sole `syed-build` runner is in
   the organization `Default` group, which allows all private repositories, and was busy with
   another repository's job during this work. An organization runner group named `Store and Space
   Build Host` has been created with selected-repository access limited to `Syed-Systems/space` and
   `Syed-Systems/vendryx-commerce`; the existing runner has not yet been moved into it. The runner
   remains in `Default`, so isolation is not complete and Store CI must not be routed to it. The
   current runner also carries `android` and `godot` labels; identify its existing consumers and
   provide a safe transition or a separate isolated runner before moving it.
3. **Trusted CI parity.** The current `-Mode rust-ci` is a partial Rust diagnostic. Full parity
   means all seven workflow jobs—`rust-static`, `web-static`, `rust-test`, `web-test`, `e2e`,
   `container`, and `compliance`—plus the aggregate `ci` rule that fails on any failed or skipped
   job. Run each corresponding host stage manually and compare its commands, toolchain versions,
   service images, permissions, timeouts, and failure behavior against `.github/workflows/ci.yml`.
   Only then add a trusted-main/manual host job. Keep the hosted jobs required and authoritative
   while measuring cold/warm results and runner recovery. A queued, busy, offline, or failed host
   job must never turn a skipped stage into a green result or hold the hosted merge gate hostage.
   The host builder serializes all projects behind one host-capacity lock and keeps the per-cache
   target lock; busy waits, daemon calls, image pulls, and service health waits have finite limits.
4. **Selective migration.** Move only a Rust build stage after parity, cancellation cleanup,
   restart recovery, concurrency, cache-loss recovery, and offline behavior have been exercised.
   Retain a bounded hosted fallback and promote the Space result to required only after the
   fallback is proven on an unavailable host.
5. **Operations and retirement.** Add disk/inode and memory monitoring, runner heartbeat and
   credential rotation, and a tested cold-cache recovery procedure. Per-volume cache growth
   ceilings, scoped source-snapshot cleanup, and orphan-resource cleanup are implemented;
   cache-volume deletion and periodic host-wide cleanup remain disabled until their retention and
   recovery policies are tested. The manual builder's resource recovery runs only after host
   isolation passes, holds the
   shared build lock, and removes stopped containers and unattached networks older than 24 hours
   only when their purpose, manager, and project labels match. It never touches running containers,
   attached networks, images, or volumes. Its mocked-Docker fixture covers dry-run, age, label and
   project mismatch, lock contention, daemon failure, and the no-volume-pruning guarantee. Rollback
   restores the prior hosted `runs-on` configuration; caches and build workspaces are disposable
   and are not rollback data.
   **Workspace retention implementation:** the manual builder now assigns every source snapshot a
   unique UTC/run identifier, so concurrent builds of identical source cannot extract into the
   same directory. After host isolation passes and before capacity is checked, it invokes Space's
   scoped cleanup helpers for that repository: retain at most the newest 20 snapshots and remove
   snapshots older than 7 days, while skipping active workspaces; remove only stale, stopped build
   containers and unattached networks with exact ownership labels. The helpers default to dry-run;
   the builder explicitly requests apply. Workspace cleanup only examines recognized names beneath
   `~/.cache/syed-build/workspaces/<project>` and never runs Docker prune or touches cache volumes.
   This lets safe retention reclaim space before a capacity rejection; a periodic monitor/sweeper
   and cache-size trend reporting remain outstanding.
   **2026-10-23 verification:** `scripts/test-cleanup-build-workspaces.sh` passed against an
   explicitly isolated temporary `HOME`, covering dry-run, age and count retention, active-lock
   preservation, project scoping, symlink safety, and invalid project rejection. PowerShell and
   Bash syntax checks passed. This validates the cleanup helper; the full remote `rust-ci` mode
   and real-host restart recovery remain unverified.
   **Orphan-resource recovery:** Space's `cleanup-build-resources.sh` is now called after the
   isolation preflight and before source packaging. It applies the 24-hour retention threshold
   while holding the host lock and validates purpose, manager, and project labels before removal.
   It only removes stopped job containers and networks without attached containers; images,
   volumes, and live or attached resources are excluded. The mocked-Docker fixture passed dry-run,
   ownership mismatch, age, lock contention, daemon failure, and no-volume-pruning cases. This is
   scoped recovery evidence only; no cleanup was run against the remote Docker daemon.
   **Per-volume cache ceilings:** Space's `check-build-cache-sizes.sh` runs before Rust or frontend
   build commands inside the isolated container. Initial hard limits are 8 GiB for Cargo registry,
   2 GiB for Cargo Git sources, 50 GiB for the Rust target, and 8 GiB for npm's package cache.
   It measures allocated bytes with a 120-second bound, fails closed on missing/unreadable sizes or
   an exceeded limit, and never deletes cache data. The mocked-size fixture passed normal and exact-
   limit cases, over-limit cases, invalid input, and measurement failure. These are operator policy
   ceilings to be reviewed against measured cache trends; no legacy rootful volume was measured or
   modified, and the current host still fails isolation before the size check can run. The six
   pre-existing Store and Space volumes on the current rootful daemon carry the legacy
   `prepare-build-host.sh` manager label and no project-ownership label, so they do not satisfy
   the new builder's ownership contract. Do not relabel or delete them. A separately provisioned
   rootless daemon has its own volume namespace; it will create correctly labeled cold caches
   there rather than reuse these rootful volumes. Space's
   `check-build-host-capacity.sh` checks disk free space, inodes, and available memory before
   source packaging and again before starting the build container: Rust jobs require 70 GiB and
   100,000 free inodes, web-static requires 20 GiB, workspace builds require 4.25 GiB available
   memory, and Rust CI requires 5.5 GiB before starting its Postgres and Redis services, then
   checks for 4.25 GiB before the build. Package-scoped Rust checks use one CPU/Cargo job and a
   3 GiB container cap with a 3.5 GiB host preflight. They do not start Postgres/Redis unless
   `-CargoTestServices` is explicitly selected. A cold package-test attempt at 2 GiB was killed
   while compiling Diesel (rustc reached about 2.4 GiB RSS); the 3 GiB retry compiled Diesel but
   was blocked at its second capacity check when another Android build reduced host availability
   below 3.5 GiB. This is not yet a passing package test. The separate `syed-build` account and
   rootless Docker service are provisioned on Space; user lingering keeps only the small systemd
   user manager alive, while the builder stops Docker after each run. No GitHub runner registration
   or workflow routing changed.
   Persistent cache sizes are checked again after the gate so a cache that grows beyond its limit
   cannot produce a passing result. Thresholds are policy safeguards; monitoring and trend alerts
   are not installed.

### Edge cases and required behavior

| Condition | Required behavior |
|---|---|
| Fork or untrusted pull request | Run hosted CI only; do not expose the Space runner or its writable caches. |
| Runner offline, busy, restarting, or disconnected | Hosted CI remains runnable and authoritative; a host result is never inferred from a timeout or skipped job. |
| Cache missing, corrupt, stale, or owned by another service | Create only the missing exact namespaced volume and rebuild cold; fail closed on ownership mismatch. Never prune, relabel, or delete unrelated data. Corruption still requires a scoped operator recovery procedure. |
| Concurrent or canceled Store builds | Use unique source workspaces and containers; serialize writes to a persistent Cargo target; hold a per-workspace lock so retention skips active snapshots; remove the job container on exit, signal, or timeout. |
| Toolchain, target, profile, features, or relevant flags change | Use a distinct cache namespace or cold cache. Lockfile checks and all repository gates still run. |
| Host disk/inodes, memory, or CPU are pressured | Check disk, inode, and memory headroom before image pulls and container start; enforce per-container resource limits; fail with capacity diagnostics without deleting unrelated Docker data; keep hosted CI available. |
| Registry outage, image pull failure, or throttling | Retry only within bounded limits, then fail the host job clearly; hosted CI remains the fallback. |
| Test database/service startup or migration fails | Treat it as a failed gate; use per-job isolated services, health checks, and cleanup. Never reuse production data or credentials. |
| Cache poisoning or partial artifacts | Treat cache data as untrusted acceleration only; source, lockfile checksums, tests, and artifacts must be tied to the checked-out commit. |
| Host restart or job timeout | Re-run from source with a cold or warm cache. On the next isolated build, source snapshots older than 7 days or outside the newest 20 are removed while active snapshots are skipped; stale stopped build containers and unused build networks older than 24 hours are removed only by exact ownership labels under the host lock. Running containers, attached networks, volumes, images, and unrelated projects remain untouched. |
| Runner credential leak or unexpected repository access | Revoke the runner registration, rotate credentials, inspect job access, and return to hosted-only builds. |

### Consequences

**Good.** Rust and frontend builds can reuse expensive compilation and package downloads across
the two company repositories, while build execution stays separate from customer-facing services.

**Bad.** The host adds maintenance, capacity, availability, and access-control work. A cache outage
must not block correctness checks or release gates; cold-cache builds remain supported.

### Reversal condition

If measured build-time savings do not justify operating the host, or if its isolation cannot meet
the repository trust boundaries, return builds to managed CI runners and retain only safe
provider-native caches.

**Cost of the exception.** Dedicated build-host and cache operations are accepted as shared
engineering infrastructure for Store and Space.

---

<a id="adr-037"></a>

## ADR-037 — Cloudflare is the edge layer only; vendor calls are confined to one provider crate and Space

**Status:** Accepted · **Date:** 2026-11-13 · **Amends:** [ADR-024](#adr-024) · **Relates to:** [ADR-012](#adr-012), [ADR-022](#adr-022), [ADR-026](#adr-026), [ADR-030](#adr-030)

### Context

[ADR-024](#adr-024) fixed the shape — compute beside Postgres, delivery by a CDN that only needs
correct HTTP caching — but named no vendor and left four things unbuilt. A review on 2026-11-13
found:

1. **No reverse proxy exists.** `docker-compose.yml` runs Postgres, Redis and PgBouncer only, yet
   `tenant_layer.rs` states tenant identity is safe only behind a proxy that controls `Host`.
2. **Nothing purges the edge.** The origin emits `Surrogate-Key` and `Cache-Tag`, but no code
   calls a purge, so an edited product stays stale for up to `s-maxage=3600`.
3. **The emitted tags are shared across tenants** (`products`, `catalog`, `collections`). A tag
   purge is zone-wide on every CDN, so one merchant's edit would flush every store.
4. **Custom-domain certificates have no issuer.** The admin domain screen reports an SSL status
   and tells merchants to CNAME to `shops.vendryx.com`, but nothing issues the certificate.
5. **Geo routing reads a vendor header.** Storefront `middleware.ts` reads `x-vercel-ip-country`.
6. **`Cache-Tag` is emitted space-separated**, the `Surrogate-Key` convention. Cloudflare reads
   `Cache-Tag` as a comma-separated list, so today's value is not a usable tag list there.
7. **The admin dashboard is specified to share an origin with the storefront**
   ([ARCHITECTURE.md](ARCHITECTURE.md) section 4). The dashboard uses cookie session auth, so a
   script injected into any storefront page could call the admin API as the signed-in merchant;
   CSRF protection does not stop a same-origin script. Platforms that host merchant-authored
   content keep the admin on an origin that serves none.
8. **The development proxy lets the client pick the tenant.** `apps/admin/vite.config.cts`
   rewrites `Host` from a `stheno_tenant_host` cookie or an `x-stheno-tenant-host` header. That
   is correct for local tenant switching and would be a tenant-impersonation hole in production.

Items 2 and 4 cannot be solved by headers alone: a purge and a certificate order are API calls.
[ADR-024](#adr-024) decision 2 ("No CDN-specific code, ever") forbids them as written.

### Options considered

| Option | For | Against |
|---|---|---|
| **Cloudflare as edge only: DNS, CDN, WAF, Tunnel, R2, custom hostnames, static marketing** (chosen) | Matches ADR-024. Origin stays portable. Zero-egress media. Certificates for merchant domains without running an ACME fleet | Two vendor API calls must exist somewhere |
| Run the engine or storefront on Workers | One vendor, no origin host | Rejected by ADR-024: every uncached request must reach the single Postgres, and the edge Next.js adapter can serve another merchant's theme ([TRD](TRD.md)) |
| No CDN; origin serves everything | No vendor at all | Egress and latency scale with page views; no certificate path for merchant domains |

### Decision

1. **Cloudflare is used only as the edge.** DNS, CDN cache, WAF, Tunnel, R2 object storage,
   custom hostnames for merchant domains, and static hosting for `apps/marketing`. The Rust
   engine, Postgres, Redis, PgBouncer, the storefront SSR process and the admin dashboard run on
   the origin. No Workers, Pages Functions, D1, KV or Durable Objects carry application logic.
2. **ADR-024 decision 2 is amended, not dropped.** Request handling still contains no
   CDN-specific code. Exactly two vendor API calls are permitted, each in one place:
   - **Cache purge** lives in a single provider crate, `crates/providers/stheno-cdn-cloudflare`,
     invoked from a subscriber. Unconfigured, it is a no-op and the engine behaves as before.
   - **Custom-hostname registration and certificate status** live in Space, which
     [ADR-030](#adr-030) already makes the owner of domain and SSL orchestration.
   Replacing the vendor means replacing that crate and that Space adapter, and nothing else.
3. **Purge is by tenant-scoped tag; hostname purge is reserved for store-wide changes.**
   - Every cache tag carries the tenant id as a prefix. An unprefixed tag is never emitted, so a
     zone-wide tag purge cannot reach another tenant.
   - Tags are entity-level where the entity is known (one product, one collection, one category)
     plus one listing tag per tenant for pages that enumerate the catalog. Storefront HTML
     responses carry the same tags as the API responses they are built from; a tag purge that
     clears JSON but leaves the rendered page is not a purge.
   - A catalog edit purges the edited entity's tags and the tenant's listing tag. Other product
     pages of the same store stay cached.
   - Changes that alter every page of a store — branding, appearance, domain changes,
     suspension, closure — purge by that tenant's hostnames instead. A suspended or closed store
     must not keep serving from the edge.
   - `Cache-Tag` is comma-separated and `Surrogate-Key` is space-separated, each in its own
     vendor's format.
   - Purge requests are durable: they are written to the existing leased outbox in the same
     transaction as the change, coalesced per tenant, batched, and retried until the vendor
     accepts them. A purge that cannot be delivered is visible to operators, never silently
     dropped. Purge APIs are rate limited per zone, which is why coalescing is a requirement and
     not an optimisation.
4. **A purge runs only after the origin's own caches are invalidated** (the in-memory L1 cache
   and the Next.js route cache). Otherwise the edge refills itself with the stale response and
   holds it for another hour.
5. **The origin is reachable only through a Cloudflare Tunnel.** No public ingress port. A
   reverse-proxy container behind the tunnel does the path routing and is the proxy
   `tenant_layer.rs` assumes. It lands in Compose and in the chart ([ADR-026](#adr-026)). The
   production proxy never derives `Host` from a cookie, header or parameter, and strips
   `x-stheno-tenant-host` and `X-Forwarded-Host` from every inbound request. The cookie-driven
   `Host` rewrite stays in the development server only.
6. **The proxy owns vendor header translation, into a vendor-neutral name.** The proxy sets
   `x-country-code` from `CF-IPCountry`, overwrites any client-supplied value, and removes the
   header when the vendor did not supply one. The storefront reads `x-country-code`; the
   reference to `x-vercel-ip-country` is removed. No application code names a CDN vendor.
7. **Edge caching is opt-in per response.** The cache rule is "use the origin `Cache-Control`
   if present, bypass if absent", with an explicit bypass for admin, cart, checkout and account
   paths. No edge TTL override is ever configured, because an override caches responses the
   origin never marked public.
8. **Media is split by bucket.** Public product media is an R2 bucket on a media hostname.
   Payment proofs and other private documents are a second bucket with no public hostname,
   reached only through authenticated, time-limited URLs
   ([COMMERCIAL_IMPLEMENTATION_PLAN.md](COMMERCIAL_IMPLEMENTATION_PLAN.md) section 8).
9. **The admin dashboard is served from its own origin, never from a storefront hostname.**
   - Each tenant gets one admin hostname under a registrable domain that serves no
     merchant-authored content. The storefront zone and merchant custom domains do not qualify.
   - The admin hostname is registered in the same domain registry, so `Host` remains the only
     source of tenant identity ([ADR-022](#adr-022)) and no second identity path is added.
   - The proxy serves the dashboard and `/admin/*` only on admin hostnames. On every storefront
     hostname and custom domain `/admin/*` returns the same 404 an unknown route returns.
   - The admin session cookie is host-only, `Secure`, `HttpOnly`, `SameSite=Lax` or stricter,
     and uses the `__Host-` prefix so a sibling hostname cannot overwrite it.
   - Admin responses send a `Content-Security-Policy` that allows scripts only from the admin
     origin, and `frame-ancestors 'none'`.
   - The dashboard is still served by the origin and cached as static assets; it does not move
     to third-party static hosting.
   This supersedes the same-origin admin route formerly described in
   [ARCHITECTURE.md](ARCHITECTURE.md) section 4.
   **Owner confirmation (2026-11-13):** one admin hostname per store was chosen over a single
   shared admin hostname. Each store's dashboard is `<store>.dashboard.vendryx.com`.
   `dashboard.vendryx.com` itself is only a sign-in entry point that redirects a merchant to
   their own store's hostname; it holds no session and serves no store data.
   `admin.vendryx.com` is reserved for the platform operator. Storefronts stay on the merchant's
   own domain or the default store subdomain. Two-level hostnames need a certificate that
   covers `*.dashboard.vendryx.com`, which is an acceptance check at setup.
10. **The edge firewall has a written baseline, and machine callers are exempt from challenges.**
    - Rate limits apply at the edge to sign-in, password reset, account creation, checkout
      submission and payment-proof submission, keyed by client address and hostname.
    - Payment-provider callbacks, courier callbacks and Meta webhooks are never given a browser
      challenge or bot check: a challenged callback is a lost payment confirmation. They are
      protected by the signature verification the engine already performs and by a rate limit.
    - The engine's own rate limits and signature checks stay. The edge is a second layer, not a
      replacement, so the origin remains correct behind any other CDN.

### Consequences

**Good.** The bill scales with cache misses. An edit clears only the pages it changed. Merchant
domains get certificates without operating certificate infrastructure. Media egress is free. The
origin has no public attack surface. A compromised storefront page cannot act as the merchant.
Vendor choice stays reversible at the cost of one crate and one Space adapter.

**Bad.** Every tenant needs a second registered hostname, and merchants sign in at an address
that is not their store's. Tag correctness becomes a standing obligation: a page that renders an
entity without carrying its tag goes stale until `s-maxage` expires. The tenant id becomes
visible in the `Surrogate-Key` response header. Custom hostnames beyond the vendor's free
allowance are a per-merchant cost.

**Not decided here.** The origin host, its region, backups, failover and recovery targets. One
origin is a single point of failure, and that needs its own record once a host is chosen.

**Unverified at decision time.** Tag and hostname purge availability and rate limits on the
chosen plan tier, the `Cache-Tag` list format and size limits, `stale-while-revalidate`
behaviour, custom-hostname pricing, wildcard certificate depth for admin hostnames, wildcard
Tunnel routing preserving `Host`, and `stheno-file-s3` against R2 have not been tested. Each is
an acceptance check in the spec, not an assumption.

### Reversal condition

If tag purge is unavailable or too tightly rate limited on an affordable tier, fall back to
hostname purge for every change inside the same provider crate; the tenant-scoped tags stay,
because they cost nothing and keep the path back open. If one admin hostname per tenant proves
unworkable, a single shared admin hostname is the alternative, and it requires a new record:
tenant identity for `/admin/*` would then come from the authenticated session, which is a second
identity path [ADR-022](#adr-022) does not currently allow. If Cloudflare itself is dropped, any
CDN that honours `Cache-Control` replaces it with a DNS change plus the two adapters.

**Cost of the exception.** Two vendor-specific API integrations are accepted, confined as stated
in decision 2.

---

<a id="adr-038"></a>

## ADR-038 — The engine renders the storefront from tokenized template themes; no Node.js storefront

**Status:** Accepted · **Date:** 2026-11-27 · **Supersedes:** [ADR-031](#adr-031) (runtime and chassis) · **Amends:** [ADR-021](#adr-021), [ADR-022](#adr-022) · **Relates to:** [ADR-006](#adr-006), [ADR-012](#adr-012), [ADR-024](#adr-024), [ADR-037](#adr-037)

### Context

[ADR-031](#adr-031) put every merchant on one Next.js runtime with `frontend-default` as the
only DOM, varied by design tokens and data attributes. Three things have changed since:

1. **New designs arrive from AI design tools as whole storefronts.** Commit `12203039`
   (2026-11-27) added three storefronts built in Google AI Studio: Maison Véla
   (`maison-cuir`), KOVA (`hydro-vessel`) and Homedine (`homedine-eco`). Each has its own
   layout, typography and interaction design. They do not fit `frontend-default`'s DOM, and
   ADR-031 gives them no legal home.
2. **That push replaced the engine-connected storefront instead of extending it.** Its tenants,
   domains, products, prices and reviews are literals in `src/core/tenant-resolver.ts` and
   `themes/homedine-eco/catalog.ts`; its checkout computes a receipt in the browser and creates
   no order. Homedine alone carries 233 colour literals and no tokens. A design channel that
   can produce this needs a contract that makes it impossible, not a review that catches it.
3. **The owner set the operating constraints on 2026-11-27:** one server (the Rust engine), no
   Node.js storefront process, no nginx for the storefront, no Cloudflare Workers; every visual
   and copy choice tokenized and managed from the admin center; data only from the engine;
   heavy edge caching to save database compute; and the AI tools only design and tokenize.

ADR-031's token principle survives all three. Its runtime (Next.js) and its single chassis
(`frontend-default`) do not.

### Options considered

| Option | For | Against |
|---|---|---|
| Keep ADR-031: Next.js runtime, `frontend-default` chassis | Already built. React designs need no conversion | A second server and language to run and patch. AI Studio designs cannot be expressed as tokens on one DOM, so every design becomes a fork or a refusal |
| Static React app built once, data fetched by the browser from the engine | One server. Designs stay React | Pages arrive empty, so search ranking and link previews (WhatsApp, Facebook) suffer, which a store cannot afford. Moves server code into browser calls |
| Engine renders Liquid templates (Shopify's model) | The format Shopify has run on since 2006. Templates cannot run code | The Rust `liquid` crate's last release is 0.26.11 (February 2025), pre-1.0, and it lacks the `{% liquid %}` and `{% echo %}` tags modern themes use |
| **Engine renders `minijinja` templates; each theme is a folder of templates plus `theme.json`** (chosen) | One server. Full HTML for search and previews. Sandboxed templates. `minijinja` 3.1.0 released 2026-11-07, actively maintained by the author of Jinja. Same model as Liquid | The six existing themes are rewritten from React into templates. A rendering layer is built in the engine |

### Decision

1. **The Rust engine renders every storefront page.** A new crate, `stheno-storefront`, maps
   storefront routes to loaders and `minijinja` templates and returns HTML. No Node.js or
   Next.js storefront process exists once the cut-over in the spec completes. No storefront
   code runs on Cloudflare Workers or Pages Functions ([ADR-037](#adr-037) decision 1).
2. **A theme is one folder, `themes/<theme-id>/`:** templates, one stylesheet, interaction
   scripts and theme art, plus `theme.json`. Adding a theme touches nothing outside its folder.
   Theme folders load from `STHENO_THEMES_DIR` at runtime, so shipping a theme is a file
   release, not an engine build.
3. **Everything visual and every piece of copy is a token declared in `theme.json`.** Token
   types are a closed set: `color`, `range`, `select`, `font`, `text`, `richtext`, `image`,
   `url`, `boolean`. Colour, font, radius, shadow and spacing literals outside `theme.json` are
   a gate failure. Fonts are keys into the curated self-hosted set, never URLs
   ([ADR-021](#adr-021) decision 2).
4. **A theme contains no data.** Products, prices, stock, collections, reviews, policies and
   store identity reach templates only as read-only objects built by engine loaders. A figure
   that does not come from a loader does not render (AGENTS.md, "no unsourced figures").
   Money is `stheno-types::Money` throughout and is formatted by the engine
   ([ADR-006](#adr-006)).
5. **Templates are sandboxed.** They receive only the documented objects and filters, cannot
   fetch, query or execute code, and render with autoescaping on. Interaction scripts call only
   same-origin `/store/*` endpoints and never compute money.
6. **Commerce boundaries stay core-owned.** Checkout, payment, sign-in and account forms and the
   Free-plan "Powered by Vendryx" attribution are core partials a theme includes and cannot
   override. This is what ADR-031 protected by forbidding DOM forks, kept by a narrower rule.
7. **Merchants change token values in the admin center.** The admin lists installed themes and
   renders the Appearance form from the active theme's `theme.json`. The engine validates every
   value against that schema before saving; nothing partial is stored. A token added to
   `theme.json` appears in the admin with no admin code change.
8. **Caching follows [ADR-037](#adr-037) unchanged.** Public pages carry tenant-prefixed tags and
   `public, s-maxage, stale-while-revalidate`. Cart, checkout, account and order pages are never
   cached. Personalised fragments (cart count, signed-in state) load through interaction scripts
   so public pages stay shareable from the edge. Appearance and branding saves purge the
   tenant's hostnames.
9. **The storefront and the Store API share an origin because one process serves both.** This
   satisfies [ADR-022](#adr-022) without the `/api/*` hairpin, and its middleware region-cache
   landmine disappears with the middleware.
10. **One automated gate, `scripts/check-theme-pack.sh`, admits a theme.** It fails on any
    literal from decision 3, any data from decision 4, any forbidden capability from decision 5,
    an override of a decision 6 partial, a file outside the theme folder, an invalid
    `theme.json`, or default tokens that fail WCAG 2.2 AA contrast.

### Consequences

**Good.** One process to run, patch and debug. Search engines and link previews get finished
HTML. A new design from an AI tool lands as one folder and cannot reach data, prices or
checkout. Merchants restyle their store from the admin without a deploy, and the edge serves
most traffic without touching Postgres.

**Bad.** The six existing themes (Maison Véla, KOVA, Homedine, Mahal, Lustre, Matrix) are
rewritten from React to templates, keeping their look. The engine gains a rendering layer and a
schema validator. Rich interactions (KOVA's WebGL hero, drawers, galleries) are written as
small scripts, not components. Until the cut-over, the Next.js storefront keeps serving, so two
storefront implementations coexist for one migration.

**Not decided here.** Whether [ADR-037](#adr-037) decision 5's reverse-proxy duties (stripping
tenant-override headers, translating `CF-IPCountry`, the admin and storefront host split) move
into the engine now that the engine serves the storefront itself. That needs its own record.

**Unverified at decision time.** Tag-purge availability on the owner's Cloudflare plan (already
open under ADR-037), Lighthouse targets on the converted themes, and `minijinja` template load
time with all themes installed. Each is an acceptance check in the spec, not an assumption.

### Implementation

Specified task by task in [STOREFRONT_THEME_ENGINE_PLAN.md](STOREFRONT_THEME_ENGINE_PLAN.md): phase 0 unblocks main,
then renderer and contract, admin Appearance and storage, theme conversion, route and checkout
parity followed by deletion of `apps/storefront`, and the authoring kit
([THEME_AUTHORING.md](THEME_AUTHORING.md)).

### Reversal condition

If engine rendering cannot meet the spec's performance or parity checks, the fallback is
ADR-031's Next.js runtime with the same `theme.json` contract and gate. Tokens, the admin
Appearance screen and the no-data rule carry over unchanged, because none of them depend on the
renderer.

**Cost of the exception.** A one-time rewrite of six themes, paid to remove a server permanently.
