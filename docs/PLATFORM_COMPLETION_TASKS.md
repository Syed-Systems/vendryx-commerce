# Platform completion — the 23 handlers between 74% and 100%

**Status:** Plan · **Date:** 2026-09-01 (Updated 2026-09-08) · **Refines:** [CONTROL_PLANE.md](CONTROL_PLANE.md) · **Space Target:** [SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md](SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md) · **Decisions:** [ADR-030](DECISIONS.md#adr-030)

> [!NOTE]
> **Context — Transition to Space (2026-09-08):**  
> The company control plane architecture has transitioned to **Space**, an independent project at
> `D:\Syed Systems\Space`. These 23 handlers complete the in-repo `stheno-api-platform` surface for Vendryx
> Commerce, providing the verified baseline that will be exposed to Space through versioned product connectors.

---

## 1. Where the control plane actually is

Two measurements, because they disagree and the gap is the point.

| Measure | Value | What it counts |
|---|---:|---|
| Surface coverage | **59 / 63 (94%)** | capabilities in CONTROL_PLANE.md §2 with a live route answering 200 |
| Substance | **66 / 89 (74%)** | handlers that read or write real data rather than returning literals |

CONTROL_PLANE.md §3 used to report **7 / 63 (11%)**, dated 2026-08-21 — three weeks stale, and
understating the work badly, since D1, D2, D3, D9 and D10 were largely built by then.
**Replaced with the table above on 2026-09-02.**

The 20-point gap between surface and substance is one specific failure mode: a route exists,
answers 200, returns a shape the frontend renders — and the values are hardcoded. Verified
example, `GET /platform/settlements/cod` on a database with zero courier remittances:

```
{"cod_settlements":[{"courier_id":"fp_steadfast",
  "collected_by_courier":{"amount":"12500.00","currency":"BDT"}, ...
```

Those literals are written at [oversight.rs:414-420](../crates/api/stheno-api-platform/src/oversight.rs#L414).
This is the same class of defect `apps/admin`'s e2e suite already guards against with
*"previously stubbed endpoints return real data"*. The control plane has no such guard.

## 2. Where the gap is concentrated

| Module | Handlers | Backed | Stubbed | Domain |
|---|---:|---:|---:|---|
| `oversight.rs` | 14 | 2 | **12** | D4 payments · D5 fulfillment |
| `support.rs` | 8 | 5 | **3** | D6 support ops |
| `lib.rs` | 26 | 24 | 2 | D1 tenants · D3 billing |
| `merchant.rs` | 6 | 4 | 2 | merchant launch (in flight) |
| `security.rs` | 8 | 6 | 2 | D9 access |
| `themes_and_config.rs` | 11 | 9 | 2 | D7 themes |
| `analytics.rs` | 6 | 6 | 0 | D10 growth |
| `launch_requests.rs` | 3 | 3 | 0 | merchant launch |
| `operators.rs` | 4 | 4 | 0 | D9 access |
| `tiers.rs` | 3 | 3 | 0 | D2 plans |
| **Total** | **89** | **66** | **23** | |

Analytics, tiers, operators and launch-requests are genuinely finished. The weakness is
exactly where CONTROL_PLANE.md predicted it: the domains it marked *"depends on P21–P24"*
and *"depends on P23"* have routes and DTOs but no wiring.

## 3. The tasks

Effort is **S** (wire an existing table), **M** (cross-tenant aggregation), **L** (needs
storage that does not exist yet). Ordered so the S tasks land first — most of the remaining
26 points are wiring, not building.

### Group A — payments oversight · 8 tasks · D4

The backing tables all exist. These are cross-tenant reads over data the payment module
already writes; nothing new has to be stored.

| # | Handler | Reads | Effort |
|---|---|---|---|
| A1 | `list_providers_oversight` | `payment.payment_provider` + `payment_provider_credential`, grouped by tenant | S |
| A2 | `get_provider_health` | `payment_provider_credential.credential_health` + last outcome | S |
| A3 | `list_outbound_ledger` | `payment.payment_provider_request`, filter `state` | S |
| A4 | `get_sweep_status` | `payment_provider_request` where `state='in_flight'`, oldest age | S |
| A5 | `trigger_reconciliation_sweep` | invoke the existing sweep, return what it converged | M |
| A6 | `list_webhook_oversight` | `payment.payment_webhook_event`, delivery + replay state | S |
| A7 | `replay_dead_letter_webhook` | re-dispatch one `payment_webhook_event` | M |
| A8 | `list_settlement_reconciliations` | `payment.payment_reconciliation_run` | S |

### Group B — fulfillment oversight · 3 tasks · D5

| # | Handler | Reads | Effort |
|---|---|---|---|
| B1 | `list_couriers_oversight` | `fulfillment.fulfillment_provider_request` grouped by provider | S |
| B2 | `list_stranded_consignments` | same table, `state='in_flight'` past threshold, cross-tenant | S |
| B3 | `list_unknown_status_alarms` | same table, unmapped courier status (P23-11) | M |

### Group C — COD settlement · 1 task · D5

| # | Handler | Reads | Effort |
|---|---|---|---|
| C1 | `list_cod_settlements` | join COD orders against courier remittance; **no settlement table exists** — decide aggregate-on-read vs. a `platform.cod_settlement` table | **L** |

CONTROL_PLANE.md calls COD reconciliation *"highest value-at-risk in the product"*. It is
currently four hardcoded numbers.

### Group D — support operations · 3 tasks · D6

| # | Handler | Reads | Effort |
|---|---|---|---|
| D1 | `get_tenant_errors` | recent errors/activity for one tenant | M |
| D2 | `trigger_manual_job` | dispatch sweep / webhook replay / search re-index by name | M |
| D3 | `list_platform_workflows` | workflow executions cross-tenant — **confirm the source table**; none matched `%workflow%` in the dev schema | **L** |

### Group E — themes · 2 tasks · D7

| # | Handler | Reads | Effort |
|---|---|---|---|
| E1 | `list_theme_catalog` | **no `theme` catalog table** — `platform.tenant_theme` holds assignment only | **L** |
| E2 | `rollout_theme_version` | needs E1 plus version pinning | **L** |

### Group F — access & compliance · 2 tasks · D9

| # | Handler | Reads | Effort |
|---|---|---|---|
| F1 | `get_operator_role` | `platform.operator.role` — the write path exists, the read is literal | S |
| F2 | `get_compliance_evidence` | derive from the RLS audit + PCI posture rather than asserting it | M |

### Group G — platform payment providers · 2 tasks · D4

| # | Handler | Reads | Effort |
|---|---|---|---|
| G1 | `list_platform_payment_providers` | the registry, cross-tenant | S |
| G2 | `upsert_platform_payment_provider_credentials` | write through `PaymentService::upsert_provider_credential` | S |

### Group H — merchant launch payment · 2 tasks · in flight

| # | Handler | Reads | Effort |
|---|---|---|---|
| H1 | `initiate_payment` | start a real gateway attempt for a Pro launch request | M |
| H2 | `payment_status` | read that attempt's state | S |

**Owned by whoever is building the launch workflow.** Listed for completeness, not for
reassignment.

### Group I — the guard that stops this recurring · 1 task

| # | Task | Effort |
|---|---|---|
| I1 | A control-plane e2e test mirroring the dashboard's *"previously stubbed endpoints return real data"* — seed a known row, assert the platform endpoint reflects **it** and not a literal | M |

Without I1 the 26-point gap can silently reopen: every stub here answers 200 today.

## 4. Sequencing

```
Group A (8, mostly S) ──┐
Group B (3, mostly S) ──┼── Group I  ← the guard, once there is something true to assert
Group F, G (4, S)     ──┘
                         Group D (3, M)
                         Group C, E (3, L — need a storage decision first)
                         Group H (2)  ← launch-workflow owner
```

**A + B + F + G is 17 of the 23**, nearly all S, all against tables that already exist. That
alone moves substance from 74% to about 93%. C, E and D3 are the genuine L items: each needs
a decision about where the data lives before code is worth writing.

## 5. Scope and ownership

`crates/api/stheno-api-platform`, `crates/modules/stheno-platform` and `apps/platform` were
under active edit while this plan was written — `App.tsx` changed 47 seconds before this
paragraph. **Two agents writing these eight files at once will lose work.** The split has to
be agreed before any of it starts; this document is the plan, not a claim on the files.
