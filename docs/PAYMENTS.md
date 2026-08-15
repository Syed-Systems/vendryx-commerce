# Payments — Design Specification

**Baseline:** `medusajs/medusa@b733400` — `packages/modules/payment`,
`packages/modules/providers/payment-stripe`, `packages/core/types/src/payment/provider.ts`

This document exists because [TRD §10](TRD.md) specified payments as an eight-method trait and
moved on. That was wrong in two ways: Medusa's provider interface is **18 methods**, and the trait
is the easy part. Payments are a distributed-systems and accounting problem wearing an API's
clothes.

---

## 1. The premise

Every other provider in this system is a function call that might fail. Payments are different in
three specific ways, and every design decision below follows from them:

1. **A failed call is not a failed operation.** A timeout on "create charge" means *unknown*, not
   *no*. Treating it as "no" and retrying is how you double-charge.
2. **State arrives twice, out of order.** The API response and the webhook both tell you what
   happened, they race, and the webhook can win. Anything that assumes response-then-webhook is
   already broken.
3. **The other party keeps their own books, and they diverge.** Dashboard refunds, disputes,
   chargebacks, and reversals all happen without your involvement. Without reconciliation you find
   out at month-end close.

The rest of this document is those three facts applied to each surface.

---

## 2. Domain model

Entities are unchanged from [DATA_MODEL §1](DATA_MODEL.md) — 8 entities. What follows is the
columns that carry the hard semantics, plus three additions.

```
payment_collection ──< payment_session ──> (provider, data jsonb)
                   └──< payment ──< capture
                                └──< refund ──> refund_reason
customer ──(link)── account_holder
```

| Entity | Semantics that matter |
|---|---|
| `payment_collection` | `amount`, `currency_code`, `status`, `authorized_amount`, `captured_amount`, `refunded_amount`. The **money ledger anchor** for one payable thing |
| `payment_session` | `provider_id`, `amount`, `currency_code`, `status` (7 states, §3.1), `data` jsonb (provider object), `authorized_at`, `context` |
| `payment` | Created **only on successful authorization**. `provider_id`, `amount`, `captured_at`, `canceled_at`, `data` |
| `capture` | `payment_id`, `amount`, `created_by`. Multiple rows per payment |
| `refund` | `payment_id`, `amount`, `refund_reason_id`, `note`, `created_by`. Multiple rows per payment |
| `account_holder` | `provider_id`, `external_id`, `data`. Provider-side customer for saved methods |

### 2.1 Three additions Medusa does not have

These close the gaps named in §1 and are Stheno-only.

**`payment_provider_request`** — the outbound idempotency ledger (§6).

| Column | Purpose |
|---|---|
| `id` | `<prefix>_<ULID>` |
| `idempotency_key` | what we sent the PSP; unique |
| `provider_id`, `operation` | `initiate \| authorize \| capture \| refund \| cancel` |
| `resource_type`, `resource_id` | our side of the reference |
| `request_hash` | SHA-256 of the canonical payload — detects key reuse with a changed body |
| `state` | `in_flight \| succeeded \| failed \| unknown` |
| `response` jsonb, `error` jsonb | |
| `attempts`, `first_sent_at`, `settled_at` | |

**`payment_webhook_event`** — raw event store (§7).

| Column | Purpose |
|---|---|
| `provider_id`, `provider_event_id` | unique together — the replay guard |
| `event_type`, `payload` jsonb, `signature_verified` | |
| `provider_created_at` | the PSP's timestamp, used for ordering, **not** ours |
| `state` | `received \| processed \| ignored \| failed` |
| `processed_at`, `error` | |

**`payment_reconciliation_run`** — drift detection (§9).

| Column | Purpose |
|---|---|
| `provider_id`, `window_start`, `window_end` | |
| `checked_count`, `drift_count` | |
| `drifts` jsonb | `[{ our_id, provider_id, field, ours, theirs }]` |
| `state` | `running \| clean \| drift_found \| failed` |

---

## 3. State machines

### 3.1 Payment session — 7 states

Exact parity with Medusa's `PaymentSessionStatus`:

```
                    ┌──────────► error
                    │
pending ──► pending_authorization ──► authorized ──► captured
   │                    │                  │
   │                    ▼                  ▼
   └──────────────► requires_more      canceled
                        │
                        └──► authorized | error | canceled
```

| State | Meaning |
|---|---|
| `pending` | Session created, no customer action yet |
| `pending_authorization` | Customer acted; PSP is processing (async methods live here for hours) |
| `requires_more` | Customer action needed — 3DS/SCA, or a partially funded intent |
| `authorized` | Funds held, capture available |
| `captured` | Funds taken |
| `canceled` | Authorization released |
| `error` | Terminal failure |

**Transitions are monotonic.** `captured` never returns to `authorized`; `error` and `canceled`
are terminal. An out-of-order webhook attempting a backwards transition is **logged and dropped**,
never applied. This single rule neutralises most webhook-ordering hazards (§7.3).

### 3.2 Webhook actions — 8 values

Parity with Medusa's `PaymentActions`. The provider maps its native events onto these; the module
never sees a Stripe event type.

`authorized` · `captured` · `failed` · `pending` · `requires_more` · `canceled` ·
`pending_authorization` · `not_supported`

`not_supported` means "an event we do not act on" — acknowledge with 200, store, do nothing.
Returning an error for unrecognised events causes PSP retry storms.

### 3.3 Money ledger invariants

Enforced by database constraints, not application checks. These are the accounting equivalent of
the module-isolation rule — violate them and the books stop balancing.

```
0 ≤ Σ captures  ≤ payment.amount
0 ≤ Σ refunds   ≤ Σ captures
payment_collection.captured_amount = Σ captures  across its payments
payment_collection.refunded_amount = Σ refunds   across its payments
```

Every capture and refund insert re-derives the aggregate under `SELECT … FOR UPDATE` on the
payment row. Over-capture and over-refund are **impossible**, not merely validated — §11 lists the
races this defeats.

---

## 4. The provider trait — all 18 methods

Medusa's interface has 11 required and 7 optional methods. The Rust equivalent splits them into
two traits, so a provider that cannot store payment methods does not stub seven functions.

```rust
#[async_trait]
pub trait PaymentProvider: Send + Sync + 'static {
    fn id(&self) -> &str;

    // ---- session lifecycle ----
    async fn initiate(&self, i: InitiateInput) -> Result<InitiateOutput>;
    async fn update(&self, i: UpdateInput) -> Result<UpdateOutput>;
    async fn delete(&self, i: DeleteInput) -> Result<()>;

    // ---- money movement ----
    async fn authorize(&self, i: AuthorizeInput) -> Result<AuthorizeOutput>;
    async fn capture(&self, i: CaptureInput) -> Result<CaptureOutput>;
    async fn refund(&self, i: RefundInput) -> Result<RefundOutput>;
    async fn cancel(&self, i: CancelInput) -> Result<CancelOutput>;

    // ---- reads ----
    async fn retrieve(&self, i: RetrieveInput) -> Result<RetrieveOutput>;
    async fn get_status(&self, i: StatusInput) -> Result<PaymentSessionStatus>;

    // ---- inbound ----
    fn verify_webhook(&self, raw: &[u8], headers: &HeaderMap) -> Result<VerifiedEvent>;
    fn map_webhook(&self, event: &VerifiedEvent) -> Result<WebhookAction>;

    // ---- capability probe: drives which flows the module offers ----
    fn capabilities(&self) -> ProviderCapabilities { ProviderCapabilities::DEFAULT }
}

/// Optional. Implemented only by providers with a vault.
#[async_trait]
pub trait PaymentVault: PaymentProvider {
    async fn create_account_holder(&self, i: CreateAccountHolderInput)   -> Result<AccountHolderOutput>;
    async fn update_account_holder(&self, i: UpdateAccountHolderInput)   -> Result<AccountHolderOutput>;
    async fn retrieve_account_holder(&self, i: RetrieveAccountHolderInput) -> Result<AccountHolderOutput>;
    async fn delete_account_holder(&self, i: DeleteAccountHolderInput)   -> Result<()>;
    async fn list_payment_methods(&self, i: ListMethodsInput)            -> Result<Vec<PaymentMethod>>;
    async fn save_payment_method(&self, i: SaveMethodInput)              -> Result<PaymentMethod>;
    async fn delete_payment_method(&self, i: DeleteMethodInput)          -> Result<()>;
}
```

Splitting `verify_webhook` from `map_webhook` (Medusa fuses both into
`getWebhookActionAndData`) is deliberate: verification is a security boundary that must run and be
audited **before** the payload is parsed, and it lets us persist the raw event even when mapping
fails.

### 4.1 Capabilities

```rust
pub struct ProviderCapabilities {
    pub partial_capture: bool,
    pub multi_capture: bool,
    pub partial_refund: bool,
    pub cancel_after_authorize: bool,
    pub vault: bool,
    pub async_settlement: bool,      // OXXO, boleto, bank transfer — settles hours/days later
    pub authorization_ttl: Option<Duration>,   // ~7 days on Stripe cards
}
```

The module reads this instead of attempting an operation and interpreting the failure. It is also
what lets the admin grey out "capture partial" for providers that cannot, rather than showing an
error after the operator commits.

---

## 5. The money boundary

**The single highest-risk function in the codebase.** Stripe takes integer minor units. We hold
`Decimal`. Getting this wrong is a silent financial error, which is PRD risk R6.

Medusa's rules, reproduced exactly from `get-smallest-unit.ts`:

| Class | Currencies | Multiplier |
|---|---|---|
| Zero-decimal | BIF, CLP, DJF, GNF, JPY, KMF, KRW, MGA, PYG, RWF, UGX, VND, VUV, XAF, XOF, XPF | 10⁰ |
| Three-decimal | BHD, IQD, JOD, KWD, OMR, TND | 10³ |
| Default | everything else | 10² |

**The three-decimal trap.** Stripe requires the final digit to be zero for these currencies, so
the converted amount is rounded **up** to the nearest ten. `1.234 KWD → 1240`, not `1234`. Miss
this and every Kuwaiti transaction is rejected by the API. Medusa handles it; naive
implementations do not.

```rust
pub fn to_minor_units(amount: Decimal, currency: CurrencyCode) -> Result<i64> {
    let exp = currency.minor_unit_exponent();          // 0 | 2 | 3
    let scaled = (amount * Decimal::from(10i64.pow(exp))).round();
    let mut v = scaled.to_i64().ok_or(Error::AmountOverflow)?;
    if exp == 3 { v = ((v + 9) / 10) * 10; }           // Stripe: last digit must be 0
    Ok(v)
}

pub fn from_minor_units(v: i64, currency: CurrencyCode) -> Decimal {
    Decimal::from(v) / Decimal::from(10i64.pow(currency.minor_unit_exponent()))
}
```

**Rules:**

- These two functions are the **only** place a money value crosses into or out of a provider.
  A provider crate calling `.to_i64()` on a `Decimal` directly is a review failure.
- Round-trip is **not** the property to test — three-decimal rounding is deliberately lossy
  upward. The property is: `from_minor_units(to_minor_units(x)) >= x` and the difference is under
  one minor unit.
- Every amount we send is also **re-read from the provider response** and compared. A mismatch
  aborts the workflow rather than recording our number (§11, E12).
- Property-tested across all 22 special currencies plus a sample of the default set.

---

## 6. Outbound idempotency

**The gap my original spec had.** We specified idempotency for callers of *our* API and forgot our
calls to *theirs*.

### The protocol

Every mutating provider call follows this, without exception:

```
1. Derive a deterministic key:
       sha256(workflow_transaction_id ‖ step_name ‖ resource_id ‖ operation)
   Deterministic, so a retried workflow step reproduces the same key.

2. INSERT payment_provider_request (key, request_hash, state='in_flight')
   ON CONFLICT (idempotency_key) DO NOTHING

3a. Insert won  → send to PSP with the key in its Idempotency-Key header
3b. Conflict    → a previous attempt exists:
        succeeded → return its stored response. Do not call the PSP.
        failed    → return its stored error.
        in_flight → the ambiguous case. Go to 5.

4. On response → UPDATE state='succeeded'|'failed', store response, settled_at=now()

5. On timeout or connection error → state='unknown'.
   NEVER assume failure. Resolve by reading provider state:
        retrieve() the object by our reference
        found & matching → treat as succeeded, backfill the response
        not found        → safe to retry with the same key
        found & differing amount → halt, flag for manual review
```

Step 5 is the whole point. A naive implementation treats a timeout as failure and retries without
a key, producing two charges.

**Key reuse with a changed payload.** `request_hash` catches the case where the same key is sent
with a different body — Stripe rejects this, and we want to fail on our side with a clear error
rather than surface theirs.

**Retention:** 30 days, which comfortably exceeds Stripe's 24-hour idempotency window. Rows outlive
the provider's memory so we can still answer "did this happen?" during a dispute.

---

## 7. Webhooks

### 7.1 The receive path

```
POST /hooks/payment/:provider
  1. Read the RAW body. Never a parsed value — signature is over exact bytes.
  2. verify_webhook() — HMAC-SHA256 with a constant-time compare, and a
     timestamp tolerance of 5 minutes to bound replay.
  3. INSERT payment_webhook_event ON CONFLICT (provider_id, provider_event_id) DO NOTHING
     → conflict means duplicate delivery. Return 200. Stop.
  4. Return 200 immediately.
  5. Process asynchronously as a workflow.
```

**Return 200 before processing, always.** A slow handler triggers PSP retries, which produce
duplicate deliveries, which is a self-inflicted thundering herd. Verification failure returns
**400** — the only case where a non-2xx is correct.

### 7.2 The convergence rule

The API response and the webhook are **two observations of one truth**, not a sequence. Both run
through the same reducer:

```rust
fn apply_observation(session: &mut PaymentSession, obs: Observation) -> Applied {
    if !session.status.can_transition_to(obs.status) {
        return Applied::Ignored(IgnoreReason::BackwardsTransition);
    }
    if obs.observed_at < session.status_observed_at {
        return Applied::Ignored(IgnoreReason::Stale);
    }
    session.status = obs.status;
    session.status_observed_at = obs.observed_at;
    Applied::Changed
}
```

`observed_at` comes from the **provider's** timestamp, never ours — our clock and their event
order are unrelated. Combined with monotonic transitions (§3.1), this makes the webhook/response
race a non-event: whichever arrives first wins, the second is a no-op.

### 7.3 Ordering

PSPs guarantee delivery, not order. `charge.refunded` can precede `payment_intent.succeeded`.

**Never derive state from event sequence.** An event is a *hint that something changed*; the
authoritative read is `retrieve()` against the provider when the transition is non-obvious. For
the expensive path (high-frequency events) trust the payload; for anything that moves money into
the ledger, re-read.

### 7.4 Events mapped

Stripe events Medusa handles, and their action:

| Event | Action |
|---|---|
| `payment_intent.created`, `.processing` | `pending` / `pending_authorization` |
| `payment_intent.requires_action` | `requires_more` |
| `payment_intent.partially_funded` | `requires_more` |
| `payment_intent.amount_capturable_updated` | `authorized` |
| `payment_intent.succeeded` | `captured` |
| `payment_intent.payment_failed` | `failed` |
| `payment_intent.canceled` | `canceled` |
| anything else | `not_supported` |

Beyond Medusa, Stheno also handles: `charge.refunded` (out-of-band refunds),
`charge.dispute.created|closed` (§10), and `payment_intent.amount_capturable_updated` amount
drift.

### 7.5 Environment isolation

A webhook whose `livemode` disagrees with our configured mode is stored and **ignored**. Test
events hitting production is common during setup and must never touch real orders.

---

## 8. Flows

### 8.1 Authorization inside checkout

Step 6 of `complete-cart` ([WORKFLOWS §2.3](WORKFLOWS.md)). It is an **async step with a 30s
timeout** precisely because of §7.2 — it may be completed by the API response or by a webhook,
whichever lands first.

```
authorize_payment (async, timeout 30s, retry_interval_awaiting 2s)
  invoke:
    idempotent_call(authorize) →
       authorized      → set_step_success
       requires_more   → suspend; the 3DS webhook or the client confirm resumes it
       failed          → set_step_failure
       unknown         → resolve via retrieve() per §6.5
  compensate:
    cancel() the authorization — idempotent, safe to run twice
```

**Timeout does not abandon the call.** Per the engine's semantics, a timed-out async step reverts
*when the response arrives*, so a late authorization is still cancelled rather than left holding
customer funds.

### 8.2 Capture

Capture is **not** part of checkout. It happens at fulfillment, or immediately when the provider
does not separate auth and capture.

- Partial capture requires `capabilities().partial_capture`.
- Multiple captures require `multi_capture`; otherwise the first capture closes the payment.
- Every capture re-derives `Σ captures ≤ payment.amount` under a row lock (§3.3).
- **Authorization expiry** (`authorization_ttl`, ~7 days on Stripe cards) is a scheduled job that
  warns at T−48h and marks the payment `requires_action` at expiry. An order sitting unfulfilled
  past the window is a silent revenue loss otherwise.

### 8.3 Refunds

- Refund amount ≤ Σ captures − Σ refunds, enforced under lock.
- Multiple partial refunds allowed; each is its own `refund` row with a reason and actor.
- A refund on an async-settlement method before settlement is **rejected**, not queued — the funds
  do not exist yet.
- Refunds triggered by an order edit or return flow through the same path; there is no second
  refund implementation.
- An out-of-band refund (dashboard) arrives as `charge.refunded` and is inserted with
  `created_by = 'provider'` so the ledger stays balanced.

### 8.4 3DS / SCA

`requires_more` suspends the workflow. The client completes the challenge; either the client
calls confirm or the PSP fires a webhook. Both converge (§7.2).

**Abandonment** is the common case and needs an explicit answer: a session in `requires_more`
past a TTL (default 30 min) is cancelled by a scheduled job, and its inventory reservations
released. Without this, abandoned 3DS attempts hold stock indefinitely.

---

## 9. Reconciliation

**The second gap in my original spec.** Ledgers drift. Detect it on a schedule, not at month-end.

Nightly job, per provider, over a 48-hour rolling window (overlapping, so nothing falls between
runs):

```
for each payment touched in the window:
    theirs = provider.retrieve(reference)
    compare: status, amount, Σ captured, Σ refunded, currency
    on mismatch → record a drift row; do not auto-correct
also:
    provider objects with no local payment   → orphan_remote
    local payments with no provider object   → orphan_local
```

Output is a `payment_reconciliation_run` row. **Drift never auto-heals** — an automatic
"correction" against a payment provider is how a bug becomes a refund. Drift raises an alert and
appears in the admin for a human.

Metrics: `payment_reconciliation_drift_count`, `payment_unknown_state_count`,
`payment_webhook_unprocessed_count`. Any non-zero value on the first two pages someone.

---

## 10. Disputes and chargebacks

Not in Medusa core. In scope here because a commerce engine without dispute records cannot answer
"why is this order paid but the money is gone."

- `charge.dispute.created` → record a `dispute` row, mark the payment `disputed`, notify.
- `charge.dispute.closed` → record the outcome; on loss, write a compensating ledger entry so
  `payment_collection` reflects reality.
- A disputed payment **blocks refunds** — refunding a disputed charge loses the money twice.

---

## 11. Edge-case atlas

The failure modes this design must survive. Each row is a test in the P4 suite.

### Ambiguity and retries

| # | Scenario | Naive failure | Required behaviour |
|---|---|---|---|
| E1 | Timeout on authorize | Retry without key → double charge | §6: key + `unknown` state + `retrieve()` resolution |
| E2 | Process crashes between PSP success and our commit | Charge exists, order does not | Workflow resumes, `payment_provider_request` shows `succeeded`, backfills |
| E3 | Customer double-clicks pay | Two intents, two charges | Deterministic key from `transaction_id` → identical key → PSP dedupes |
| E4 | Same key, different amount | PSP rejects with an opaque error | `request_hash` mismatch → our clear error before the call |
| E5 | PSP returns 429 | Hammering makes it worse | Exponential backoff with jitter, honour `Retry-After`, cap attempts |
| E6 | PSP outage mid-checkout | Cart stuck, inventory held | Step fails after retries → compensation releases inventory, cart stays usable |

### Webhooks

| # | Scenario | Naive failure | Required behaviour |
|---|---|---|---|
| E7 | Webhook before API response | Response overwrites with stale state | §7.2 convergence + monotonic transitions |
| E8 | Out-of-order events | Refund applied, then "succeeded" resets it | Provider timestamp ordering; backwards transitions dropped |
| E9 | Duplicate delivery | Double ledger entry | Unique `(provider_id, provider_event_id)` |
| E10 | Replayed old event | Accepted | 5-minute timestamp tolerance in signature check |
| E11 | Event for an unknown session | 500 → PSP retry storm | Store, mark `ignored`, return 200 |
| E12 | Amount in webhook ≠ our record | Silent financial drift | Halt, flag for review, never overwrite |
| E13 | Test event to production | Real order mutated | `livemode` check (§7.5) |
| E14 | Invalid signature | Forged state transition | 400, store nothing beyond an audit line |

### Money

| # | Scenario | Naive failure | Required behaviour |
|---|---|---|---|
| E15 | JPY amount | ×100 → 100× overcharge | Zero-decimal table (§5) |
| E16 | KWD amount | PSP rejects on last digit | Round up to nearest ten (§5) |
| E17 | `f64` anywhere in the path | Cent drift | `Decimal` only; clippy denies float arithmetic |
| E18 | Cart total changes after session created | Charge ≠ order total | Re-validate amount at authorize; mismatch → update session or fail |
| E19 | Order currency ≠ provider account currency | Silent FX | Reject at session creation unless the provider declares support |
| E20 | Amount exceeds `i64` minor units | Overflow | Explicit `AmountOverflow` error |

### Lifecycle and accounting

| # | Scenario | Naive failure | Required behaviour |
|---|---|---|---|
| E21 | Refunds summing over captures | Negative balance | DB invariant (§3.3) under row lock |
| E22 | Capture over authorized | PSP rejects, ledger already written | Same invariant, checked before the call |
| E23 | Two concurrent partial captures | Both read stale totals | `SELECT … FOR UPDATE` on the payment |
| E24 | Authorization expires before fulfillment | Silent revenue loss | TTL job: warn at T−48h, mark `requires_action` |
| E25 | Refund on unsettled async method | PSP error surfaces to operator | Capability check → reject with a clear reason |
| E26 | Refund after dispute | Money lost twice | Disputed payments block refunds (§10) |
| E27 | Dashboard refund | Ledger diverges | `charge.refunded` ingested, `created_by='provider'` |
| E28 | Order cancelled while capture in flight | Captured for a cancelled order | Cancel waits for in-flight requests to settle |
| E29 | 3DS abandoned | Inventory held forever | 30-min TTL job cancels session, releases reservations |
| E30 | Two sessions on one collection | Double authorization | One active session per collection; creating a second cancels the first |
| E31 | Cart completed by a parallel request | Two orders | Cart completion is idempotent on `cart_id` ([WORKFLOWS §1.7](WORKFLOWS.md)) |
| E32 | API key rotated mid-flight | In-flight ops fail opaquely | Keys read per-request from config; rotation drains rather than cuts |
| E33 | Provider object deleted upstream | `retrieve()` 404 during reconcile | `orphan_local` drift row, human review |
| E34 | Saved method expired | Failure at authorize | `listPaymentMethods` surfaces expiry; storefront prompts before checkout |

---

## 12. Library decision

**Recommendation: hand-roll the Stripe client on `reqwest`. No new dependency.**

`async-stripe` is the de-facto community SDK and it is now **split into per-domain crates**
(`stripe-core`, `stripe-billing`, `stripe-payment`, …) with per-resource feature flags,
specifically to control compile times. That mitigates my earlier concern — the modular version is
a far more reasonable proposition than the monolith was.

It is still the wrong call here, for reasons that are about this codebase rather than that crate:

1. **Surface.** We need ~20 endpoints: PaymentIntent create/retrieve/confirm/capture/cancel,
   Refund, Customer, SetupIntent, PaymentMethod attach/list/detach, plus webhook verification.
   Even feature-gated, we would import a generated API surface to use a twentieth of it.
2. **The money boundary is the point.** §5 must be *our* code — one audited conversion with the
   three-decimal rule, property-tested. Inheriting generated `i64` types puts the highest-risk
   function in the system outside our test suite.
3. **Idempotency is not optional here.** §6 requires the key on every mutating call plus the
   `unknown`-state resolution protocol. That is a wrapper around any SDK regardless, so the SDK
   saves less than it appears to.
4. **Compile time.** PRD risk R5. Even split, it is a large addition for twenty endpoints.
5. **Blast radius is already contained.** `PaymentProvider` is the abstraction; the client lives
   in one crate.

Realistically 600–900 lines: a typed `reqwest` client with form encoding, ~20 request/response
structs, HMAC-SHA256 signature verification (~20 lines), and the money boundary. That is ladder
rung 5 — `reqwest`, `serde`, `hmac`, `sha2` are already in the stack.

**Reverse this if:** you need Stripe Billing, Connect, Tax, or Issuing (hundreds of endpoints), or
three-plus PSPs on a tight schedule. Then take `async-stripe` with feature gates and keep §5 and
§6 as your own wrapper regardless.

### PCI scope

Card data never reaches our servers. The storefront uses Stripe Elements / Payment Element; we see
tokens and intent ids only. This keeps the deployment in **SAQ-A**, the lightest PCI-DSS
self-assessment. Any design that would put a PAN in a request to our API is rejected on those
grounds alone — it moves the merchant into SAQ-D and a different compliance universe.

---

## 13. Testing

Beyond the module checklist in [IMPLEMENTATION_PLAN §5](IMPLEMENTATION_PLAN.md):

| Layer | Requirement |
|---|---|
| Money | Property tests across all 22 special currencies; the three-decimal rule explicitly |
| Idempotency | Every mutating call twice with the same key → one provider effect |
| Ambiguity | Fault-injection proxy dropping responses after the PSP commits → E1, E2 |
| Webhooks | Replay, reorder, duplicate, unknown-session, wrong-livemode, bad signature — E7–E14 |
| Concurrency | 100 parallel captures and refunds against one payment → invariants hold, E21–E23 |
| Ledger | Property test: for random capture/refund sequences, `Σ` invariants hold at every step |
| Reconciliation | Seeded drift is detected and never auto-corrected |
| Contract | `wiremock` fixtures recorded from Stripe test mode, not hand-written |
| End-to-end | Stripe test mode: auth → partial capture → partial refund → dispute |

**No test may call the live Stripe API.** Test mode via recorded fixtures in CI; a nightly job
runs against real Stripe test mode to catch API drift.
