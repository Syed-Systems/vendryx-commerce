# Phase 2 Task Decomposition

**212 tasks across 10 phases (P11–P20).** Each is 2 hours to 2 days, has explicit dependencies,
and ends in a **command that exits 0** — never a judgement call.

[PHASE2_PLAN.md](PHASE2_PLAN.md) says what each phase achieves and why. This document is what an
agent or engineer actually picks up.

Task IDs continue from Phase 1 and never collide with it. [TASKS.md](../TASKS.md) P0–P10 IDs remain
valid references.

---

## 1. The loop protocol

Unchanged from [TASKS.md §1](../TASKS.md). Rules **L1–L8** apply verbatim, including the one that
matters most:

> **L3 — Never edit the exit command to make it pass.** Changing the gate is a spec change and
> needs sign-off.

Phase 1 ended with three gate scripts that asserted numbers nobody measured and two gate scripts
that were never written. That is what L3 exists to prevent, and it is why every exit command below
either measures something or fails.

### 1.1 The universal gate, Phase 2 edition

Runs on **every** task before its specific exit command. Two additions from Phase 1, both
non-negotiable from P11 onward:

```bash
# DATABASE_URL must be set. A task that passes without it has tested nothing.
test -n "$DATABASE_URL" || { echo "DATABASE_URL unset"; exit 1; }

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                    # now runs against real Postgres
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-module-contracts.sh
./scripts/check-dto-separation.sh
./scripts/check-schema-drift.sh           # blocking from P11-22, not advisory
./scripts/check-no-domain-gates.sh
./scripts/check-no-in-memory-stores.sh    # new: fails on RwLock<HashMap> in a migrated module
```

`check-no-in-memory-stores.sh` is written in P11-21 and takes a list of modules declared migrated.
It is what stops a module from silently regressing to a `HashMap` after its phase closes.

### 1.2 Reading a task row

Unchanged. **ID** is stable and never renumbered. **Deps** must be `DONE` first. **Exit** is the
command that decides, assuming repo root.

Where an exit command names a test that does not exist yet, **writing that test is part of the
task.**

---

## P11 — Persistence foundations (22 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P11-01 | CI Postgres service container; `DATABASE_URL` in the test job; document local setup | — | `./scripts/check-ci-postgres.sh` |
| P11-02 | **Fork upstream verification** — establish a real, pinned Medusa dashboard tag that builds; record commit in `apps/admin/UPSTREAM.md`. Unblocks P18 | — | `./scripts/check-upstream-pin.sh` |
| P11-03 | `create_pool` called from the real boot path; `DATABASE_URL` read; pool size configurable | P11-01 | `cargo test -p stheno-server --test boot_pool` |
| P11-04 | **Decide and implement the `Id<T>` wire representation** — prefix preserved end to end, no `ends_with` scans. ADR amendment first (L8) | — | `cargo test -p stheno-types --test id_roundtrip` |
| P11-05 | `diesel_migrations` embedded; migrations run on boot behind `--migrate` | P11-03 | `cargo test -p stheno-db --test migration_runner` |
| P11-06 | `--check-migrations` mode exits non-zero on schema drift | P11-05 | `./scripts/check-schema-drift.sh` |
| P11-07 | `PgCrudRepository<T>`: `retrieve`, `create`, `update`, `delete` | P11-05, P11-04 | `cargo test -p stheno-db --test pg_crud_basic` |
| P11-08 | `PgCrudRepository<T>`: `soft_delete`, `restore`, `upsert`, `create_many` | P11-07 | `cargo test -p stheno-db --test pg_crud_lifecycle` |
| P11-09 | `FilterSpec` → SQL for equality, comparison, and null operators | P11-07 | `cargo test -p stheno-db --test filter_sql_scalar` |
| P11-10 | `FilterSpec` → SQL for `in`, `like`, range, and negation operators | P11-09 | `cargo test -p stheno-db --test filter_sql_set` |
| P11-11 | `FilterSpec` → SQL for ordering, offset pagination, and `list_and_count` | P11-10 | `cargo test -p stheno-db --test filter_sql_paging` |
| P11-12 | **`FilterSpec` viability spike against the 20 hardest list endpoints.** If it cannot express them, STOP and escalate — this is PRD R3 coming due | P11-11 | `cargo test -p stheno-db --test filter_hard_cases` |
| P11-13 | `with_savepoint` proven under three levels of nesting with partial rollback | P11-07 | `cargo test -p stheno-db --test transaction_nesting` |
| P11-14 | `SERIALIZABLE` retry helper with bounded backoff | P11-13 | `cargo test -p stheno-db --test serializable_retry` |
| P11-15 | Two concurrent writers produce a serialisation failure that the helper retries; both commit | P11-14 | `cargo test -p stheno-db --test concurrent_writers` |
| P11-16 | `TestApp::spawn()` provisions an isolated schema, migrates into it, drops it on teardown. It currently generates a schema name and ignores it | P11-05 | `cargo test -p stheno-server --test test_harness_isolation` |
| P11-17 | Migrate `region` onto `PgCrudRepository` | P11-08, P11-11, P11-16 | `cargo test -p stheno-region` |
| P11-18 | Migrate `currency` onto `PgCrudRepository` | P11-17 | `cargo test -p stheno-currency` |
| P11-19 | Migrate `product` — products, variants, options, option values | P11-18 | `cargo test -p stheno-product --test persistence` |
| P11-20 | Migrate `product` — categories with materialized paths, collections, tags, types | P11-19 | `cargo test -p stheno-product --test taxonomy_persistence` |
| P11-21 | Write `check-no-in-memory-stores.sh` and add it to the universal gate | P11-20 | `./scripts/check-no-in-memory-stores.sh` |
| P11-22 | **Restart durability test**: boot, create a product, restart the process, retrieve the same id | P11-19 | `./scripts/restart-durability.sh` |

**Phase gate:** `./scripts/gate-p11.sh`

---

## P12 — Persistence: catalog, pricing, inventory, config (24 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P12-01 | Write the migration playbook from P11's three modules; every later migration follows it unamended | P11-22 | `test -f docs/MIGRATION_PLAYBOOK.md && ./scripts/check-playbook.sh` |
| P12-02 | `pricing` schema: price sets, prices, price lists, price rules | P12-01 | `cargo test -p stheno-pricing --test schema` |
| P12-03 | Migrate `pricing` service onto the repository | P12-02 | `cargo test -p stheno-pricing` |
| P12-04 | Price calculation hot path: benchmark generic repository vs `sql_query` | P12-03 | `cargo bench -p stheno-pricing --bench price_calc` |
| P12-05 | Implement whichever P12-04 chose; record the decision in DECISIONS.md | P12-04 | `cargo test -p stheno-pricing --test hot_path` |
| P12-06 | `inventory` schema: items, levels, reservations | P12-01 | `cargo test -p stheno-inventory --test schema` |
| P12-07 | Migrate `inventory` service; reservation quantities under concurrency | P12-06 | `cargo test -p stheno-inventory` |
| P12-08 | Concurrent reservation test: 50 parallel reservations against stock of 10 → exactly 10 succeed | P12-07 | `cargo test -p stheno-inventory --test concurrent_reservation` |
| P12-09 | `stock-location` schema and migration | P12-06 | `cargo test -p stheno-stock-location` |
| P12-10 | `sales-channel` schema and migration | P12-01 | `cargo test -p stheno-sales-channel` |
| P12-11 | `tax` schema: regions, rates, overrides | P12-01 | `cargo test -p stheno-tax` |
| P12-12 | `store` schema: store, store currencies, store locales | P12-01 | `cargo test -p stheno-store` |
| P12-13 | Variant option-value uniqueness as a database unique index; delete the `variant_hashes` map | P11-20 | `cargo test -p stheno-product --test variant_unique_index` |
| P12-14 | Concurrent duplicate-variant insert rejected by the index, not by a read-then-write check | P12-13 | `cargo test -p stheno-product --test variant_concurrent_insert` |
| P12-15 | Indexes for every filterable column on catalog tables | P12-03, P12-07 | `./scripts/check-filter-indexes.sh` |
| P12-16 | Seed harness: generate 10k products with variants and prices | P12-15 | `cargo run -p stheno-cli -- seed --products 10000` |
| P12-17 | Catalog list latency budget at 10k products with realistic filters | P12-16 | `k6 run tests/load/catalog.js --quiet` |
| P12-18 | Cross-module list filter: products filtered by price range through a real join | P12-15 | `cargo test -p stheno-query --test cross_module_filter` |
| P12-19 | Store catalog parity re-verified against Postgres | P12-13 | `./scripts/parity.sh --group store-catalog` |
| P12-20 | Store pricing parity re-verified against Postgres | P12-05 | `./scripts/parity.sh --group store-pricing` |
| P12-21 | Admin catalog parity re-verified against Postgres | P12-19 | `./scripts/parity.sh --group admin-catalog` |
| P12-22 | Admin inventory parity re-verified against Postgres | P12-08 | `./scripts/parity.sh --group admin-inventory` |
| P12-23 | Admin pricing parity re-verified against Postgres | P12-20 | `./scripts/parity.sh --group admin-pricing` |
| P12-24 | Admin config parity re-verified against Postgres | P12-11, P12-12 | `./scripts/parity.sh --group admin-config` |

**Phase gate:** `./scripts/gate-p12.sh`

---

## P13 — Persistence: cart, order, payment, fulfilment (26 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P13-01 | `WorkflowStore` schema: executions, step states, leases | P12-01 | `cargo test -p stheno-workflow --test store_schema` |
| P13-02 | Migrate `WorkflowStore` onto Postgres | P13-01 | `cargo test -p stheno-workflow --test durable_store` |
| P13-03 | Lease acquisition and expiry across two processes | P13-02 | `cargo test -p stheno-workflow --test lease_contention` |
| P13-04 | Compensation stack persisted; recoverable after restart | P13-02 | `cargo test -p stheno-workflow --test compensation_durability` |
| P13-05 | **Kill-resume against real restarts**, not simulated ones | P13-04 | `./scripts/kill-resume.sh` |
| P13-06 | `cart` schema: carts, line items, adjustments, tax lines, shipping methods | P12-01 | `cargo test -p stheno-cart --test schema` |
| P13-07 | Migrate `cart` service onto the repository | P13-06 | `cargo test -p stheno-cart` |
| P13-08 | `promotion` schema: promotions, rules, campaigns, budgets | P12-01 | `cargo test -p stheno-promotion --test schema` |
| P13-09 | Migrate `promotion`; campaign budget as an atomic conditional update | P13-08 | `cargo test -p stheno-promotion` |
| P13-10 | Campaign budget concurrency: 100 parallel redemptions against a budget of 10 | P13-09 | `cargo test -p stheno-promotion --test budget_concurrency` |
| P13-11 | `payment` schema: collections, sessions, payments, refunds, captures | P12-01 | `cargo test -p stheno-payment --test schema` |
| P13-12 | Migrate `payment` service onto the repository | P13-11 | `cargo test -p stheno-payment` |
| P13-13 | Idempotency keys persisted; survive a restart | P13-12 | `cargo test -p stheno-payment --test idempotency_durability` |
| P13-14 | Duplicate capture with the same key across a restart charges once | P13-13 | `cargo test -p stheno-payment --test outbound_idempotency` |
| P13-15 | `order` schema: orders, items, summaries, transactions, credit lines | P12-01 | `cargo test -p stheno-order --test schema` |
| P13-16 | Migrate `order` service onto the repository | P13-15 | `cargo test -p stheno-order` |
| P13-17 | Order versioning and `get_order_at_version` against real rows | P13-16 | `cargo test -p stheno-order --test versioning` |
| P13-18 | `order-change` schema and migration: changes, actions, previews | P13-16 | `cargo test -p stheno-order-change` |
| P13-19 | `fulfillment` schema: fulfilments, sets, providers, shipping options and profiles | P12-01 | `cargo test -p stheno-fulfillment --test schema` |
| P13-20 | Migrate `fulfillment` service onto the repository | P13-19 | `cargo test -p stheno-fulfillment` |
| P13-21 | Checkout transactional integrity: cart completion, payment, order, reservation commit or roll back together | P13-07, P13-12, P13-16, P12-08 | `cargo test -p stheno-flows --test checkout_atomicity` |
| P13-22 | Concurrent completion of the same cart produces exactly one order | P13-21 | `cargo test -p stheno-flows --test cart_completion_race` |
| P13-23 | Money precision through `NUMERIC`, property-tested at every ISO 4217 precision | P13-16 | `cargo test -p stheno-order --test money_precision_db` |
| P13-24 | Store cart, order, and return parity re-verified against Postgres | P13-21 | `./scripts/parity.sh --group store-cart && ./scripts/parity.sh --group store-order` |
| P13-25 | Admin order, payment, and fulfilment parity re-verified against Postgres | P13-18, P13-20 | `./scripts/parity.sh --group admin-orders && ./scripts/parity.sh --group admin-payments` |
| P13-26 | Checkout latency budget against Postgres | P13-21 | `k6 run tests/load/checkout.js --quiet` |

**Phase gate:** `./scripts/gate-p13.sh`

---

## P14 — Persistence: platform modules and link tables (22 tasks)

Runs parallel with P13. Depends on P12, not P13.

| ID | Task | Deps | Exit |
|---|---|---|---|
| P14-01 | `customer` schema and migration: customers, groups, addresses | P12-01 | `cargo test -p stheno-customer` |
| P14-02 | `auth` schema and migration: identities, providers, tokens, MFA factors | P12-01 | `cargo test -p stheno-auth` |
| P14-03 | Token version invalidation persisted; a revoked token stays revoked after restart | P14-02 | `cargo test -p stheno-auth --test token_revocation_durability` |
| P14-04 | `user` schema and migration: users, invites | P12-01 | `cargo test -p stheno-user` |
| P14-05 | `api-key` schema and migration; publishable and secret keys | P12-01 | `cargo test -p stheno-api-key` |
| P14-06 | `rbac` schema and migration: roles, policies, assignments | P14-04 | `cargo test -p stheno-rbac` |
| P14-07 | `file` schema and migration | P12-01 | `cargo test -p stheno-file` |
| P14-08 | `notification` schema and migration; delivery log and suppression | P12-01 | `cargo test -p stheno-notification` |
| P14-09 | `settings` schema and migration | P12-01 | `cargo test -p stheno-settings` |
| P14-10 | `translation` schema and migration: locales, translations, overrides | P12-01 | `cargo test -p stheno-translation` |
| P14-11 | `audit` schema, append-only, with PII redaction at write time | P12-01 | `cargo test -p stheno-audit` |
| P14-12 | `analytics` schema and migration | P12-01 | `cargo test -p stheno-analytics` |
| P14-13 | `search` index persistence and full rebuild from source rows | P12-01 | `cargo test -p stheno-search` |
| P14-14 | `index` denormalised tables, incremental sync, lag metric | P14-13 | `cargo test -p stheno-index` |
| P14-15 | `caching` backed by Redis with a Postgres fallback path | P12-01 | `cargo test -p stheno-caching` |
| P14-16 | `locking` on Postgres advisory locks, held on a pinned connection | P12-01 | `cargo test -p stheno-locking` |
| P14-17 | Two processes contend for one advisory lock; exactly one proceeds | P14-16 | `cargo test -p stheno-locking --test advisory_contention` |
| P14-18 | Migrations for all **22 writable link tables**, each with a unique index on the pair | P12-01 | `cargo test -p stheno-links --test link_tables_exist` |
| P14-19 | `LinkStore` backed by the real tables; cascade behaviour honoured per definition | P14-18 | `cargo test -p stheno-links` |
| P14-20 | The 14 read-only link declarations wired into the query engine as traversals | P14-19 | `cargo test -p stheno-query --test readonly_link_traversal` |
| P14-21 | Erasure cascade and verification against real rows | P14-01, P14-11 | `cargo test -p stheno-flows --test erasure_cascade --test erasure_verification` |
| P14-22 | No `RwLock<HashMap>` remains anywhere under `crates/modules/` | P14-21, P13-20 | `./scripts/check-no-in-memory-stores.sh --all` |

**Phase gate:** `./scripts/gate-p14.sh`

---

## P15 — Quota counters and entitlement enforcement (12 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P15-01 | `entitlement_usage` table and migration | P14-22 | `cargo test -p stheno-db --test entitlement_usage_schema` |
| P15-02 | Atomic conditional increment; zero rows returned means exhausted | P15-01 | `cargo test -p stheno-db --test quota_increment` |
| P15-03 | Decrement on delete | P15-02 | `cargo test -p stheno-db --test quota_decrement` |
| P15-04 | `CheckQuotaStep` as a workflow step, not a service branch | P15-02 | `cargo test -p stheno-flows --test check_quota_step` |
| P15-05 | Wire `CheckQuotaStep` into product creation | P15-04 | `cargo test -p stheno-flows --test product_quota` |
| P15-06 | Wire `CheckQuotaStep` into staff-seat creation | P15-04, P14-04 | `cargo test -p stheno-flows --test seat_quota` |
| P15-07 | Rejection is exact: creation blocked at the limit, not at limit+1 | P15-05 | `cargo test -p stheno-flows --test quota_boundary` |
| P15-08 | **Concurrency: 100 parallel creates against a limit of 10 → exactly 10 succeed** | P15-07 | `cargo test -p stheno-flows --test quota_concurrency` |
| P15-09 | Over-quota error in Medusa's `{ type, message }` shape with the upsell in the message | P15-07 | `cargo test -p stheno-api-admin --test over_quota_error` |
| P15-10 | Downgrade and grandfathering: 412 products on a limit of 100 all readable, editable, 413th rejected | P15-08 | `cargo test -p stheno-flows --test grandfathering` |
| P15-11 | `GET /admin/entitlements` reports real usage; remove the hardcoded `used: 0` | P15-08 | `cargo test -p stheno-api-admin-platform --test entitlements` |
| P15-12 | Nightly reconciliation job; detects a deliberately corrupted counter and alerts | P15-03 | `cargo test -p stheno-flows --test quota_reconciliation` |

**Phase gate:** `./scripts/gate-p15.sh`

---

## P16 — Reporting module (14 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P16-01 | Scaffold `stheno-reporting` crate; MODULE.toml; wire into the registry | P13-16 | `cargo test -p stheno-reporting` |
| P16-02 | Daily rollup schema: revenue, orders, units, by day and currency | P16-01 | `cargo test -p stheno-reporting --test rollup_schema` |
| P16-03 | Incremental rollup refresh from `order.placed` | P16-02 | `cargo test -p stheno-reporting --test rollup_incremental` |
| P16-04 | **Rollup invalidation** on order edits, returns, refunds, and cancellations | P16-03 | `cargo test -p stheno-reporting --test rollup_invalidation` |
| P16-05 | Revenue-over-time aggregate | P16-03 | `cargo test -p stheno-reporting --test revenue_over_time` |
| P16-06 | Orders-over-time aggregate | P16-03 | `cargo test -p stheno-reporting --test orders_over_time` |
| P16-07 | Top-products aggregate | P16-03 | `cargo test -p stheno-reporting --test top_products` |
| P16-08 | Average-order-value aggregate | P16-03 | `cargo test -p stheno-reporting --test average_order_value` |
| P16-09 | Window gate: `analytics_window_days` bounds the query in SQL, no domain branching | P16-05, P15-11 | `cargo test -p stheno-reporting --test window_gate` |
| P16-10 | A Free deployment cannot retrieve data older than 30 days | P16-09 | `cargo test -p stheno-reporting --test free_tier_horizon` |
| P16-11 | A Pro deployment retrieves full history | P16-09 | `cargo test -p stheno-reporting --test pro_tier_horizon` |
| P16-12 | `GET /admin/reports/*` in Medusa's envelope shape | P16-08 | `./scripts/parity.sh --group admin-reports` |
| P16-13 | CSV export of every report, UTF-8 BOM, reusing the P10-03e path | P16-12 | `cargo test -p stheno-reporting --test report_csv_export` |
| P16-14 | Analytics latency budget at 100k orders | P16-12 | `k6 run tests/load/reporting.js --quiet` |

**Phase gate:** `./scripts/gate-p16.sh`

---

## P17 — Admin API completion (32 tasks)

Route groups follow their storage: catalog groups after P12, order groups after P13.

| ID | Task | Deps | Exit |
|---|---|---|---|
| P17-01 | Re-record the parity corpus against the pinned Medusa version | P12-01 | `./scripts/parity.sh --record --group all` |
| P17-02 | Route-coverage test: fails when a route in API_SPEC is unmounted | P17-01 | `cargo test -p stheno-api-admin --test route_coverage` |
| P17-03 | Admin products: full CRUD, batch, import, export | P12-21 | `./scripts/parity.sh --group admin-products` |
| P17-04 | Admin product-variants and product-options | P17-03 | `./scripts/parity.sh --group admin-variants` |
| P17-05 | Admin product-tags, product-types, collections | P17-03 | `./scripts/parity.sh --group admin-taxonomy` |
| P17-06 | Admin product-categories, including tree operations | P17-05 | `./scripts/parity.sh --group admin-categories` |
| P17-07 | Admin inventory-items and reservations | P12-22 | `./scripts/parity.sh --group admin-inventory` |
| P17-08 | Admin price-lists | P12-23 | `./scripts/parity.sh --group admin-price-lists` |
| P17-09 | Admin price-preferences | P17-08 | `./scripts/parity.sh --group admin-price-preferences` |
| P17-10 | Admin orders: list, detail, filters | P13-25 | `./scripts/parity.sh --group admin-orders` |
| P17-11 | Admin order lifecycle transitions | P17-10 | `./scripts/parity.sh --group admin-order-lifecycle` |
| P17-12 | Admin order-edits and preview endpoints | P17-11, P13-18 | `./scripts/parity.sh --group admin-order-edits` |
| P17-13 | Admin returns and return-reasons | P17-11 | `./scripts/parity.sh --group admin-returns` |
| P17-14 | Admin claims | P17-12 | `./scripts/parity.sh --group admin-claims` |
| P17-15 | Admin exchanges | P17-12 | `./scripts/parity.sh --group admin-exchanges` |
| P17-16 | Admin draft-orders, distinct from the social-draft path | P17-10 | `./scripts/parity.sh --group admin-draft-orders` |
| P17-17 | Admin fulfillments and fulfillment-sets | P13-20 | `./scripts/parity.sh --group admin-fulfillments` |
| P17-18 | Admin shipping-options, profiles, and option-types | P17-17 | `./scripts/parity.sh --group admin-shipping` |
| P17-19 | Admin fulfillment-providers and stock-locations | P17-17 | `./scripts/parity.sh --group admin-locations` |
| P17-20 | Admin payments and payment-collections | P13-25 | `./scripts/parity.sh --group admin-payments` |
| P17-21 | Admin refund-reasons | P17-20 | `./scripts/parity.sh --group admin-refund-reasons` |
| P17-22 | Admin customers and customer-groups | P14-01 | `./scripts/parity.sh --group admin-customers` |
| P17-23 | Admin promotions and campaigns | P13-09 | `./scripts/parity.sh --group admin-marketing` |
| P17-24 | Admin regions, currencies, sales-channels, stores | P12-24 | `./scripts/parity.sh --group admin-config` |
| P17-25 | Admin tax regions, rates, and overrides | P12-24 | `./scripts/parity.sh --group admin-tax` |
| P17-26 | Admin locales and translations | P14-10 | `./scripts/parity.sh --group admin-i18n` |
| P17-27 | Admin users and invites | P14-04 | `./scripts/parity.sh --group admin-users` |
| P17-28 | Admin api-keys | P14-05 | `./scripts/parity.sh --group admin-api-keys` |
| P17-29 | Admin RBAC roles and policies | P14-06 | `./scripts/parity.sh --group admin-rbac` |
| P17-30 | Admin uploads and notifications | P14-07, P14-08 | `./scripts/parity.sh --group admin-platform` |
| P17-31 | Conditional mounting applied to every gateable group; tier matrix still passes at full surface | P17-30 | `cargo test -p stheno-api-router --test tier_matrix` |
| P17-32 | Admin list latency budgets at realistic volumes | P17-31 | `k6 run tests/load/admin.js --quiet` |

**Phase gate:** `./scripts/gate-p17.sh`

---

## P18 — Admin dashboard fork (28 tasks)

Track C, starting at P11. P11-02 must be `DONE` before anything here begins.

| ID | Task | Deps | Exit |
|---|---|---|---|
| P18-01 | Fork the pinned dashboard into `apps/admin`; record the real commit | P11-02 | `cd apps/admin && npm ci && npm run build` |
| P18-02 | Strip Enterprise paths per ADR-008; extend `check-no-enterprise-paths.sh` to the real tree | P18-01 | `./scripts/check-no-enterprise-paths.sh` |
| P18-03 | Point the SDK at `stheno-server`; environment configuration | P18-01 | `npm --prefix apps/admin run test:e2e -- --suite smoke` |
| P18-04 | Playwright harness against a live backend, replacing the removed placeholder runner | P18-03 | `npm --prefix apps/admin run test:e2e -- --suite harness` |
| P18-05 | Auth and session flow | P18-03 | `npm --prefix apps/admin run test:e2e -- --suite auth` |
| P18-06 | Product list and detail screens | P18-05, P17-03 | `npm --prefix apps/admin run test:e2e -- --suite products` |
| P18-07 | Product create and edit mutations | P18-06 | `npm --prefix apps/admin run test:e2e -- --suite product-mutations` |
| P18-08 | Variant and option management | P18-07, P17-04 | `npm --prefix apps/admin run test:e2e -- --suite variants` |
| P18-09 | Category tree and collections | P18-06, P17-06 | `npm --prefix apps/admin run test:e2e -- --suite taxonomy` |
| P18-10 | Inventory and reservations | P18-06, P17-07 | `npm --prefix apps/admin run test:e2e -- --suite inventory` |
| P18-11 | Price lists and the rule builder | P18-06, P17-08 | `npm --prefix apps/admin run test:e2e -- --suite pricing` |
| P18-12 | Order list and detail | P18-05, P17-10 | `npm --prefix apps/admin run test:e2e -- --suite orders` |
| P18-13 | Order edit flow with preview | P18-12, P17-12 | `npm --prefix apps/admin run test:e2e -- --suite order-edits` |
| P18-14 | Returns, claims, exchanges | P18-13, P17-15 | `npm --prefix apps/admin run test:e2e -- --suite returns` |
| P18-15 | Draft orders | P18-12, P17-16 | `npm --prefix apps/admin run test:e2e -- --suite draft-orders` |
| P18-16 | Fulfilment and shipping | P18-12, P17-18 | `npm --prefix apps/admin run test:e2e -- --suite fulfillment` |
| P18-17 | Customers and groups | P18-05, P17-22 | `npm --prefix apps/admin run test:e2e -- --suite customers` |
| P18-18 | Promotions and campaigns | P18-05, P17-23 | `npm --prefix apps/admin run test:e2e -- --suite marketing` |
| P18-19 | Settings: regions, currencies, channels, tax | P18-05, P17-24 | `npm --prefix apps/admin run test:e2e -- --suite settings` |
| P18-20 | **Clean-room RBAC screens** — rebuilt, not forked | P18-05, P17-29 | `npm --prefix apps/admin run test:e2e -- --suite rbac` |
| P18-21 | **Clean-room SSO screens** — Google, GitHub, OIDC | P18-20 | `npm --prefix apps/admin run test:e2e -- --suite sso` |
| P18-22 | Entitlements settings screen and over-quota grace banner | P18-05, P15-11 | `npm --prefix apps/admin run test:e2e -- --suite entitlements` |
| P18-23 | **Feature-flag hiding: a Free backend hides Pro screens with no dashboard code change** | P18-22 | `npm --prefix apps/admin run test:e2e -- --suite feature-flags` |
| P18-24 | Onboarding flow | P18-05 | `npm --prefix apps/admin run test:e2e -- --suite onboarding` |
| P18-25 | Spreadsheet import with dry-run preview | P18-06 | `npm --prefix apps/admin run test:e2e -- --suite import` |
| P18-26 | Social queue: draft orders, `Ref` matching, convert-to-order | P18-15 | `npm --prefix apps/admin run test:e2e -- --suite social-queue` |
| P18-27 | Analytics screens rendering the four aggregates | P18-05, P16-12 | `npm --prefix apps/admin run test:e2e -- --suite analytics` |
| P18-28 | WCAG 2.2 AA audit with a real axe run | P18-27 | `npm --prefix apps/admin run test:a11y` |

**Phase gate:** `./scripts/gate-p18.sh`

---

## P19 — Storefront completion and e2e (14 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P19-01 | Storefront pinned and building against the persisted backend | P13-24 | `npm --prefix apps/storefront ci && npm --prefix apps/storefront run build` |
| P19-02 | Playwright harness against a live backend | P19-01 | `npm --prefix apps/storefront run test:e2e -- --suite smoke` |
| P19-03 | Product listing and PDP render from real data | P19-02 | `npm --prefix apps/storefront run test:e2e -- --suite catalog` |
| P19-04 | Collections and categories browse | P19-03 | `npm --prefix apps/storefront run test:e2e -- --suite collections` |
| P19-05 | Cart add, update, remove | P19-03 | `npm --prefix apps/storefront run test:e2e -- --suite cart` |
| P19-06 | Full purchase journey against Stripe test mode | P19-05 | `npm --prefix apps/storefront run test:e2e -- --suite checkout` |
| P19-07 | Customer account: register, login, order history | P19-05, P14-01 | `npm --prefix apps/storefront run test:e2e -- --suite account` |
| P19-08 | **Order Now**: address form, COD-only payment section unless Pro | P19-05 | `npm --prefix apps/storefront run test:e2e -- --suite order-now` |
| P19-09 | Social checkout: WhatsApp `wa.me?text=` pre-fill | P19-08 | `npm --prefix apps/storefront run test:e2e -- --suite social-checkout` |
| P19-10 | Messenger and Instagram: clipboard then open app. `m.me` does not pre-fill; do not try | P19-09 | `npm --prefix apps/storefront run test:e2e -- --suite social-clipboard` |
| P19-11 | Source attribution: `?ref=` → cart metadata → order metadata | P19-08 | `npm --prefix apps/storefront run test:e2e -- --suite attribution` |
| P19-12 | End-to-end social path: storefront draft → admin queue → convert → **no double notification** | P19-09, P18-26 | `npm --prefix apps/storefront run test:e2e -- --suite social-roundtrip` |
| P19-13 | Lighthouse ≥ 95 and CWV gates on the PDP, measured | P19-03 | `npm --prefix apps/storefront run test:lighthouse` |
| P19-14 | Storefront returns and exchanges | P19-07, P17-13 | `npm --prefix apps/storefront run test:e2e -- --suite returns` |

**Phase gate:** `./scripts/gate-p19.sh`

---

## P20 — Hardening, load, and GA (18 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P20-01 | Install k6 and helm in CI; both gates currently fail on missing tooling | — | `./scripts/check-tooling.sh` |
| P20-02 | Encrypted backup and restore drill against the real database | P14-22 | `./scripts/restore-drill.sh` |
| P20-03 | Erasure replay verified inside the restore drill | P20-02, P14-21 | `./scripts/restore-drill.sh` |
| P20-04 | Distroless image built, size and cold start measured | P13-26 | `./scripts/check-image.sh` |
| P20-05 | Helm chart linted and templated in CI | P20-01, P20-04 | `helm lint deploy/chart && helm template deploy/chart` |
| P20-06 | Kubernetes deployment smoke test against the chart | P20-05 | `./scripts/check-deploy.sh` |
| P20-07 | Load suite: catalog read budget | P20-01, P12-17 | `k6 run tests/load/catalog.js --quiet` |
| P20-08 | Load suite: cart write budget | P20-01, P13-26 | `k6 run tests/load/cart.js --quiet` |
| P20-09 | Load suite: checkout budget | P20-08 | `k6 run tests/load/checkout.js --quiet` |
| P20-10 | Load suite: admin list budget | P20-01, P17-32 | `k6 run tests/load/admin.js --quiet` |
| P20-11 | Load suite: search budget | P20-01, P14-13 | `k6 run tests/load/search.js --quiet` |
| P20-12 | Load suite: reporting budget | P20-01, P16-14 | `k6 run tests/load/reporting.js --quiet` |
| P20-13 | **All seven latency budgets together on reference hardware** | P20-07..P20-12 | `k6 run tests/load/all.js --quiet` |
| P20-14 | Soak: 500 rps for one hour, RSS ≤ 400 MB, against Postgres | P20-13 | `./scripts/soak.sh --duration 1h --rps 500` |
| P20-15 | Redis outage degradation and recovery | P14-15 | `./scripts/redis-outage.sh` |
| P20-16 | Penetration test; no high or critical findings outstanding | P20-04 | Report filed in `docs/PEN_TEST.md`, findings closed |
| P20-17 | PCI SAQ A re-verified against the shipped checkout surface | P19-06 | `./scripts/check-pci-saq-a.sh` |
| P20-18 | `security.txt`, disclosure policy, SBOM regenerated for the release | P20-04 | `./scripts/check-disclosure.sh` |

**Phase gate:** `./scripts/gate-p20.sh`, then the full 1.0 checklist in
[PARITY_MATRIX §4](../PARITY_MATRIX.md).

---

## 3. Task summary

| Phase | Tasks | Weeks | Track |
|---|---:|---:|---|
| P11 | 22 | 5 | A |
| P12 | 24 | 5 | A |
| P13 | 26 | 6 | A |
| P14 | 22 | 4 | A |
| P15 | 12 | 2 | A |
| P16 | 14 | 3 | B |
| P17 | 32 | 8 | C |
| P18 | 28 | 6 | C |
| P19 | 14 | 3 | B |
| P20 | 18 | 4 | A+B+C |
| **Total** | **212** | **46** | |

---

## 4. The five tasks that decide the schedule

Everything else is execution. These are the ones where being wrong costs a month or more:

| Task | Why |
|---|---|
| **P11-12** | `FilterSpec` viability. Specified in P0, never exercised, now load-bearing for 25 modules. PRD R3 coming due |
| **P11-04** | `Id<T>` representation. Decide once, or encode a table-scan workaround into 25 migrations |
| **P13-02** | Durable `WorkflowStore`. ADR-003's entire premise, and it has never been durable |
| **P16-04** | Rollup invalidation. Get it wrong and every report silently lies |
| **P11-02** | Dashboard upstream pin. Half a day, unblocks six weeks, already got wrong once |

---

## 5. What this decomposition deliberately does not do

- **It does not re-estimate Phase 1.** Those numbers describe a program that ran without a
  database; they are not a baseline for anything here.
- **It does not assume the Phase 1 latency budgets survive.** Every persistence phase re-measures.
  The gate is that numbers are measured and committed, not that they are unchanged.
- **It does not parallelise P17 across engineers.** 240 routes against one parity corpus conflicts
  more than it gains.
- **It does not add features.** Every task closes a gap between the specs and the code that
  already exists. Anything genuinely new — courier dispatch, fraud scoring — is post-1.0 and
  named as such in ENTITLEMENTS.md.
