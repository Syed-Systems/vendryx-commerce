# Phase 2 Implementation Plan

**~46 engineer-weeks of work · ~26 calendar weeks with three parallel tracks · 10 phases (P11–P20)**

Phase 1 ([IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)) built the engine's domain logic and
stopped short of making it durable. This document covers everything between that and 1.0.

Same staffing assumption: **4–6 engineers** — 3 Rust backend, 1–2 frontend, 1 shared
QA/infrastructure. Phase 2 leans harder on the frontend pair than Phase 1 did, because P18 is a
dashboard fork rather than a backend grind.

---

**Task-level breakdown:** [PHASE2_TASKS.md](PHASE2_TASKS.md) decomposes these phases into 212
tasks, each ending in a command that exits 0.

---

## 0. Where Phase 1 actually finished

Stated plainly, because the plan below only makes sense against an honest baseline. Measured, not
estimated:

| Claim | Reality at the end of Phase 1 |
|---|---|
| ADR-004: Diesel across 127 entities | **0 of 25** persisting modules use the repository trait. All 25 hold `Arc<RwLock<HashMap>>` |
| `CrudRepository<T>` abstraction | Exists in `stheno-db`, threads `Conn<'_>` through every method, and **nothing implements it but `InMemoryCrudStore`** |
| `FilterSpec` | Defined, and **ignored** by `InMemoryCrudStore::list` and `list_and_count` |
| Migrations | 7 modules have SQL; ~25 need it. One Diesel `schema.rs` exists |
| Admin API | 38 routes against ~278 specified |
| Admin dashboard | Component sources only. No entry point, router, bundler, or deps |
| Reporting | Does not exist. Analytics emits events; nothing produces a report |

The engine passes 320 tests and loses every product, order, and customer on restart. Phase 2 is
mostly about closing that one sentence.

### The seam is already there, and that is the good news

`CrudRepository<T>` was written with `Option<&mut Conn<'_>>` on every method. That is the exact
shape a Diesel-backed implementation needs, and it means Phase 2 does **not** design a persistence
abstraction — it implements the second impl of one that already exists and moves 25 services onto
it. That is a substantially cheaper shape of work than it would have been without the trait, and
it is why P11 is 5 weeks and not 10.

---

## 1. How the plan is structured

Unchanged from Phase 1. Each phase declares **Deliverables**, an **Exit gate** that is binary and
testable, **Parallelism**, and **Risk**.

Two rules carry over, plus one new one:

1. **The load suite runs at every gate.** A phase meeting its functional gate but missing its
   latency budget ([PRD §7.1](PRD.md)) is not complete.
2. **The parity suite runs at every gate.** Every endpoint added must pass its recorded Medusa
   fixtures before the phase closes.
3. **New: no phase may ship a module that passes only against `InMemoryCrudStore`.** From P12
   onward, a module's tests run against a real Postgres in CI. A module that is green in memory
   and untested on Postgres is not migrated, it is decorated.

---

## 2. Phase map

| # | Phase | Weeks | Depends on | Track |
|---|---|---:|---|---|
| **P11** | Persistence foundations | 5 | P10 | A |
| **P12** | Persistence: catalog, pricing, inventory, config | 5 | P11 | A |
| **P13** | Persistence: cart, order, payment, fulfilment | 6 | P12 | A |
| **P14** | Persistence: platform modules and link tables | 4 | P12 | A |
| **P15** | Quota counters and entitlement enforcement | 2 | P14 | A |
| **P16** | Reporting module | 3 | P13 | **B** (parallel from P13) |
| **P17** | Admin API completion | 8 | P12 | **C** (parallel from P12) |
| **P18** | Admin dashboard fork | 6 | P11 | **C** (fork starts early) |
| **P19** | Storefront completion and e2e | 3 | P13, P17 | B |
| **P20** | Hardening, load, and GA | 4 | all | A+B+C |
| | **Sequential total** | **46** | | |
| | **With 3 tracks** | **~26** | | |

```
Track A  P11──P12──P13──────────────P15───────┐
                 └───P14──────────────┘       │
Track B                  └──P16───P19─────────┤──P20
Track C          └──P17───────────────────────┤
         └──P18───────────────────────────────┘
```

**P18 starts at P11, not later.** Forking `packages/admin/dashboard`, pinning it, stripping the
Enterprise paths, and getting it to boot against the current 38 admin routes is independent of
persistence. Waiting until P17 finishes wastes the frontend pair for four months.

**P14 forks off P12, not P13.** The platform modules (file, notification, search, settings,
translation, audit) touch none of the transactional machinery. Blocking them behind the order
schema is false sequencing.

---

## P11 — Persistence foundations (5 weeks)

The phase that decides whether the other four persistence phases are grinding or firefighting.

### Deliverables

1. **`PgCrudRepository<T>`** — the Diesel-backed second implementation of the existing
   `CrudRepository<T>` trait. One generic implementation, not 25 hand-written ones.
2. **`FilterSpec` → SQL** — the translation layer, and the thing `InMemoryCrudStore` never did.
   Validated against the 20 hardest list endpoints, as P0-05 originally intended.
3. **Migration runner wired into boot** — `diesel_migrations` embedded, run on startup behind a
   flag, with a `--check-migrations` mode that exits non-zero on drift.
4. **Real connection lifecycle** — `create_pool` called from the actual boot path, `DATABASE_URL`
   read, pool size configurable, health check reporting pool saturation.
5. **Transaction manager** — `with_savepoint` proven under nesting, `SERIALIZABLE` retry helper
   with bounded backoff, and a test that two concurrent writers produce a serialisation failure
   that the helper retries correctly.
6. **The Postgres test harness** — `TestApp::spawn()` provisions an isolated schema per test,
   runs migrations into it, and drops it on teardown. Currently `TestApp` creates a schema name
   and never uses it.
7. **Three reference modules migrated end to end** — `region`, `currency`, `product`. Two trivial,
   one with the nastiest shape in the catalog (variants, options, option-value hashing,
   collections, categories with materialized paths).
8. **The migration playbook** — a written, followed-once procedure for moving a module from
   `HashMap` to `PgCrudRepository`, so P12–P14 are execution rather than invention.
9. **CI runs Postgres.** Service container, `DATABASE_URL` in the test job, schema-drift check
   promoted from advisory to blocking.

### Exit gate

- `cargo test --workspace` passes with `DATABASE_URL` set, against a real Postgres, in CI.
- `stheno-server` boots, runs migrations, serves `/store/products`, is **restarted**, and returns
  the same product ids. This is the single assertion that Phase 1 could not make.
- `FilterSpec` translates every operator in `filter.rs` to SQL, verified per operator.
- A `SERIALIZABLE` conflict between two concurrent writers is retried and both commit.
- `--check-migrations` exits non-zero against a deliberately drifted schema.
- `region`, `currency`, and `product` have zero `RwLock<HashMap>` remaining in their services.
- Latency budgets for catalog reads hold against Postgres, not memory. Expect them to move; the
  gate is that they are **measured and committed**, not that they are unchanged.

### Risk

**`FilterSpec` is the same gamble it was in P0, and it is now due.** It was specified, then never
exercised — `InMemoryCrudStore` ignores it. If it cannot express product-with-variant-price or
order-with-item-customer filters, decide **in P11** to fall back to per-module hand-written query
builders. Discovering it in P13 costs a month, exactly as PRD R3 warned.

**Second risk: the `Id<T>` prefix loss.** `Id::from_string` strips the `prod_` prefix and
serialises the bare ULID, and services compensate with `ends_with` suffix matching. Against a
`HashMap` that is merely ugly. Against an indexed Postgres column it is a full table scan on every
retrieve. Decide the id representation in P11-04 before 25 modules encode the workaround.

---

## P12 — Persistence: catalog, pricing, inventory, config (5 weeks)

Execution of the P11 playbook across the read-heavy half of the domain.

### Deliverables

1. **Migrated modules** — `product` (finished in P11), `pricing`, `inventory`, `stock-location`,
   `sales-channel`, `tax`, `store`, `region`, `currency`.
2. **Full migrations per module** — every entity in [DATA_MODEL §2](DATA_MODEL.md), with the
   indexes the list endpoints actually need, not just primary keys.
3. **The pricing hot path** — `sql_query` escape hatch per ADR-004, benchmarked against the
   generic repository, with the decision recorded either way.
4. **Variant option-value uniqueness enforced by a database constraint**, not a side `HashMap`.
   The current implementation keeps the hash in `variant_hashes` and reverse-scans it.
5. **Cross-module list filters** through `FilterSpec` on real joins.

### Exit gate

- Every listed module: no `RwLock<HashMap>` in `service.rs`, tests green against Postgres.
- `./scripts/check-schema-drift.sh` clean.
- A duplicate variant option-value combination is rejected by a unique index, proven by a test
  that inserts concurrently.
- Catalog list endpoints hold their latency budgets at 10k products with realistic filters.
- Parity suite green for every store and admin catalog, pricing, and inventory group.

### Risk

Pricing is where the generic repository is most likely to be too slow. Budget for the `sql_query`
fallback rather than discovering it under load in P20.

---

## P13 — Persistence: cart, order, payment, fulfilment (6 weeks)

The transactional heart, and the phase where correctness stops being negotiable.

### Deliverables

1. **Migrated modules** — `cart`, `promotion`, `payment`, `order`, `order-change`, `fulfillment`.
2. **Workflow durability** — `WorkflowStore` on Postgres. The SAGA engine is currently an
   in-memory `HashMap`, which means a restart mid-checkout loses the compensation stack. This is
   the highest-severity item in Phase 2.
3. **Transactional integrity across the checkout path** — cart completion, payment authorisation,
   order creation, and inventory reservation commit or roll back together.
4. **Idempotency keys persisted**, so a retried payment capture after a restart is still idempotent.
5. **Order versioning and `get_order_at_version`** backed by real rows.
6. **Money precision preserved through the database** — `NUMERIC`, never float, asserted.

### Exit gate

- Kill the process mid-checkout; on restart the SAGA resumes or compensates. `./scripts/kill-resume.sh`
  passes against Postgres with real restarts, not simulated ones.
- A duplicate payment capture with the same idempotency key across a restart charges once.
- Order totals round-trip through the database with zero precision loss, property-tested.
- Concurrent cart completion for the same cart produces exactly one order.
- Parity suite green for all store and admin cart, order, payment, and fulfilment groups.
- Latency budgets hold for cart and checkout endpoint classes.

### Risk

**Durable workflows are the reason ADR-003 exists, and they have never been durable.** If the
Postgres SAGA store cannot hit the checkout latency budget, the fallback is a write-behind journal
with bounded replay — decide in P13, not P20.

---

## P14 — Persistence: platform modules and link tables (4 weeks)

Parallel with P13. Touches none of the transactional machinery.

### Deliverables

1. **Migrated modules** — `customer`, `auth`, `user`, `api-key`, `rbac`, `file`, `notification`,
   `search`, `index`, `caching`, `locking`, `translation`, `settings`, `analytics`, `audit`.
2. **The 22 writable link tables** from [DATA_MODEL §3.1](DATA_MODEL.md), with the unique index on
   each pair. `ALL_LINKS` declares them; no table has ever existed.
3. **Read-only link traversal** — the 14 declarations in §3.2 wired into the query engine.
4. **Distributed locking on Postgres advisory locks**, replacing the in-memory lock.
5. **Search index persistence and rebuild** against real rows.
6. **Audit log append-only**, with the PII redaction the erasure cascade assumes.

### Exit gate

- All 15 modules green against Postgres, no `RwLock<HashMap>` remaining anywhere in
  `crates/modules/`.
- All 22 writable link tables exist, migrate, and enforce pair uniqueness.
- `cargo test -p stheno-links --test registry` passes with tables actually present.
- Two processes contend for the same advisory lock; exactly one proceeds.
- Erasure cascade verified against real rows, not maps.

### Risk

`locking` and `caching` currently pass because nothing is concurrent. Advisory locks behave
differently across pooled connections — the lock must be held on the same connection for its
lifetime, which the pool will fight.

---

## P15 — Quota counters and entitlement enforcement (2 weeks)

Small, well specified, and blocked on P14. [ENTITLEMENTS.md §6.2](ENTITLEMENTS.md) already
prescribes the mechanism.

### Deliverables

1. **`entitlement_usage` counter table** and the atomic conditional `UPDATE`.
2. **`CheckQuotaStep`** — a workflow step, never an `if` in a service. `check-no-domain-gates.sh`
   stays green.
3. **`GET /admin/entitlements` reporting real usage.** It currently hardcodes `used: 0`.
4. **Downgrade and grandfathering** — existing data readable and editable, creates blocked.
5. **Nightly reconciliation job** comparing counters to actual counts, with drift alerting.
6. **Over-quota errors in Medusa's shape**, so the forked dashboard renders them unmodified.

### Exit gate

- 100 parallel creates against a limit of 10 → exactly 10 succeed. Not 11.
- Creation is rejected at the limit, not at limit+1.
- A downgraded store with 412 products on a limit of 100 serves all 412 and rejects the 413th.
- Reconciliation detects a deliberately corrupted counter.
- `check-no-domain-gates.sh` still passes.

### Risk

Low. The pattern is the same atomic conditional update as campaign budgets, and that one works.

---

## P16 — Reporting module (3 weeks)

[ENTITLEMENTS.md §5c](ENTITLEMENTS.md) names this as **missing entirely** and as the Free-vs-Pro
differentiator. Nothing in the tier plan is sellable without it.

### Deliverables

1. **`stheno-reporting` module** — revenue over time, orders over time, top products, average
   order value.
2. **Window-gated queries** — `analytics_window_days` bounds the horizon in SQL, per
   [ENTITLEMENTS.md §4.2](ENTITLEMENTS.md). Zero branching in domain logic.
3. **`GET /admin/reports/*`** in Medusa's envelope shape.
4. **Materialised daily rollups** with incremental refresh, because scanning the order table per
   dashboard load will not hold the budget.
5. **CSV export of every report**, UTF-8 BOM, reusing the P10-03e path.

### Exit gate

- Four aggregates computed correctly against a seeded 100k-order dataset.
- A Free-tier deployment cannot retrieve data older than 30 days — enforced in the query, verified
  by test.
- A Pro deployment retrieves full history.
- Report endpoints hold the analytics latency budget at 100k orders.
- Exported CSV opens in Excel on Windows with correct encoding.

### Risk

Rollup invalidation on order edits, returns, and refunds. An order that changes after the rollup is
computed silently corrupts every later report. Decide the invalidation strategy before writing the
first aggregate.

---

## P17 — Admin API completion (8 weeks)

The largest phase by task count and the most mechanical. ~240 routes across the resource groups in
[API_SPEC §3.2](API_SPEC.md).

### Deliverables

1. **All 51 admin resource groups**, in Medusa's exact shapes.
2. **Batch endpoints** workflow-backed internally, Medusa's response shape externally.
3. **Order mutation surface** — edits, returns, claims, exchanges, and their preview endpoints.
4. **Draft orders**, distinct from the social-draft path in `stheno-flows-order`.
5. **Admin access surface** — users, invites, api-keys, RBAC.
6. **Conditional mounting** of every gateable group through `stheno-api-router`.

### Exit gate

- `./scripts/parity.sh --group all` green across all 51 groups.
- Route count reconciles against API_SPEC, verified by a test that fails when a spec'd route is
  unmounted.
- The tier matrix test still passes with the full surface mounted.
- Admin list endpoints hold their latency budgets at realistic data volumes.

### Risk

Volume, not difficulty. The failure mode is drift: 240 routes written fast against a parity corpus
recorded once. Re-record the corpus at the phase midpoint.

---

## P18 — Admin dashboard fork (6 weeks)

Track C, starting at P11. Frontend pair.

### Deliverables

1. **The fork** — `packages/admin/dashboard` from a **pinned, verified** Medusa tag, with the
   commit recorded in `apps/admin/UPSTREAM.md`. The previous claim of a fork cited a repository
   that does not exist and a fabricated commit; this task starts by establishing a real upstream.
2. **Enterprise paths excluded**, per ADR-008, with `check-no-enterprise-paths.sh` extended to the
   real tree.
3. **Clean-room RBAC and SSO screens** — genuinely rebuilt, not forked.
4. **Entitlements settings screen** and the over-quota grace banner.
5. **Operator screens** — onboarding, spreadsheet import, social queue, analytics, billing.
6. **A real e2e runner** — Playwright against a live backend. The previous `test-runner.js`
   printed hardcoded checkmarks and has been removed.

### Exit gate

- `npm --prefix apps/admin ci && npm run build` produces a bundle.
- The dashboard boots against `stheno-server` and lists products, orders, and customers.
- A Free-tier backend hides Pro screens with **no dashboard code change** — the ADR-002 claim,
  tested.
- WCAG 2.2 AA audit passes with a real axe run.
- Playwright suite green against a live backend.

### Risk

The fork's upstream must be established first and honestly. Everything else in this phase depends
on that one task being real.

---

## P19 — Storefront completion and e2e (3 weeks)

### Deliverables

1. **Full purchase journey** against Stripe test mode.
2. **Order Now** — address form, COD-only payment section unless Pro.
3. **Social checkout** — WhatsApp `wa.me?text=` pre-fill; clipboard-and-open for Messenger and
   Instagram, wired to `stheno-flows-order`.
4. **Source attribution** — `?ref=` → cart metadata → order metadata.
5. **Lighthouse ≥ 95 and CWV gates** on the PDP.

### Exit gate

- Playwright: browse → cart → checkout → order confirmation, green.
- A social draft created from the storefront converts in the admin queue and does **not**
  double-notify.
- `?ref=` survives to order metadata.
- Lighthouse ≥ 95, INP and CLS within budget, measured.

### Risk

Low. The storefront is a fork and the backend contract is settled by P17.

---

## P20 — Hardening, load, and GA (4 weeks)

Everything that currently cannot run because a prerequisite is missing.

### Deliverables

1. **Restore drill** — now executable, because there is a database. `restore-drill.sh` already
   fails loudly for exactly this reason.
2. **Full load suite** — all seven latency budgets, k6 installed in CI.
3. **Soak** — 500 rps for one hour, RSS ≤ 400 MB, against Postgres.
4. **Distroless image** — measured size and cold start, `check-image.sh` already does this.
5. **Helm chart** linted and templated in CI.
6. **Penetration test** — no high or critical findings outstanding.
7. **PCI SAQ A** and disclosure artefacts re-verified against the shipped surface.

### Exit gate

- Every script in `scripts/` exits 0 **having measured something**. No gate asserts an unobserved
  number.
- `gate-p11.sh` through `gate-p20.sh` all green.
- Restore drill restores a real encrypted dump and verifies row counts.
- Seven latency budgets met on reference hardware.
- Pen test report filed, findings closed.

### Risk

The load budgets were measured against in-memory storage. Expect them to be missed on first run
against Postgres, and expect P20 to surface index work that belongs in P12–P14. Budget for it.

---

## 3. Staffing

| Track | Who | Phases |
|---|---|---|
| A | 2 Rust backend | P11 → P12 → P13 → P15, plus P14 |
| B | 1 Rust backend | P16, then P19 backend support |
| C | 1–2 frontend | P18 from P11, P19 |
| C | 1 Rust backend | P17 |
| Shared | 1 QA/infra | Postgres CI, load harness, P20 |

P17 is one engineer for eight weeks by design. It is high-volume and low-coupling, which makes it
the one phase that parallelises badly across people — two engineers writing 120 routes each against
the same parity corpus will conflict more than they gain.

---

## 4. Sequencing rules

1. **No module leaves a persistence phase green-in-memory.** Postgres or not migrated.
2. **P13 does not start until the P11 playbook has been followed twice** in P12 without amendment.
   If the playbook still needs edits, the abstraction is wrong.
3. **P17 route groups follow their storage.** Catalog groups after P12, order groups after P13.
   Writing admin routes against `HashMap`s means writing them twice.
4. **P18's upstream fork task is the first thing done in Phase 2**, not the first thing in P18.
   It is a half-day of verification that unblocks a six-week phase, and it has already been got
   wrong once.
5. **Re-measure budgets at every persistence gate.** The Phase 1 numbers describe a different
   program.

---

## 5. What would make this take longer

- **`FilterSpec` cannot express the hard joins.** Adds ~4 weeks for hand-written builders. This is
  the single largest schedule risk and it lands in P11.
- **The `Id<T>` prefix decision is deferred.** Retrofitting id representation across 25 migrated
  modules costs more than deciding it once in P11.
- **Durable workflows miss the checkout budget.** Adds ~2 weeks for a write-behind journal.
- **The Medusa upstream for the dashboard cannot be pinned** to something that builds. P18 becomes
  a rebuild, not a fork, and ADR-002 needs revisiting — that is a quarter, not a phase.
- **Report rollup invalidation is got wrong.** Silent data corruption is the most expensive class
  of bug to find late.

---

## 6. Definition of done, per module

A module is migrated when all of these hold:

- No `RwLock<HashMap>` in its `service.rs`.
- Every entity in DATA_MODEL has a migration, a Diesel model, and an index for each filterable
  column.
- Its tests run against Postgres in CI and pass.
- Its list endpoints honour `FilterSpec`, verified per operator.
- Its rows survive a process restart, verified by a restart test.
- `check-schema-drift.sh` is clean for it.
- Its latency budget is re-measured and committed.
