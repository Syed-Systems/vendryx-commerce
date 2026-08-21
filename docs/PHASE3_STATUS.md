# Phase 3 — where we actually are, and what is left

**Date:** 2026-08-21 · **Plan:** [PHASE3_PLAN.md](PHASE3_PLAN.md) · **Tasks:** [PHASE3_TASKS.md](PHASE3_TASKS.md)

[PHASE3_TASKS.md](PHASE3_TASKS.md) is the task list and does not change. This document is the
*state* of that list, plus the work discovered since it was written that is not in it.

---

## 1. Done

**15 of 95 tasks**, each verified by its own exit command.

| Phase | Done | Evidence |
|---|---|---|
| **P25** storefront core/theme split | **13 of 13** | `./scripts/gate-p25.sh` exits 0, committed `b2ca1ba` |
| **P26** branding tier | 1 of 9 | P26-01 — `rls_audit` 4/4 |
| **P21** provider platform | 1 of 14 | P21-01 — `payment_webhook_route` 2/2 |

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

## 2. The admin section — it does not exist yet

This is the question that keeps coming up, so it gets its own section.

**Current state: two hardcoded stubs.**

| Endpoint | Reality |
|---|---|
| `GET /admin/payment-providers` | `vec![]` of three literals — `pp_system_default`, `stripe`, `manual_payment` |
| `GET /store/payment-providers` | a different hardcoded pair — `pp_system_default`, `pp_stripe_stripe` |

Neither reads the provider registry. Neither is tenant-scoped. There is no credential storage,
no enable/disable, and no UI. `apps/admin/src/routes/` has no provider or payments settings page.

**This is P21-11 and P21-12**, and it is the thing that makes every gateway usable by a merchant
rather than by us. It needs, in order:

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

**The larger finding**, recorded while doing the above: the business has no way to charge a
merchant. [ENTITLEMENTS.md §9](ENTITLEMENTS.md) scoped billing out of the engine, correctly,
and nothing else picked it up. An operator can move a merchant onto Pro; nothing then invoices
them, nothing notices non-payment, and `platform.tenant.status` has a `suspended` value that no
code path writes. That is [PHASE4_TASKS.md](PHASE4_TASKS.md) P30, 17 tasks, and it is the
largest open question in the product. Item 4 lands with the
phases that create the surfaces.

---

## 3. What is left, in order

### Track A — providers and money

| Phase | Tasks left | Depends on | Notes |
|---|---:|---|---|
| **P21** provider platform | 13 | — | Includes the admin section above. **No credentials needed** |
| **P22** BD gateways | 18 | P21 | bKash and SSLCommerz fully specified; Nagad needs one PDF |
| **P23** couriers | 13 | P21 | Forks off P21, not P22 |
| **P24** COD settlement | 12 | P22 + P23 | Highest value-at-risk code in the phase |

### Track B — storefront

| Phase | Tasks left | Notes |
|---|---:|---|
| **P26** branding tier | 8 | Runtime tokens: colour, font, logo, store name per tenant, no rebuild |
| **P27** theme packages | 10 | A real second theme, conformance pack, per-tenant deploy, theme switching |

### Both

| Phase | Tasks left | Notes |
|---|---:|---|
| **P28** hardening, PCI, GA | 5 | CSP origins for every new gateway, SAQ-A re-attestation, failure modes, runbooks |

**80 of 95 remain.**

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
now ── P21 (13) ──┬── P22 (18) ──┐
                  └── P23 (13) ──┴── P24 (12) ──┐
                                                ├── P28 (5)
     P26 (8) ── P27 (10) ─────────────────────┘
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
