# The Control Plane — the business's only control centre

**Status:** Draft · **Date:** 2026-08-21 · **Refines:** [ADR-012](DECISIONS.md#adr-012),
[ADR-016](DECISIONS.md#adr-016) · **Tasks:** [PHASE4_TASKS.md](PHASE4_TASKS.md)

---

## 1. What this document decides

`apps/platform` began as a tenant list with a tier dropdown. This document widens it to
what it actually has to be: **the only place the business is operated from.**

Not "an admin panel for tenants". The whole company runs on this surface — provisioning,
getting paid, chasing the merchants who have not paid, watching money move through six
gateways and three couriers, answering support tickets, deciding what a plan costs, and
knowing which merchants are about to leave.

There is no second tool. There is no spreadsheet that is the real answer. If a business
question cannot be answered here, it cannot be answered.

### 1.1 The two surfaces, restated

| | **Dashboard** — `apps/admin` | **Control plane** — `apps/platform` |
|---|---|---|
| Who | the merchant, running one store | the business, running the platform |
| Scope | one tenant, enforced by RLS | every tenant, mounted outside the tenant layer |
| Auth | user accounts, RBAC, MFA | named operators (P29), previously one shared token |
| Failure mode | one merchant sees wrong data | **every** merchant's data is one bug away |

The asymmetry in that last row is the reason this document exists. The dashboard has had
RBAC and MFA since P14. The surface that can read and rewrite every merchant's billing had
a single static string, no identity, and no audit trail.

### 1.2 What the business actually is

From [PRODUCT.md §1](PRODUCT.md): Bangladeshi social-commerce sellers, 20–300 products,
running a business out of a Facebook page. They pay a **monthly subscription** for the
paid tier — courier integration and COD fraud protection are the two acute daily losses,
and they are what the paid tier sells.

That makes this a **subscription business whose customers are merchants**, and it has two
completely separate money flows that are routinely confused:

| | Flow A — the merchant's commerce | Flow B — our subscription revenue |
|---|---|---|
| Who pays whom | a buyer pays a merchant | a merchant pays **us** |
| Rails | bKash, Nagad, SSLCommerz, COD | bKash, SSLCommerz (recurring) |
| Built? | Phase 2 + Phase 3, extensively | **Nothing. Not one line.** |
| Lives in | `payment` module, tenant-scoped | must live in `platform`, cross-tenant |
| Control plane's job | *oversight* — is money moving, is it reconciled | *operation* — invoice, collect, dun, suspend |

**Flow B does not exist.** [ENTITLEMENTS.md §9](ENTITLEMENTS.md) says explicitly:
"No usage-based metering or billing. Entitlements say what is *allowed*; billing systems
say what is *owed*. Integrate a billing provider; do not build one here."

That was a correct scoping decision for the engine and it leaves the business with no way
to charge anyone. Today an operator can move a merchant onto Pro; nothing then bills them,
nothing notices when they do not pay, and nothing moves them back off. **Phase 4 P30 is
where that gets resolved** — and the resolution is deliberately *thin*: collect through
the gateways already integrated for Flow A, and do not build a billing engine.

---

## 2. The ten domains

Everything the control plane must eventually do, grouped by the question it answers.
Status is as of 2026-08-21.

### D1 — Tenant lifecycle · *"who are our merchants and what state are they in"*

| Capability | State |
|---|---|
| Provision a merchant (tenant + subscription + domain, one scope) | ✅ `POST /platform/tenants` |
| List every merchant with its plan | ✅ `GET /platform/tenants` |
| Search, filter, sort, paginate that list | ❌ returns all rows, unsorted |
| Suspend / reactivate / close | ❌ `status` CHECK accepts `active,suspended,closed`; **nothing writes it** |
| Soft-delete, retention window, hard purge | ❌ `deleted_at` exists, no route |
| Domain management — add, remove, re-point primary, TLS state | ❌ one domain, create-time only |
| Merchant health at a glance — orders/day, GMV, last activity | ❌ |
| Onboarding funnel state — created → product → payment → first order | ❌ |
| Support impersonation, time-boxed and audited | ❌ |

The suspend gap is the sharpest one: a merchant who stops paying cannot be turned off, so
the business has no enforcement behind its own pricing.

### D2 — Plans, subscriptions, entitlements · *"what is each merchant entitled to"*

| Capability | State |
|---|---|
| Tier definitions as data, editable without a release ([ADR-016](DECISIONS.md#adr-016)) | ✅ `GET/PUT/DELETE /platform/tiers` |
| Per-merchant quota / window / feature overrides | ✅ `PATCH .../subscription` |
| Effective-state preview — plan × overrides, resolved | ✅ same resolver the request path uses |
| Live on every node with no restart, no cache | ✅ read per request |
| Write-boundary key validation — a typo is a 400, not a silent outage | ✅ |
| Trials: start, extend, expire | ⚠️ dates are patchable; nothing *expires* them |
| Upgrade / downgrade with proration | ❌ needs D3 |
| Current usage against each quota, per merchant | ❌ counters exist; no cross-tenant view |

This is the one domain that is genuinely finished, and it is finished well.

### D3 — Subscription billing · *"do we get paid"* — **the largest gap**

| Capability | State |
|---|---|
| Plan pricing (amount, currency, interval) as data | ❌ tiers have no price at all |
| Invoice generation on a billing cycle | ❌ |
| Collection from merchants via bKash / SSLCommerz | ❌ |
| Dunning ladder: fail → retry → grace → suspend | ❌ `status='grace'` exists; nothing drives it |
| Proration on mid-cycle plan change | ❌ |
| Credits, discounts, coupons on the subscription | ❌ |
| VAT / Mushak-6.3 (BD, 15%) | ❌ |
| Revenue reporting — MRR, ARR, churn, expansion, LTV | ❌ |
| Payment-method-on-file for a merchant | ❌ |

Nine of nine missing. This is what turns a working product into a business.

### D4 — Payments oversight · *"is money moving correctly for our merchants"*

Flow A, seen from above. Depends on P21–P24.

| Capability | State |
|---|---|
| Provider registry: what exists, what a tenant has enabled per region | ❌ `/admin/payment-providers` returns three hardcoded literals |
| Per-tenant gateway credentials, encrypted at rest | ❌ P21-11/12 |
| Credential health — valid / expiring / failing auth | ❌ |
| Webhook delivery health, replay, dead-letter queue | ⚠️ route exists (P21-01); no cross-tenant view |
| Outbound-ledger oversight — `in_flight` / stranded rows, every tenant | ⚠️ ledger + sweep exist; **the sweep is never scheduled** |
| Gateway settlement vs. our ledger reconciliation | ❌ P24 |
| Refund and chargeback oversight | ❌ |
| Global gateway kill switch for an incident | ❌ |

### D5 — Fulfillment oversight · *"are parcels moving"* — depends on P23

| Capability | State |
|---|---|
| Courier registry + per-tenant credentials | ❌ |
| `fulfillment.fulfillment_provider_request` stranded consignments, cross-tenant | ⚠️ table + ledger landed 2026-08-21; no view |
| Lookup by merchant reference — resolve a stranded consignment | ❌ §4 item 2 of PHASE3_STATUS |
| Unknown-courier-status alarm (P23-11) | ❌ |
| COD collection reconciliation against courier remittance | ❌ P24 — highest value-at-risk in the product |

### D6 — Support operations · *"a merchant is on the phone"*

| Capability | State |
|---|---|
| Impersonate a merchant, time-boxed, consented, audited | ❌ |
| Read one merchant's recent errors and activity | ❌ |
| Re-run a job by hand — reconciliation sweep, webhook replay, search re-index | ❌ |
| Workflow-execution observability across tenants | ⚠️ exists per-tenant on `/admin/*` |
| Put one merchant read-only during an incident | ❌ |

### D7 — Themes and branding · *"what does each storefront look like"* — depends on P26/P27

| Capability | State |
|---|---|
| Theme catalog and per-tenant assignment (P27-09) | ❌ "a control-plane action with no home today" |
| Theme version pinning, staged rollout | ❌ |
| Branding tier gate (`custom_theme`, `branding` in `tiers.toml`) | ❌ §4 item 5 — **the gate must ship before the feature** |

### D8 — Platform configuration · *"deployment-wide settings"*

| Capability | State |
|---|---|
| Global feature-flag catalog and kill switches | ❌ |
| Regions, currencies, tax defaults | ⚠️ modules exist; no control-plane surface |
| Email / SMS provider config and templates | ❌ |
| Broadcast announcement to every merchant | ❌ |

### D9 — Access and compliance · *"who can do this and what did they do"*

| Capability | State |
|---|---|
| Named operator credentials, revocable individually | 🔨 **P29 — landing now** |
| Control-plane audit trail | 🔨 **P29 — landing now** |
| Operator roles (support / billing / engineer) | ❌ deferred; the table has room |
| MFA on the control plane | ❌ |
| IP allowlist, session TTL | ❌ |
| Per-tenant data export / deletion request | ❌ |
| RLS audit + PCI SAQ-A evidence surfaced here | ⚠️ `rls_audit` passes 4/4; not surfaced |

### D10 — Growth analytics · *"is the business working"*

| Capability | State |
|---|---|
| Signup → activation → paid funnel | ❌ |
| Total GMV across every merchant | ❌ |
| Plan distribution, top merchants, cohort retention | ❌ |
| Usage-vs-quota heatmap as an upsell signal | ❌ |
| Churn cohorts and reasons | ❌ |

---

## 3. Scorecard

| Domain | Built | Total | |
|---|---:|---:|---|
| D1 Tenant lifecycle | 2 | 9 | ▓▓░░░░░░░ |
| D2 Plans & entitlements | 5 | 8 | ▓▓▓▓▓▓░░ |
| D3 Subscription billing | 0 | 9 | ░░░░░░░░░ |
| D4 Payments oversight | 0 | 8 | ░░░░░░░░ |
| D5 Fulfillment oversight | 0 | 5 | ░░░░░ |
| D6 Support operations | 0 | 5 | ░░░░░ |
| D7 Themes & branding | 0 | 3 | ░░░ |
| D8 Platform configuration | 0 | 4 | ░░░░ |
| D9 Access & compliance | 0 | 7 | ░░░░░░░ |
| D10 Growth analytics | 0 | 5 | ░░░░░ |
| **Total** | **7** | **63** | **11%** |

---

## 4. Sequencing, and why this order

```
P29 access ──┬── P30 billing ── P31 lifecycle ──┬── P34 support
             │                                  │
             └── P32 money oversight ───────────┴── P35 config
                        (needs P21–P24)          │
                 P33 growth ───────────────────  ┘
                                                 P36 hardening
```

**P29 first.** Everything after it writes billing-affecting changes across every merchant.
Doing that without knowing who acted is not a gap, it is an incident waiting for a second
operator. It also needs nothing: no credentials, no vendor, no other phase.

**P30 second, and it is the business.** A subscription product that cannot invoice is a
hobby. It is placed before lifecycle because *suspend* is meaningless until something can
decide a merchant has not paid.

**P31 third.** Suspend, close, purge, domains — the enforcement end of P30, plus the
lifecycle operations support needs daily.

**P32 waits on P21–P24** and cannot be pulled forward. Its value is oversight of code that
does not exist yet.

**P33 growth runs in parallel** with anything — it is read-only over data already stored.

**P36 last**: MFA, roles, IP allowlist. Deliberately after the surface stops changing
shape, because hardening a moving target is done twice.

---

## 5. The rule this surface is built under

> **One bug here is a cross-tenant bug.** ([ADR-012](DECISIONS.md#adr-012))

Three things follow, and they are not negotiable per-task:

1. **Every write is attributed and audited.** No exceptions, including reads of sensitive
   data once impersonation exists.
2. **Every cross-tenant query is deliberate and reviewed.** The control plane runs outside
   the tenant layer; that is a licence to leak unless each query is written as though it
   were the one that leaks.
3. **Fails closed.** No credential means the surface is off, never open. This already
   holds and must survive every change.

---

## 6. What is deliberately out of scope

- **A billing engine.** P30 collects through gateways already integrated and stores
  invoices. Proration is a formula, not a product. If subscription billing ever needs
  metered usage, revenue recognition, or multi-currency dunning, integrate a vendor —
  ENTITLEMENTS.md §9 was right about that.
- **Operator RBAC beyond a flat role**, until there are more than about five operators.
  Knowing *who* acted is the requirement; restricting *what* they may do is a later one.
- **A merchant-facing billing portal.** Merchants see their plan on `/admin/*`; they do
  not need our invoicing UI.
- **BI tooling.** D10 answers a fixed set of business questions. Anything past that is a
  warehouse and a BI tool, not a Rust route.
