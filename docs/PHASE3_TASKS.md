# Phase 3 Task Decomposition

**Status:** Final · **Date:** 2026-08-19 · **Plan:** [PHASE3_PLAN.md](PHASE3_PLAN.md)

**95 tasks across 8 phases (P21–P28).** Each is 2 hours to 2 days, has explicit dependencies, and
ends in a **command that exits 0** — never a judgement call.

[PHASE3_PLAN.md](PHASE3_PLAN.md) says what each phase achieves and why. This document is what an
agent or engineer actually picks up.

Task IDs continue from Phase 2 and never collide with it. [TASKS.md](TASKS.md) P0–P10 and
[PHASE2_TASKS.md](PHASE2_TASKS.md) P11–P20 IDs remain valid references.

---

## 1. The loop protocol

Unchanged from [TASKS.md §1](TASKS.md). Rules **L1–L8** apply verbatim, including the two that
matter most here:

> **L3 — Never edit the exit command to make it pass.** Changing the gate is a spec change and
> needs sign-off.

> **L8 — A decision that changes an ADR is an ADR amendment first.** ADR-013 is written and
> accepted before P21-01 and P25-01. Both tracks depend on it.

### 1.1 The universal gate, Phase 3 edition

Runs on **every** backend task before its specific exit command. Two additions from Phase 2:

```bash
test -n "$DATABASE_URL" || { echo "DATABASE_URL unset"; exit 1; }

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-module-contracts.sh
./scripts/check-dto-separation.sh
./scripts/check-schema-drift.sh
./scripts/check-no-domain-gates.sh
./scripts/check-no-in-memory-stores.sh
./scripts/check-pci-saq-a.sh
./scripts/check-provider-pci.sh          # new: written in P21-13
./scripts/check-provider-conformance.sh  # new: written in P21-08
```

For **frontend** tasks (P25–P27) the gate is:

```bash
npm --prefix apps/storefront run lint
npm --prefix apps/storefront run typecheck
npm --prefix apps/storefront run test
./scripts/check-storefront-boundaries.sh   # new: written in P25-09
./scripts/check-checkout-scripts.sh
```

`check-provider-conformance.sh` runs the generic suite against **every** registered provider. It is
what stops a gateway from being merged with a hand-written test that proves nothing.

`check-storefront-boundaries.sh` is the frontend equivalent of `check-module-isolation.sh`. It is
what stops a theme from reaching into core.

### 1.2 Reading a task row

Unchanged. **ID** is stable and never renumbered. **Deps** must be `DONE` first. **Exit** is the
command that decides, assuming repo root.

Where an exit command names a test or script that does not exist yet, **writing it is part of the
task.**

### 1.3 `[BP]` — breaking points

Sixteen task rows are marked **`[BP]`**. They implement the thirteen breaking points in
[PHASE3_PLAN.md §8](PHASE3_PLAN.md#8-breaking-points--where-the-lazy-answer-is-the-wrong-answer),
and they share one property: **a plausible implementation passes its own exit command and fails in
production, silently.** Wrong money, a stuck order, a free product, or another tenant's data on the
page — no crash, no red build.

On a `[BP]` task, three rules replace the normal ones:

1. **The exit command is the floor, not the ceiling.** Green is necessary and not sufficient.
2. **Write the negative test first** — the one proving the failure mode is caught — before the
   test proving the happy path works.
3. **Report what you proved, not that it passed.** A `[BP]` task closed with "gate green" and no
   account of the failure mode is not closed. If you took a shortcut, name it.

The ladder in [AGENTS.md §1](../AGENTS.md) still applies to *how much code you write*. It has
never applied to how much you read, and on these sixteen it does not apply to how much you verify.

```bash
grep -nE '^\| P[0-9]+-[0-9a-z]+ \| [*][*]\[BP\]' docs/PHASE3_TASKS.md   # the sixteen
```

---

## 0. Pre-phase — not engineering work, on the critical path anyway

| ID | Task | Deps | Exit |
|---|---|---|---|
| P3-00a | **File bKash merchant sandbox application.** Business documents, not a signup form | — | Credentials received and recorded in the secrets store |
| P3-00b | **File Nagad merchant sandbox application** | — | Credentials received and recorded in the secrets store |
| P3-00c | **File SSLCommerz sandbox application** | — | Credentials received and recorded in the secrets store |
| P3-00d | **Open Pathao, Steadfast, RedX merchant accounts** | — | Credentials received and recorded in the secrets store |
| P3-00e | **Write and accept [ADR-013](DECISIONS.md#adr-013)** — tiers, checkout-is-core, providers compile-time / enablement as data, redirect-only for PCI | — | ADR-013 merged with status Accepted and an index row |

These five are not counted in the 95. **P3-00a–d are started in week 1 regardless of engineering
readiness** — see the P22 risk. P3-00e blocks P21-01 and P25-01.

---

## P21 — Provider platform completion (14 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P21-01 | `POST /hooks/payment/:provider_id` route, raw body extractor, wired to `PaymentService::receive_webhook` | P3-00e | `cargo test -p stheno-api-store --test payment_webhook_route` |
| P21-02 | Route returns 401 on signature failure and writes no row; 200 on replay without reprocessing | P21-01 | `cargo test -p stheno-api-store --test payment_webhook_semantics` |
| P21-03 | **[BP]** **Raw-body preservation assertion** — a byte-exact fixture survives the middleware stack unparsed | P21-01 | `cargo test -p stheno-api-store --test raw_body_integrity` |
| P21-04 | `POST /hooks/fulfillment/:provider_id` route and `FulfillmentService::receive_webhook` with its own replay guard and event table | P21-01 | `cargo test -p stheno-fulfillment --test fulfillment_webhook` |
| P21-05 | Migration: `fulfillment.fulfillment_webhook_event`, tenant-owned, RLS + FORCE, unique on `(provider_id, provider_event_id)` | P21-04 | `./scripts/check-schema-drift.sh && cargo test -p stheno-db --test rls_audit` |
| P21-06 | `ProviderCapabilities.redirect: bool`, defaulting false; Stripe and offline declare theirs | P3-00e | `cargo test -p stheno-payment --test capabilities_redirect` |
| P21-07 | Document and type the `InitiateOutput.data.redirect_url` convention; offline declares none, a fixture redirect provider declares one | P21-06 | `cargo test -p stheno-payment --test redirect_contract` |
| P21-08 | **[BP]** **Provider conformance suite**: session lifecycle, idempotent capture, webhook replay, refund paths | P21-02 | `cargo test -p stheno-payment --test conformance_suite` |
| P21-09 | **[BP]** Conformance suite asserts **capability honesty** — every `true` in `capabilities()` is exercised and must succeed; every `false` must be refused | P21-08 | `cargo test -p stheno-payment --test conformance_capabilities` |
| P21-10 | Stripe and offline pass the suite with zero provider-specific exemptions | P21-09 | `./scripts/check-provider-conformance.sh` |
| P21-11 | Admin routes: list available providers, enable/disable per region, per tenant | P21-05 | `cargo test -p stheno-api-admin --test provider_config_routes` |
| P21-12 | Provider credentials encrypted at rest; plaintext never in logs or audit rows | P21-11 | `cargo test -p stheno-payment --test credential_encryption` |
| P21-13 | `scripts/check-provider-pci.sh` — fails on any provider crate declaring a card-data-accepting flow. Includes a deliberately non-compliant fixture that must fail | P21-07 | `./scripts/check-provider-pci.sh` |
| P21-14 | **Tenant isolation test**: tenant A enabling a provider is invisible to tenant B, asserted by query | P21-11 | `cargo test -p stheno-payment --test provider_tenant_isolation` |

**Phase gate:** `./scripts/gate-p21.sh`

---

## P22 — Bangladesh payment gateways (18 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P22-01 | `stheno-payment-bkash` crate skeleton, registered, `id()` and `capabilities()` honest | P21-10, P3-00a | `cargo test -p stheno-payment-bkash --test registration` |
| P22-02 | bKash grant-token lifecycle with refresh and expiry handling | P22-01 | `cargo test -p stheno-payment-bkash --test token_lifecycle` |
| P22-03 | bKash `initiate` → create payment, returns `redirect_url` | P22-02 | `cargo test -p stheno-payment-bkash --test initiate` |
| P22-04 | bKash `authorize`/`capture` via execute + query-payment convergence | P22-03 | `cargo test -p stheno-payment-bkash --test capture` |
| P22-05 | **[BP]** bKash `verify_webhook` / `map_webhook`, including a tampered-signature rejection test | P22-04 | `cargo test -p stheno-payment-bkash --test webhook` |
| P22-06 | bKash refund; declare `partial_refund` truthfully and refuse what is not supported | P22-04 | `cargo test -p stheno-payment-bkash --test refund` |
| P22-07 | `stheno-payment-nagad` crate skeleton, registered, capabilities honest | P21-10, P3-00b | `cargo test -p stheno-payment-nagad --test registration` |
| P22-08 | Nagad checkout initialise with its key-exchange/signing scheme | P22-07 | `cargo test -p stheno-payment-nagad --test initialise` |
| P22-09 | Nagad complete + verification callback → capture | P22-08 | `cargo test -p stheno-payment-nagad --test capture` |
| P22-10 | **[BP]** Nagad `verify_webhook` / `map_webhook` with tampered-signature rejection | P22-09 | `cargo test -p stheno-payment-nagad --test webhook` |
| P22-11 | Nagad refund and capability declaration | P22-09 | `cargo test -p stheno-payment-nagad --test refund` |
| P22-12 | `stheno-payment-sslcommerz` crate skeleton, registered, capabilities honest | P21-10, P3-00c | `cargo test -p stheno-payment-sslcommerz --test registration` |
| P22-13 | SSLCommerz hosted session creation returning `redirect_url` | P22-12 | `cargo test -p stheno-payment-sslcommerz --test session` |
| P22-14 | **[BP]** SSLCommerz IPN validation and transaction-query convergence | P22-13 | `cargo test -p stheno-payment-sslcommerz --test ipn` |
| P22-15 | SSLCommerz refund and capability declaration | P22-14 | `cargo test -p stheno-payment-sslcommerz --test refund` |
| P22-16 | **All three pass the conformance suite unmodified** | P22-06, P22-11, P22-15 | `./scripts/check-provider-conformance.sh` |
| P22-17 | **[BP]** BDT end to end: decimal totals, minor-unit conversion only at the provider boundary, rounding asserted | P22-16 | `cargo test -p stheno-payment --test bdt_money` |
| P22-18 | Recorded sandbox fixtures for all three so CI does not call a third party | P22-16 | `./scripts/check-provider-fixtures.sh` |

**Phase gate:** `./scripts/gate-p22.sh`

---

## P23 — Courier providers and tracking (13 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P23-01 | Courier status vocabulary: one engine-side enum, plus an explicit `Unknown(String)` that logs and alerts | P21-04 | `cargo test -p stheno-fulfillment --test status_vocabulary` |
| P23-02 | `stheno-fulfillment-pathao` crate, registered, options and serviceable-area lookup | P23-01, P3-00d | `cargo test -p stheno-fulfillment-pathao --test options` |
| P23-03 | Pathao `calculate_price` — inside-city, outside-city, sub-city tiers | P23-02 | `cargo test -p stheno-fulfillment-pathao --test rates` |
| P23-04 | Pathao `create_fulfillment` returning a consignment/tracking id; cancel and return flows | P23-03 | `cargo test -p stheno-fulfillment-pathao --test consignment` |
| P23-05 | Pathao status webhook mapping, every documented status covered by a fixture | P23-04 | `cargo test -p stheno-fulfillment-pathao --test status_mapping` |
| P23-06 | `stheno-fulfillment-steadfast` crate: options, rates, consignment, cancel, return | P23-01, P3-00d | `cargo test -p stheno-fulfillment-steadfast` |
| P23-07 | Steadfast status mapping with fixtures | P23-06 | `cargo test -p stheno-fulfillment-steadfast --test status_mapping` |
| P23-08 | `stheno-fulfillment-redx` crate: options, rates, consignment, cancel, return | P23-01, P3-00d | `cargo test -p stheno-fulfillment-redx` |
| P23-09 | RedX status mapping with fixtures | P23-08 | `cargo test -p stheno-fulfillment-redx --test status_mapping` |
| P23-10 | **Polling fallback** for couriers that do not push, sharing the webhook reducer — not a second code path | P23-05 | `cargo test -p stheno-fulfillment --test status_polling` |
| P23-11 | **[BP]** **Unknown status is loud**: an unmapped status raises an alert and leaves the fulfillment in a named state, never a silent no-op | P23-10 | `cargo test -p stheno-fulfillment --test unknown_status_alerts` |
| P23-12 | Shipping-option configuration per courier zone expressed through existing shipping options | P23-03, P23-06, P23-08 | `cargo test -p stheno-fulfillment --test zone_options` |
| P23-13 | Recorded sandbox fixtures for all three couriers | P23-09 | `./scripts/check-courier-fixtures.sh` |

**Phase gate:** `./scripts/gate-p23.sh`

---

## P24 — COD settlement and reconciliation (12 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P24-01 | Migration: courier settlement cycle and settlement line tables, tenant-owned, RLS + FORCE | P22-16, P23-11 | `./scripts/check-schema-drift.sh && cargo test -p stheno-db --test rls_audit` |
| P24-02 | Order carries three distinct amounts: order total, collected, remitted. Never conflated | P24-01 | `cargo test -p stheno-order --test cod_amounts` |
| P24-03 | **[BP]** Delivery confirmation authorises capture on the COD provider — **one flow, courier-agnostic** | P24-02 | `cargo test -p stheno-payment --test cod_capture_on_delivery` |
| P24-04 | A returned COD order produces no capture and no settlement expectation | P24-03 | `cargo test -p stheno-payment --test cod_return_no_capture` |
| P24-05 | **[BP]** Settlement report import, per courier, idempotent on re-import | P24-01 | `cargo test -p stheno-payment --test settlement_import` |
| P24-06 | Reconciliation run writes to `payment.payment_reconciliation_run` with three buckets: matched, collected-not-remitted, remitted-not-matched | P24-05 | `cargo test -p stheno-payment --test reconciliation_buckets` |
| P24-07 | **[BP]** Courier fee accounting: `sum(remitted) + sum(fees) == sum(collected)` over a full cycle, asserted in decimal | P24-06 | `cargo test -p stheno-payment --test settlement_balance` |
| P24-08 | A deliberately mismatched settlement file produces a discrepancy row, not a silent balance | P24-06 | `cargo test -p stheno-payment --test settlement_mismatch` |
| P24-09 | Discrepancy alerting: an unmatched line older than the cycle length raises an alert | P24-08 | `cargo test -p stheno-payment --test settlement_alerting` |
| P24-10 | Admin reconciliation routes: per-cycle owed, matched, and unmatched | P24-06 | `cargo test -p stheno-api-admin --test reconciliation_routes` |
| P24-11 | Admin reconciliation UI | P24-10 | `npm --prefix apps/admin run test -- reconciliation` |
| P24-12 | **End-to-end COD**: order → delivered → captured → settled from courier input alone, decimal-exact | P24-09 | `./scripts/cod-lifecycle.sh` |

**Phase gate:** `./scripts/gate-p24.sh`

---

## P25 — Storefront core/theme split (13 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P25-01 | Workspace restructure: `packages/storefront-core`, `packages/theme-default`, app shell consuming both. No code moved yet | P3-00e | `npm --prefix apps/storefront run build` |
| P25-02 | **Theme B component inventory drawn up now**, before the contract is frozen — the list theme B will need, from the designer, not from core's authors | P3-00e | `test -f docs/THEME_CONTRACT_INPUTS.md` |
| P25-03 | Move `src/lib/data`, `src/lib/context`, `src/lib/hooks`, `middleware.ts` into core, unchanged | P25-01 | `npm --prefix apps/storefront run typecheck` |
| P25-04 | **[BP]** Move the `app/` route tree into core. Routing, region resolution and SEO are not theme-editable | P25-03 | `npm --prefix apps/storefront run build` |
| P25-05 | Define the theme contract: one typed interface enumerating every required export and its props | P25-02, P25-04 | `npm --prefix apps/storefront run typecheck` |
| P25-06 | Move `src/modules` into `theme-default`, conforming it to the contract | P25-05 | `npm --prefix apps/storefront run typecheck` |
| P25-07 | Theme resolution point — the single place core binds a theme. Deleting the theme errors here and nowhere else | P25-06 | `./scripts/check-theme-resolution.sh` |
| P25-08 | Checkout components live in core and are styled by theme tokens, per ADR-013 — flow, steps and validation are not overridable | P25-06 | `npm --prefix apps/storefront run test -- checkout-ownership` |
| P25-09 | `scripts/check-storefront-boundaries.sh` — fails on a theme importing core internals or the SDK. Includes a violating fixture theme that must fail | P25-07 | `./scripts/check-storefront-boundaries.sh` |
| P25-10 | Assert `theme-default` contains no data fetching, no server actions, no checkout state | P25-09 | `./scripts/check-theme-purity.sh` |
| P25-11 | Core/theme versioning policy: themes pin a core major, core changes additive within a major | P25-07 | `test -f docs/THEME_VERSIONING.md && ./scripts/check-theme-pin.sh` |
| P25-12 | Existing e2e pack passes unmodified against the restructured storefront | P25-08 | `npm --prefix apps/storefront run test:e2e` |
| P25-13 | **[BP]** **Zero-behaviour-change proof**: pre- and post-extraction route snapshots are byte-identical for a fixed catalog | P25-12 | `./scripts/check-extraction-parity.sh` |

**Phase gate:** `./scripts/gate-p25.sh`

---

## P26 — Branding tier, runtime tokens (9 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P26-01 | **[BP]** **Confirm ADR-012's tenancy audit is green** before any branding work begins | P25-13 | `cargo test -p stheno-db --test rls_audit` |
| P26-02 | Migration: tenant branding table — name, logo, favicon, font key, colour tokens. Tenant-owned, RLS + FORCE | P26-01 | `./scripts/check-schema-drift.sh && cargo test -p stheno-db --test rls_audit` |
| P26-03 | Store API exposes resolved branding for the current tenant; absent row yields documented defaults | P26-02 | `cargo test -p stheno-api-store --test branding_endpoint` |
| P26-04 | Curated font set, self-hosted, licences recorded. Merchants choose a key; they do not upload font files | P26-02 | `./scripts/check-font-licences.sh` |
| P26-05 | Core resolves branding per request and emits CSS custom properties. No per-tenant build, no dynamic component loading | P26-03 | `npm --prefix apps/storefront run test -- branding-tokens` |
| P26-06 | Logo and favicon upload through the existing file module, with type and dimension validation at the boundary | P26-03 | `cargo test -p stheno-file --test branding_asset_validation` |
| P26-07 | Admin branding UI with live preview | P26-06 | `npm --prefix apps/admin run test -- branding` |
| P26-08 | **Two tenants, one process, different renders** — name, font, logo and colour all differ | P26-05 | `./scripts/check-multi-tenant-branding.sh` |
| P26-08a | **Branding applies on checkout routes** and the script manifest stays clean while it does — self-hosted fonts only, per ADR-013 decision 2a | P26-08 | `./scripts/check-checkout-branding.sh && ./scripts/check-checkout-scripts.sh` |
| P26-09 | Lighthouse does not regress against the pre-P26 baseline; branding resolution costs no render-path round trip | P26-08 | `npm --prefix apps/storefront run test:lighthouse` |

**Phase gate:** `./scripts/gate-p26.sh`

---

## P27 — Theme packages and conformance (10 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P27-01 | **[BP]** **Theme conformance e2e pack**, written against the theme contract, not against markup: browse → product → cart → checkout → order confirmed | P26-09 | `npm --prefix apps/storefront run test:conformance -- --theme=default` |
| P27-02 | Retrofit the pack to `theme-default` and make it the gate for that theme too, before theme B exists | P27-01 | `./scripts/check-theme-conformance.sh` |
| P27-03 | `packages/theme-<b>` skeleton conforming to the contract, from the P25-02 inventory | P27-02 | `npm --prefix apps/storefront run typecheck` |
| P27-04 | Theme B catalog and product surfaces | P27-03 | `npm --prefix apps/storefront run test:conformance -- --theme=b` |
| P27-05 | Theme B cart and account surfaces | P27-04 | `npm --prefix apps/storefront run test:conformance -- --theme=b` |
| P27-06 | Theme B checkout **skinning** — restyled pages, unaltered flow, steps and scripts | P27-05 | `./scripts/check-checkout-scripts.sh && npm --prefix apps/storefront run test -- checkout-ownership` |
| P27-07 | **Log every core change theme B required.** More than two means the P25 contract was drawn wrong and is fixed here | P27-06 | `test -f docs/THEME_B_CORE_DELTAS.md && ./scripts/check-core-delta-budget.sh` |
| P27-08 | Per-tenant themed build and deploy pipeline: one command, one target, theme pinned to a core major | P27-06 | `./scripts/deploy-themed-storefront.sh --dry-run` |
| P27-09 | Tenant switched from theme A to theme B by configuration plus one deploy | P27-08 | `./scripts/check-theme-switch.sh` |
| P27-10 | Theme authoring documentation, checkout rule stated first; time-to-new-theme measured and recorded | P27-09 | `test -f docs-site/docs/theme-authoring.md && ./scripts/check-theme-docs.sh` |

**Phase gate:** `./scripts/gate-p27.sh`

---

## P28 — Hardening, PCI re-attestation, GA (5 tasks)

| ID | Task | Deps | Exit |
|---|---|---|---|
| P28-01 | **[BP]** Add every new gateway redirect and iframe origin to the CSP and to `docs/payment-page-scripts.md` | P22-18, P27-10 | `./scripts/check-checkout-scripts.sh` |
| P28-02 | SAQ-A re-attestation covering all registered providers | P28-01 | `./scripts/check-pci-saq-a.sh` |
| P28-03 | Provider failure modes — gateway timeout, gateway 500, callback never arriving, courier API down — each with a defined tested outcome | P24-12, P23-13 | `cargo test --workspace --test provider_failure_modes` |
| P28-04 | Load test: multiple tenants and themes in one runtime, budgets held with branding on the render path | P26-09, P28-03 | `./scripts/load-suite.sh --multi-tenant` |
| P28-05 | Runbooks: reconciliation discrepancy, stuck consignment, gateway outage | P28-03 | `./scripts/check-runbooks.sh` |

**Phase gate:** `./scripts/gate-p28.sh`

---

## 2. Task count by phase

| Phase | Tasks |
|---|---:|
| P21 | 14 |
| P22 | 18 |
| P23 | 13 |
| P24 | 12 |
| P25 | 13 |
| P26 | 10 |
| P27 | 10 |
| P28 | 5 |
| **Total** | **95** |

Plus five pre-phase items (P3-00a–e) that are not engineering work and are on the critical path
anyway.
