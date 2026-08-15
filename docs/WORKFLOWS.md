# Workflow Engine and Catalog

**249 workflows · 288 named steps · one durable SAGA orchestrator on Postgres**

Counts measured from `medusajs/medusa@b733400` (`packages/core/core-flows`). This document
specifies the engine, then catalogues what must run on it.

---

## Part 1 — The engine

### 1.1 Why an engine at all

A checkout touches cart, inventory, payment, order, promotion, and fulfillment. Those live in
six modules with six separate table groups and no shared transaction that spans a payment
provider's network call. Something has to guarantee that a failure at step 4 undoes steps 1–3.

That something is the workflow engine. It is not infrastructure decoration — it is the mechanism
that makes a modular monolith safe to write into.

### 1.2 Semantics to reproduce

Medusa's orchestrator supports every option below. Stheno's must support all of them, because the
249 workflows use them.

**Per step:**

| Option | Behaviour |
|---|---|
| `max_retries` | Retry count on transient failure. Default 0 |
| `retry_interval` | Seconds between retries. Default: immediate |
| `auto_retry` | Retry without external intervention. Default true |
| `retry_interval_awaiting` | Poll interval for a step sitting in `WAITING` |
| `max_awaiting_retries` | Cap on awaiting polls |
| `timeout` | Seconds to wait for a response. On expiry: mark `TIMEOUT` and revert **when the response arrives** — the step is never abandoned mid-flight |
| `async` | Step suspends; completion arrives externally via `set_step_success` / `set_step_failure` |
| `background_execution` | An async step that completes itself, with no external signal |
| `compensate_async` | The compensation function is itself async |
| `no_compensation` | No rollback for this step |
| `continue_on_permanent_failure` | Mark `PERMANENT_FAILURE`, run subsequent steps anyway |
| `skip_on_permanent_failure` | `true` skips all remaining steps; a step name skips forward to that step |
| `no_wait` | Do not wait for sibling parallel steps before advancing |
| `nested` | The step contains a sub-transaction |
| `save_response` | Persist the step output. Default true |

**Per workflow:** `timeout` (global), `store` (persist state at all), `retention_time`, `idempotent`.

### 1.3 State machines

```
Transaction:
  not_started ──► invoking ──► done
                     │
                     ├──► waiting_to_compensate ──► compensating ──► reverted
                     └──► failed

Step:
  not_started ──► invoking ──► done
                     │            └──► compensating ──► reverted
                     ├──► failed / permanent_failure
                     ├──► timeout
                     └──► skipped / skipped_failure
```

Both enums serialise to the same strings Medusa uses, so an imported `workflow_execution` row
stays readable.

### 1.4 Rust API

Steps are types, so their input and output are checkable and storable:

```rust
#[async_trait]
pub trait Step: Send + Sync + 'static {
    type Input:  Serialize + DeserializeOwned + Send;
    type Output: Serialize + DeserializeOwned + Send;

    const NAME: &'static str;
    const CONFIG: StepConfig = StepConfig::DEFAULT;

    async fn invoke(&self, ctx: &StepContext, input: Self::Input) -> Result<Self::Output>;

    /// Receives the step's own output. Must be idempotent — it may run more than once.
    async fn compensate(&self, _ctx: &StepContext, _output: Self::Output) -> Result<()> { Ok(()) }
}
```

`StepContext` carries the transaction id, attempt number, the connection, the app context, and a
`tracing::Span`.

Workflows are declared with a macro that preserves type-checking between producer and consumer:

```rust
workflow! {
    name: "complete-cart",
    input: CompleteCartInput,
    options: { timeout: 60s, store: true, idempotent: true },
    steps: {
        cart      = ValidateCartStep(input.cart_id),
        promo     = ValidatePromotionsStep(cart.promotions),
        reserve   = ReserveInventoryStep(cart.items)        [retries: 2, timeout: 10s],
        payment   = AuthorizePaymentStep(cart.payment_collection)
                        [timeout: 30s, async, retry_interval_awaiting: 2s],
        order     = CreateOrderStep { cart: cart, reservations: reserve, payment: payment },
        after     = [ EmitOrderPlacedStep(order),
                      MarkCartCompletedStep(cart.id),
                      ConsumeCampaignBudgetStep(promo) ],   // parallel
    },
    output: order.order,
}
```

The macro emits a `StepDefinition` DAG plus a typed executor. A mismatch between what one step
produces and the next consumes is a **compile error**, which is the main advantage over Medusa's
runtime-checked composition.

### 1.5 Persistence

```sql
CREATE TABLE workflow_execution (
    id               text PRIMARY KEY,
    workflow_id      text        NOT NULL,
    transaction_id   text        NOT NULL,
    run_id           text        NOT NULL,
    state            text        NOT NULL,
    context          jsonb       NOT NULL,   -- step inputs, outputs, compensation stack
    attempts         int         NOT NULL DEFAULT 0,
    errors           jsonb,
    run_at           timestamptz NOT NULL DEFAULT now(),
    locked_by        text,
    locked_until     timestamptz,
    retention_time   int,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    UNIQUE (workflow_id, transaction_id, run_id)
);

CREATE INDEX ON workflow_execution (state, run_at)
    WHERE state IN ('invoking', 'compensating');
```

A checkpoint is one `UPDATE` per step boundary. That write is the engine's dominant cost and it is
the price of durability — a workflow with `store: false` skips it entirely, and pure-read flows
should use that.

### 1.6 Distributed execution

Every instance runs a poller:

```sql
UPDATE workflow_execution SET locked_by = $1, locked_until = now() + interval '30 seconds'
WHERE id IN (
    SELECT id FROM workflow_execution
    WHERE state IN ('invoking','compensating')
      AND run_at <= now()
      AND (locked_until IS NULL OR locked_until < now())
    ORDER BY run_at
    FOR UPDATE SKIP LOCKED
    LIMIT 10
)
RETURNING *;
```

`SKIP LOCKED` is the whole scheduler. No Redis, no leader election, no Temporal cluster
([ADR-003](DECISIONS.md#adr-003)).

**Crash recovery:** an instance dies holding a lock; the lock expires; another instance claims the
row and resumes from the last checkpoint. The step that was mid-flight re-invokes, which is why
**every step must be idempotent** — a rule stated in [AGENTS.md](../AGENTS.md) and enforced by the
kill-and-resume test harness.

**Lock renewal:** a long-running step renews `locked_until` on a heartbeat so it is not stolen
mid-execution.

### 1.7 Idempotency

`transaction_id` is unique per workflow. Starting `complete-cart` with an existing
`transaction_id` returns the existing execution instead of starting a second. The HTTP layer maps
the client's `Idempotency-Key` header — or the cart id, for cart completion — onto it.

This is what makes FR-CRT-7 structural: a double-submitted checkout cannot double-charge, because
the second call never becomes a second execution.

### 1.8 Observability (FR-PLUS-1)

`GET /admin/workflow-executions/:id` returns the full step timeline: name, state, attempts,
duration, input, output, error, and compensation history. The admin renders it as a timeline with
a **replay-from-step** control.

This is the single clearest operational win over Medusa, where a failed workflow is largely opaque
without reading logs. Each execution is also one OpenTelemetry trace, with a span per step.

### 1.9 Testing the engine

Non-negotiable gates, all in P1:

1. **Compensation completeness** — for every workflow, inject a failure at each step and assert the
   compensation stack unwinds in exact reverse order.
2. **Crash resumption** — SIGKILL the process at every step boundary; assert the workflow completes
   or fully reverts after restart. No partial states.
3. **Idempotency** — invoke every step twice with identical input; assert identical end state.
4. **Concurrency** — 100 concurrent starts with the same `transaction_id` produce exactly one
   execution.
5. **Timeout** — an async step exceeding its timeout reverts when the late response arrives.

---

## Part 2 — Workflow catalog

Distribution by domain, measured. Ordered by size, because size is where the risk is.

| Domain | Workflows | Steps | Phase |
|---|---:|---:|---|
| **order** | **68** | **38** | P5, P6 |
| product | 27 | 35 | P2 |
| cart | 14 | 27 | P4 |
| promotion | 11 | 12 | P4 |
| draft-order | 9 | 3 | P6 |
| rbac | 9 | 12 | P7 |
| tax | 9 | 13 | P3 |
| customer | 8 | 6 | P7 |
| fulfillment | 8 | 17 | P5 |
| settings | 8 | 9 | P8 |
| user | 8 | 5 | P7 |
| price-list | 7 | 8 | P3 |
| payment-collection | 6 | 8 | P4 |
| translation | 5 | 7 | P8 |
| api-key | 4 | 6 | P7 |
| common | 4 | 12 | P1 |
| inventory | 4 | 10 | P2 |
| invite | 4 | 6 | P7 |
| payment | 4 | 7 | P4 |
| customer-group, pricing, product-category, region, reservation, return-reason, sales-channel, stock-location, store | 3 each | 3–5 | P2–P7 |
| auth, file | 2 each | 2–4 | P7, P8 |
| line-item | 1 | 2 | P4 |
| defaults, locking, notification, shipping-options, shipping-profile | 0 | 1–3 | steps only |
| **Total** | **249** | **288** | |

**The order domain is 27% of all workflows.** It is PRD risk R1 and it drives the phase ordering:
the order-change model is built in P1 as a standalone crate with a fixture corpus, before any UI
depends on it.

### 2.1 Order workflows — the 68

Grouped by what they actually do:

| Group | Workflows |
|---|---|
| **Lifecycle** | create, cancel, archive, complete, update, transfer (request/accept/decline/cancel) |
| **Edits** | begin, request, confirm, cancel, add items, update items, remove items, add shipping, update shipping, remove shipping |
| **Returns** | begin, request, confirm, cancel, add items, update items, remove items, begin receive, receive items, confirm receive, cancel receive, dismiss items |
| **Claims** | begin, request, confirm, cancel, add inbound/outbound items, update items, remove items, add shipping |
| **Exchanges** | begin, request, confirm, cancel, add inbound/outbound items, update items, remove items |
| **Fulfillment** | create, cancel, mark shipped, mark delivered, create shipment |
| **Payment** | capture, refund, mark paid, create payment collection, cancel payment collection |
| **Change control** | create change, cancel change, confirm change, decline change, delete change actions |

Every one of these follows the same shape, which is why they can share step implementations:

```
open order_change → append order_change_action[] → preview (no write)
                                                 → confirm: apply, bump version,
                                                   recompute summary, write transaction
```

### 2.2 Cross-cutting step library

Steps reused across many workflows. Build these once, in P1–P2:

| Step | Used by |
|---|---|
| `use_remote_query` / `use_query_graph` | almost everything — the read primitive |
| `create_links`, `dismiss_links`, `update_links` | every workflow crossing a module boundary |
| `emit_event` | every mutating workflow |
| `create_entities`, `update_entities`, `delete_entities` | generic CRUD steps |
| `validate_presence` | precondition checks |
| `acquire_lock` / `release_lock` | inventory, promotion budget |
| `parallelize` | fan-out |
| `transform` / `when` | data shaping and conditional branches |

`when` deserves a note: Medusa's workflows branch on runtime data via `when(...).then(...)`. In
Rust the DAG is static, so a branch is modelled as a step whose config marks downstream steps
`skipped` when the predicate is false. Same observable behaviour, still statically typed.

### 2.3 The checkout workflow, end to end

The one that matters most. Reproduced here because it exercises every engine feature:

```
complete-cart (timeout 60s, idempotent on cart_id)
│
├─ 1. validate_cart                    compensate: none
│     cart exists, not completed, has items, has shipping + billing address
├─ 2. validate_shipping_methods        compensate: none
├─ 3. validate_promotions              compensate: none
├─ 4. recalculate_totals               compensate: none
├─ 5. reserve_inventory                compensate: release_reservations
│     retries 2, SERIALIZABLE, SELECT FOR UPDATE per level
├─ 6. authorize_payment                compensate: cancel_authorization
│     async, timeout 30s — resumes on webhook or poll
│     may return requires_more → 3DS step-up, workflow suspends
├─ 7. consume_campaign_budget          compensate: release_budget
├─ 8. create_order                     compensate: delete_order
├─ 9. create_order_links               compensate: dismiss_links
│     order↔cart, order↔payment_collection, order↔promotion
├─10. mark_cart_completed              compensate: unmark
└─11. [parallel] emit_order_placed · send_confirmation · sync_search_index
       no_compensation — side effects that must not block the order
```

Failure at step 6 releases the inventory reserved at step 5 and stops. Failure at step 8 cancels
the payment authorisation. A crash between 8 and 10 resumes at 10 — the order exists, and the
engine knows it.

Step 11 is deliberately `no_compensation`: a failed confirmation email must never roll back a paid
order. Getting that boundary right is the difference between a robust checkout and a fragile one.
