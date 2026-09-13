# Control Plane — remediation plan and full dashboard scope

**Date:** 2026-08-22 (Updated 2026-09-08) · **Scope doc:** [CONTROL_PLANE.md](CONTROL_PLANE.md) ·
**Phase 4 tasks:** [PHASE4_TASKS.md](PHASE4_TASKS.md) · **Space Target:** [SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md](SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md) ·
**Decisions:** [ADR-030](DECISIONS.md#adr-030)

> **Current status source:** [STATUS.md](STATUS.md). This plan is a remediation design and
> should not be used as the live completion dashboard.
>
> [!NOTE]
> **Evolution to Space (2026-09-08):**  
> The company-wide control plane has been updated to **Space**, living as an independent project at
> `D:\Syed Systems\Space`. The handler remediation documented here stabilizes the in-repo `stheno-api-platform`
> surface for Vendryx Commerce, ensuring verified business logic is available to wrap into Space connectors during the
> phased migration.

---

## 1. The finding, in one table

Every capability in CONTROL_PLANE.md now has a route. **66 routes exist under `/platform`.**
The surface is essentially complete. The substance is not, and the split is measurable —
count the handlers in each module against the handlers that call a service:

| Module | Domain | Handlers | Touch a service | Verdict |
|---|---|---:|---:|---|
| `tiers.rs` | D2 plans | 3 | 2 | **real** |
| `operators.rs` | D9 access | 4 | 3 | **real** |
| `analytics.rs` | D10 growth | 6 | 6 | **real** |
| `oversight.rs` | D4 + D5 money | 13 | **1** | 12 hollow |
| `support.rs` | D6 support | 6 | **0** | 6 hollow |
| `themes_and_config.rs` | D7 + D8 config | 11 | **1** | 10 hollow |

**28 of 43 handlers in the three newest modules touch no data at all.** They return literals
or read process-global `HashMap`s. That is the deadwood.

### 1.1 Eight registries that die on restart

```
oversight.rs          killed_gateways
support.rs            sessions              (impersonation tokens)
support.rs            read_only_tenants
themes_and_config.rs  tenant_themes
themes_and_config.rs  global_flags
themes_and_config.rs  platform_defaults
themes_and_config.rs  messaging_config
themes_and_config.rs  announcements
```

Each is a `static OnceLock<RwLock<HashMap<..>>>`. Restart the server and every theme
assignment, every kill switch, every announcement and every live support session is gone. On
more than one node they were never shared to begin with.

`scripts/check-no-in-memory-stores.sh` passes only because it scans `crates/modules/*/src`
and has never looked at `crates/api/`. **P37-01 fixes the gate first**, so this class of
regression cannot recur while the rest of the plan is being worked.

### 1.2 Switches that are wired to nothing

| Switch | Route | Read by |
|---|---|---|
| Emergency tenant read-only | `POST /platform/tenants/{id}/read-only` | **nothing** — no production caller of `is_tenant_read_only` |
| Global gateway kill switch | `POST /platform/gateways/{id}/kill-switch` | only the control plane's own listing route |
| Impersonation token | `POST /platform/tenants/{id}/impersonate` | **nothing** — no merchant surface accepts it |

Their tests pass because each asserts the flag it just set. `read_only_tenant.rs` is titled
*"writes refused, reads served"* and never attempts a write;
`global_gateway_kill_switch_toggles_and_enforces_fail_closed` proves the toggle and not the
fail-closed. This is the exact failure mode [PHASE4_TASKS.md](PHASE4_TASKS.md) §1.2 marks
`[BP]` for.

### 1.3 Numbers that are invented

[oversight.rs](../crates/api/stheno-api-platform/src/oversight.rs) hardcodes five provider
literals — `pp_bkash`, `pp_sslcommerz`, `pp_stripe`, `fp_redx`, `fp_steadfast` — and returns
`total_calls_24h: 124` as provider health. Four of those five providers **have no crate in
this repo**; they exist only as strings in this file and its tests.

This is the same stub pattern P21-L1/L2 spent a phase deleting from `/admin/payment-providers`
and `/store/payment-providers`, reintroduced one crate over.

---

## 2. What "properly works" means

A capability counts as built when all five hold. Anything less is deadwood however green its
test is.

1. **Durable** — state lives in Postgres. Surviving a restart is not a feature, it is the
   floor. No domain state in a process-global map.
2. **Enforced** — a switch changes what the engine does, proven by a **negative test**: set
   the switch, attempt the thing, assert it is refused. Asserting the flag reads back is not
   evidence.
3. **Real** — every number on the dashboard traces to a row. No literal in a handler, no
   invented count, no placeholder timestamp.
4. **Attributed** — every write names the operator and lands in `platform.operator_audit`.
   `./scripts/check-control-plane-audit.sh` must pass.
5. **Honestly absent** — a capability whose engine does not exist yet renders as
   *unavailable*, never as zeros. **A dashboard that invents numbers is worse than one that
   admits a gap**, because the operator cannot tell the two apart.

Rule 5 decides what happens to D4 settlement and D5 courier oversight, and it is not
negotiable: those surfaces get removed and re-added with P24, not filled with
plausible-looking data.

---

## 3. Full scope — all 63 capabilities, re-scored

Legend: ✅ real · ⚠️ hollow (route exists, substance does not) · ❌ missing ·
⛔ blocked on an engine that does not exist

### D1 — Tenant lifecycle · *who are our merchants and what state are they in*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Provision a merchant (tenant + subscription + domain, one scope) | ✅ | ✅ | |
| List every merchant with its plan | ✅ | ✅ | |
| Search, filter, sort, paginate | ❌ | ✅ | `list_tenants_paged` |
| Suspend / reactivate / close | ❌ | ⚠️ | writes land; **unaudited** — P37-10 |
| Soft-delete, retention window, hard purge | ❌ | ✅ | |
| Domain management — add, remove, re-point primary, TLS state | ❌ | ⚠️ | backend real; **UI never wired** — P37-20 |
| Merchant health — orders/day, GMV, last activity | ❌ | ⚠️ | backend real; UI never wired |
| Onboarding funnel state | ❌ | ⚠️ | backend real; UI never wired |
| Support impersonation, time-boxed and audited | ❌ | ⚠️ | see D6 |

### D2 — Plans, subscriptions, entitlements · *what is each merchant entitled to*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Tier definitions as data, editable without a release | ✅ | ✅ | [ADR-016](DECISIONS.md#adr-016) |
| Per-merchant quota / window / feature overrides | ✅ | ✅ | |
| Effective-state preview — plan × overrides, resolved | ✅ | ✅ | same resolver as the request path |
| Live on every node, no restart, no cache | ✅ | ✅ | laid over per request in `tenant_layer.rs` |
| Write-boundary key validation | ✅ | ✅ | |
| Trials: start, extend, expire | ⚠️ | ⚠️ | dates patchable; nothing expires them — P37-24 |
| Upgrade / downgrade with proration | ❌ | ✅ | `Money::allocate_proportionally` |
| Current usage against each quota, per merchant | ❌ | ✅ | `/platform/analytics/quota-headroom` |

**This domain works and is the reference standard for the rest.** Two defects, both small:
`ALL_FEATURES` in [stheno-api-router](../crates/api/stheno-api-router/src/lib.rs) is
`[Feature; 8]` while `Feature` has ten variants — `CustomTheme` and `Branding` are silently
omitted from the map the dashboard reads, against that constant's own stated invariant
(P37-22). And route-mount gating is decided once at boot from process-wide entitlements, so
it cannot vary per tenant (P37-23).

### D3 — Subscription billing · *do we get paid*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Plan pricing as data | ❌ | ✅ | price is a TOML **string**, not a float |
| Invoice generation on a billing cycle | ❌ | ✅ | idempotent on `(tenant_id, period_start)` |
| Collection via bKash / SSLCommerz | ❌ | ✅ | both crates built, wiremock-tested |
| Dunning ladder: fail → retry → grace → suspend | ❌ | ✅ | |
| Proration on mid-cycle plan change | ❌ | ✅ | |
| Credits, discounts, coupons | ❌ | ✅ | |
| VAT / Mushak-6.3 | ❌ | ⛔ | needs the business's BIN and the current form |
| Revenue reporting — MRR, ARR, churn, LTV | ❌ | ⚠️ | **currency bug** — P37-21 |
| Payment-method-on-file for a merchant | ❌ | ⛔ | bKash tokenised agreement is a separate application |

D3 went from 0/9 to 6/9 and is the largest genuine advance on the branch. The one defect is
real money: `get_gmv_metrics` accumulates into `Money::zero("USD")` and folds with
`(total + inv.total).unwrap_or(total)`, so **a BDT invoice is silently dropped from revenue** —
in the only currency the business charges in.

### D4 — Payments oversight · *is money moving correctly for our merchants*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Provider registry — what exists, what a tenant enabled per region | ❌ | ⚠️ | five hardcoded literals — P37-11 |
| Per-tenant gateway credentials, encrypted at rest | ❌ | ✅ | P21: AES-256-GCM, RLS forced |
| Credential health — valid / expiring / failing auth | ❌ | ⚠️ | invented; real derivation exists in admin — P37-12 |
| Webhook delivery health, replay, dead-letter | ⚠️ | ⚠️ | route exists; no real read — P37-13 |
| Outbound-ledger oversight, cross-tenant | ⚠️ | ⚠️ | table is real; the view is not — P37-13 |
| Sweep is actually scheduled | ⚠️ | ⚠️ | `sweep-status` reports; verify the schedule — P37-14 |
| Gateway settlement vs. our ledger | ❌ | ⛔ | **P24 — do not fake it** |
| Global gateway kill switch | ❌ | ⚠️ | in RAM, enforced nowhere — P37-02, P37-06 |

### D5 — Fulfillment oversight · *are parcels moving*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Courier registry + per-tenant credentials | ❌ | ⛔ | **no courier crate exists** — P23 |
| Stranded consignments, cross-tenant | ⚠️ | ⚠️ | ledger table real; view stubbed — P37-15 |
| Lookup by merchant reference | ❌ | ⚠️ | only the `None` branch is tested — P37-16 |
| Unknown-courier-status alarm | ❌ | ⛔ | needs the P23-01 status vocabulary |
| COD reconciliation against courier remittance | ❌ | ⛔ | **P24 — highest value-at-risk in the product** |

D5 is the clearest case for rule 5. Four of five capabilities oversee an engine that does not
exist. **P37-17 removes those surfaces from the dashboard** and they return with P23/P24.

### D6 — Support operations · *a merchant is on the phone*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Impersonate a merchant, time-boxed, consented, audited | ❌ | ⚠️ | tokens in RAM; no surface accepts them — P37-03, P37-07 |
| Read one merchant's recent errors and activity | ❌ | ⚠️ | P37-18 |
| Re-run a job by hand | ❌ | ⚠️ | P37-18 |
| Workflow-execution observability across tenants | ⚠️ | ⚠️ | P37-18 |
| Put one merchant read-only during an incident | ❌ | ⚠️ | in RAM, enforced nowhere — P37-04, P37-05 |

Zero of six handlers in this module touch a service. D6 is entirely deadwood today.

### D7 — Themes and branding · *what does each storefront look like*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Theme catalog and per-tenant assignment | ❌ | ⚠️ | assignment in RAM — P37-08 |
| Theme version pinning, staged rollout | ❌ | ⚠️ | in RAM — P37-08 |
| Branding tier gate (`custom_theme`, `branding`) | ❌ | ⚠️ | flags exist; **omitted from `ALL_FEATURES`** — P37-22 |

The gate shipping before the feature was the right call and it happened. What it gates is
P26/P27 and does not exist yet.

### D8 — Platform configuration · *deployment-wide settings*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Global feature-flag catalog and kill switches | ❌ | ⚠️ | in RAM, never reaches the engine — P37-08, P37-09 |
| Regions, currencies, tax defaults | ⚠️ | ⚠️ | `config/defaults` in RAM — P37-08 |
| Email / SMS provider config and templates | ❌ | ⚠️ | in RAM — P37-08 |
| Broadcast announcement to every merchant | ❌ | ⚠️ | in RAM; no merchant surface reads it — P37-08, P37-19 |

### D9 — Access and compliance · *who can do this and what did they do*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Named operator credentials, revocable individually | 🔨 | ✅ | |
| Control-plane audit trail | 🔨 | ✅ | ULID cursor, no skip/repeat |
| Operator roles (support / billing / engineer) | ❌ | ⚠️ | route exists; verify it denies — P37-25 |
| MFA on the control plane | ❌ | ⚠️ | route exists; verify enrolment is required — P37-25 |
| IP allowlist, session TTL | ❌ | ⚠️ | route exists; verify enforcement — P37-25 |
| Per-tenant data export / deletion request | ❌ | ✅ | export + purge |
| RLS audit + PCI SAQ-A evidence surfaced here | ⚠️ | ⚠️ | verify it reads the real gate — P37-25 |

The bootstrap token stays valid after named operators exist, on purpose, and there is a test
named `an_operator_cannot_lock_everyone_out_by_revoking_the_last_credential` that will stop
anyone who "fixes" it. Retiring it deliberately is P36-05.

### D10 — Growth analytics · *is the business working*

| Capability | Was | Now | Note |
|---|:--:|:--:|---|
| Signup → activation → paid funnel | ❌ | ✅ | |
| Total GMV across every merchant | ❌ | ⚠️ | currency fold drops BDT — P37-21 |
| Plan distribution, top merchants, cohort retention | ❌ | ✅ | |
| Usage-vs-quota heatmap | ❌ | ✅ | |
| Churn cohorts | ❌ | ✅ | |

Six of six handlers read real rows. D10 is real, with one money bug.

### 3.1 Scorecard

| Domain | Real | Hollow | Blocked | Total |
|---|---:|---:|---:|---:|
| D1 Tenant lifecycle | 4 | 5 | 0 | 9 |
| D2 Plans & entitlements | 7 | 1 | 0 | 8 |
| D3 Subscription billing | 6 | 1 | 2 | 9 |
| D4 Payments oversight | 1 | 6 | 1 | 8 |
| D5 Fulfillment oversight | 0 | 2 | 3 | 5 |
| D6 Support operations | 0 | 5 | 0 | 5 |
| D7 Themes & branding | 0 | 3 | 0 | 3 |
| D8 Platform configuration | 0 | 4 | 0 | 4 |
| D9 Access & compliance | 3 | 4 | 0 | 7 |
| D10 Growth analytics | 4 | 1 | 0 | 5 |
| **Total** | **25** | **32** | **6** | **63** |

**25 of 63 real (40%), up from 7.** Thirty-two are hollow and are what P37 exists to fix. Six
are honestly blocked on P22–P24 or on paperwork and must render as unavailable until then.

---

## 4. P37 — the task list

The universal gate from [PHASE4_TASKS.md](PHASE4_TASKS.md) §1.1 applies to every row, **with
the two additions that were being skipped**:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-dto-separation.sh
./scripts/check-schema-drift.sh
./scripts/check-no-in-memory-stores.sh --all
./scripts/check-control-plane-audit.sh        # was never run
npm --prefix apps/platform run typecheck      # was never run; `build` does not typecheck
npm --prefix apps/platform run build
```

`npm run build` is `vite build`. It transpiles and does not type-check. Running it instead of
`typecheck` is how 32 type errors reached the tree under a reported-green gate.

### 4.0 Progress

| ID | State | Evidence |
|---|---|---|
| P37-01 | **done** | gate extended; passes today, fails on a new violation and on a stale ledger entry |
| P37-10 | **done** | not by this plan — landed separately; `check-control-plane-audit.sh` passes, 34 handlers |
| P37-16 | **done** | `cargo test -p stheno-fulfillment --test lookup_by_reference` — 1 passed |
| P37-22 | **done** | `cargo test -p stheno-api-router` — 16 passed, 0 failed |
| P37-26 | **done** | `--test distinct_shipments_are_not_deduped` — RED with `idempotency_key_payload_mismatch`, then 1 passed; `lifecycle` and `lookup_by_reference` unregressed |
| P37-02/03/04/08 | **done** | ledger empty: **0** process-global registries in `crates/api`; all 7 boundary gates pass |
| P37-25 (persistence half) | **done** | operator role/MFA/allowlist on `platform.operator`; the unknown-operator case now denies instead of defaulting to `owner` |

**Class B is complete.** Nine registries became rows across two migrations
([control_plane_persistence](../migrations/20260822000001_control_plane_persistence/),
[operator_security](../migrations/20260822000002_operator_security/)), five entity models,
sixteen service methods and four rewired handler files. The ledger in
`check-no-in-memory-stores.sh` is now empty, which makes `crates/api` permanently subject to
the same rule as `crates/modules`.

**Three defects the move exposed, all fixed on the way through:**

- The impersonation token was `imp_{tenant_id}_{timestamp_nanos}` — derived entirely from
  values an attacker already has. It is 32 random bytes now, and only its SHA-256 is stored.
  `impersonation.rs` had been asserting `starts_with("imp_ten_test_")`, so the test was
  holding the weakness in place; it now asserts the token does **not** contain the tenant id.
- `get_security_profile` answered an unknown operator id with a default carrying
  `role: Owner` — the unknown case was the most privileged one. It returns `Option` now and
  callers deny.
- `set_operator_role` used `or_insert_with`, so a typo in a path segment silently created a
  new privileged principal. A role can only be set on an operator that exists.

**What class B does not do.** The state is durable; it is not yet *enforced*. Nothing reads
`is_tenant_read_only` on the request path and no payment path consults the kill switch. That
is class C, and it is what turns these from records into switches.

Two things the work turned up that were not in the original list:

**`tier_matrix.rs` was already red** and had been since P35-01. Its stated job is to fail when
"a tier quietly gains a feature"; `custom_theme` and `branding` were added to `[pro]` in
`config/tiers.toml` and the expected list was never updated. It did exactly what it was built
to do and nobody was running `cargo test --workspace` to hear it. Fixed with P37-22.

**P37-26 is new** — see 4.7. Found by writing the P37-16 test.

### 4.1 Class A — make the gates true (do these first)

| ID | Task | Exit |
|---|---|---|
| P37-01 | Extend `check-no-in-memory-stores.sh` to scan `crates/api/` as well as `crates/modules/` | passes today, fails on a new or grown violation, fails on a stale ledger entry |

P37-01 comes first on purpose. Until the gate can see `crates/api/`, every fix below can be
undone by the next handler someone writes.

**Implemented as a ratchet, not a hard red.** The plan first called for a gate that fails on
today's tree and goes green after class B. That would have been red for as long as class B
takes, during which the honest response and the dishonest one — weakening the gate — look
equally green, and a red shared gate invites the second. Instead the nine known registries are
recorded in a ledger inside the script, and three things fail: a violation in an unlisted file,
more violations in a listed file than recorded, and a listed file with none left. The third
rule is what stops the ledger outliving the debt: each class-B task deletes its row, and the
gate refuses to keep a row it can no longer justify. The guarantee is the same — no new
process-global state — and it holds from today instead of from the end of class B.

### 4.2 Class B — persistence · **[BP]**

One migration, four tables and one column. Nothing here is a new idea; each replaces a
registry that already exists in memory.

| ID | Task | Exit |
|---|---|---|
| P37-02 | Migration: `platform.global_flag` (id, enabled, reason, updated_by, updated_at) — serves both the gateway kill switch and D8 flags | `./scripts/check-schema-drift.sh` |
| P37-03 | Migration: `platform.impersonation_session` — token **hash**, tenant, operator, scopes, `expires_at`, `revoked_at` | `./scripts/check-schema-drift.sh` |
| P37-04 | `platform.tenant.is_read_only` + `read_only_reason` — tenant state belongs on the tenant, not in a side map | `./scripts/check-schema-drift.sh` |
| P37-08 | Migration: `platform.tenant_theme`, `platform.deployment_config` (singleton: defaults + messaging), `platform.announcement`; move all five `themes_and_config.rs` registries onto them | `cargo test -p stheno-platform --test config_persistence` |

Store the impersonation **token hash**, never the token. A support token sitting in a table an
operator can read is a credential leak with an audit trail attached.

### 4.3 Class C — enforcement · **[BP]** · *write the negative test first*

This is the class that turns switches into switches. Every row's exit command must fail
before the fix and pass after.

| ID | Task | Exit |
|---|---|---|
| P37-05 | Read-only mode enforced in `tenant_layer.rs`: writes refused with a documented status, reads served, reversible | `cargo test -p stheno-server --test read_only_refuses_writes` |
| P37-06 | Kill switch consulted in `PaymentService::get_provider` before any provider is resolved — fails closed | `cargo test -p stheno-payment --test kill_switch_refuses_provider` |
| P37-07 | Impersonation token accepted on the merchant surface with reduced scope, expiring server-side, and the merchant sees a notice ([ADR-018](DECISIONS.md#adr-018)) | `cargo test -p stheno-api-admin --test impersonation_scope_is_limited` |
| P37-09 | Global feature flags reach the engine — a disabled flag changes behaviour, not just the flag list | `cargo test -p stheno-server --test global_flag_is_enforced` |
| P37-19 | An announcement is readable on the merchant surface | `cargo test -p stheno-api-admin --test announcement_visible_to_merchant` |

Supersedes the toggle-only tests `read_only_tenant.rs` and `gateway_kill_switch.rs`. **Do not
edit those to pass** — rule L3. Replace them with tests that attempt the action.

### 4.4 Class D — real data

| ID | Task | Exit |
|---|---|---|
| P37-11 | `list_providers_oversight` reads `PaymentService::list_factories()`. **No provider literal survives in any handler** | `cargo test -p stheno-api-platform --test provider_oversight`, and `pp_bkash` appears zero times in `oversight.rs` |
| P37-12 | Provider health derived from real `payment.payment_provider_request` outcomes — reuse the derivation already written for admin (P21-L3), do not write a second one | `cargo test -p stheno-api-platform --test credential_health` |
| P37-13 | Outbound-ledger and webhook-health views read `payment.payment_provider_request` | `cargo test -p stheno-api-platform --test ledger_oversight` |
| P37-14 | Prove `reconcile_unsettled_requests` is actually scheduled and `sweep-status` reports the real last run | `cargo test -p stheno-api-platform --test sweep_is_scheduled` |
| P37-15 | Stranded-consignment view reads `fulfillment.fulfillment_provider_request` | `cargo test -p stheno-api-platform --test consignment_oversight` |
| P37-16 | `lookup_by_reference` positive case tested — a consignment that exists is found | `cargo test -p stheno-fulfillment --test lookup_by_reference` |
| P37-18 | D6 triage — tenant errors, manual jobs, workflow oversight — read real rows | `cargo test -p stheno-api-platform --test tenant_errors` |
| P37-21 | GMV accumulates in the invoice's own currency; a mixed-currency set returns per-currency totals and **never** silently drops one | `cargo test -p stheno-platform --test revenue_metrics` |

P37-21 is the money row. `unwrap_or(total)` on a currency mismatch must become an error or a
per-currency bucket — never a silent discard.

### 4.5 Class E — honest absence

| ID | Task | Exit |
|---|---|---|
| P37-17 | Remove D4-settlement and D5-courier surfaces from the dashboard; the API returns a documented `unavailable` state and the UI renders "not configured", never zeros. Re-added by P23/P24 | `npm --prefix apps/platform run typecheck`, and no fabricated row renders |

### 4.6 Class F — audit, access, frontend

| ID | Task | Exit |
|---|---|---|
| P37-10 | `close_tenant`, `reactivate_tenant`, `suspend_tenant` call `record()` | `./scripts/check-control-plane-audit.sh` |
| P37-20 | `apps/platform` typechecks clean; the unwired D1 UI (domains, health, onboarding) is connected, or its imports removed and the gap recorded | `npm --prefix apps/platform run typecheck` |
| P37-22 | `ALL_FEATURES` covers every `Feature` variant, enforced by an exhaustive match so it cannot drift again | `cargo test -p stheno-api-router --test all_features_exhaustive` |
| P37-23 | Decide route-mount gating: per-request, or documented as per-deployment with a test asserting no gated group varies by tier | `cargo test -p stheno-server --test tier_matrix` |
| P37-24 | Trials expire — a lapsed trial resolves to the free tier without an operator touching it | `cargo test -p stheno-platform --test trial_expiry` |
| P37-25 | D9 verification sweep: an operator role denies, MFA is required once enrolled, the IP allowlist refuses, compliance evidence reads the real gate | `cargo test -p stheno-api-platform --test operator_identity` |

### 4.7 Found while implementing

| ID | Task | Exit |
|---|---|---|
| P37-26 | **[BP]** `create_fulfillment` must not collapse two distinct shipments onto one idempotency key | `cargo test -p stheno-fulfillment --test distinct_shipments_are_not_deduped` |

`FulfillmentService::create_fulfillment` derives its ledger key as
`create_fulfillment:{provider_id}:{shipping_option_id or "no_reference"}`. Two genuinely
different parcels for the same courier with no shipping option therefore share a key, and the
second is refused with `idempotency_key_payload_mismatch`.

The comment above it says the caller "must supply one to get dedupe rather than merely a
durable record" — but the behaviour is the opposite of that intent. Absent a caller reference
the calls do not lose dedupe, they *share* it, which is worse than having none: a real second
shipment is rejected as a replay. A missing reference should produce a unique key, not a
shared constant.

Found by writing the P37-16 test, which could not create three consignments for one courier.
It is a behavioural change in the P21 provider track's file, so it is written down here rather
than fixed in passing.

---

## 5. Order of work

```
P37-01  gate sees crates/api/          ← first, or everything below can regress
   │
   ├── Class B  persistence (02, 03, 04, 08)          ← 1 migration, 4 tables + 1 column
   │      │
   │      └── Class C  enforcement (05, 06, 07, 09, 19)   ← [BP], negative test first
   │
   ├── Class D  real data (11–16, 18, 21)             ← parallel with B/C, no shared files
   │
   ├── Class E  honest absence (17)                   ← anytime; smallest diff on the list
   │
   └── Class F  audit + frontend (10, 20, 22, 23, 24, 25)
```

**Cheapest first, if you want momentum:** P37-10 (three `record()` calls), P37-22 (two enum
variants), P37-16 (one test), P37-17 (deletion). All four together are under an hour, and two
of them turn a red gate green.

**Highest value:** P37-05 and P37-06. An incident switch that does not switch anything is the
only defect on this list that is worse than not having the feature at all — it will be trusted
in the one moment it matters.

---

## 6. What stays blocked, and by what

| Capability | Blocked on | Kind |
|---|---|---|
| COD settlement reconciliation | P24 | sequencing |
| Courier registry, credentials, status alarms | P23 — no courier crate exists | sequencing |
| Gateway settlement vs. ledger | P24 | sequencing |
| VAT / Mushak-6.3 invoices | the business's BIN + current form layout | paperwork |
| Payment-method-on-file | bKash tokenised agreement application | paperwork |
| Branding and theme surfaces having an effect | P26 / P27 | sequencing |

Six of 63. Every other capability on this list is buildable today with what is already in the
repo — the credential seam, the factory registry, the outbound ledger and the audit trail are
all built and working.

---

## 7. The rule this plan is built under

From [CONTROL_PLANE.md](CONTROL_PLANE.md) §5, unchanged, plus one addition earned this week:

1. Every write is attributed and audited.
2. Every cross-tenant query is deliberate and reviewed.
3. Fails closed. No credential means the surface is off, never open.
4. **A control that does not control anything is a defect, not a placeholder.** Ship it wired,
   or ship it visibly absent. Never ship it looking finished.
