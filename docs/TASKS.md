# Task Decomposition

**282 tasks across 11 phases.** Each is 2 hours to 2 days, has explicit dependencies, and ends in
a **command that exits 0** — never a judgement call.

[IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) says what each phase achieves and why. This
document is what an agent or engineer actually picks up.

---

## 1. The loop protocol

```
  ┌─────────────────────────────────────────────────┐
  │ 1. Pick the lowest-numbered task whose deps are  │
  │    all DONE                                      │
  │ 2. Read its spec references. Walk the ladder.    │
  │ 3. Implement                                     │
  │ 4. Run: the universal gate + the task exit cmd   │
  │ 5. Exit 0?  ── yes ──► mark DONE, go to 1        │
  │       │                                          │
  │       no                                         │
  │       ▼                                          │
  │ 6. Same failure as last attempt?                 │
  │       no  ──► fix, go to 4                       │
  │       yes ──► STOP. The spec is wrong. Escalate. │
  └─────────────────────────────────────────────────┘
```

### 1.1 Rules that make the loop terminate

These are what stop "it keeps developing" from becoming "it never stops."

| # | Rule |
|---|---|
| **L1** | **One task in progress at a time.** Parallelism happens across tracks, not within a worker |
| **L2** | **The exit command is the definition of done.** Not "looks right," not "tests mostly pass" |
| **L3** | **Never edit the exit command to make it pass.** Changing the gate is a spec change and needs sign-off. This is the single most important rule here |
| **L4** | **Two identical consecutive failures = stop.** The same error twice means the approach or the spec is wrong. Looping a third time burns budget on a wrong premise |
| **L5** | **Five attempts on one task = stop**, even if the failures differ |
| **L6** | **Blocked ≠ failed.** Missing dependency, ambiguous spec, or a decision that is not yours → escalate immediately. Do not guess and proceed |
| **L7** | **Never mark DONE without running the command** and seeing exit 0 |
| **L8** | A task that reveals the spec is wrong **updates the spec first**, in its own commit, then implements |

### 1.2 The universal gate

Runs on **every** task before its specific exit command. A task is not done if this fails:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-module-isolation.sh     # no lateral or upward module edges
./scripts/check-sql-boundaries.sh       # no cross-schema raw SQL
./scripts/check-shared-kernel.sh        # stheno-types stays starved
./scripts/check-module-contracts.sh     # MODULE.toml matches migrations and code
./scripts/check-dto-separation.sh
./scripts/check-schema-drift.sh
```

### 1.3 Escalation format

When L4, L5, or L6 trips, stop and report exactly this — no more:

```
BLOCKED  <task-id>
attempts: <n>
command:  <exit command>
output:   <verbatim last 20 lines>
read:     <what you believe is wrong in the spec, one paragraph>
proposal: <the smallest spec change that would unblock it>
```

### 1.4 Status values

`TODO` · `IN PROGRESS` · `DONE` · `BLOCKED` · `SPEC-CHANGE` (paused pending a doc update)

---

## 2. Reading a task row

| Field | Meaning |
|---|---|
| **ID** | Stable. Never renumber — other tasks and commits reference it |
| **Task** | One sentence, one deliverable |
| **Deps** | Must be `DONE` first. `—` means it can start immediately |
| **Exit** | The command that decides. Assume repo root |

Where an exit command names a test that does not exist yet, **writing that test is part of the
task.** That is deliberate: it forces the acceptance criterion to be written before the code.

---

## P0 — Foundations and harnesses (27 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P0-01 | Cargo workspace, shared `[workspace.dependencies]`, empty crate skeletons | — | `cargo build --workspace` |
| P0-02 | CI pipeline: fmt, clippy `-D warnings`, test, `cargo deny`, `cargo audit` | P0-01 | CI green on a trivial PR |
| P0-03 | `stheno-types`: `Money` with `Decimal`, no `From<f64>`, currency-aware ops | P0-01 | `cargo test -p stheno-types money` |
| P0-04 | `Money` largest-remainder allocation + proptest across all 22 special-precision currencies | P0-03 | `cargo test -p stheno-types --features proptest allocation` |
| P0-05 | `to_minor_units` / `from_minor_units` incl. zero- and three-decimal rules ([PAYMENTS §5](PAYMENTS.md)) | P0-03 | `cargo test -p stheno-types minor_units` |
| P0-06 | `Id<T>`: prefixed ULID, typed per entity, serde round-trip | P0-01 | `cargo test -p stheno-types ids` |
| P0-07 | `stheno-db`: deadpool pool, config, health check | P0-01 | `cargo test -p stheno-db pool` |
| P0-08 | Transaction manager with explicit `Conn<'_>`, `SERIALIZABLE` retry helper | P0-07 | `cargo test -p stheno-db transaction` |
| P0-09 | `diesel_migrations` embedded; `stheno migrate` under advisory lock; **creates one Postgres schema per module** | P0-07 | `cargo run -p stheno-cli -- migrate && cargo run -p stheno-cli -- migrate` |
| P0-10 | Migration phase linter — reject unpaired destructive statements | P0-09 | `./scripts/check-migration-phases.sh` |
| P0-11 | **`FilterSpec` derive macro** → `BoxedSelectStatement`, incl. subquery relation filters | P0-08 | `cargo test -p stheno-db filterspec` |
| P0-12 | **Validate `FilterSpec` against the 20 hardest endpoints** ([TRD §5.3](TRD.md)). Go/no-go on Diesel | P0-11 | `cargo test -p stheno-db filterspec_hard_cases` |
| P0-13 | Filter limits: `$in` ≤ 100, depth ≤ 3, ≤ 20 predicates, trailing-wildcard only | P0-11 | `cargo test -p stheno-db filter_limits` |
| P0-14 | `stheno-http`: axum app, Medusa error renderer, `garde` extractor, offset pagination | P0-03 | `cargo test -p stheno-http` |
| P0-15 | Middleware stack: trace, request-id, CORS allowlist, body limits, security headers | P0-14 | `cargo test -p stheno-http middleware` |
| P0-16 | **CSRF middleware** — double-submit + Origin + custom header ([SECURITY §6](SECURITY.md)) | P0-15 | `cargo test -p stheno-http csrf` |
| P0-17 | Rate limiting with `TRUSTED_PROXIES` resolution, per-IP and per-actor buckets | P0-15 | `cargo test -p stheno-http rate_limit` |
| P0-18 | `Owned<T, Scope>` extractor + `check-dto-separation.sh` CI check | P0-14 | `cargo test -p stheno-http owned && ./scripts/check-dto-separation.sh` |
| P0-19 | Observability: `tracing`, OTLP, RED metrics, redaction layer | P0-14 | `cargo test -p stheno-http redaction` |
| P0-20 | **Parity harness**: recording proxy, seeded Medusa, fixture store, replay/diff runner | P0-14 | `./scripts/parity.sh --record --replay --smoke` |
| P0-21 | **Load harness** + measured Medusa baselines on reference hardware, committed | P0-20 | `k6 run tests/load/baseline.js && test -f tests/load/BASELINES.md` |
| P0-22 | **Boundary enforcement scripts**: `check-module-isolation.sh` (lateral + upward edges), `check-shared-kernel.sh` | P0-01 | `./scripts/check-module-isolation.sh && ./scripts/check-shared-kernel.sh` |
| P0-23 | `MODULE.toml` format + `check-module-contracts.sh` — declared tables/events/links match reality | P0-09 | `./scripts/check-module-contracts.sh` |
| P0-24 | `check-sql-boundaries.sh` — scan every `sql_query` literal for foreign schema prefixes | P0-09 | `./scripts/check-sql-boundaries.sh` |
| P0-26 | `Entitlements` type: `config/tiers.toml` loader (trial/free/pro), env overrides, fail-closed validation at boot | P0-01 | `cargo test -p stheno-types entitlements` |
| P0-27 | `check-no-domain-gates.sh` — no `Entitlements` reference under `crates/modules/` | P0-26 | `./scripts/check-no-domain-gates.sh` |
| P0-25 | **Negative tests** for all five boundary scripts — each must fail on a deliberately planted violation | P0-22, P0-23, P0-24, P0-27 | `./scripts/test-boundary-checks.sh` |

**Phase gate:** `./scripts/gate-p0.sh` — universal gate, plus a two-table demo module round-tripping through HTTP with filtering, pagination, validation, CSRF, and both error paths. **P0-25 must prove every boundary check actually fails on a planted violation** — an enforcement script that has never failed is not enforcement.

---

## P1 — Platform (25 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P1-01 | `audit_log` table, append-only grants, writer API | P0-09 | `cargo test -p stheno-audit append_only` |
| P1-02 | `workflow_execution` schema + migration + claim index | P0-09 | `cargo test -p stheno-workflow schema` |
| P1-03 | `Step` trait, `StepConfig` with all 15 options | P0-03 | `cargo test -p stheno-workflow step_config` |
| P1-04 | Transaction + step state machines, monotonic transitions | P1-03 | `cargo test -p stheno-workflow state_machine` |
| P1-05 | `workflow!` macro → typed DAG; producer/consumer type mismatch fails compile | P1-03 | `cargo test -p stheno-workflow macro && cargo test -p stheno-workflow --test compile_fail` |
| P1-06 | Sequential + parallel execution, checkpoint per step boundary | P1-04, P1-02 | `cargo test -p stheno-workflow execute` |
| P1-07 | Compensation stack, reverse-order unwind | P1-06 | `cargo test -p stheno-workflow compensate` |
| P1-08 | **Compensation completeness**: inject failure at every step position | P1-07 | `cargo test -p stheno-workflow compensation_matrix` |
| P1-09 | Retries, backoff, `max_retries`, `retry_interval` | P1-06 | `cargo test -p stheno-workflow retry` |
| P1-10 | Step and workflow timeouts, incl. revert-on-late-response | P1-06 | `cargo test -p stheno-workflow timeout` |
| P1-11 | Async steps, `set_step_success` / `set_step_failure`, `background_execution` | P1-06 | `cargo test -p stheno-workflow async_steps` |
| P1-12 | `continue_on_permanent_failure`, `skip_on_permanent_failure`, `no_wait`, `nested` | P1-06 | `cargo test -p stheno-workflow failure_modes` |
| P1-13 | `SKIP LOCKED` claim loop, lock renewal, expiry recovery | P1-06 | `cargo test -p stheno-workflow claim` |
| P1-14 | **Kill-and-resume harness**: SIGKILL at every step boundary, 100 runs | P1-13 | `./scripts/kill-resume.sh --runs 100` |
| P1-15 | Idempotency by `transaction_id`; 100 concurrent starts → 1 execution | P1-13 | `cargo test -p stheno-workflow idempotency_concurrent` |
| P1-16 | Workflow context retention window + PII redaction hooks ([SECURITY §11.1](SECURITY.md)) | P1-02 | `cargo test -p stheno-workflow retention` |
| P1-17 | `stheno-links`: `LinkDefinition`, all 36 declared, link CRUD steps | P0-09 | `cargo test -p stheno-links registry` |
| P1-18 | `stheno-query`: field-selection parser, depth/node limits | P0-14 | `cargo test -p stheno-query parser` |
| P1-19 | Query planner → `FetchNode` tree using the link registry | P1-17, P1-18 | `cargo test -p stheno-query planner` |
| P1-20 | Batched level executor + hash joins; **asserted query count, no N+1** | P1-19 | `cargo test -p stheno-query no_n_plus_one` |
| P1-21 | `stheno-events`: trait, in-process + Redis, transactional buffering, dead-letter. **No event types in core** | P0-07 | `cargo test -p stheno-events` |
| P1-21b | `crates/subscribers/` crate + registration; CI check that no module declares a subscriber | P1-21 | `cargo test -p stheno-subscribers && ./scripts/check-module-isolation.sh` |
| P1-22 | Scheduler: cron parsing, advisory-lock single execution | P1-13 | `cargo test -p stheno-workflow scheduler` |
| P1-23 | **Order-change engine, standalone**: apply/preview, fixture corpus from Medusa | P0-09 | `cargo test -p stheno-order-change corpus` |
| P1-24 | `AppContext` + composition root in `stheno-server` | P1-21 | `cargo run -p stheno-server -- --check-config` |

**Phase gate:** `./scripts/gate-p1.sh` — P1-08, P1-14, P1-15, P1-20, P1-23 all green.

---

## P2 — Catalog and inventory (27 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P2-01 | `stheno-product` crate + 12 entity models + migration + `schema.rs` | P1-24 | `cargo test -p stheno-product models` |
| P2-02 | `SthenoService` derive macro (list/retrieve/create/update/delete/soft-delete/restore) | P1-24 | `cargo test -p stheno-db service_derive` |
| P2-03 | `ProductService` + hand-written domain methods | P2-01, P2-02 | `cargo test -p stheno-product service` |
| P2-04 | Variant ⇄ option-value identity, unique index on sorted option-value hash | P2-01 | `cargo test -p stheno-product variant_identity` |
| P2-05 | Concurrent variant creation does not duplicate | P2-04 | `cargo test -p stheno-product variant_identity_concurrent` |
| P2-06 | Category tree: `mpath`, ranking, ancestor/descendant queries | P2-01 | `cargo test -p stheno-product categories` |
| P2-07 | Collections, tags, types, options services | P2-02 | `cargo test -p stheno-product taxonomy` |
| P2-08 | Product/variant images, per-variant assignment | P2-01 | `cargo test -p stheno-product images` |
| P2-09 | `FilterSpec` for products, variants, categories | P0-11, P2-01 | `cargo test -p stheno-product filters` |
| P2-10 | Product domain events | P1-21, P2-03 | `cargo test -p stheno-product events` |
| P2-11 | `stheno-inventory`: 3 entities, service | P1-24 | `cargo test -p stheno-inventory` |
| P2-12 | Reservations, release on cancel/fulfil | P2-11 | `cargo test -p stheno-inventory reservations` |
| P2-13 | **Oversell test**: 100 concurrent reservations vs 1 unit → exactly 1 wins | P2-12 | `cargo test -p stheno-inventory no_oversell` |
| P2-14 | `stheno-stock-location`, `stheno-sales-channel` | P1-24 | `cargo test -p stheno-stock-location -p stheno-sales-channel` |
| P2-15 | Links: product↔channel, variant↔inventory-item, channel↔location | P1-17, P2-14 | `cargo test -p stheno-links catalog_links` |
| P2-16 | 27 product + 3 category workflows | P1-05, P2-03 | `cargo test -p stheno-flows product` |
| P2-17 | 4 inventory + 3 stock-location + 3 sales-channel workflows | P1-05, P2-11 | `cargo test -p stheno-flows inventory` |
| P2-18 | Batch product ops: 1,000 products, per-row errors, one workflow | P2-16 | `cargo test -p stheno-flows batch_products` |
| P2-19 | Store catalog endpoints (products, variants, categories, collections, tags, types, options) | P2-09, P0-14 | `./scripts/parity.sh --group store-catalog` |
| P2-20 | Upload pipeline: allowlist, magic-byte sniffing, re-encode, no SVG, separate origin | P0-14 | `cargo test -p stheno-file upload_security` |
| P2-21 | Egress guard: scheme, resolved-IP denylist incl. 169.254.0.0/16, redirect re-check | P0-14 | `cargo test -p stheno-http egress_guard` |
| P2-21b | `entitlement_usage` counter table + atomic `CheckQuotaStep` workflow step | P1-05, P0-26 | `cargo test -p stheno-flows-common quota_step` |
| P2-21c | Quota concurrency: 100 parallel creates against a limit of 10 → exactly 10 succeed | P2-21b | `cargo test -p stheno-flows-common quota_concurrent` |
| P2-21d | `entitlement_state` table + daily job: trial→free at day 30, day-23/29 warnings, audit-logged transitions | P2-21b, P1-22 | `cargo test -p stheno-flows-common trial_lifecycle` |
| P2-21e | **Grandfathering test**: merchant with 412 products downgraded to a 100 limit keeps all 412 live; only creation blocked | P2-21d | `cargo test -p stheno-flows-common grandfathering` |
| P2-21f | Window gates: `analytics_window_days` bounds reporting queries | P0-26 | `cargo test -p stheno-flows-common window_gate` |
| P2-22 | `stheno new-module` scaffold CLI — emits crate, `MODULE.toml`, schema migration, service, dto, events, tests, registry entry | P2-01, P0-23 | `cargo run -p stheno-cli -- new-module demo && cargo build -p stheno-demo && ./scripts/check-module-contracts.sh` |

**Phase gate:** parity green for store-catalog; P2-13 green; `GET /store/products` p95 ≤ 25 ms.

---

## P3 — Pricing, tax, region, currency (17 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P3-01 | `stheno-currency` + ISO 4217 precision table | P1-24 | `cargo test -p stheno-currency` |
| P3-02 | `stheno-region`: region, country, service | P1-24 | `cargo test -p stheno-region` |
| P3-03 | `stheno-store`: store, currency, locale | P1-24 | `cargo test -p stheno-store` |
| P3-04 | `stheno-pricing`: 6 entities + migration | P1-24 | `cargo test -p stheno-pricing models` |
| P3-05 | Price rules, `rules_count`, specificity ranking | P3-04 | `cargo test -p stheno-pricing rules` |
| P3-06 | Quantity tiers (`min_quantity`/`max_quantity`) | P3-04 | `cargo test -p stheno-pricing tiers` |
| P3-07 | Price lists: sale/override, date windows, list rules | P3-04 | `cargo test -p stheno-pricing price_lists` |
| P3-08 | Price preferences → tax-inclusive resolution | P3-04 | `cargo test -p stheno-pricing preferences` |
| P3-09 | **`calculated_price` as ONE CTE query**, ranked in SQL | P3-05, P3-07 | `cargo test -p stheno-pricing calculated_price` |
| P3-10 | **Query-count assertion**: 100 variants × 3 regions × 2 groups = 1 query | P3-09 | `cargo test -p stheno-pricing single_query` |
| P3-11 | Pricing fixture corpus vs Medusa (tiers, overlaps, list precedence, ties) | P3-09 | `cargo test -p stheno-pricing corpus` |
| P3-12 | `stheno-tax`: 4 entities, hierarchical regions | P1-24 | `cargo test -p stheno-tax models` |
| P3-13 | Rate resolution: rules, combinable, overrides | P3-12 | `cargo test -p stheno-tax resolution` |
| P3-14 | **Tax-inclusive reverse computation, exact**, proptest at every precision | P3-13, P0-04 | `cargo test -p stheno-tax inclusive_exact` |
| P3-15 | `TaxProvider` trait + system provider | P3-13 | `cargo test -p stheno-tax provider` |
| P3-16 | Links: variant↔price-set, shipping-option↔price-set, region↔payment-provider | P1-17, P3-04 | `cargo test -p stheno-links pricing_links` |
| P3-17 | 10 pricing + 9 tax + 3 region + 3 store workflows | P1-05, P3-09 | `cargo test -p stheno-flows pricing tax region store` |
| P3-18 | Store endpoints: regions, currencies; products with prices | P3-09, P2-19 | `./scripts/parity.sh --group store-pricing` |
| P3-19 | **Checkout threat-modelling session** ([THREAT_MODEL_CHECKOUT.md](THREAT_MODEL_CHECKOUT.md)) | P3-18 | Findings appended to SECURITY.md as F30+ with owners |

**Phase gate:** P3-10, P3-11, P3-14 green; product list with pricing still ≤ 25 ms p95.

---

## P4 — Cart, promotion, payment (31 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P4-01 | `stheno-cart`: 9 entities + migration | P3-18 | `cargo test -p stheno-cart models` |
| P4-02 | **Totals engine as a pure function** over a cart snapshot | P4-01, P0-03 | `cargo test -p stheno-cart totals` |
| P4-03 | Totals proptest: line sums equal order totals exactly, every precision | P4-02 | `cargo test -p stheno-cart totals_proptest` |
| P4-04 | Line items, adjustments, tax lines; denormalised product fields | P4-01 | `cargo test -p stheno-cart line_items` |
| P4-05 | Credit lines (gift card / store credit) | P4-01 | `cargo test -p stheno-cart credit_lines` |
| P4-06 | **Cart ownership**: `cart_secret`, cookie binding, claim-on-login (F6) | P4-01 | `cargo test -p stheno-cart ownership` |
| P4-07 | Cart TTL job + creation rate limit (F22) | P4-06 | `cargo test -p stheno-cart ttl` |
| P4-08 | `stheno-promotion`: 7 entities + migration | P3-18 | `cargo test -p stheno-promotion models` |
| P4-09 | Rule engine, 8 operators, cart/item/customer context | P4-08 | `cargo test -p stheno-promotion rules` |
| P4-10 | Application methods: fixed/percentage × each/across × 3 targets | P4-09 | `cargo test -p stheno-promotion application` |
| P4-11 | Buyget: buy-rules, target-rules, `apply_to_quantity` | P4-10 | `cargo test -p stheno-promotion buyget` |
| P4-12 | **Deterministic stacking order**, documented + fixture-tested | P4-10 | `cargo test -p stheno-promotion stacking` |
| P4-13 | Campaigns + **atomic budget consumption** | P4-08 | `cargo test -p stheno-promotion budget` |
| P4-14 | **Budget concurrency**: 100 orders vs 50-use budget → exactly 50 | P4-13 | `cargo test -p stheno-promotion budget_concurrent` |
| P4-15 | Promo-code abuse limits: per-cart, per-IP, constant-time compare (F10) | P4-09 | `cargo test -p stheno-promotion abuse_limits` |
| P4-16 | `stheno-payment`: 8 entities + migration | P3-18 | `cargo test -p stheno-payment models` |
| P4-17 | **`payment_provider_request`** table + `idempotent_call()` ([PAYMENTS §6](PAYMENTS.md)) | P4-16 | `cargo test -p stheno-payment outbound_idempotency` |
| P4-18 | **Ambiguity resolution**: `unknown` state → `retrieve()` reconciliation | P4-17 | `cargo test -p stheno-payment ambiguous_timeout` |
| P4-19 | **`payment_webhook_event`** store + replay guard + retention | P4-16 | `cargo test -p stheno-payment webhook_store` |
| P4-20 | **Convergence reducer**: monotonic transitions, provider-timestamp ordering | P4-19 | `cargo test -p stheno-payment convergence` |
| P4-21 | **Ledger invariants** as DB constraints; capture/refund under row lock | P4-16 | `cargo test -p stheno-payment ledger_invariants` |
| P4-22 | Ledger concurrency: 100 parallel captures/refunds hold invariants | P4-21 | `cargo test -p stheno-payment ledger_concurrent` |
| P4-23 | `PaymentProvider` + `PaymentVault` traits (18 methods) + capabilities | P4-16 | `cargo test -p stheno-payment traits` |
| P4-23b | **COD / offline payment provider** — order placed unpaid, fulfil before payment, manual mark-as-paid, no gateway refund path | P4-23 | `cargo test -p stheno-payment-offline` |
| P4-24 | Stripe provider on `reqwest`: ~20 endpoints, wiremock fixtures. **Gated: Pro only** | P4-23, P4-17 | `cargo test -p stheno-payment-stripe` |
| P4-25 | Webhook endpoint: raw body, HMAC constant-time, 5-min tolerance, livemode, pre-verify rate limit | P4-19, P4-24 | `cargo test -p stheno-payment-stripe webhooks` |
| P4-26 | **Edge atlas E1–E34** ([PAYMENTS §11](PAYMENTS.md)) as a test suite | P4-25 | `cargo test -p stheno-payment --test edge_atlas` |
| P4-27 | Card-testing abuse limits on payment-session creation | P4-25 | `cargo test -p stheno-payment card_testing` |
| P4-28 | 15 cart + 11 promotion + 10 payment + 1 line-item workflows | P1-05, P4-24 | `cargo test -p stheno-flows cart promotion payment` |
| P4-29 | Store endpoints: carts, line items, promotions, payment collections/providers | P4-28 | `./scripts/parity.sh --group store-cart` |
| P4-30 | Checkout script manifest + CI enforcement (PCI, F25) | — | `./scripts/check-checkout-scripts.sh` |

**Phase gate:** P4-03, P4-14, P4-22, P4-26 green; line-item mutation ≤ 30 ms p95; Stripe test-mode auth→capture→partial refund→3DS resume.

---

## P5 — Order, fulfillment, sales channel (22 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P5-01 | `stheno-order`: 23 entities + migration | P4-29 | `cargo test -p stheno-order models` |
| P5-02 | Wire the P1 order-change engine into the order module | P5-01, P1-23 | `cargo test -p stheno-order change_integration` |
| P5-03 | Versioned `order_item`, snapshot load at version N | P5-01 | `cargo test -p stheno-order versioning` |
| P5-04 | `order_summary` computation from transactions + items | P5-03 | `cargo test -p stheno-order summary` |
| P5-05 | `transaction` ledger; payment/fulfillment status independence | P5-01 | `cargo test -p stheno-order ledger` |
| P5-06 | **Sequential gap-free `display_id`** under 50-way concurrency | P5-01 | `cargo test -p stheno-order display_id_concurrent` |
| P5-07 | `stheno-fulfillment`: 12 entities + migration | P4-29 | `cargo test -p stheno-fulfillment models` |
| P5-08 | Fulfillment sets, service zones, geo zones (country/province/city/zip) | P5-07 | `cargo test -p stheno-fulfillment zones` |
| P5-09 | Shipping options, rules, types, profiles | P5-08 | `cargo test -p stheno-fulfillment shipping_options` |
| P5-10 | Eligible-option resolution by cart, address, channel | P5-09, P4-01 | `cargo test -p stheno-fulfillment eligibility` |
| P5-11 | `FulfillmentProvider` trait + manual provider | P5-07 | `cargo test -p stheno-fulfillment-manual` |
| P5-12 | Calculated shipping pricing through the provider | P5-11, P3-09 | `cargo test -p stheno-fulfillment calculated` |
| P5-13 | Fulfillment creation, items, labels, cancel; pickup options | P5-11 | `cargo test -p stheno-fulfillment lifecycle` |
| P5-14 | Order links: cart, payment-collection, promotion, fulfillment | P1-17, P5-01 | `cargo test -p stheno-links order_links` |
| P5-15 | **`complete-cart` workflow, all 11 steps** ([WORKFLOWS §2.3](WORKFLOWS.md)) | P5-02, P4-28 | `cargo test -p stheno-flows complete_cart` |
| P5-16 | Compensation boundaries: step 11 `no_compensation` verified | P5-15 | `cargo test -p stheno-flows complete_cart_compensation` |
| P5-17 | **Crash resumption at each of the 11 steps** | P5-15, P1-14 | `./scripts/kill-resume.sh --workflow complete-cart --all-steps` |
| P5-18 | **No double charge**: duplicate complete → same order, verified vs Stripe test mode | P5-15, P4-18 | `cargo test -p stheno-flows complete_cart_idempotent` |
| P5-19 | Order lifecycle workflows: create, cancel, archive, complete, update | P5-02 | `cargo test -p stheno-flows order_lifecycle` |
| P5-20 | 8 fulfillment workflows | P5-13 | `cargo test -p stheno-flows fulfillment` |
| P5-21 | Order transfer: request/accept/decline/cancel | P5-02 | `cargo test -p stheno-flows order_transfer` |
| P5-22 | Store endpoints: orders, transfer, shipping options, returns | P5-19 | `./scripts/parity.sh --group store-order` |

**Phase gate:** guest checkout end-to-end; P5-17, P5-18, P5-06 green; `complete-cart` ≤ 250 ms p95.

---

## P6 — Order lifecycle (20 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P6-01 | Order edit: begin, add/update/remove items | P5-22 | `cargo test -p stheno-flows order_edit_items` |
| P6-02 | Order edit: add/update/remove shipping | P6-01 | `cargo test -p stheno-flows order_edit_shipping` |
| P6-03 | **Edit preview — computed, zero writes** | P6-01 | `cargo test -p stheno-flows order_edit_preview_no_writes` |
| P6-04 | Edit confirm: version bump, summary recompute, payment delta | P6-03 | `cargo test -p stheno-flows order_edit_confirm` |
| P6-05 | Edit delta collects on increase, refunds on decrease | P6-04, P4-21 | `cargo test -p stheno-flows order_edit_delta` |
| P6-06 | Edit cancel/discard leaves version untouched | P6-01 | `cargo test -p stheno-flows order_edit_cancel` |
| P6-07 | Return: request, add/update/remove items | P5-22 | `cargo test -p stheno-flows return_request` |
| P6-08 | Return: begin receive, receive items, confirm, cancel | P6-07 | `cargo test -p stheno-flows return_receive` |
| P6-09 | **Partial receipt** with damaged quantities → correct refund + restock | P6-08 | `cargo test -p stheno-flows return_partial` |
| P6-10 | Return dismiss / write-off | P6-08 | `cargo test -p stheno-flows return_dismiss` |
| P6-11 | Claim: begin, inbound/outbound items, evidence images | P5-22 | `cargo test -p stheno-flows claim_items` |
| P6-12 | Claim confirm: replace and refund kinds | P6-11 | `cargo test -p stheno-flows claim_confirm` |
| P6-13 | Exchange: begin, inbound/outbound items | P5-22 | `cargo test -p stheno-flows exchange_items` |
| P6-14 | Exchange confirm with computed difference | P6-13 | `cargo test -p stheno-flows exchange_confirm` |
| P6-15 | Draft orders: create, edit, 9 workflows | P5-22 | `cargo test -p stheno-flows draft_order` |
| P6-16 | Draft → live order conversion | P6-15 | `cargo test -p stheno-flows draft_convert` |
| P6-17 | Order change cancel/decline for every subtype | P6-04 | `cargo test -p stheno-flows change_control` |
| P6-18 | **Timeline reconstruction** from `order_change_action` | P6-17 | `cargo test -p stheno-order timeline` |
| P6-19 | **Medusa fixture corpus** for all four mutation families | P6-14, P6-16 | `cargo test -p stheno-flows order_corpus` |
| P6-20 | Store endpoints: returns | P6-08 | `./scripts/parity.sh --group store-returns` |

**Phase gate:** P6-03, P6-09, P6-19 green; every mutation previewable with a correct delta.

---

## P7 — Auth, user, customer, API keys, RBAC (19 tasks · Track B)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P7-01 | `stheno-auth`: 6 entities + migration | P1-24 | `cargo test -p stheno-auth models` |
| P7-02 | Argon2id hashing + **scrypt verification for imports** + transparent re-hash | P7-01 | `cargo test -p stheno-auth hashing` |
| P7-03 | emailpass provider | P7-02 | `cargo test -p stheno-auth-emailpass` |
| P7-04 | Google + GitHub OAuth providers | P7-01 | `cargo test -p stheno-auth-google -p stheno-auth-github` |
| P7-05 | **OIDC provider, clean-room from the spec** ([ADR-008](DECISIONS.md#adr-008)) | P7-01 | `cargo test -p stheno-auth-oidc` |
| P7-06 | JWT issue/verify with `actor_id`, `actor_type`, `token_version` | P7-01 | `cargo test -p stheno-auth jwt` |
| P7-07 | Refresh rotation with **reuse detection** invalidating the family | P7-06 | `cargo test -p stheno-auth refresh_reuse` |
| P7-08 | `token_version` invalidation on password/MFA/email change (F13) | P7-06 | `cargo test -p stheno-auth invalidation` |
| P7-09 | MFA: TOTP factors, recovery codes, challenge/verify, encrypted at rest | P7-06 | `cargo test -p stheno-auth mfa` |
| P7-10 | Password reset: single-use, expiring, hashed tokens | P7-02 | `cargo test -p stheno-auth reset` |
| P7-11 | **Anti-enumeration**: identical responses and timing (F14) | P7-03, P7-10 | `cargo test -p stheno-auth no_enumeration` |
| P7-12 | Breached-password check via k-anonymity range query | P7-02 | `cargo test -p stheno-auth breached` |
| P7-13 | Auth brute-force limits and lockout | P7-03 | `cargo test -p stheno-auth lockout` |
| P7-14 | `stheno-customer`: 4 entities, groups, guest→registered upgrade | P1-24 | `cargo test -p stheno-customer` |
| P7-15 | `stheno-user`: user, invite | P1-24 | `cargo test -p stheno-user` |
| P7-16 | `stheno-api-key`: publishable + secret, hashed, **scopes** (F20) | P1-24 | `cargo test -p stheno-api-key` |
| P7-17 | Publishable-key → sales-channel scope middleware; out-of-scope → 404 | P7-16, P2-14 | `cargo test -p stheno-http scope_resolution` |
| P7-18 | **`stheno-rbac` clean-room**: roles, policies, inheritance closure, deny-beats-allow | P1-24 | `cargo test -p stheno-rbac` |
| P7-19 | MFA required for sensitive permissions; audit-logged | P7-18, P1-01 | `cargo test -p stheno-rbac sensitive` |
| P7-20 | **Authorization matrix test**: every role × endpoint × ownership | P7-18 | `cargo test --test authz_matrix` |
| P7-21 | 39 auth/user/customer/invite/api-key/rbac workflows | P1-05, P7-18 | `cargo test -p stheno-flows auth customer user invite api_key rbac` |
| P7-22 | Auth endpoints, Medusa-compatible shape | P7-21 | `./scripts/parity.sh --group auth` |

**Phase gate:** P7-07, P7-11, P7-17, P7-18, P7-20 green; imported scrypt hash authenticates then re-hashes.

---

## P8 — Platform services (27 tasks · Track B)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P8-01 | `stheno-file` + local provider | P2-20 | `cargo test -p stheno-file-local` |
| P8-02 | S3 provider + presigned uploads | P8-01, P2-21 | `cargo test -p stheno-file-s3` |
| P8-03 | `stheno-notification` + templates + delivery records | P1-24 | `cargo test -p stheno-notification` |
| P8-04 | Local + SendGrid notification providers | P8-03 | `cargo test -p stheno-notification-local -p stheno-notification-sendgrid` |
| P8-04b | **Web push provider** (VAPID) — zero marginal cost, no app. Primary owner-alert channel | P8-03 | `cargo test -p stheno-notification-webpush` |
| P8-05 | `stheno-caching` trait + in-process provider | P1-24 | `cargo test -p stheno-caching` |
| P8-06 | Redis cache provider | P8-05 | `cargo test -p stheno-caching-redis` |
| P8-07 | `stheno-locking` + Postgres and Redis providers | P1-24 | `cargo test -p stheno-locking` |
| P8-08 | **Redis-outage degradation**: kill Redis, zero failed requests single-node | P8-06, P8-07 | `./scripts/redis-outage.sh` |
| P8-09 | `stheno-search` trait + Postgres full-text provider | P1-24 | `cargo test -p stheno-search-local` |
| P8-10 | Event-driven search index sync | P8-09, P1-21 | `cargo test -p stheno-search sync` |
| P8-11 | Search convergence ≤ 5 s after 1,000-product bulk update | P8-10 | `cargo test -p stheno-search convergence` |
| P8-12 | `stheno-index`: 4 entities, denormalised projection | P1-24 | `cargo test -p stheno-index models` |
| P8-13 | Incremental index update + full rebuild + lag metric | P8-12, P1-21 | `cargo test -p stheno-index sync` |
| P8-14 | Cross-module filter/sort resolving via `index_data` | P8-13, P1-19 | `cargo test -p stheno-index cross_module_filter` |
| P8-15 | `stheno-translation`: 3 entities, locale-scoped field overrides | P1-24 | `cargo test -p stheno-translation` |
| P8-16 | `stheno-settings`: view configs, layouts, property labels, preferences | P1-24 | `cargo test -p stheno-settings` |
| P8-17 | `stheno-analytics` + local and PostHog providers | P1-24 | `cargo test -p stheno-analytics` |
| P8-18 | **Erasure pipeline**: pseudonymisation cascade per [SECURITY §11.2](SECURITY.md) | P8-13, P7-14 | `cargo test --test erasure_cascade` |
| P8-19 | **Erasure verification**: no PII in any inventory store; financials unchanged | P8-18 | `cargo test --test erasure_verification` |
| P8-20 | Retention jobs for every store in the data inventory + staleness alert | P8-18 | `cargo test --test retention_jobs` |
| P8-21 | Customer data export endpoint, re-auth + rate limited | P7-14 | `cargo test --test data_export` |
| P8-22 | **Payment reconciliation job** ([PAYMENTS §9](PAYMENTS.md)); drift never auto-corrects | P4-24, P1-22 | `cargo test -p stheno-payment reconciliation` |
| P8-22b | **Reporting: 4 aggregates** — orders by status, revenue (confirmed only), top 10 products, orders by source ([PRODUCT §7](PRODUCT.md)) | P5-22 | `cargo test -p stheno-flows-platform reporting` |
| P8-22c | Reporting window gate — horizon bound by `analytics_window_days` | P8-22b, P2-21f | `cargo test -p stheno-flows-platform reporting_window` |
| P8-22d | Web push subscriber on `order.placed` — **must skip `metadata.source == \"social\"`** ([PRODUCT §5.5](PRODUCT.md) trap 3). Test both paths | P8-04b, P5-19 | `cargo test -p stheno-subscribers push_order_placed` |
| P8-23 | 13 settings/translation/file workflows | P1-05, P8-16 | `cargo test -p stheno-flows settings translation file` |

**Phase gate:** P8-08, P8-11, P8-19, P8-22 green; index lag < 2 s at 500 rps.

---

## P9 — Admin API, forked dashboard, and product screens (40 tasks · Track C)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P9-01 | Admin response envelopes in **Medusa's exact shapes** | P5-22 | `cargo test -p stheno-api-core envelopes` |
| P9-02 | Admin catalog routes (products, variants, options, tags, types, collections, categories) | P9-01, P2-19 | `./scripts/parity.sh --group admin-catalog` |
| P9-03 | Admin inventory, reservations, stock-locations | P9-01, P2-11 | `./scripts/parity.sh --group admin-inventory` |
| P9-04 | Admin pricing: price-lists, price-preferences | P9-01, P3-07 | `./scripts/parity.sh --group admin-pricing` |
| P9-05 | Admin orders: list, detail, lifecycle | P9-01, P5-19 | `./scripts/parity.sh --group admin-orders` |
| P9-06 | Admin order mutations: edits, returns, claims, exchanges, **preview endpoints** | P9-05, P6-19 | `./scripts/parity.sh --group admin-order-mutations` |
| P9-07 | Admin draft-orders | P9-05, P6-16 | `./scripts/parity.sh --group admin-draft-orders` |
| P9-08 | Admin fulfillment, shipping options/profiles/types | P9-01, P5-13 | `./scripts/parity.sh --group admin-fulfillment` |
| P9-09 | Admin payments, payment-collections, refund/return reasons | P9-01, P4-24 | `./scripts/parity.sh --group admin-payments` |
| P9-10 | Admin customers, customer-groups | P9-01, P7-14 | `./scripts/parity.sh --group admin-customers` |
| P9-11 | Admin promotions, campaigns | P9-01, P4-13 | `./scripts/parity.sh --group admin-marketing` |
| P9-12 | Admin config: regions, currencies, sales-channels, stores, tax, locales, translations | P9-01, P3-02 | `./scripts/parity.sh --group admin-config` |
| P9-13 | Admin access: users, invites, api-keys, rbac | P9-01, P7-18 | `./scripts/parity.sh --group admin-access` |
| P9-14 | Admin platform: uploads, notifications, workflows-executions, views, layouts, property-labels | P9-01, P8-16 | `./scripts/parity.sh --group admin-platform` |
| P9-14b | `GET /admin/feature-flags` in Medusa's shape; forked dashboard hides disabled screens unmodified | P9-01, P0-26 | `./scripts/parity.sh --group admin-feature-flags` |
| P9-14c | Additive `GET /admin/entitlements` — features, quotas, usage, grace state | P9-14b, P2-21b | `cargo test -p stheno-api-admin-platform entitlements` |
| P9-14d | Conditional router mounting per feature; disabled routes 404 | P9-14b | `cargo test -p stheno-api-router gated_mounting` |
| P9-14e | **Tier matrix test**: each tier boots and mounts exactly its declared routes, no more | P9-14d | `cargo test --test tier_matrix` |
| P9-14f | Dashboard fork: entitlements settings screen + over-quota grace banner | P9-14c, P9-19 | `npm --prefix apps/admin run test:e2e -- --suite entitlements` |
| P9-15 | Additive workflow-observability endpoints (timeline, retry, cancel, run) | P9-14, P1-13 | `cargo test -p stheno-api-admin-platform workflow_obs` |
| P9-16 | **Fork the dashboard** at a pinned version; record in `apps/admin/UPSTREAM.md` | P9-14 | `test -f apps/admin/UPSTREAM.md && cd apps/admin && npm ci && npm run build` |
| P9-17 | **Strip every Enterprise path**; CI check that none return | P9-16 | `./scripts/check-no-enterprise-paths.sh` |
| P9-18 | **Unmodified dashboard runs against Stheno** — the compatibility gate | P9-16 | `npm --prefix apps/admin run test:e2e -- --suite smoke` |
| P9-19 | Point at Stheno; patch API client to send `X-Stheno-CSRF` | P9-18, P0-16 | `npm --prefix apps/admin run test:e2e -- --suite mutations` |
| P9-20 | **Clean-room roles + policies screens** and permissions provider | P9-17, P7-18 | `npm --prefix apps/admin run test:e2e -- --suite rbac` |
| P9-21 | **Clean-room SSO login** component | P9-17, P7-05 | `npm --prefix apps/admin run test:e2e -- --suite sso` |
| P9-22 | Rebrand: name, logo, colour tokens | P9-19 | `npm --prefix apps/admin run build` |
| P9-23 | Deepen the workflow-executions screen (timeline, replay-from-step) | P9-15, P9-19 | `npm --prefix apps/admin run test:e2e -- --suite workflows` |
| P9-24 | Verify `admin-vite-plugin` extension system works against Stheno | P9-19 | `npm --prefix apps/admin run test:extensions` |
| P9-25 | WCAG 2.2 AA audit + fixes | P9-22 | `npm --prefix apps/admin run test:a11y` |
| P9-26 | **Priya's five jobs** ([PRD §4.1](PRD.md)) end-to-end | P9-23 | `npm --prefix apps/admin run test:e2e -- --suite operator-jobs` |
| P9-27 | Onboarding route: Google sign-in → store name + WhatsApp → subdomain with live uniqueness check | P9-19, P7-04 | `npm --prefix apps/admin run test:e2e -- --suite onboarding` |
| P9-28 | Spreadsheet upload: accept `.xlsx` **and** `.csv`, parse headers | P9-27 | `npm --prefix apps/admin run test:e2e -- --suite import-upload` |
| P9-29 | **Column mapping**: their headers ↔ our fields, **drop unmapped**, **inject sales channel**, persist mapping, 5-row preview, errors in the seller's own column names | P9-28 | `npm --prefix apps/admin run test:e2e -- --suite import-mapping` |
| P9-30 | Import acceptance: a real messy spreadsheet imports unedited, and the products are **visible on the storefront** | P9-29 | `npm --prefix apps/admin run test:e2e -- --suite import-acceptance` |
| P9-31 | Social queue screen: draft orders, `Ref` code matching, convert-to-order | P9-19, P9-32 | `npm --prefix apps/admin run test:e2e -- --suite social-queue` |
| P9-32 | Social draft creation — call the order workflow **directly**, no email/customer_id, buyer in `metadata`, `sales_channel_id` set at creation, `metadata.source = "social"` ([PRODUCT §5.5](PRODUCT.md) traps 1–3) | P5-19 | `cargo test -p stheno-flows-order social_draft` |
| P9-33 | **Trap-2 regression test**: create a social draft, then convert it — reservation must resolve | P9-32 | `cargo test -p stheno-flows-order social_draft_convert` |
| P9-34 | Analytics screen rendering the 4 aggregates, horizon from the tier | P8-22c, P9-19 | `npm --prefix apps/admin run test:e2e -- --suite analytics` |
| P9-35 | Billing/entitlements screen: tier, trial countdown, quota usage, upgrade | P9-14f | `npm --prefix apps/admin run test:e2e -- --suite billing` |

**Phase gate:** P9-18 first, then P9-20, P9-25, P9-26; every admin parity group green.

---

## P10 — Storefront, migration, hardening, GA (16 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P10-01 | Fork Medusa's Next.js starter; pin and record upstream | P5-22 | `cd apps/storefront && npm ci && npm run build` |
| P10-02 | Point at Stheno; **unmodified storefront runs** — the compatibility gate | P10-01 | `npm --prefix apps/storefront run test:e2e -- --suite smoke` |
| P10-03 | Full purchase journey e2e against Stripe test mode | P10-02, P5-18 | `npm --prefix apps/storefront run test:e2e -- --suite checkout` |
| P10-03b | **Order Now**: address form, COD-only payment section unless Pro | P10-02, P4-23b | `npm --prefix apps/storefront run test:e2e -- --suite order-now` |
| P10-03c | **Social checkout**: WhatsApp `wa.me?text=` pre-fill; **clipboard + open app** for Messenger and Instagram ([PRODUCT §5.4](PRODUCT.md)) | P10-02, P9-32 | `npm --prefix apps/storefront run test:e2e -- --suite social-checkout` |
| P10-03d | Source attribution: `?ref=` → cart metadata → order metadata | P10-03b | `npm --prefix apps/storefront run test:e2e -- --suite attribution` |
| P10-03e | CSV export written with a **UTF-8 BOM** — verified opening in Excel on Windows | P9-29 | `cargo test -p stheno-flows-platform csv_export_bom` |
| P10-04 | Lighthouse ≥ 95 and CWV gates on PDP | P10-02 | `npm --prefix apps/storefront run test:lighthouse` |
| P10-05 | `stheno import-medusa` schema mapping, all tables, IDs preserved | P8-19 | `cargo test -p stheno-cli import_mapping` |
| P10-06 | Import money conversion with no-precision-loss assertion | P10-05, P0-05 | `cargo test -p stheno-cli import_money` |
| P10-07 | Import `--dry-run` fidelity report | P10-05 | `cargo run -p stheno-cli -- import-medusa --dry-run --source $MEDUSA_DB` |
| P10-08 | **Import verification**: order/payment/refund totals match source exactly | P10-06 | `cargo test -p stheno-cli import_totals_exact` |
| P10-09 | Post-import index + search rebuild | P10-05, P8-13 | `cargo test -p stheno-cli import_reindex` |
| P10-10 | Encrypted backups; **restore drill + erasure replay** | P8-18 | `./scripts/restore-drill.sh` |
| P10-11 | Distroless image ≤ 60 MB, non-root, cold start ≤ 1 s | P0-01 | `./scripts/check-image.sh` |
| P10-12 | Helm chart + deployment docs | P10-11 | `helm lint deploy/chart && helm template deploy/chart` |
| P10-13 | **Full load suite: all 7 latency budgets** on reference hardware | all | `k6 run tests/load/all.js --quiet` |
| P10-14 | Memory ≤ 400 MB at 500 rps sustained 1 hour | P10-13 | `./scripts/soak.sh --duration 1h --rps 500` |
| P10-15 | Penetration test; no high or critical | P10-11 | Report filed, findings closed |
| P10-16 | **PCI SAQ A** incl. script-attack criterion + PSP confirmation on file | P4-30 | `./scripts/check-pci-saq-a.sh` |
| P10-17 | `security.txt`, disclosure policy, SBOM per release | P10-11 | `./scripts/check-disclosure.sh` |
| P10-18 | Docs site: getting started, module authoring, provider authoring, migration | all | `npm --prefix docs-site run build` |

**Phase gate:** the full 1.0 checklist in [PARITY_MATRIX §4](PARITY_MATRIX.md).

---

## 3. Task summary

| Phase | Tasks | Weeks | Track |
|---|---:|---:|---|
| P0 | 27 | 3 | A |
| P1 | 25 | 5 | A |
| P2 | 27 | 5 | A |
| P3 | 19 | 4 | A |
| P4 | 31 | 6 | A |
| P5 | 22 | 6 | A |
| P6 | 20 | 6 | A |
| P7 | 22 | 4 | B |
| P8 | 27 | 5 | B |
| P9 | 40 | 7 | C |
| P10 | 22 | 5 | A+B+C |
| **Total** | **282** | **56** | |

---

## 4. What this decomposition deliberately does not do

- **It does not estimate individual tasks.** Task size varies; the phase week counts are the
  estimate. Per-task estimates would be false precision.
- **It does not sequence within a phase beyond dependencies.** Anything with satisfied deps can be
  picked up.
- **It does not replace the phase gates.** A phase is done when its gate passes, not when its last
  task is `DONE` — the gate tests properties no single task owns.
- **It does not cover spec changes.** When a task reveals the spec is wrong (rule L8), that is a
  documentation commit first, then implementation.
