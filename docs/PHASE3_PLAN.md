# Phase 3 Implementation Plan

**Status:** Final · **Date:** 2026-08-19 · **Governed by:** [ADR-013](DECISIONS.md#adr-013)

**~25 engineer-weeks of work · ~14 calendar weeks with two parallel tracks · 8 phases (P21–P28)**

Phase 2 ([PHASE2_PLAN.md](PHASE2_PLAN.md)) made the engine durable and multi-tenant. It did not
make it *sellable in more than one country* or *wearable in more than one skin*. Phase 3 is those
two things, and they are genuinely independent work, which is why this phase parallelises better
than Phase 2 did.

Staffing assumption changes: **4–5 engineers** — 2 Rust backend, 2 frontend, 1 shared QA/infra.
Phase 3 is the first phase where the frontend track is not support work.

---

**Task-level breakdown:** [PHASE3_TASKS.md](PHASE3_TASKS.md) decomposes these phases into 95
tasks, each ending in a command that exits 0.

**Decision record:** [ADR-013](DECISIONS.md#adr-013) states the modularity contract this phase
implements. It is written **before** P21-01, per rule L8.

---

## 0. Where Phase 3 actually starts

Measured, not estimated. This baseline is unusually good, and the plan is short because of it.

| Capability | Reality at the start of Phase 3 |
|---|---|
| `PaymentProvider` trait | **Complete.** 12 methods, `crates/modules/stheno-payment/src/dto.rs:336`. Three impls ship |
| `FulfillmentProvider` trait | **Complete.** 6 methods, `crates/modules/stheno-fulfillment/src/provider.rs:13` |
| Webhook ingestion | Service layer **done** — `receive_webhook`, replay guard, `payment.payment_webhook_event`. **No HTTP route exposes it** |
| Reconciliation storage | `payment.payment_reconciliation_run` table exists. Nothing writes to it |
| Outbound idempotency | `payment.payment_provider_request` ledger exists and is used |
| Per-region provider enablement | `links.link_region_payment_provider` exists |
| Provider registration | Compile-time, three lines in `stheno-server/src/lib.rs:449` |
| Storefront core/theme seam | **Does not exist.** `src/lib/data` and `src/modules` are one Next application |
| Per-tenant branding | Does not exist. `tailwind.config.js` is a build-time constant |
| Theme conformance testing | One `e2e/storefront.spec.ts`, written against the one theme |

### The seam is already there, twice, and that is the good news

Both provider traits were designed for this. Adding bKash is **one crate and one
`register_provider` line** — Phase 3 does not design a payment abstraction, it writes
implementations of one that already has three. That is why P21 is 2 weeks and not 8.

The storefront has no such luck. That is real extraction work and it is the bulk of this phase.

---

## 1. How the plan is structured

Unchanged from Phase 2. **Deliverables**, a binary **Exit gate**, **Parallelism**, **Risk**.

Three rules carry over, plus two new ones:

1. **The load suite runs at every gate.**
2. **The parity suite runs at every gate.**
3. **No module ships green-in-memory.**
4. **New: no payment provider merges without passing the shared conformance suite.** A provider
   that works in a hand-written test and not in the generic suite is not integrated, it is
   attached.
5. **New: no theme merges without passing the shared checkout e2e pack.** Same reasoning. This is
   the rule that stops N designs from becoming N broken checkouts.

---

## 2. Phase map

| # | Phase | Weeks | Depends on | Track |
|---|---|---:|---|---|
| **P21** | Provider platform completion | 2 | P20, ADR-013 | A |
| **P22** | Bangladesh payment gateways | 4 | P21 | A |
| **P23** | Courier providers and tracking | 3 | P21 | **A′** (parallel from P21) |
| **P24** | COD settlement and reconciliation | 3 | P22, P23 | A |
| **P25** | Storefront core/theme split | 4 | ADR-013 | **B** (parallel from day 1) |
| **P26** | Branding tier — runtime tokens | 3 | P25, ADR-012 audit green | B |
| **P27** | Theme packages and conformance | 4 | P26 | B |
| **P28** | Hardening, PCI re-attestation, GA | 2 | all | A+B |
| | **Sequential total** | **25** | | |
| | **With 2 tracks** | **~14** | | |

```
Track A   P21──P22──────────P24───┐
               └──P23────────┘    │
                                  ├──P28
Track B   P25────P26────P27───────┘
```

**P25 starts on day 1, not after P21.** The storefront extraction depends on no backend work
whatsoever — it consumes the Store API that has been stable since ADR-002. Blocking the frontend
pair behind gateway integration wastes eleven weeks of their time.

**P23 forks off P21, not P22.** Couriers and payment gateways share only the webhook route built
in P21. Sequencing them is false dependency.

**P24 depends on both**, because COD settlement is exactly the seam where a courier performs a
payment capture. It cannot be built until both sides exist.

---

## P21 — Provider platform completion (2 weeks)

The phase that turns two finished traits into a platform anyone can plug into. Short by design:
almost everything here already exists and is unwired.

### Deliverables

1. **Inbound webhook routes.** `POST /hooks/payment/:provider_id` and
   `POST /hooks/fulfillment/:provider_id`, raw-body preserving, routed to the existing
   `receive_webhook`. Signature failures return 401 without touching the database; replays return
   200 without reprocessing. Both behaviours are already implemented in the service — this exposes
   them.
2. **Redirect flow modelled, not improvised.** `ProviderCapabilities` gains `redirect: bool`.
   `InitiateOutput.data` gains a documented `redirect_url` convention. The checkout reads it in
   **one** place in storefront core, because checkout is core ([ADR-013](DECISIONS.md#adr-013)).
3. **The provider conformance suite.** A generic harness in `stheno-payment` that any
   `PaymentProvider` impl is run through: session lifecycle, idempotent capture, partial refund
   where declared, webhook replay, and capability honesty — a provider declaring
   `partial_refund: true` must actually partial-refund. Stripe and offline are the first two
   subjects and must pass unmodified.
4. **Per-tenant provider enablement proven under RLS.** `link_region_payment_provider` rows are
   tenant-owned; a tenant enabling bKash must not make it visible to another tenant. This is a
   test, not a review.
5. **Admin routes for provider configuration** — list available providers, enable per region,
   store credentials encrypted at rest.
6. **The PCI rule made mechanical.** `scripts/check-provider-pci.sh` fails the build if a provider
   crate declares a card-data-accepting flow. Redirect and iframe only, per
   [PCI_SAQ_A.md](PCI_SAQ_A.md) §1.

### Exit gate

- A signed Stripe webhook delivered over HTTP reaches `receive_webhook` and transitions a payment.
- The same webhook delivered twice produces one state change and two 200s.
- An unsigned webhook returns 401 and writes no row.
- Stripe and offline both pass the conformance suite with no provider-specific exemptions.
- Tenant A enabling a provider is invisible to tenant B, verified by query, not inspection.
- `check-provider-pci.sh` fails against a deliberately non-compliant fixture provider.

### Risk

**The conformance suite is the deliverable that decides P22's cost.** If it is written loosely,
three gateways will each need bespoke debugging in P22 and the phase doubles. Write it against the
two existing providers first, and make it strict enough that Stripe passing means something.

**Raw body preservation is the classic mistake here.** Any middleware that parses and re-serialises
JSON before the handler breaks every signature check, and it breaks it *silently in production*
rather than in tests where fixtures are byte-exact. Assert on it in P21-03.

---

## P22 — Bangladesh payment gateways (4 weeks)

Execution of the P21 conformance suite across three real gateways.

### Deliverables

1. **bKash** — tokenized checkout (grant token, create, execute), redirect flow, query-payment
   reconciliation, refund.
2. **Nagad** — checkout initialise/complete, redirect flow, verification callback, refund.
3. **SSLCommerz** — hosted session, IPN validation, refund, and the multi-method aggregation it
   fronts (cards, mobile banking, net banking).
4. **Sandbox integration tests** for each, recorded as fixtures so CI does not depend on a third
   party being up.
5. **BDT currency and rounding verified end to end** — decimal per [ADR-006](DECISIONS.md#adr-006),
   minor-unit conversion at the provider boundary only.

### Exit gate

- All three pass the P21 conformance suite unmodified.
- Each completes a sandbox payment end to end: initiate → redirect → callback → captured order.
- Each correctly rejects a tampered callback signature.
- Refund succeeds against sandbox for all three. Where a gateway cannot partial-refund, it declares
  `partial_refund: false` and the engine refuses the operation rather than half-doing it.
- `check-provider-pci.sh` clean for all three.

### Risk

**Sandbox credentials are the schedule, not the code.** bKash and Nagad merchant sandbox access is
an application process with a real-world turnaround measured in weeks, and it requires business
documents, not a signup form. **Start those applications before P21 begins.** This is the single
most likely cause of Phase 3 slipping and it is entirely non-technical.

**Documentation quality is materially worse than Stripe's.** Budget discovery time per gateway, not
implementation time. The integration is 300 lines; finding out what the callback actually sends is
the week.

---

## P23 — Courier providers and tracking (3 weeks)

### Deliverables

1. **Pathao, Steadfast, RedX** implementations of `FulfillmentProvider` — serviceable-area and rate
   lookup, consignment creation returning a tracking id, cancellation, return consignment.
2. **Status ingestion** via the P21 fulfillment webhook route, mapped to the engine's fulfillment
   states. Polling fallback for couriers that do not push.
3. **Zone and rate configuration** — inside-city, outside-city, sub-city tiers, expressed through
   existing shipping options rather than a new abstraction.
4. **Recorded fixtures** for every status transition each courier emits, including the ugly ones:
   partial delivery, return-in-transit, hold.

### Exit gate

- A shipping rate for a Dhaka address is calculated live through each provider.
- A consignment is created in sandbox and returns a tracking number stored on the fulfillment.
- Every documented status each courier emits maps to a known engine state — **an unrecognised
  status is a logged, alerting failure, never a silent no-op.**
- Cancellation and return flows verified per provider.

### Risk

**Status vocabularies do not agree and are not fully documented.** The failure mode is an order
stuck in a state the engine does not know exists, discovered by a customer. The exit gate is
written to force the unknown-status path to be loud.

---

## P24 — COD settlement and reconciliation (3 weeks)

The phase nobody scopes. In Bangladesh, COD is the default payment method, which means **the
courier collects the money** — a fulfillment provider performing a payment capture, and a payment
whose settlement depends on a courier's payout report.

### Deliverables

1. **Courier-collected payment path** — delivery confirmation from P23 authorises capture on the
   offline/COD provider. One flow, not three courier-specific ones.
2. **Settlement import** — courier payout reports ingested per cycle and matched to orders.
3. **Reconciliation runs** written to the existing `payment.payment_reconciliation_run`, producing
   three explicit buckets: matched, collected-not-remitted, remitted-not-matched.
4. **Courier fee accounting** — the remittance is net of the courier's charge. The order total, the
   collected amount, and the remitted amount are three different numbers and all three are stored.
5. **Admin reconciliation surface** — a merchant can see, per cycle, what is owed to them and what
   did not match.
6. **Discrepancy alerting** — an unmatched item older than the cycle length raises an alert.

### Exit gate

- A COD order transitions delivered → captured → settled from courier input alone.
- A settlement file with a deliberate mismatch produces a discrepancy row and an alert, not a
  silent balance.
- Courier fees reconcile: `sum(remitted) + sum(fees) == sum(collected)` for a full cycle, asserted
  in decimal, never float.
- A returned COD order produces no capture and no settlement expectation.

### Risk

**This is the highest value-at-risk code in Phase 3, and the only place money can silently vanish.**
A wrong reconciliation does not crash; it produces a plausible number. Every assertion in this
phase is on decimal equality against a fixture with known-correct totals.

---

## P25 — Storefront core/theme split (4 weeks)

The extraction. No new features — if a merchant can see a difference after P25, something went
wrong.

### Deliverables

1. **`@stheno/storefront-core`** — `src/lib/data`, `src/lib/context`, `src/lib/hooks`, the
   middleware, and **the `app/` route tree**. Routing, region resolution, SEO, and the entire
   checkout flow live here and are not theme-editable, per [ADR-013](DECISIONS.md#adr-013).
2. **`@stheno/theme-default`** — everything currently in `src/modules`, converted to a theme
   package exporting a typed component map.
3. **The theme contract** — one typed interface enumerating every component a theme must export and
   the props it receives. A theme is complete when it type-checks against it.
4. **A boundary rule with teeth** — `scripts/check-storefront-boundaries.sh` in CI, failing on a
   theme importing core internals or reaching for the SDK directly. This is the frontend equivalent
   of `check-module-isolation.sh` and exists for the same reason: review does not catch it at 2am
   before a launch.
5. **Versioning policy written down** — themes pin a core major; core changes are additive within a
   major. Without this, every engine feature becomes an N-theme migration.

### Exit gate

- The storefront builds and serves from `core` + `theme-default` with zero behaviour change.
- The existing e2e pack passes unmodified.
- `check-storefront-boundaries.sh` fails against a fixture theme that imports `core/lib/data`.
- `theme-default` contains **no** data fetching, no server actions, and no checkout state.
- Deleting `theme-default` produces type errors in exactly one place — the theme resolution point.

### Risk

**Scope creep is the entire risk of this phase.** Extraction invites redesign, and the storefront
has plenty worth redesigning. It is not this phase. The exit gate says *zero behaviour change* for
that reason.

**Second risk: the route tree is contested.** Putting routes in the theme feels more flexible and
is wrong — it hands URL structure, region routing and checkout flow to whoever writes the theme.
[ADR-013](DECISIONS.md#adr-013) settles this; do not relitigate it in code review.

---

## P26 — Branding tier, runtime tokens (3 weeks)

The free/base tier from [ADR-013](DECISIONS.md#adr-013): store name, typography, colour. Data, not
code — resolved per request in one shared runtime, no build, no deploy.

### Deliverables

1. **Tenant branding record** — store name, logo, favicon, font selection from a curated set, colour
   tokens, with defaults. Tenant-owned, RLS-covered.
2. **Runtime token resolution** — branding resolved per request and emitted as CSS custom
   properties. No per-tenant build, no dynamic component loading.
3. **Curated font set**, self-hosted and licence-cleared. Merchants choose from it; they do not
   upload arbitrary font files.
4. **Admin branding UI** with live preview.
5. **Asset handling** for logos through the existing file module, with dimension and type validation
   at the trust boundary.

### Exit gate

- Two tenants served by **one** running storefront process render different names, fonts, logos and
  colours.
- **Branding applies on checkout routes**, not only on catalog pages — and
  `check-checkout-scripts.sh` stays green while it does. Self-hosted fonts only, per
  [ADR-013](DECISIONS.md#adr-013) decision 2a.
- A branding change is visible on the next request with no rebuild and no restart.
- A tenant with no branding row renders the default theme, not an error.
- Branding reads are tenant-scoped — verified by an RLS test, not by inspection.
- Lighthouse scores do not regress against the pre-P26 baseline. Per-request token resolution must
  not cost a round trip on the render path.

### Risk

**This phase assumes [ADR-012](DECISIONS.md#adr-012)'s audit test is green.** Branding is the first
feature where one process serves visibly different tenants, so it is where an isolation gap becomes
a screenshot. Confirm the tenancy audit passes before P26-01, not during.

---

## P27 — Theme packages and conformance (4 weeks)

The Pro tier: a real second design, and the machinery that makes the tenth one cheap.

### Deliverables

1. **A second reference theme**, visually unrelated to the default — different layout, different
   components, its own animation. Built by the frontend pair as the proof the seam holds. If it
   requires a single change to core, the contract was wrong and gets fixed here.
2. **The theme conformance e2e pack** — browse → product → add to cart → checkout → order confirmed,
   written against the theme contract so it runs unmodified against *any* theme. **A theme ships
   when this is green.** This is the highest-leverage artifact in Phase 3.
3. **Per-tenant build and deploy pipeline** for themed storefronts — one command, one deploy target,
   the theme pinned to a core version.
4. **Theme authoring documentation** — what a theme may and may not do, with the checkout rule
   stated first.
5. **Checkout skinning verified** — themes restyle checkout pages; they cannot alter the flow, the
   steps, or the scripts. `check-checkout-scripts.sh` runs against both themes.

### Exit gate

- Both themes pass the same conformance pack with no theme-specific branches in it.
- Building theme B required zero changes to `storefront-core`.
- A tenant is switched from theme A to theme B by configuration and one deploy.
- The checkout script manifest holds on both themes — the CSP allowlist is identical.
- Time to stand up a new themed storefront from an existing theme is measured and recorded.

### Risk

**A second theme built by the same people who built core will not exercise the contract honestly.**
They will unconsciously avoid the awkward parts. Mitigation: theme B's component list is drawn up
before core's contract is frozen in P25, and any core change it needs is logged rather than quietly
merged.

---

## P28 — Hardening, PCI re-attestation, GA (2 weeks)

### Deliverables

1. **SAQ-A re-attestation** covering the three new gateways. Every redirect and iframe origin added
   to the CSP and to [payment-page-scripts.md](payment-page-scripts.md), with
   `check-checkout-scripts.sh` updated and green.
2. **Provider failure-mode testing** — gateway timeout, gateway 500, callback never arriving,
   courier API down. Each has a defined, tested outcome.
3. **Load testing with multiple tenants and themes** in one runtime.
4. **Runbooks** — reconciliation discrepancy, stuck consignment, gateway outage.
5. **Docs**: provider authoring updated with the PCI rule and the conformance suite; theme authoring
   published.

### Exit gate

- SAQ-A attestation re-signed and `check-pci-saq-a.sh` green with all providers registered.
- Every failure mode above has a test and a runbook entry.
- Latency budgets hold with branding resolution on the render path.
- No provider or theme in the tree lacks a conformance run in CI.

---

## 3. Staffing

| Track | Who | Phases |
|---|---|---|
| A | 1 Rust backend | P21 → P22 → P24 |
| A′ | 1 Rust backend | P23, then P24 support |
| B | 2 frontend | P25 → P26 → P27 |
| Shared | 1 QA/infra | Conformance harnesses, sandbox accounts, P28 |

The QA/infra engineer owns **gateway sandbox procurement from week 1**. It is not an engineering
task and it is the critical path for P22.

---

## 4. Sequencing rules

1. **[ADR-013](DECISIONS.md#adr-013) is written and accepted before P21-01 and P25-01.** Both tracks
   depend on the checkout-is-core rule; discovering disagreement in code review costs the
   extraction.
2. **Sandbox applications are filed before Phase 3 starts.** See P22's risk.
3. **No gateway merges without the conformance suite.** The suite exists first, in P21.
4. **No theme merges without the e2e pack.** The pack exists in P27 and is retrofitted to
   `theme-default` before theme B is written, not after.
5. **P25 changes no behaviour.** Any improvement discovered during extraction is filed, not made.
6. **Confirm ADR-012's tenancy audit is green before P26.**

---

## 5. What would make this take longer

- **Sandbox credentials arrive late.** The most likely slip, entirely non-technical, and it can cost
  3–4 weeks of Track A idling. Mitigated only by filing early.
- **A gateway requires card data on our page.** That gateway is out of SAQ-A scope, and either gets
  dropped or moves the whole platform to SAQ-A-EP. That is not a schedule change, it is a different
  compliance posture — decide by refusing the gateway.
- **The storefront extraction turns into a redesign.** Adds 3+ weeks and produces a theme system
  nobody validated. Rule 5 exists for this.
- **Theme B needs core changes.** If it needs more than two, the contract in P25 was drawn wrong and
  P27 becomes a re-extraction.
- **COD reconciliation is got wrong.** Silent money loss is the most expensive class of bug here and
  the least likely to be caught by a passing test suite.

---

## 6. Definition of done, per provider

A payment or fulfillment provider is integrated when all of these hold:

- It implements its trait fully, with no `todo!()`.
- It passes the shared conformance suite with no provider-specific exemption.
- Its `capabilities()` are honest — every declared capability is exercised by a test.
- Mutating external calls use `idempotent_call()` with a unique key, per AGENTS.md §5.
- Amounts cross the boundary through `to_minor_units()` / `from_minor_units()`, never ad hoc.
- Its webhook signature verification has a passing test and a failing-signature test.
- It has recorded sandbox fixtures, so CI does not depend on a third party.
- `check-provider-pci.sh` is clean for it.

---

## 7. Definition of done, per theme

A theme is shippable when all of these hold:

- It type-checks against the theme contract with no gaps.
- It passes the shared checkout e2e pack, unmodified.
- It contains no data fetching, no server actions, and no checkout state.
- `check-storefront-boundaries.sh` is clean for it.
- `check-checkout-scripts.sh` is clean for it — the CSP allowlist is unchanged.
- It pins a core major version.
- It renders correctly with default branding and with a tenant's branding applied.

---

## 8. Breaking points — where the lazy answer is the wrong answer

Most of the 95 tasks in [PHASE3_TASKS.md](PHASE3_TASKS.md) are ordinary. Take the shortest path
that works, per [AGENTS.md §1](../AGENTS.md), and move on.

**Thirteen are not.** Each one below shares a signature: **a plausible implementation passes its
own test and fails in production, silently.** No crash, no red build, no stack trace — a wrong
number, a stuck order, a free product, or another merchant's brand on your page. They are the
tasks where the ladder shortens the *solution* and must never shorten the *reading*.

Every one is marked `**[BP]**` in the task file. Find them with:

```bash
grep -nE '^\| P[0-9]+-[0-9a-z]+ \| [*][*]\[BP\]' docs/PHASE3_TASKS.md
```

**The rule for a `[BP]` task:** the exit command is the floor, not the ceiling. Read every caller
before editing. Write the negative test — the one that proves the failure mode is caught — before
the positive one. Do not mark it DONE on a green gate alone; state in the report what you proved
and how, and if you took a shortcut, say which. A `[BP]` task reported as "gate green" with no
account of the failure mode is not finished.

---

### BP1 · P21-03 — Raw body integrity

**The reasonable implementation:** add the webhook route, take the parsed JSON body, verify the
signature against it.

**How it fails:** any middleware that deserialises and re-serialises JSON reorders keys and
changes whitespace. The bytes the handler signs are no longer the bytes the gateway signed.
Signature verification fails for **every** live webhook — but passes in tests, because fixtures are
constructed byte-exact and never travel the middleware stack. The failure is 100% and appears only
in production.

**Required effort:** assert on bytes through the **full** router and middleware stack, not on the
handler in isolation. The test must send a request through the real app, not call the function.

---

### BP2 · P21-08, P21-09 — Conformance suite strictness

**The reasonable implementation:** a suite that runs the happy path for each provider method.

**How it fails:** it passes for everything and proves nothing. Then three gateways in P22 each
need bespoke debugging, the phase doubles, and the suite becomes a formality people route around.
This is the highest-leverage artifact in Track A and the easiest to write uselessly.

**Required effort:** **capability honesty is the core of it.** A provider declaring
`partial_refund: true` must be made to actually partial-refund inside the suite and the result
asserted; a provider declaring `false` must be shown to *refuse*, not to silently full-refund.
Write it against Stripe and offline first and make it strict enough that Stripe passing means
something.

---

### BP3 · P22-05, P22-10, P22-14 — Callback signature verification

**The reasonable implementation:** parse the callback, find the order, mark it paid.

**How it fails:** this is the **free-goods path**. An attacker who can POST to a public callback
URL creates paid orders without paying. It works perfectly in every test, because every test sends
a legitimate callback.

**Required effort:** a **negative test per gateway** — tampered payload, tampered signature, and
replayed-with-different-amount — each asserting no state change. Never trust an amount or an order
id from a callback body without verifying it against a provider-side query. This is a trust
boundary; [AGENTS.md §1](../AGENTS.md) forbids simplifying it.

---

### BP4 · P22-17 — BDT minor-unit conversion

**The reasonable implementation:** convert the decimal to minor units wherever it is convenient.

**How it fails:** rounding at the wrong boundary loses paisa per transaction. Nothing errors. The
storefront, the order, and the gateway each show a defensible number, and then P24's reconciliation
never balances — and the cause is three phases upstream. [ADR-006](DECISIONS.md#adr-006) exists
because of exactly this.

**Required effort:** conversion happens **only** at the provider boundary, via
`to_minor_units()` / `from_minor_units()`, never ad hoc. Assert round-trip identity on values
chosen to be hostile: amounts ending in 0.005, three-decimal inputs, and the largest order total
the platform allows.

---

### BP5 · P23-11 — Unknown courier status

**The reasonable implementation:** match the known statuses, ignore the rest.

**How it fails:** a courier adds or renames a status and every affected order silently stops
moving. Nothing logs, nothing alerts, the dashboard looks normal, and it is discovered by a
customer asking where their parcel is — weeks later, across an unknown number of orders.

**Required effort:** unmapped status is a **named state plus an alert**, never a no-op. The test
feeds a deliberately invented status string and asserts the alert fires. Assume the courier's
documented list is incomplete, because it is.

---

### BP6 · P24-03 — Capture on delivery

**The reasonable implementation:** on a delivered event, capture the COD payment.

**How it fails:** couriers re-send delivery events, and *delivered → returned* is a normal
sequence, not an anomaly. A non-idempotent capture double-charges the merchant's books; a capture
that ignores a later return books revenue that physically came back.

**Required effort:** capture is idempotent on the delivery event id, and the return path is tested
explicitly — including the ugly ordering where the return arrives before the system has finished
processing the delivery.

---

### BP7 · P24-05 — Settlement re-import idempotency

**The reasonable implementation:** parse the courier's payout file and insert the rows.

**How it fails:** operations re-uploads a file, or a courier re-sends a corrected cycle. Every
line double-counts. The reconciliation still *balances internally*, which is what makes this one
genuinely nasty — the numbers look consistent and are wrong.

**Required effort:** import is keyed on a courier-side settlement identifier and re-import is
asserted to be a no-op, including for a **partially overlapping** file, which is the real-world
case and the one a naive unique constraint misses.

---

### BP8 · P24-07 — Courier fee accounting

**The reasonable implementation:** store what the courier remitted and treat it as the payment.

**How it fails:** remittance is net of the courier's fee. Conflating collected with remitted
understates revenue on every single COD order — a small, consistent, entirely invisible error
that compounds across every merchant on the platform.

**Required effort:** order total, collected, and remitted are **three stored numbers**, and
`sum(remitted) + sum(fees) == sum(collected)` is asserted in decimal over a full cycle against a
fixture with hand-verified totals. Float anywhere on this path is a defect regardless of whether a
test catches it.

---

### BP9 · P25-04 — The route tree stays in core

**The reasonable implementation:** put the `app/` routes in the theme. It is more flexible, and it
makes themes feel more powerful.

**How it fails:** it hands URL structure, region resolution, SEO and the checkout flow to whoever
writes the theme. Every guarantee in [ADR-013](DECISIONS.md#adr-013) evaporates, and it evaporates
quietly — the build stays green and the consequence arrives when the fourth theme has four URL
schemes and the sitemap is meaningless.

**Required effort:** none technically — this one costs discipline, not work.
[ADR-013](DECISIONS.md#adr-013) decision 4 settles it. **Do not relitigate it in code review**, and
do not accept "just this route" from a theme.

---

### BP10 · P25-13 — Zero behaviour change

**The reasonable implementation:** while moving the code, fix the things that are obviously wrong.

**How it fails:** the extraction and the improvements land in one diff, so when something breaks
nobody can tell which half did it. A four-week phase becomes six, and the storefront regressions
are found by merchants.

**Required effort:** file every improvement, make none. The exit gate is a **byte-identical route
snapshot** against a fixed catalog before and after. If the snapshot differs, the phase is not
done, regardless of how much better the new version is.

---

### BP11 · P26-01 — RLS audit green before branding

**The reasonable implementation:** start branding work; tenancy is already built.

**How it fails:** branding is the first feature where one process renders *visibly* different
tenants. An isolation gap that was previously an abstract risk becomes a screenshot of another
merchant's logo on a customer's checkout page. That is not a bug report, it is a breach
disclosure — see [ADR-012](DECISIONS.md#adr-012)'s "company-ending" note.

**Required effort:** run the audit and read the result before P26-02. Not "it passed last month" —
run it. Every branding read is tenant-scoped and proven by an RLS test, never by inspection.

---

### BP12 · P27-01 — Conformance pack written against the contract

**The reasonable implementation:** write the e2e pack against theme A — its CSS selectors, its
DOM structure, its button text.

**How it fails:** it is worthless the moment theme B exists, because theme B has none of those
selectors. The pack then either gets theme-specific branches — which destroys the one guarantee it
exists to provide — or gets skipped. Either way the rule "no theme merges without passing the
pack" becomes decorative, and N designs become N broken checkouts, which is the exact outcome all
of Phase 3 exists to prevent.

**Required effort:** the pack targets the **theme contract** — stable test ids and semantic roles
that the contract *requires* every theme to expose — never markup. Retrofit it to `theme-default`
in P27-02 and prove it green there **before** theme B exists, because that is the only moment you
can still tell the difference between a contract test and a theme test.

---

### BP13 · P28-01 — CSP origins per gateway

**The reasonable implementation:** checkout breaks for a new gateway in staging, so widen the CSP
until it works.

**How it fails:** two outcomes, both bad. Forget an origin and that gateway's checkout fails in
production for real customers. Widen the CSP to a wildcard and SAQ-A eligibility is gone —
`check-pci-saq-a.sh` may still pass while the attestation it represents has become false.

**Required effort:** each gateway's redirect and iframe origins are added **explicitly and
individually** to the CSP and to [payment-page-scripts.md](payment-page-scripts.md). A wildcard on
a checkout route is a rejected change, not a temporary unblock. Per
[ADR-013](DECISIONS.md#adr-013) decision 3, this is a compliance boundary, not a configuration
preference.

---

### Summary

| BP | Task | Fails as | Class |
|---|---|---|---|
| BP1 | P21-03 | Every live webhook rejected, tests green | Silent, total, production-only |
| BP2 | P21-08/09 | Suite proves nothing; P22 doubles | Schedule + false confidence |
| BP3 | P22-05/10/14 | Free orders for anyone who can POST | **Security** |
| BP4 | P22-17 | Paisa lost per transaction | **Money** |
| BP5 | P23-11 | Orders stuck in an unknown state | Silent, customer-discovered |
| BP6 | P24-03 | Double capture, or revenue on returns | **Money** |
| BP7 | P24-05 | Double-counted settlement that still balances | **Money** |
| BP8 | P24-07 | Revenue understated on every COD order | **Money** |
| BP9 | P25-04 | ADR-013's guarantees quietly void | Architecture |
| BP10 | P25-13 | Extraction and regressions indistinguishable | Schedule + trust |
| BP11 | P26-01 | Another tenant's brand on a customer's page | **Breach** |
| BP12 | P27-01 | The one rule preventing N broken checkouts becomes decorative | Architecture |
| BP13 | P28-01 | Checkout down, or SAQ-A silently false | **Compliance** |

Four money, one security, one breach, one compliance. **Seven of thirteen breaking points are
places where the system keeps running and something valuable is quietly lost.** That is the class
of failure this section exists to slow you down for.
