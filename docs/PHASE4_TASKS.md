# Phase 4 — Control Plane Task Decomposition

**Status:** Draft · **Date:** 2026-08-21 (Updated 2026-09-08) · **Scope:** [CONTROL_PLANE.md](CONTROL_PLANE.md) · **Space Target:** [SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md](SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md) · **Decisions:** [ADR-030](DECISIONS.md#adr-030)

> **Current status source:** [STATUS.md](STATUS.md). This file defines acceptance criteria and
> task order. It should not carry independent progress percentages.
>
> [!NOTE]
> **Space Standalone Target (2026-09-08):**  
> The company control plane is now established as **Space**, a separate standalone project at `D:\Syed Systems\Space`.
> Tasks in this file complete the transitional in-repo `stheno-api-platform` service. Greenfield company-wide
> control plane development is tracked under Space tickets (`SPACE-...`) in `D:\Syed Systems\Space\docs\IMPLEMENTATION_BACKLOG.md`.

**87 tasks across 8 phases (P29–P36).** Each is 2 hours to 2 days, has explicit
dependencies, and ends in a **command that exits 0** — never a judgement call.

[CONTROL_PLANE.md](CONTROL_PLANE.md) says what the control plane is and why each domain
matters. This document is what an agent or engineer actually picks up.

Task IDs continue from Phase 3 and never collide with it. [TASKS.md](TASKS.md) P0–P10,
[PHASE2_TASKS.md](PHASE2_TASKS.md) P11–P20 and [PHASE3_TASKS.md](PHASE3_TASKS.md) P21–P28
remain valid references.

---

## 1. The loop protocol

Unchanged from [TASKS.md §1](TASKS.md). Rules **L1–L8** apply verbatim, including:

> **L3 — Never edit the exit command to make it pass.** Changing the gate is a spec change
> and needs sign-off.

### 1.1 The universal gate, Phase 4 edition

Runs on **every** backend task before its specific exit command:

```bash
test -n "$DATABASE_URL" || { echo "DATABASE_URL unset"; exit 1; }

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-dto-separation.sh
./scripts/check-schema-drift.sh
./scripts/check-no-in-memory-stores.sh
./scripts/check-control-plane-audit.sh   # new: written in P29-08
```

For **frontend** tasks the gate is:

```bash
npm --prefix apps/platform run typecheck
npm --prefix apps/platform run build
```

There is no `lint` script in `apps/platform`; `tsc --noEmit` plus a production build is
what the package actually offers, and naming a script that does not exist would make the
gate a lie on its first run.

`check-control-plane-audit.sh` greps every write handler in `stheno-api-platform` for a
`record(` call. It is what stops route number nineteen from shipping unaudited because the
author of route nineteen did not read this document.

### 1.2 `[BP]` — breaking points

Nineteen task rows are marked **`[BP]`**. They share one property: **a plausible
implementation passes its own exit command and fails in production, silently.** On this
surface that means one merchant charged for another's plan, a suspended merchant still
serving, a revoked operator still acting, or a cross-tenant read rendered as a dashboard.

On a `[BP]` task, three rules replace the normal ones:

1. **The exit command is the floor, not the ceiling.** Green is necessary, not sufficient.
2. **Write the negative test first** — the one proving the failure mode is caught.
3. **Report what you proved, not that it passed.**

```bash
grep -nE '^\| P[0-9]+-[0-9a-z]+ \| [*][*]\[BP\]' docs/PHASE4_TASKS.md   # the nineteen
```

### 1.3 The standing review question

Every task on this surface answers one extra question before it is closed:

> **If this handler is wrong, how many merchants does it affect?**

If the answer is "all of them", it is a `[BP]` whether or not the table says so.

---

## P29 — Operator identity and audit (10 tasks) · **D9**

*Nothing after this phase should write across every merchant without a name attached.*
Needs no credentials, no vendor, no other phase.

| ID | Task | Deps | Exit |
|---|---|---|---|
| P29-01 | Migration: `platform.operator` (hashed token, `revoked_at`) and `platform.operator_audit` (append-only, global, no `deleted_at`) | — | `./scripts/check-schema-drift.sh` |
| P29-02 | `Operator` / `OperatorAudit` models + DTOs; issue, list, revoke, authenticate, record, read-trail on `PlatformService` | P29-01 | `cargo test -p stheno-platform` |
| P29-03 | **[BP]** `actor()` replaces `authorised()`: named operator **or** bootstrap token, revoked resolves to nothing, unknown still 404s | P29-02 | `cargo test -p stheno-api-platform --test operator_identity` |
| P29-04 | Routes: `GET/POST /platform/operators`, `DELETE /platform/operators/{id}`, `GET /platform/audit` | P29-03 | `cargo test -p stheno-api-platform --test operator_identity` |
| P29-05 | **[BP]** Every existing write handler records to the trail: tenant create, tier set, subscription patch, tier define, tier reset | P29-04 | `cargo test -p stheno-api-platform --test audit_trail` |
| P29-06 | **[BP]** The trail never contains a live credential, and a refused write records nothing | P29-05 | `cargo test -p stheno-api-platform --test audit_trail` |
| P29-07 | Control-plane UI: operators page (issue, show-once token, revoke, redacted list) and audit page with entity filter | P29-04 | `npm --prefix apps/platform run build` |
| P29-08 | `scripts/check-control-plane-audit.sh` — fails on any write handler with no `record(` call; includes a fixture that must fail | P29-05 | `./scripts/check-control-plane-audit.sh` |
| P29-09 | **[BP]** Trail pagination by cursor, not offset — the trail grows at the end being read, so an offset page two re-serves rows page one showed | P29-04 | `cargo test -p stheno-api-platform --test audit_pagination` |
| P29-10 | README + [DEPLOYMENT.md](DEPLOYMENT.md) §3.1: issuing the first operator, rotating, what the bootstrap token is for | P29-07 | `./scripts/check-docs-links.sh` |

**Status: P29 is COMPLETE as of 2026-08-21.** 10 of 10.

19 tests green across `operator_identity` (6), `audit_trail` (7) and `audit_pagination` (6),
plus the pre-existing `subscription_journey`. Two new gates:
`./scripts/check-control-plane-audit.sh` (7 write handlers audited, with a fixture that
must fail) and `./scripts/check-docs-links.sh` (312 relative links resolve).

Three things worth knowing before building on it:

* **The bootstrap token stays valid** once named operators exist. Revoking the last named
  credential must not be able to lock the owner out of their own control plane. P36-05
  adds the flag that retires it deliberately.
* **The audit trail is `platform.operator_audit`, not `audit.audit_log`.** The latter is
  RLS-forced, so "everything this operator changed" is unanswerable there by construction.
* **The cursor is `id`, not `created_at`.** ULIDs sort by time and are unique, so
  `id < cursor` cannot skip a row or repeat one — which `created_at` with an `OFFSET`
  cannot promise on a table that grows at the end being read.

---

## P30 — Subscription billing (17 tasks) · **D3** · *the business*

*A subscription product that cannot invoice is a hobby.* Deliberately thin: collect
through gateways already integrated for Flow A; store invoices; **do not build a billing
engine** ([CONTROL_PLANE.md §6](CONTROL_PLANE.md)).

| ID | Task | Deps | Exit |
|---|---|---|---|
| P30-01 | **Write and accept ADR-017** — subscription billing is first-party, collected on existing gateways, no vendor; state what is *not* built (metering, rev-rec, multi-currency dunning) | P29-05 | ADR-017 merged, Accepted, index row present |
| P30-02 | Plan pricing as data: `price` (decimal), `currency`, `interval` on tier definitions + `config/tiers.toml`; absent price means the plan is not sold, never that it is free | P30-01 | `cargo test -p stheno-api-core --test tier_pricing` |
| P30-03 | Migration: `platform.invoice`, `platform.invoice_line`, `platform.billing_attempt`. Global, append-mostly, money as `numeric(20,6)` + currency per [ADR-006](DECISIONS.md#adr-006) — **not** minor units | P30-01 | `./scripts/check-schema-drift.sh` |
| P30-04 | **[BP]** `PlatformService::generate_invoice(tenant, period)` — in `stheno-platform`, which owns the `platform` schema; a `stheno-billing` crate would own a `billing` schema and fail `check-sql-boundaries.sh` — **idempotent on `(tenant_id, period_start)`**; running the cycle twice bills once | P30-03 | `cargo test -p stheno-platform --test invoice_idempotency` |
| P30-05 | **[BP]** Proration on mid-cycle plan change: credit the unused remainder, charge the new plan pro rata, and prove the two sum to the period | P30-04 | `cargo test -p stheno-platform --test proration` |
| P30-06 | VAT line (BD 15%, Mushak-6.3 fields) computed on the invoice, never on the line total | P30-04 | `cargo test -p stheno-platform --test vat` |
| P30-07 | **[BP]** Collection goes through `idempotent_call` / the outbound ledger — a timed-out charge is never retried into a double charge | P30-04, P21-01 | `cargo test -p stheno-platform --test collection_ledger` |
| P30-08 | bKash recurring / tokenised collection for subscriptions | P30-07, P22-02 | `cargo test -p stheno-platform --test bkash_collection` |
| P30-09 | SSLCommerz collection for subscriptions | P30-07, P22-11 | `cargo test -p stheno-platform --test sslcommerz_collection` |
| P30-10 | **[BP]** Dunning ladder as a state machine: `active → past_due → grace → suspended`, retry schedule, every transition audited and reversible on payment | P30-07 | `cargo test -p stheno-platform --test dunning_ladder` |
| P30-11 | **[BP]** Dunning drives `subscription.status` and **nothing else does** — an operator override is an explicit, audited exception | P30-10 | `cargo test -p stheno-platform --test dunning_is_sole_writer` |
| P30-12 | Merchant notifications on each dunning rung (email + SMS), templated, per rung | P30-10 | `cargo test -p stheno-platform --test dunning_notifications` |
| P30-13 | Credits, discounts and coupons applied to an invoice; a coupon cannot make a total negative | P30-04 | `cargo test -p stheno-platform --test credits` |
| P30-14 | Routes: `GET /platform/invoices`, `GET /platform/tenants/{id}/invoices`, `POST .../invoices/{id}/retry`, `POST .../credit` — all audited | P30-10 | `cargo test -p stheno-api-platform --test billing_routes` |
| P30-15 | Scheduled billing run: a job that closes periods and issues invoices, safe to run twice | P30-04 | `cargo test -p stheno-platform --test billing_run` |
| P30-16 | **[BP]** MRR / ARR / churn / expansion computed from invoices, not from tier assignment — a merchant on Pro who has not paid is not revenue | P30-14 | `cargo test -p stheno-platform --test revenue_metrics` |
| P30-17 | Control-plane UI: billing page — invoices, attempts, dunning state, manual retry and credit | P30-14 | `npm --prefix apps/platform run build` |

---

## P31 — Tenant lifecycle (12 tasks) · **D1**

*The enforcement end of P30, plus what support needs daily.*

| ID | Task | Deps | Exit |
|---|---|---|---|
| P31-01 | **[BP]** `POST /platform/tenants/{id}/suspend` and `/reactivate` — writes `platform.tenant.status`, audited, reason required | P29-05 | `cargo test -p stheno-api-platform --test tenant_lifecycle` |
| P31-02 | **[BP]** A suspended tenant's `/store/*` and `/admin/*` requests are refused at the tenant layer — **assert the storefront actually stops serving**, not just that the column changed | P31-01 | `cargo test -p stheno-server --test suspended_tenant_is_offline` |
| P31-03 | Suspension is visible to the merchant with a reason and a way to pay, not a blank 404 | P31-02 | `cargo test -p stheno-server --test suspension_notice` |
| P31-04 | `POST /platform/tenants/{id}/close` — terminal, requires typing the tenant id, audited | P31-01 | `cargo test -p stheno-api-platform --test tenant_lifecycle` |
| P31-05 | **[BP]** Retention window then hard purge: a closed tenant's rows are deleted from **every** RLS table after N days, and the purge proves it left nothing | P31-04 | `cargo test -p stheno-platform --test tenant_purge` |
| P31-06 | Per-tenant data export (JSON/CSV bundle) before purge, for a deletion request | P31-04 | `cargo test -p stheno-platform --test tenant_export` |
| P31-07 | Domain management: add, remove, re-point primary; `GET/POST/DELETE /platform/tenants/{id}/domains` | P29-05 | `cargo test -p stheno-api-platform --test domain_routes` |
| P31-08 | **[BP]** A domain can belong to exactly one tenant, enforced by a unique index and asserted by a concurrent double-claim test | P31-07 | `cargo test -p stheno-platform --test domain_uniqueness` |
| P31-09 | Domain verification state (DNS pointed, TLS issued) surfaced per domain | P31-07 | `cargo test -p stheno-api-platform --test domain_routes` |
| P31-10 | Tenant list: search, filter by tier/status, sort, cursor pagination | P29-05 | `cargo test -p stheno-api-platform --test tenant_listing` |
| P31-11 | Merchant health column: orders in the last 7 days, GMV, last activity, computed cross-tenant | P31-10 | `cargo test -p stheno-api-platform --test tenant_health` |
| P31-12 | Onboarding funnel state per merchant: created → product → payment connected → first order | P31-11 | `cargo test -p stheno-api-platform --test onboarding_state` |

---

## P32 — Money and fulfillment oversight (14 tasks) · **D4 + D5**

*Blocked on P21–P24. Its value is oversight of code that does not exist yet.*

| ID | Task | Deps | Exit |
|---|---|---|---|
| P32-01 | `GET /platform/providers` — the real registry, not the three hardcoded literals, with per-tenant enablement per region | P21-11 | `cargo test -p stheno-api-platform --test provider_oversight` |
| P32-02 | **[BP]** Credential health per tenant per provider: valid / expiring / failing auth — **derived from real call outcomes**, never from "a row exists" | P32-01, P21-12 | `cargo test -p stheno-api-platform --test credential_health` |
| P32-03 | **[BP]** Credential values are never rendered, logged, or returned by any control-plane route; only a health verdict is | P32-02 | `cargo test -p stheno-api-platform --test credential_opacity` |
| P32-04 | Cross-tenant outbound-ledger view: `payment_provider_request` rows in `in_flight`, aged, grouped by tenant and provider | P21-01 | `cargo test -p stheno-api-platform --test ledger_oversight` |
| P32-05 | **[BP]** **Schedule `reconcile_unsettled_requests`** — the sweep exists and nothing calls it periodically. A convergence path that never runs is not one | P32-04 | `cargo test -p stheno-payment --test sweep_is_scheduled` |
| P32-06 | Sweep outcome surfaced: last run, rows converged, rows still stranded | P32-05 | `cargo test -p stheno-api-platform --test sweep_status` |
| P32-07 | Webhook delivery health cross-tenant; dead-letter list with a replay action, audited | P21-02 | `cargo test -p stheno-api-platform --test webhook_oversight` |
| P32-08 | **[BP]** Global gateway kill switch — disable one provider across every tenant for an incident, and prove in-flight requests fail closed rather than half-complete | P32-01 | `cargo test -p stheno-api-platform --test gateway_kill_switch` |
| P32-09 | Settlement reconciliation view: gateway statement vs. our ledger, per tenant, with the delta | P24-01 | `cargo test -p stheno-api-platform --test settlement_view` |
| P32-10 | **[BP]** COD settlement oversight: courier remittance vs. collected, per tenant — the highest value-at-risk number in the product | P24-01 | `cargo test -p stheno-api-platform --test cod_oversight` |
| P32-11 | Courier registry + per-tenant credentials, same opacity rule as P32-03 | P23-01 | `cargo test -p stheno-api-platform --test courier_oversight` |
| P32-12 | Cross-tenant stranded-consignment view over `fulfillment.fulfillment_provider_request` | P23-01 | `cargo test -p stheno-api-platform --test consignment_oversight` |
| P32-13 | **Courier lookup by merchant reference** — without it a stranded consignment can only be refused, never resolved | P32-12 | `cargo test -p stheno-fulfillment --test lookup_by_reference` |
| P32-14 | Unknown-courier-status alarm surfaced here (P23-11), so an unmapped status is loud rather than silently complete | P23-11 | `cargo test -p stheno-api-platform --test unknown_status_alarm` |

---

## P33 — Growth analytics (8 tasks) · **D10** · *runs in parallel with anything*

Read-only over data already stored.

| ID | Task | Deps | Exit |
|---|---|---|---|
| P33-01 | Cross-tenant aggregation helper — one reviewed place where a query crosses the tenant boundary, so there is one thing to audit rather than twelve | P29-05 | `cargo test -p stheno-platform --test cross_tenant_aggregate` |
| P33-02 | **[BP]** That helper returns **aggregates only** — a shape that can carry one merchant's rows out of it is refused at the type level | P33-01 | `cargo test -p stheno-platform --test aggregate_only` |
| P33-03 | Total GMV across merchants, by day/week/month | P33-02 | `cargo test -p stheno-api-platform --test gmv_metrics` |
| P33-04 | Signup → activation → paid funnel with conversion rates | P31-12 | `cargo test -p stheno-api-platform --test funnel_metrics` |
| P33-05 | Plan distribution and top merchants by GMV | P33-03 | `cargo test -p stheno-api-platform --test plan_distribution` |
| P33-06 | Churn cohorts by signup month | P30-16 | `cargo test -p stheno-api-platform --test churn_cohorts` |
| P33-07 | Usage-vs-quota heatmap — merchants near a limit, as an upsell signal | P33-01 | `cargo test -p stheno-api-platform --test quota_headroom` |
| P33-08 | Control-plane UI: the business dashboard — MRR, GMV, funnel, churn on one page | P33-06 | `npm --prefix apps/platform run build` |

---

## P34 — Support operations (10 tasks) · **D6**

| ID | Task | Deps | Exit |
|---|---|---|---|
| P34-01 | **Write and accept ADR-018** — impersonation: consent model, time box, what is visible, what is forbidden | P29-05 | ADR-018 merged, Accepted, index row present |
| P34-02 | **[BP]** `POST /platform/tenants/{id}/impersonate` issues a **time-boxed, scope-limited** token; expiry is enforced server-side, never by the client | P34-01 | `cargo test -p stheno-api-platform --test impersonation` |
| P34-03 | **[BP]** Impersonation is audited on **start, every write made under it, and end** — a support session must be reconstructable afterwards | P34-02 | `cargo test -p stheno-api-platform --test impersonation_audit` |
| P34-04 | **[BP]** An impersonating operator cannot read payment credentials, cannot change billing, and cannot issue operator credentials | P34-02 | `cargo test -p stheno-api-platform --test impersonation_limits` |
| P34-05 | The merchant sees that a support session happened, in their own dashboard | P34-03 | `cargo test -p stheno-api-admin --test support_session_notice` |
| P34-06 | Per-tenant recent-errors view for support triage | P29-05 | `cargo test -p stheno-api-platform --test tenant_errors` |
| P34-07 | Manual job triggers — reconciliation sweep, webhook replay, search re-index — each audited and rate-limited | P32-05 | `cargo test -p stheno-api-platform --test manual_jobs` |
| P34-08 | Cross-tenant workflow-execution observability (the `/admin/*` view, widened) | P29-05 | `cargo test -p stheno-api-platform --test workflow_oversight` |
| P34-09 | **[BP]** Put one merchant read-only during an incident — writes refused, reads served, reversible | P31-01 | `cargo test -p stheno-server --test read_only_tenant` |
| P34-10 | Control-plane UI: support page — impersonate, errors, jobs, read-only toggle | P34-07 | `npm --prefix apps/platform run build` |

---

## P35 — Platform configuration and themes (9 tasks) · **D7 + D8**

| ID | Task | Deps | Exit |
|---|---|---|---|
| P35-01 | **`custom_theme` and `branding` features in `config/tiers.toml`** — ship the gate before the feature, or the first branding UI ships ungated | — | `cargo test -p stheno-api-core --test tier_features` |
| P35-02 | Theme catalog: which themes this deployment has, with versions | P27-01 | `cargo test -p stheno-api-platform --test theme_catalog` |
| P35-03 | **[BP]** Per-tenant theme assignment (P27-09) — audited, and a tenant on a theme its tier does not include is refused at the write, not at render | P35-02, P35-01 | `cargo test -p stheno-api-platform --test theme_assignment` |
| P35-04 | Theme version pinning per tenant, and a staged rollout across tenants | P35-03 | `cargo test -p stheno-api-platform --test theme_rollout` |
| P35-05 | Global feature-flag catalog and kill switches, deployment-wide | P29-05 | `cargo test -p stheno-api-platform --test global_flags` |
| P35-06 | Region, currency and tax defaults editable from the control plane | P29-05 | `cargo test -p stheno-api-platform --test platform_defaults` |
| P35-07 | Email / SMS provider config and templates | P29-05 | `cargo test -p stheno-api-platform --test messaging_config` |
| P35-08 | Broadcast announcement to every merchant dashboard, with an expiry | P35-05 | `cargo test -p stheno-api-platform --test announcements` |
| P35-09 | Control-plane UI: configuration and themes pages | P35-06 | `npm --prefix apps/platform run build` |

---

## P36 — Hardening and access control (7 tasks) · **D9** · *last, deliberately*

Placed after the surface stops changing shape. Hardening a moving target is done twice.

| ID | Task | Deps | Exit |
|---|---|---|---|
| P36-01 | Operator roles — `owner`, `billing`, `support`, `engineer`; the table already has room | P34-04 | `cargo test -p stheno-api-platform --test operator_roles` |
| P36-02 | **[BP]** Every route declares its required role, and a route with no declaration **fails closed** rather than defaulting to open | P36-01 | `cargo test -p stheno-api-platform --test role_coverage` |
| P36-03 | MFA on the control plane, reusing the dashboard's TOTP implementation | P36-01 | `cargo test -p stheno-api-platform --test operator_mfa` |
| P36-04 | Optional IP allowlist and session TTL on operator credentials | P36-01 | `cargo test -p stheno-api-platform --test operator_network_limits` |
| P36-05 | Retire the bootstrap token once ≥2 operators exist: a loud warning, and a config flag to disable it | P36-03 | `cargo test -p stheno-api-platform --test bootstrap_retirement` |
| P36-06 | RLS-audit and PCI-SAQ-A evidence surfaced on the control plane instead of only in CI | P29-05 | `cargo test -p stheno-api-platform --test compliance_evidence` |
| P36-07 | Control-plane runbook: incident response, credential compromise, purge reversal | P36-05 | `./scripts/check-docs-links.sh` |

---

## 2. Totals and sequencing

| Phase | Domain | Tasks | Blocked on |
|---|---|---:|---|
| **P29** operator identity + audit | D9 | 10 | — (**complete**) |
| **P30** subscription billing | D3 | 17 | P29 |
| **P31** tenant lifecycle | D1 | 12 | P30 for enforcement |
| **P32** money + fulfillment oversight | D4, D5 | 14 | **P21–P24** |
| **P33** growth analytics | D10 | 8 | — (parallel) |
| **P34** support operations | D6 | 10 | P31 |
| **P35** configuration + themes | D7, D8 | 9 | P27 for themes |
| **P36** hardening | D9 | 7 | P34 |
| | | **87** | |

```
P29 (10) ──┬── P30 (17) ── P31 (12) ──┬── P34 (10) ── P36 (7)
           │                          │
           ├── P32 (14) ──────────────┤        [needs P21–P24]
           │                          │
           ├── P33 (8) ───────────────┘        [parallel, read-only]
           │
           └── P35 (9)                          [needs P27 for themes]
```

## 3. What is next

1. **P30-01** — ADR-017. The billing decision is the largest open question in the product
   and it is a document, not code. Write it before anyone writes an invoice table.
2. **P35-01** — one line of TOML. The branding gate must exist before P26 ships a branding
   UI, and it costs nothing today.
3. **P32-05** — schedule `reconcile_unsettled_requests`. The sweep exists and nothing
   calls it periodically. This one does not wait for P21: the method is already written.

## 4. What is blocked on someone else

1. **Merchant-side recurring payment agreements.** P30-08 needs bKash tokenised /
   agreement-based collection, which is a separate merchant application from the checkout
   integration. Blocks P30-08 only.
2. **VAT registration and Mushak-6.3 format.** P30-06 needs the business's own BIN and the
   current form layout. Blocks P30-06 only; the rest of P30 proceeds with VAT at zero.
3. **P21–P24** for all of P32. Not a third-party block — a sequencing one.
