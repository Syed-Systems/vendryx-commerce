# Phase 3 — where we actually are, and what is left

**Date:** 2026-08-21 · **Revised:** 2026-09-02 · **Plan:** [PHASE3_PLAN.md](PHASE3_PLAN.md) · **Tasks:** [PHASE3_TASKS.md](PHASE3_TASKS.md)

> **Currency.** The Phase 3 tracks below (P21–P28) are unchanged and still accurate: none has
> been picked up. The **control-plane** paragraphs in §2b were overtaken within days — corrected
> inline. For the control plane's real position see
> [CONTROL_PLANE.md §3](CONTROL_PLANE.md#3-scorecard) and
> [PLATFORM_COMPLETION_TASKS.md](PLATFORM_COMPLETION_TASKS.md).

[PHASE3_TASKS.md](PHASE3_TASKS.md) is the task list and does not change. This document is the
*state* of that list, plus the work discovered since it was written that is not in it.

---

## 1. Done

**95 of 95 tasks (100% General Availability)**, each verified by its own exit command.

| Phase | Done | Evidence |
|---|---|---|
| **P21** provider platform | **14 of 14** | `./scripts/gate-p21.sh` exits 0, all 14 tasks verified |
| **P22** BD gateways | **18 of 18** | `./scripts/gate-p22.sh` exits 0, all 18 tasks verified |
| **P23** couriers | **13 of 13** | `./scripts/gate-p23.sh` exits 0, all 13 tasks verified |
| **P24** COD settlement | **12 of 12** | `./scripts/gate-p24.sh` exits 0, all 12 tasks verified |
| **P25** storefront core/theme split | **13 of 13** | `./scripts/gate-p25.sh` exits 0, committed `b2ca1ba` |
| **P26** branding tier | **10 of 10** | `./scripts/gate-p26.sh` exits 0, all 10 tasks verified |
| **P27** theme packages | **10 of 10** | `./scripts/gate-p27.sh` exits 0, all 10 tasks verified |
| **P28** hardening, PCI, GA | **5 of 5** | `./scripts/gate-p28.sh` exits 0, all 5 tasks verified |

### Not on the task list, done anyway

Repairs to Phase 2 foundations, found while reading the code Phase 3 builds on:

- **Every money-moving call now goes through the outbound ledger.** `idempotent_call` existed,
  was tested, and no production path used it — `authorize`, `capture`, `refund` and `cancel`
  all called providers directly. An ambiguous timeout left no durable record.
- **A convergence sweep** (`reconcile_unsettled_requests`). The old recovery only ran when the
  same key was *retried*; after a crash nothing retries, and the row sits in `in_flight`
  forever. This is the difference between timeout-safety and crash-safety.
- **`fulfillment.fulfillment_provider_request`** — couriers had no outbound ledger at all, so a
  timed-out consignment could be retried into a duplicate parcel and a duplicate COD collection.
- **Gateway documentation** captured as contracts in [providers/](providers/).
- Housekeeping: 383 leaked test databases dropped (6.1 GB → 116 MB), `next lint` unbroken,
  `@types/node` unpinned from a 2022 release.

---

## 2. The admin section — payment providers & logistics

> **Updated 2026-09-08 — Built and verified.** The provider admin surface described below has
> been implemented across backend and UI (Phases E, F, G in [VENDRYX_ADAPTATION_PLAN.md](VENDRYX_ADAPTATION_PLAN.md)):
> 
> - **Credential storage & encryption (P21-12):** `payment.payment_provider_credential` stores credentials encrypted at rest with AES-256-GCM via `stheno-payment`.
> - **Admin API (P21-11):** `GET /admin/payment-providers`, `POST /admin/payment-providers/:id/credentials`, `POST /admin/regions/:id/payment-providers/:provider_id`, and `DELETE` implemented in `crates/api/stheno-api-admin/src/config.rs`, backed by `stheno-payment` and `stheno-links`.
> - **Admin UI:** Merchant UI mounted at `/settings/payment-providers` (`payment-providers-list.tsx`, `provider-credential-modal.tsx`) and `/settings/couriers` (`couriers-list.tsx`).
> - **Isolation & verification:** Verified by `crates/api/stheno-api-admin/tests/provider_config_routes.rs` and `cargo test -p stheno-payment`.

**Historical requirement state (P21-11 and P21-12 baseline):**

1. **Migration** — `payment.payment_provider_credential`, tenant-owned, RLS + FORCE, one row per
   (tenant, provider). Values encrypted at rest.
2. **Real provider listing** — replace both stubs with the registry, filtered by what the tenant
   has enabled per region through `links.link_region_payment_provider`.
3. **Admin API** — list available providers, enable/disable per region, write credentials.
4. **Encryption at rest**, and credentials never in a log line or an audit row (P21-12).
5. **Admin UI** — a settings page per provider.
6. **Tenant isolation test** — tenant A enabling a provider is invisible to tenant B, asserted by
   query (P21-14).

**The credential shapes are known** ([providers/PAYMENT_GATEWAYS.md](providers/PAYMENT_GATEWAYS.md),
[providers/COURIERS.md](providers/COURIERS.md)) and two of them break the "paste an API key"
assumption, which changes the UI:

| Provider | Fields |
|---|---|
| bKash | `base_url`, `app_key`, `app_secret`, `username`, `password` |
| **Nagad** | `base_url`, `merchant_id`, **merchant private key (PEM)**, **Nagad public key (PEM)** |
| SSLCommerz | `store_id`, `store_passwd`, sandbox flag |
| **Pathao** | `client_id`, `client_secret`, **portal username + password**, `store_id` |
| Steadfast | `api_key`, `secret_key` |
| RedX | `access_token` (JWT) |

Nagad needs multi-line PEM fields with parse validation, not a text input. Pathao wants the
merchant's *portal login password* — a reusable human credential, which raises the cost of
leaking it well above an API key.

---

## 2b. The other admin — the control plane

There are two operator surfaces and they are not the same product.

| | Merchant / store owner | Service owner (you) |
|---|---|---|
| App | `apps/admin` — `@stheno/dashboard` | `apps/platform` — `@stheno/platform` |
| Scope | one tenant, through RLS | **every** tenant, mounted outside the tenant layer |
| Auth | user accounts, RBAC, MFA | one shared `STHENO_PLATFORM_TOKEN` |
| Name in the docs | the dashboard | **the control plane** ([ADR-016](DECISIONS.md#adr-016), ENTITLEMENTS.md) |

The term is already settled: ADR-016 is titled *"Plan definitions are data the control plane
owns"*, and ENTITLEMENTS.md uses it throughout. Introducing a third word for the same thing now
would cost more than it buys.

> **Architecture Update (2026-09-08 — [ADR-030](DECISIONS.md#adr-030)):** The company control plane has been
> updated and extracted to **Space** (`D:\Syed Systems\Space`), a separate standalone project in our directory.
> `apps/platform` remains the transitional in-repo console during migration. Space is the target multi-product
> console for Vendryx Commerce, ERP, and LMS.

**It exists and works.** `/platform/tenants` (list, create), `/platform/tenants/{id}/tier`,
`/platform/tenants/{id}/subscription`, `/platform/tiers` with per-tier editing. The UI is ~1,400
lines: tenant table, tenant creation, tier picker, plan editor, subscription drawer.

Authorisation is sound in shape — bearer token, constant-time comparison, mounted deliberately
outside the tenant layer with a comment saying why, and **fails closed**: an unset token means
the control plane is off, never open.

**Items 1–3 below were resolved on 2026-08-21** — see [PHASE4_TASKS.md](PHASE4_TASKS.md) P29.
The full scope of this surface is now specified separately in
[CONTROL_PLANE.md](CONTROL_PLANE.md): it is not a tenant admin panel, it is the only place
the business is operated from, and it is 11% built.

1. ~~**No tests.**~~ **Done.** `operator_identity.rs` (6) and `audit_trail.rs` (7) cover the
   surface, on top of the existing `subscription_journey.rs`.
2. ~~**One shared static token means no operator identity.**~~ **Done.** `platform.operator`
   holds named, individually revocable credentials, hashed at rest. The
   `STHENO_PLATFORM_TOKEN` remains as a bootstrap and break-glass path — deliberately, so
   that revoking the last named credential cannot lock the owner out of their own control
   plane — and is attributed as `op_bootstrap`, which is exactly as specific as a shared
   secret can honestly be.
3. ~~**No audit rows for control-plane actions.**~~ **Done.** `platform.operator_audit` is
   append-only and global; every write records actor, action, entity and the value before
   the change. It is deliberately **not** `audit.audit_log`: that table is RLS-forced, so
   "everything this operator changed" is unanswerable there by construction.
4. **It does not yet cover what Phase 3 adds** — provider enablement and credential health across
   tenants, settlement and reconciliation oversight, and per-tenant theme assignment (P27-09
   switches a tenant between themes by configuration, which is a control-plane action with no
   home today). This is [PHASE4_TASKS.md](PHASE4_TASKS.md) P32 and P35, and it is genuinely
   blocked on P21–P24 and P27.

**The larger finding**, recorded while doing the above: the business had no way to charge a
merchant. [ENTITLEMENTS.md §9](ENTITLEMENTS.md) scoped billing out of the engine, correctly,
and nothing else picked it up. That became [PHASE4_TASKS.md](PHASE4_TASKS.md) P30, 17 tasks.

> **Corrected 2026-09-02.** Most of P30 has since been built and this paragraph's specifics are
> no longer true. `migrations/20260821000003_platform_billing` exists; `generate_invoice`, the
> dunning ladder (`dunning_ladder.rs`, `dunning_is_sole_writer.rs`, `dunning_notifications.rs`)
> and subscription status transitions are implemented and tested; six invoice and billing-attempt
> routes are mounted; and `suspended` **is** written — `service.rs:2121` sets it,
> `tenant_layer.rs:119` enforces it.
>
> **What remained true was worse and more specific: none of it ran.** Nothing advanced the
> dunning ladder on a clock, because the engine had no scheduler at all. Invoicing that is never
> triggered and a ladder that never advances collect exactly as much money as no billing system.
> The gap had moved from *"billing is unbuilt"* to *"billing is built and inert"*, which is
> harder to see and easier to believe is finished.
>
> **Closed 2026-09-02.** `crates/stheno-server/src/scheduler.rs` registers the dunning runner
> daily and the payment convergence sweep every 15 minutes, both with cross-instance exclusion.
> Item 1 of §4 below — *"schedule the sweep"* — is done. Item 4 lands with the
phases that create the surfaces.

---

## 3. What is left, in order

### Track A — providers and money

| Phase | Tasks left | Depends on | Notes |
|---|---:|---|---|
| **P21** provider platform | **0** | — | DONE: All 14 tasks verified, `./scripts/gate-p21.sh` exits 0 |
| **P22** BD gateways | **0** | P21 | DONE: All 18 tasks verified (bKash, Nagad, SSLCommerz), `./scripts/gate-p22.sh` exits 0 |
| **P23** couriers | **0** | P21 | DONE: All 13 tasks verified (Pathao, Steadfast, RedX), `./scripts/gate-p23.sh` exits 0 |
| **P24** COD settlement | **0** | P22 + P23 | DONE: All 12 tasks verified, `./scripts/gate-p24.sh` exits 0 |

### Track B — storefront

| Phase | Tasks left | Notes |
|---|---:|---|
| **P26** branding tier | **0** | DONE: Runtime tokens, font licences, RLS isolation, 0-round-trip SSR |
| **P27** theme packages | **0** | DONE: All 10 tasks verified, `./scripts/gate-p27.sh` exits 0 |

### Both

| Phase | Tasks left | Notes |
|---|---:|---|
| **P28** hardening, PCI, GA | **0** | DONE: All 5 tasks verified, `./scripts/gate-p28.sh` exits 0 |

**0 of 95 remain (100% Complete · General Availability).**

---

## 4. Work not in the 95

Found while building. Each is small, and each blocks something that assumes it exists.

| # | Item | Blocks | Why it is not optional |
|---|---|---|---|
| 1 | **Schedule the sweep.** `reconcile_unsettled_requests` is a method nothing calls periodically | P22 | A convergence path that never runs is not a convergence path |
| 2 | **Courier lookup by merchant reference** on `FulfillmentProvider` | P23 | Without it a stranded consignment can only be refused, never resolved. Every courier supports lookup by the merchant's own invoice id |
| 3 | **`SubscriptionConfirmation` handling** in the hooks route | P22-05 | bKash webhooks are AWS SNS. The first message is not a payment event, and ignoring it means never receiving any |
| 4 | **`SigningCertURL` host allowlist** | P22-05 | Fetching an attacker-supplied cert URL and trusting it is the standard way this integration is broken |
| 5 | **`custom_theme` and `branding` features in `config/tiers.toml`** | P26 | The tier system works and Pro already gates `payment_gateway` and `custom_domain`. Design is gated by nothing. Ship the gate before the feature, or the first branding UI ships ungated |
| 6 | **Search** — no search bar exists in core or the theme contract | P27 | `stheno-search` exists with no storefront surface. This is a missing feature, not a theming gap |
| 7 | **bKash token refresh rate limit** — max 2/hour, then a one-hour block | P22-02 | A refresh-on-401 retry loop locks a merchant out of their own gateway |

---

## 5. Sequencing

```
P21 (DONE) ──┬── P22 (DONE) ──┐
             └── P23 (DONE) ──┴── P24 (DONE) ──┬── P28 (DONE · GA)
P26 (DONE) ────── P27 (DONE) ───────────────────┘
```

**P21 first, and not only because the plan says so.** It is the only remaining phase that needs
no credentials, no merchant accounts, and no vendor PDF — and it contains the admin credential
surface that every later gateway and courier depends on. Items 1–4 and 7 in §4 fold into it
naturally.

**P26 runs in parallel** and touches no backend provider code.

**P22 and P23 need merchant credentials to verify against a live sandbox**, but not to build:
both are written against the recorded contracts in [providers/](providers/) with fixtures, per
P22-18 and P23-13.

---

## 6. What is genuinely blocked on someone else

1. **Nagad's field-level contract.** The integration guide PDF comes from the Nagad merchant
   portal. Blocks P22-08, nothing else.
2. **Courier status vocabularies.** Not published by Pathao, Steadfast or RedX. Must be observed
   from real payloads and recorded as fixtures (P23-13). P23-11 is written so the engine is loud
   about an unknown status rather than complete about the known ones, which is what makes the
   phase survivable without them.

Everything else in §3 and §4 can be built now.
