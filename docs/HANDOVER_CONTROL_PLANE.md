# Handover — Control Plane, Phase 4

**Date:** 2026-08-21 (Updated 2026-09-08) · **Branch:** `fix/storefront-checkout-and-alignment`  
**Scope doc:** [CONTROL_PLANE.md](CONTROL_PLANE.md) · **Task list:** [PHASE4_TASKS.md](PHASE4_TASKS.md) ·
**Space Architecture:** [SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md](SPACE_CONTROL_PLANE_IMPLEMENTATION_PLAN.md) ·
**Decisions:** [ADR-030](DECISIONS.md#adr-030)

> [!IMPORTANT]
> **Space Standalone Project Handoff (2026-09-08):**  
> Long-term company-wide control plane development now occurs in **Space** at `D:\Syed Systems\Space`.
> Space has its own workspace, `apps/space`, `crates/space-server`, migrations, and tests.
> The tasks below for `apps/platform` and `stheno-api-platform` cover the in-repo transitional surface
> for Vendryx Commerce. When working on new company control plane features, work in `D:\Syed Systems\Space`.

This is a mid-phase handover. **P29 is complete and verified. P30 is 2 of 17 done, with a
third task half-landed.** Read §4 before writing any code — there is one migration on disk
whose Rust half does not exist yet, and it will confuse you if you find it cold.

---

## 1. What this work is

`apps/platform` — the control plane — was a tenant list with a tier dropdown, gated on one
shared environment string, with no operator identity and no audit trail.

Two documents were written first, and they are the map:

- **[CONTROL_PLANE.md](CONTROL_PLANE.md)** — what the surface is *for*. It is not a tenant
  admin panel; it is the only place the business is operated from. Ten domains (D1–D10),
  63 capabilities, **7 built at the time of writing (11%)** — now **59/63 surface, 66/89
  substance**, see [CONTROL_PLANE.md §3](CONTROL_PLANE.md#3-scorecard).
- **[PHASE4_TASKS.md](PHASE4_TASKS.md)** — 87 tasks across P29–P36, same format as
  PHASE3_TASKS.md: stable IDs, explicit deps, an exit command that exits 0, `[BP]` markers
  on the ones where a plausible implementation passes its own test and fails in production.

**The single biggest finding:** the business has no way to charge anyone.
[ENTITLEMENTS.md §9](ENTITLEMENTS.md) scoped billing out of the *engine* — correctly — and
nothing picked up the *product* side. An operator can move a merchant onto Pro; nothing
invoices them, nothing notices non-payment, and `platform.tenant.status` shipped with a
`suspended` value **no code path writes**. That is P30, 17 tasks, and it is the reason the
phase is ordered the way it is.

---

## 2. DONE and verified — P29, operator identity and audit (10/10)

### What changed

| Thing | Where |
|---|---|
| `platform.operator`, `platform.operator_audit` | [migrations/20260821000002_platform_operator/](../migrations/20260821000002_platform_operator/) |
| Models, DTOs, 6 service methods | [stheno-platform](../crates/modules/stheno-platform/src/) — `models.rs`, `dto.rs`, `service.rs` |
| `actor()` replacing `authorised()`, `record()` | [stheno-api-platform/src/lib.rs](../crates/api/stheno-api-platform/src/lib.rs) |
| Operator + audit routes | [stheno-api-platform/src/operators.rs](../crates/api/stheno-api-platform/src/operators.rs) |
| UI: two new tabs | [apps/platform/src/operators.tsx](../apps/platform/src/operators.tsx), [audit.tsx](../apps/platform/src/audit.tsx) |
| Gate: every write is audited | [scripts/check-control-plane-audit.sh](../scripts/check-control-plane-audit.sh) |
| Gate: doc links resolve | [scripts/check-docs-links.sh](../scripts/check-docs-links.sh) |

### Three decisions you must not undo by accident

1. **The bootstrap token stays valid after named operators exist.** This looks like a
   security hole and is not. Revoking the last named credential must not be able to lock
   the owner out of their own control plane. It is attributed as `op_bootstrap` — exactly
   as specific as a shared secret honestly is. Retiring it deliberately is **P36-05**.
   There is a test named
   `an_operator_cannot_lock_everyone_out_by_revoking_the_last_credential`; if you "fix" the
   bootstrap path, that is the test that will stop you, and it is right.

2. **The audit trail is `platform.operator_audit`, not `audit.audit_log`.** The obvious
   reuse was checked and rejected: `audit.audit_log` is tenant-scoped and RLS-**forced**,
   so "everything this operator changed" — a cross-tenant read — is unanswerable there by
   construction. Half of what the control plane does (redefining a plan) also belongs to no
   merchant at all.

3. **The audit cursor is `id`, not `created_at` + `OFFSET`.** `id` is a ULID: its leading
   bits are the timestamp, so it sorts by time *and* is unique. `id < cursor` therefore
   cannot skip or repeat a row. The trail grows at exactly the end being read, so an offset
   page two re-serves rows page one already showed. Test:
   `a_row_written_between_pages_does_not_shift_the_next_one`.

### Gotcha that cost time

`PgCrudRepository::retrieve` hardcodes `AND t.deleted_at IS NULL`
([pg_repository.rs:188](../crates/core/stheno-db/src/pg_repository.rs#L188)). Any table you
call `retrieve` on **must** have a `deleted_at` column even if nothing ever writes it.
`platform.operator` carries one for that reason and documents why.
`platform.operator_audit` deliberately does **not** — an audit table must not have the
column that lets rows be soft-deleted, and only `create`/`list` are ever called on it.

### Verification

```
cargo test -p stheno-api-platform          # 19: operator_identity 6, audit_trail 7, audit_pagination 6
cargo test -p stheno-server --test subscription_journey   # 1, pre-existing regression
./scripts/check-control-plane-audit.sh     # 7 write handlers audited + failing fixture
./scripts/check-docs-links.sh              # 319 relative links resolve
```

---

## 3. DONE — P30-01 and P30-02

### P30-01 — [ADR-017](DECISIONS.md#adr-017), Accepted, indexed

**Subscription billing is first-party, collected on the gateways already integrated.**

The decision turns on one fact: **no hosted billing vendor can collect BDT from bKash or
Nagad.** Stripe Billing does not process BDT; Paddle is a card/PayPal merchant-of-record;
Chargebee orchestrates gateways that do not include the BD wallets. A vendor would collect
nothing — we would write the collection path anyway *and* pay for a duplicate invoice
record. The integration cost is not avoided by the vendor, it is duplicated by it.

[ENTITLEMENTS.md §9](ENTITLEMENTS.md) was amended in place so the two documents do not
contradict each other. "Do not build a billing *engine*" still stands: metering, revenue
recognition and multi-currency dunning are explicitly out of scope.

### P30-02 — plan pricing as data (7 tests, green)

`config/tiers.toml` gained `price` / `currency` / `interval` on `[pro]`.

**`price` is a TOML string, not a float, and this is not stylistic.** TOML has no decimal
type, so `price = 1500.00` parses as `f64`, which [ADR-006](DECISIONS.md#adr-006) forbids
anywhere near money — and it would forbid it *silently*, because `1500.00` survives a float
round trip and `1499.95` does not. There is a test (`a_price_that_a_float_would_corrupt_survives_intact`)
whose only job is to fail if someone changes this back.

**A plan with no price is NOT sold — it is not free.** Free is a price of zero. A billing
run must skip an unpriced plan rather than invoice it for nothing, so that a plan whose
price goes missing in an edit does not quietly become free for everyone on it.

### Also done — `CurrencyCode::Bdt` added

[crates/core/stheno-types/src/currency.rs](../crates/core/stheno-types/src/currency.rs).
BDT was **not in the enum**. The product sells to Bangladesh and the money type could not
express the only currency the business actually charges in. Two decimals, so it needs no
`minor_unit_exponent` entry.

---

## 4. ⚠️ HALF-LANDED — P30-03. Read this before you write anything

**[migrations/20260821000003_platform_billing/](../migrations/20260821000003_platform_billing/)
exists, applies cleanly, and passes `check-schema-drift.sh`. There is no Rust that reads or
writes any of it yet.**

It creates:

| Table | Purpose |
|---|---|
| `platform.invoice` | one period, one merchant, one bill. **`UNIQUE (tenant_id, period_start)`** |
| `platform.invoice_line` | signed amounts, so lines sum to the total |
| `platform.billing_attempt` | the outbound ledger for collection |

and alters `platform.subscription`: adds `past_due` to the status CHECK, plus
`dunning_attempts`, `next_attempt_at`, `current_period_start`.

**Your options are both fine, pick one deliberately:**

- **Continue P30-04 onward** and build the Rust against this schema as designed. The
  migration is the spec; the comments in `up.sql` explain every non-obvious column.
- **Revert it** (`git rm -r migrations/20260821000003_platform_billing`) and redesign. It
  is committed but nothing depends on it, so this is cheap *today* and expensive once
  anything reads it.

### Two constraints already discovered that shape the Rust

**1. Billing code goes in `stheno-platform`, NOT a new `stheno-billing` crate.**
`check-sql-boundaries.sh` derives a module's schema from its crate name, so `stheno-billing`
would own a `billing` schema and touching `platform.*` from it is a boundary violation.
`check-module-isolation.sh` separately forbids module→module dependencies. The task rows in
PHASE4_TASKS.md were corrected from `cargo test -p stheno-billing` to
`cargo test -p stheno-platform` for this reason.

**2. Collection cannot reuse `PaymentService::idempotent_call`.** Two independent gates say
so, and they are pointing at the same fact — our revenue is not one of the merchant's
payments:

- `payment.payment_provider_request` is tenant-scoped and RLS-forced. Writing our own
  revenue collection into it puts it inside the merchant's row set, reachable through the
  merchant's own surface. ADR-017 decision 1 forbids exactly this.
- `stheno-platform` may not depend on `stheno-payment` (module isolation).

`platform.billing_attempt` therefore mirrors the *pattern* — idempotency key, request hash,
`in_flight → succeeded | failed | unknown`, plus a convergence sweep — in its own table.
**ADR-017 decision 2 was amended to say this.** If you find yourself about to `use
stheno_payment::` from `stheno-platform`, that is the wall, and it is load-bearing.

### Useful thing you already have

`Money::allocate_proportionally` ([money.rs](../crates/core/stheno-types/src/money.rs))
implements largest-remainder allocation and **guarantees the parts sum exactly to the
whole**. That is precisely what P30-05 proration needs — do not write a second one.

---

## 5. NOT STARTED — the remaining 75 tasks

| Phase | Domain | Tasks | Blocked on |
|---|---|---:|---|
| **P30** subscription billing | revenue | 15 left | — (P30-04 next) |
| **P31** tenant lifecycle | ops | 12 | P30 for enforcement |
| **P32** money + fulfillment oversight | money | 14 | **P21–P24, genuinely** |
| **P33** growth analytics | growth | 8 | — runs in parallel |
| **P34** support operations | support | 10 | P31; needs ADR-018 first |
| **P35** config + themes | config | 9 | P27 for the theme half |
| **P36** hardening — roles, MFA | access | 7 | P34 |

**Suggested next three:**

1. **P30-04** — `PlatformService::generate_invoice`, idempotent on `(tenant_id,
   period_start)`. The unique index already enforces it; the Rust must catch the conflict
   and return the existing invoice rather than erroring. `[BP]`.
2. **P35-01** — add `custom_theme` and `branding` to `config/tiers.toml`. One line, no
   dependencies. The branding gate must exist *before* P26 ships a branding UI, or the
   first one ships ungated.
3. **P32-05** — schedule `reconcile_unsettled_requests`. The method exists in
   `stheno-payment` and **nothing calls it periodically**. This one does not wait for P21.
   A convergence path that never runs is not a convergence path.

---

## 6. Standing rules for this surface

From [CONTROL_PLANE.md §5](CONTROL_PLANE.md). These are not per-task preferences:

1. **Every write is attributed and audited.** `check-control-plane-audit.sh` enforces it
   mechanically — add a route on POST/PUT/PATCH/DELETE without a `record(` call and the
   gate fails.
2. **Every cross-tenant query is deliberate and reviewed.** The control plane runs *outside*
   the tenant layer. That is a licence to leak unless each query is written as though it
   were the one that leaks. P33-01 exists to funnel these through one reviewed helper.
3. **Fails closed.** No credential means the surface is off, never open. This holds today
   and must survive every change.

**The standing review question:** *if this handler is wrong, how many merchants does it
affect?* If the answer is "all of them", it is a `[BP]` whether or not the table says so.

---

## 7. Known issues not caused by this work

- **`cargo fmt --all -- --check` fails** on `crates/api/stheno-api-core/src/entitlements.rs`,
  `tenant_entitlements.rs`, and two pre-existing lines of
  `crates/api/stheno-api-platform/src/lib.rs` (386, 398). Verified against `HEAD`: this is
  rustfmt 1.9.0 drift on already-committed code, not from this branch. Left alone
  deliberately — reformatting would bloat the diff and CI may run a different rustfmt.
  **If you run `cargo fmt` on a whole crate you will sweep these in.** Format individual
  files with `rustfmt --edition 2021 <file>` instead. (`--edition 2024` will reorder your
  imports to the new style, which is wrong for this workspace.)

- **Uncommitted Phase 3 work from a previous session** is in the same tree: `hooks.rs`,
  `payment_webhook_route.rs`, `stranded_request_recovery.rs`, `docs/providers/`,
  `migrations/20260821000001_fulfillment_provider_request/`, and edits to
  `stheno-fulfillment` / `stheno-payment` / `stheno-payment-stripe`. It is described in
  [PHASE3_STATUS.md](PHASE3_STATUS.md) §1. Not written by this session, not verified by it.

- Two links in `docs/providers/` pointed at `PHASE3_TASKS.md` from the wrong directory.
  Fixed while writing `check-docs-links.sh`, which is how they were found.

---

## 8. How to run everything

```bash
docker compose up -d postgres
export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno

cargo test -p stheno-api-platform          # 19 control-plane tests
cargo test -p stheno-api-core --test tier_pricing   # 7 pricing tests
cargo test -p stheno-server --test subscription_journey

./scripts/check-control-plane-audit.sh
./scripts/check-docs-links.sh
./scripts/check-schema-drift.sh
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh

npm --prefix apps/platform run typecheck
npm --prefix apps/platform run build
```

The control plane needs a credential to be reachable at all:

```bash
STHENO_PLATFORM_TOKEN=dev-platform-token cargo run -p stheno-server
cd apps/platform && npm run dev     # :5174
```

Then issue yourself a named credential and stop using the bootstrap token — see
[README](../README.md) “Two kinds of credential”.
