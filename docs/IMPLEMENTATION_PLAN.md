# Implementation Plan

**~56 engineer-weeks of work · ~30 calendar weeks with three parallel tracks · 11 phases**

Estimates assume a team of **4–6 engineers**: 3 Rust backend, 1–2 frontend, 1 shared
QA/infrastructure. They assume competence with Rust and async, not expertise with Diesel or
commerce domains — R7 in the [PRD](PRD.md) risk register is priced into P0 and P1.

Anyone quoting a quarter for this is reading a different document.

---

**Task-level breakdown:** [TASKS.md](TASKS.md) decomposes these phases into 247 tasks, each ending
in a command that exits 0, plus the loop protocol and its stall rules.

---

## 0. How the plan is structured

Each phase declares:

- **Deliverables** — what exists at the end
- **Exit gate** — a binary, testable condition. Not "mostly working"
- **Parallelism** — what can run alongside it
- **Risk** — what specifically goes wrong here

**Two rules apply to every phase**, and they are what stop the plan from quietly slipping:

1. **The load suite runs at every gate.** A phase that meets its functional gate but misses its
   latency budget ([PRD §7.1](PRD.md)) is not complete. Performance regressions are cheapest to fix
   in the phase that caused them.
2. **The parity suite runs from P2 onward.** Every Store endpoint added must pass its recorded
   Medusa fixtures before the phase closes.

---

## 1. Phase map

| # | Phase | Weeks | Depends on | Track |
|---|---|---:|---|---|
| **P0** | Foundations and harnesses | 3 | — | A |
| **P1** | Platform: workflow, query, links, events | 5 | P0 | A |
| **P2** | Catalog and inventory | 5 | P1 | A |
| **P3** | Pricing, tax, region, currency | 4 | P2 | A |
| **P4** | Cart, promotion, payment | 6 | P3 | A |
| **P5** | Order, fulfillment, sales channel | 6 | P4 | A |
| **P6** | Order lifecycle: edits, returns, claims, exchanges, drafts | 6 | P5 | A |
| **P7** | Auth, user, customer, API keys, RBAC | 4 | P1 | **B** (parallel from P2) |
| **P8** | Platform services: file, notification, search, index, cache, lock, i18n, settings | 5 | P1 | **B** (parallel from P4) |
| **P9** | Admin API (Medusa dialect) + forked dashboard + product screens | 7 | P5, P7 | **C** (parallel from P6) |
| **P10** | Forked storefront, migration, hardening, GA | 5 | all | A+B+C |
| | **Sequential total** | **56** | | |
| | **With 3 tracks** | **~30** | | |

```
Track A  P0──P1──P2──P3──P4──P5──P6──────────────┐
Track B          └──P7──────P8──────────────────┤──P10
Track C                          └──P9──────────┘
```

Track B starts once P1 lands the platform primitives. Track C starts once P5 gives the admin real
orders to render. Starting either earlier produces work that gets rewritten.

---

## P0 — Foundations and harnesses (3 weeks)

The phase everyone wants to skip. Skipping it is how this project fails.

### Deliverables

1. **Workspace** — Cargo workspace, shared `[workspace.dependencies]`, `stheno-types`,
   `stheno-db`, `stheno-http`, `stheno-server` skeleton.
2. **`Money` type** — `rust_decimal`-backed, no `From<f64>`, currency-aware, largest-remainder
   allocation. Property tests from day one.
3. **`Id<T>`** — prefixed ULID, wire-identical to Medusa, typed by entity.
4. **Database layer** — `deadpool` pool, transaction manager with explicit connections,
   `diesel_migrations` embedded, `SERIALIZABLE` retry helper.
5. **The `FilterSpec` macro** — and its validation against the 20 hardest list endpoints.
6. **HTTP layer** — axum app, middleware stack, both error renderers, `garde` validation
   extractor, cursor and offset pagination.
7. **Observability** — `tracing`, OTLP export, RED metrics, structured logs with redaction.
8. **CI** — fmt, clippy `-D warnings`, test, `cargo deny`, `cargo audit`, schema-drift check,
   **module-isolation check**, compile-time budget, SAST, secret scanning.
9. **`Entitlements`** ([ENTITLEMENTS.md](ENTITLEMENTS.md)) — tier config loader, env overrides,
   fail-closed boot validation, and `check-no-domain-gates.sh`. Cheap now; retrofitting gates into
   300 routes is not.
10. **The security baseline** ([SECURITY §16](SECURITY.md)) — security headers, CORS allowlists,
   **CSRF middleware**, rate limiting with trusted-proxy resolution, log redaction, the
   `Owned<T, Scope>` extractor, the DTO/entity separation CI check, and filter limits. F1 is a
   launch blocker and the middleware is cheapest to write before there are 300 routes.
11. **The parity harness** — recording proxy, seeded Medusa v2 instance, fixture storage, replay
   and diff runner.
12. **The load harness** — k6 scripts and budget assertions, plus **measured Medusa baselines on
    identical hardware**.

### Exit gate

- `cargo build` produces a binary that serves `/health/ready` against a real Postgres.
- A trivial two-table module round-trips through the HTTP layer with filtering, pagination,
  validation, and both error formats.
- `Money` passes property tests: allocation sums exactly, at every ISO 4217 precision.
- The parity harness records and replays at least one real Medusa endpoint end to end.
- Baseline numbers for all seven endpoint classes in [PRD §7.1](PRD.md) are **measured and
  committed**, not estimated.
- CI rejects a deliberately introduced `modules/* → modules/*` dependency.
- CI rejects a handler whose request body type is an entity struct.
- A cookie-authenticated state-changing request without a CSRF token and matching `Origin` is
  rejected, verified by test.

### Risk

**The `FilterSpec` macro is the gamble** (PRD R3). If it cannot express product-with-variant-price
filters or order-with-item-customer filters cleanly, decide **in P0** to fall back to per-module
hand-written builders. Discovering this in P5 costs a month.

---

## P1 — Platform: workflow, query, links, events (5 weeks)

Everything downstream is built on this. It is also where the hardest domain modelling happens.

### Deliverables

0. **`audit_log`** — append-only table and writer, wired into the workflow engine so every
   money-moving or permission-changing step is recorded from the first day there is one.
1. **`stheno-workflow`** — the full SAGA orchestrator per [WORKFLOWS.md](WORKFLOWS.md): every step
   option, both state machines, Postgres checkpointing, `SKIP LOCKED` claiming, lock renewal,
   idempotency, the `workflow!` macro.
2. **`stheno-links`** — the `LinkDefinition` registry, all 36 definitions declared, link CRUD steps.
3. **`stheno-query`** — field-selection parser, planner, batched level executor, hash joins,
   depth and node limits.
4. **`stheno-events`** — `EventBus` trait, in-process and Redis backends, transactional buffering,
   subscriber registration, dead-letter table.
5. **The order-change model as a standalone crate** — `order_change`, `order_change_action`, the
   apply/preview engine, with a fixture corpus extracted from Medusa's integration tests.
6. **Scheduler** — cron parsing, advisory-lock single execution.
7. **`stheno-registry`** — `AppContext` and the composition root.

### Exit gate

- A 5-step workflow with parallel branches, retries, and an async step completes durably.
- **Kill-and-resume:** SIGKILL at every step boundary; the workflow either completes or fully
  reverts. Zero partial states across 100 runs.
- Compensation unwinds in exact reverse order for every injected failure position.
- 100 concurrent starts with one `transaction_id` produce exactly one execution.
- The query engine resolves a 3-level cross-module selection in a **bounded, asserted** number of
  SQL queries — the N+1 assertion is part of the test.
- The order-change engine reproduces every fixture in the corpus: preview matches, confirm
  produces the correct version bump and summary.

### Risk

**R1, the order-change model.** It is 27% of all workflows and the least documented part of
Medusa. Building it here — standalone, fixture-driven, before any UI depends on it — is the single
most important sequencing decision in this plan. If it slips, everything from P5 slips, so give it
the strongest engineer and do not parallelise it away.

---

## P2 — Catalog and inventory (5 weeks)

First real domain. Establishes the module pattern everything else copies.

### Deliverables

- `stheno-product` — 12 entities, service, filters, events
- `stheno-inventory` — 3 entities, reservations, concurrency-safe level adjustment
- `stheno-stock-location` — 2 entities
- `stheno-sales-channel` — 1 entity
- 27 product + 4 inventory + 3 category + 3 stock-location workflows
- Store endpoints: products, variants, categories, collections, tags, types, options
- Links: `product_sales_channel`, `product_variant_inventory_item`, `sales_channel_location`
- `stheno new-module` scaffold CLI, extracted from doing this once by hand

### Exit gate

- All store catalog endpoints pass the parity suite.
- `GET /store/products` with 20 items and variant expansion meets **25 ms p95** on the reference
  catalog.
- **Oversell test:** 100 concurrent reservations against 1 unit of stock — exactly 1 succeeds,
  99 fail cleanly.
- Batch product create of 1,000 products completes as one workflow with per-row error reporting.

### Risk

Variant/option-value identity (the unique-across-a-join-set constraint in
[DATA_MODEL §2.1](DATA_MODEL.md)) is easy to get subtly wrong and produces duplicate variants under
concurrency. Test it explicitly.

---

## P3 — Pricing, tax, region, currency (4 weeks)

The highest-leverage performance work in the project.

### Deliverables

- `stheno-pricing` — 6 entities, price lists, rules, preferences
- `stheno-tax` — 4 entities, rate resolution, provider trait
- `stheno-region`, `stheno-currency`, `stheno-store`
- 7 price-list + 3 pricing + 9 tax + 3 region + 3 store workflows
- `calculated_price` as **one CTE query** with rule specificity ranked in SQL
- **The checkout threat-modelling session** ([THREAT_MODEL_CHECKOUT.md](THREAT_MODEL_CHECKOUT.md)) —
  run at the end of this phase so its findings land before P4 writes payment code
- Links: `product_variant_price_set`, `shipping_option_price_set`, `region_payment_provider`

### Exit gate

- `calculated_price` for 100 variants across 3 regions and 2 customer groups resolves in **one
  query**, asserted by a query counter in the test.
- Price resolution matches Medusa on a fixture corpus covering: quantity tiers, overlapping rules,
  price-list override vs sale, and tie-breaking.
- **Tax-inclusive reverse computation** is exact — line taxes sum to the order tax with zero
  residual, property-tested at every currency precision.
- `GET /store/products` with prices still meets 25 ms p95 with pricing in the path.
- Threat-modelling session held; findings appended to [SECURITY §2](SECURITY.md) with owners and
  phases assigned.

### Risk

Rule specificity ranking is the subtlest logic in the system. Fetching candidates and ranking in
Rust is the tempting shortcut and it is what makes the endpoint slow. Rank in SQL.

---

## P4 — Cart, promotion, payment (6 weeks)

### Deliverables

- `stheno-cart` — 9 entities, the totals engine as a pure function
- `stheno-promotion` — 7 entities, rule evaluation, buyget, atomic campaign budgets
- `stheno-payment` — 8 entities, session lifecycle, captures, refunds, account holders
- `stheno-payment-stripe` — hand-rolled on `reqwest`; cards, wallets, SCA, webhooks, async methods
- The payment safety layer per [PAYMENTS.md](PAYMENTS.md): `payment_provider_request` (outbound
  idempotency), `payment_webhook_event` (raw store), the convergence reducer, the money boundary,
  and DB-enforced ledger invariants
- 14 cart + 11 promotion + 4 payment + 6 payment-collection + 1 line-item workflows
- Store endpoints: carts, line items, promotions, shipping methods, payment collections/providers
- Webhook ingestion with signature verification and replay protection

### Exit gate

- A cart with 10 items, 2 promotions, and tax across 3 regions produces totals **identical to
  Medusa**, fixture-verified.
- **Campaign budget:** 100 concurrent orders against a 50-use budget — exactly 50 succeed.
- `POST /carts/:id/line-items` meets **30 ms p95** including full totals recompute.
- Stripe test-mode authorise → capture → partial refund works end to end, including a 3DS
  step-up that suspends and resumes the workflow.
- A replayed webhook produces no duplicate side effects.
- **The edge atlas passes.** All 34 cases in [PAYMENTS §11](PAYMENTS.md), specifically: a
  fault-injection proxy that drops responses *after* Stripe commits produces exactly one charge
  (E1, E2); webhooks delivered before the API response, out of order, and duplicated all converge
  to correct state (E7–E9); JPY and KWD amounts convert correctly (E15, E16); 100 concurrent
  captures and refunds never breach the ledger invariants (E21–E23).
- Money property tests pass across all 22 special-precision currencies.

### Risk

Promotion stacking order. Multiple applicable promotions must resolve deterministically or totals
become irreproducible. Document the order, then fixture-test it — do not leave it emergent.

Second risk, and the more expensive one: **payment ambiguity**. The idempotency ledger and
convergence reducer must land *before* the Stripe provider, not after. Retrofitting idempotency
onto a working integration means auditing every call site, and the bug it prevents — double
charging — is one you discover from customers rather than from tests.

---

## P5 — Order, fulfillment, sales channel (6 weeks)

### Deliverables

- `stheno-order` — 23 entities, wired to the P1 order-change engine
- `stheno-fulfillment` — 12 entities, geo zones, shipping options, rules, provider trait
- `stheno-fulfillment-manual` provider
- The **`complete-cart` workflow** in full ([WORKFLOWS §2.3](WORKFLOWS.md))
- 8 fulfillment + the lifecycle subset of the 68 order workflows
- Store endpoints: orders, order transfer, returns, shipping options
- Links: order↔cart, order↔payment_collection, order↔promotion, order↔fulfillment

### Exit gate

- Guest checkout end to end: browse → cart → shipping → Stripe → order, on a seeded catalog.
- **Crash resumption:** SIGKILL at each of the 11 checkout steps; every run either completes with
  a correct order or fully reverts with inventory released and payment voided.
- Duplicate `POST /carts/:id/complete` returns the same order. **No double charge**, verified
  against Stripe test mode.
- `complete-cart` meets **250 ms p95**.
- `display_id` is sequential and gap-free under 50-way concurrent checkout.

### Risk

The compensation boundary at step 11 ([WORKFLOWS §2.3](WORKFLOWS.md)). A failed confirmation email
must never roll back a paid order. Get the `no_compensation` markers right or you will refund
customers for SMTP outages.

---

## P6 — Order lifecycle (6 weeks)

The remaining bulk of the 68 order workflows.

### Deliverables

- Order edits: begin, add/update/remove items and shipping, preview, confirm, cancel
- Returns: request, approve, receive with per-item received/damaged, dismiss, refund
- Claims: replace and refund kinds, inbound/outbound items, evidence images
- Exchanges: inbound return + outbound items with computed difference
- `stheno-draft-order` — 9 workflows, authoring and conversion
- Order transfer between customers
- `GET …/preview` for every mutation — computed, zero writes

### Exit gate

- Every mutation is previewable with a correct payment delta before commit.
- Fixture corpus from Medusa passes for all four mutation families.
- Partial return receipt (3 of 5 items, 1 damaged) produces the correct refund and correct
  inventory restock.
- An order edit that increases the total collects the difference; one that decreases it refunds.
- The order timeline reconstructs exactly from `order_change_action` rows.

### Risk

This phase is where scope creep lives — each family has edge cases (partial receipt, mixed
dispositions, cancel-after-partial) that multiply. The fixture corpus from P1 is the scope
boundary: if Medusa does it, it is in; if not, it is 1.1.

---

## P7 — Auth, user, customer, API keys, RBAC (4 weeks, Track B)

Runs parallel to P2–P3 once P1 lands.

### Deliverables

- `stheno-auth` — 6 entities, emailpass + Google + GitHub + OIDC providers
- `stheno-user`, `stheno-customer`, `stheno-api-key`, `stheno-rbac`
- MFA: TOTP factors, recovery codes, challenge/verify
- JWT issue and refresh rotation with reuse detection
- RBAC: policies, roles, inheritance closure, deny-beats-allow evaluation
- 8 customer + 8 user + 4 invite + 4 api-key + 9 rbac + 2 auth + 3 customer-group workflows
- Publishable-key → sales-channel scope resolution middleware
- **scrypt verification + transparent Argon2id re-hash** (FR-AUTH-10)

### Exit gate

- All four auth providers issue working JWTs.
- Refresh-token reuse invalidates the family and forces re-auth.
- RBAC denies correctly across a 4-level inheritance chain with a deny at depth 3.
- A publishable key scopes store queries to its sales channels — verified by attempting to fetch
  an out-of-scope product and getting a 404.
- An imported Medusa scrypt hash authenticates, then silently becomes Argon2id.
- Auth endpoints rate-limit and lock out under brute force.

---

## P8 — Platform services (5 weeks, Track B)

Runs parallel to P4–P5.

### Deliverables

- `stheno-file` + local and S3 providers, presigned uploads
- `stheno-notification` + local and SendGrid providers, templates, delivery records
- `stheno-search` + Postgres full-text provider, event-driven index sync
- `stheno-index` — the denormalised read index, incremental update, full rebuild, lag metric
- `stheno-caching` + in-process and Redis providers
- `stheno-locking` + Postgres and Redis providers
- `stheno-translation` — locale-scoped field overrides
- `stheno-settings` — view configs, layouts, property labels, user preferences
- **The erasure pipeline and retention jobs** ([SECURITY §11](SECURITY.md)) — pseudonymisation
  cascade, the customer export endpoint, and retention enforcement for webhook events, workflow
  context, logs, and carts. F2 is a launch blocker
- `stheno-analytics` + local and PostHog providers
- 8 settings + 5 translation + 2 file workflows

### Exit gate

- The search index converges within 5 s of a 1,000-product bulk update.
- Read-index lag is exported as a metric and stays under 2 s at 500 rps.
- **Redis-outage test:** kill Redis; the system degrades to in-process cache, Postgres locks, and
  in-process events with zero failed requests on a single node.
- A translated product returns locale-specific fields on the store API.
- **Erasure verification:** after erasing a test customer, no personal data remains in any store
  listed in the data inventory — including the read index, search index, workflow context, and
  webhook payloads — while order totals and financial records are unchanged.
- Every retention job runs and is alerted on if stale.

---

## P9 — Admin API and forked dashboard (6 weeks, Track C)

Starts once P5 provides real orders. The largest single phase.

### Deliverables

**Backend (3 weeks):** ~278 admin routes across 51 resource groups, emitting **Medusa's exact JSON
shapes**. Offset pagination, `{ type, message }` errors, Medusa's batch semantics. Additive
workflow-observability endpoints. OpenAPI generation for third-party integrators.

**Fork and adapt (1.5 weeks):** fork `packages/admin/dashboard` at a pinned Medusa version, strip
every Enterprise path ([ADR-008](DECISIONS.md#adr-008)), point it at Stheno, patch the API client to
send the CSRF header, rebrand.

**Rebuild Enterprise UI clean-room (1.5 weeks):** roles, policies, the permissions provider,
required-permissions sections, and SSO login. **This is the only genuinely new UI in 1.0**, and it
cannot reference Medusa's implementation.

### Exit gate

- **The unmodified Medusa dashboard runs against Stheno** before any fork changes are applied.
  This is the gate that proves admin compatibility, and it must pass first.
- **Priya's five jobs** ([PRD §4.1](PRD.md)) complete unaided on the forked dashboard.
- Every admin endpoint passes the parity corpus.
- Clean-room RBAC and SSO screens work, with no code derived from Enterprise paths — verified by
  review, and recorded.
- WCAG 2.2 AA audit passes; gaps inherited from upstream are fixed in the fork.
- Workflow execution timeline renders with replay-from-step working.

### Risk

**Fork drift.** Pin the upstream Medusa version and record it. Merging upstream later is a
deliberate, scheduled task — tracking `develop` turns this phase into a permanent chase.

The clean-room RBAC UI is the schedule risk here, not the fork. It has no reference implementation
and cannot borrow one.

---

## P10 — Storefront, migration, hardening, GA (5 weeks)

### Deliverables

- `apps/storefront` — **forked** from Medusa's Next.js starter, using `@medusajs/js-sdk` unmodified
- `stheno import-medusa` — full importer with `--dry-run`, ID preservation, money-precision
  assertions, scrypt hash carry-over, index rebuild
- Security review against OWASP ASVS L2, plus an external penetration test
- Performance pass against every budget
- Docker images, Helm chart, deployment docs
- Documentation site: getting started, module authoring, provider authoring, migration guide

### Exit gate

- **An unmodified Medusa storefront AND an unmodified Medusa admin dashboard both run against
  Stheno with zero code changes** (G2). This is the headline claim and it is binary.
- A production-shaped Medusa database imports with 100% entity fidelity; order, payment, and
  refund totals match the source **exactly**.
- All seven latency budgets met on reference hardware.
- Memory ≤ 400 MB at 500 rps sustained for 1 hour.
- Storefront: Lighthouse ≥ 95, LCP < 2.0 s, INP < 150 ms, CLS < 0.05 at p75.
- Penetration test yields no high or critical findings.
- SAQ A self-assessment complete, including the script-attack eligibility criterion: written PSP
  confirmation on file **and** the checkout script manifest enforced in CI.
- EAA / WCAG 2.2 AA audit passes on the storefront and admin.
- Authorization matrix test covers every role × endpoint × ownership case.
- Restore drill completed; erasure replay after restore verified.
- Full parity matrix at 100% of MUST rows.

---

## 2. Staffing

| Role | Count | Phases | Focus |
|---|---:|---|---|
| Rust lead | 1 | P0–P10 | Workflow engine, order-change model, query engine, architecture calls |
| Rust backend | 2 | P2–P8 | Domain modules, workflows, providers |
| Frontend | 1–2 | P9–P10 | Admin dashboard, storefront |
| QA / infra | 1 | P0–P10 | Parity harness, load harness, CI, deployment |

**The Rust lead builds P1 personally.** The workflow engine and order-change model are the two
components where a wrong abstraction costs months, and they are not delegable to someone learning
the domain at the same time.

---

## 3. Sequencing rules

1. **Platform before domain.** No module starts before P1 lands. Building modules against a
   changing workflow API means rewriting them.
2. **Parity harness before endpoints.** P0, not P5. Retrofitting parity means asserting what you
   built rather than what Medusa does.
3. **Order-change model before order UI.** P1, standalone, fixture-driven.
4. **Shared admin patterns before admin routes.** P9's estimate depends on it.
5. **Load suite at every gate.** Not a P10 activity.
6. **Track B and C start when their dependency lands, not earlier.** Idle engineers are cheaper
   than rewritten code.

---

## 4. What would make this take longer

Stated plainly, since a plan that only describes the happy path is not a plan:

| Trigger | Cost |
|---|---|
| `FilterSpec` macro fails its P0 validation | +2 weeks, per-module builders |
| Order-change model needs a second attempt | +4 weeks, and everything after P5 shifts |
| Team is new to Rust | +4–6 weeks across P0–P2 |
| Medusa's Store API drifts materially before GA | +1–2 weeks to re-record and reconcile |
| A second payment provider becomes 1.0 scope | +2 weeks |
| B2B or subscriptions get pulled into 1.0 | +8 weeks minimum. Say no |

---

## 5. Definition of done, per module

A module is complete when **all** of the following hold. No partial credit:

- [ ] Every entity migrated, `schema.rs` checked in, drift check green
- [ ] Service covers list, retrieve, create, update, delete, soft-delete, restore
- [ ] `FilterSpec` declared for every list endpoint
- [ ] Every workflow in the domain implemented, with compensation
- [ ] Domain events emitted, transactionally buffered
- [ ] Store endpoints pass the parity suite
- [ ] Admin endpoints implemented with cursor pagination and problem+json
- [ ] Integration tests against real Postgres, ≥ 85% line coverage
- [ ] Money paths at 100% coverage with property tests
- [ ] Latency budget met under load
- [ ] **No dependency on another module crate** — CI-verified
- [ ] `MODULE.toml` accurate: owned tables, published events, links
- [ ] Owns exactly one Postgres schema; no SQL references another module's
- [ ] `models.rs` is `pub(crate)`; only `dto.rs` crosses the boundary
- [ ] Subscribes to no events from inside the module
- [ ] OpenAPI annotations complete; generated SDK diff is empty
