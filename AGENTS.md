# AGENTS.md — Build rules for agents working on Stheno

Read this before writing code. It is not background reading; the rules below are enforced by CI,
and a change that violates them fails the build rather than earning a review comment.

**Spec package:** [docs/PRD.md](docs/PRD.md) · [docs/TRD.md](docs/TRD.md) ·
[docs/DATA_MODEL.md](docs/DATA_MODEL.md) · [docs/API_SPEC.md](docs/API_SPEC.md) ·
[docs/WORKFLOWS.md](docs/WORKFLOWS.md) · [docs/PAYMENTS.md](docs/PAYMENTS.md) · [docs/ENTITLEMENTS.md](docs/ENTITLEMENTS.md) · [docs/SECURITY.md](docs/SECURITY.md) · [docs/SITEMAP.md](docs/SITEMAP.md) ·
[docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) · [docs/TASKS.md](docs/TASKS.md) · [docs/DECISIONS.md](docs/DECISIONS.md)

**Working a task?** [TASKS.md §1](docs/TASKS.md) is the loop protocol. Rule L3 — never edit an exit
command to make it pass — is the one that keeps the loop honest.

**Handed a design to build?** Read [§12](#12-frontends-adapting-a-design) first. `ADAPT` means a new
frontend module built from that design — not `frontend-default` repainted. Guessing that one wrong has
cost three rebuilds, and `./scripts/check-frontend-not-forked.sh` now fails the build over it.

---

## 1. Before any line of code — the ladder

Walk this and **stop at the first rung that answers the need**. State the rung you stopped at in
your response.

1. **Does this need to exist at all?** Speculative need → skip it, say so in one line.
2. **Already in this codebase?** A helper, type, step, or pattern that already exists → reuse it.
   Re-implementing what lives two crates over is the most common failure here.
3. **Standard library does it?** Use it.
4. **Native platform feature covers it?** A Postgres constraint over application validation, a
   database default over an insert-time branch, a partial index over a filtered scan.
5. **An already-declared dependency solves it?** Use it. **Never add a crate for what twenty lines
   do** — see §9.
6. **Can it be one line?** Write the one line.
7. **Only then** — the minimum implementation that works.

**Never cut on the ladder:** input validation at trust boundaries, error handling that prevents
data loss, money correctness, security, or accessibility. Rung 7 means *minimum*, not *incomplete*.

When a rung above 7 fired, say which: *"rung 2 — `allocate_proportionally` already exists in
`stheno-types::money`."*

---

## 2. The rules that fail the build

### 2.1 Module isolation — the four channels

**A crate in `crates/modules/` must never depend on another crate in `crates/modules/`.** Modules
are leaves.

```toml
# crates/modules/stheno-cart/Cargo.toml
[dependencies]
stheno-types = { workspace = true }      # ✅ core
stheno-db    = { workspace = true }      # ✅ core
stheno-product = { workspace = true }    # ❌ BUILD FAILURE
```

Cross-module interaction has exactly four legal channels:

| Need | Channel | Where the code goes |
|---|---|---|
| Write across modules | **Workflow** | `crates/flows/stheno-flows-<domain>/` |
| Read across modules | **Query** | `stheno-query`, via link definitions |
| React to a change | **Event** | `crates/subscribers/` — **never inside a module** |
| Associate two records | **Link** | `stheno-links` |

If you need product data inside a cart operation, you are writing a workflow, not a service method.

**The event trap:** putting a subscriber inside a module forces that module to depend on the
emitter's crate for the event type. That defeats isolation through the back door. All subscribers
live in `crates/subscribers/`.

Checked by `scripts/check-module-isolation.sh` on every CI run.

### 2.2 SQL may not cross a module schema

Each module owns one Postgres schema. Raw SQL is the one path that can cross a boundary while still
compiling.

```rust
// in stheno-pricing
sql_query("SELECT … FROM pricing.price p JOIN product.product_variant v ON …")  // ❌ FAILS CI
sql_query("SELECT … FROM pricing.price p JOIN pricing.price_rule r ON …")       // ✅
```

Need data from another module? That is the query engine's job, and it is the only crate allowed to
read across schemas. `scripts/check-sql-boundaries.sh` scans every `sql_query` literal.

### 2.3 Do not widen the shared kernel

`stheno-types` has **zero dependencies** and holds primitives only — `Money`, `Id<T>`, `Error`,
`Metadata`. No `Product`, no `Order`, no domain logic.

Two modules needing the same domain type is a signal the **boundary is wrong**. Fix the boundary or
raise it; do not put the type in the kernel. `scripts/check-shared-kernel.sh` enforces this.

### 2.4 Never gate a feature inside domain logic

Plan tiers are enforced by **not mounting** things, never by branching.

```rust
// crates/modules/stheno-promotion/src/service.rs
if !ctx.entitlements.has(Feature::Promotions) { return Err(...) }   // ❌ BUILD FAILURE

// crates/api/stheno-api-router/src/lib.rs
if ent.has(Feature::Promotions) { app = app.merge(promotion_routes()); }   // ✅
```

`crates/modules/` may not reference `Entitlements` at all — `scripts/check-no-domain-gates.sh`
fails the build. Domain code must not know that pricing tiers exist. A disabled route returns
**404**, never 403. See [ENTITLEMENTS.md §5.1](docs/ENTITLEMENTS.md).

### 2.5 No floating point in money

```rust
let total = price * 1.2;                       // ❌ f64
let total = Money::new(dec!(19.99), Currency::USD) * dec!(1.2);   // ✅
```

`Money` has no `From<f64>` and no `as f64`. Not in tests, not in fixtures, not in a quick
calculation. Clippy denies `float_arithmetic` in every domain crate.

### 2.6 Every workflow step is idempotent

A step re-invokes after a crash. A compensation may run twice. Both must be safe.

```rust
async fn invoke(&self, ctx: &StepContext, input: Reserve) -> Result<Reservation> {
    if let Some(r) = self.repo.find_by_line_item(ctx.conn(), &input.line_item_id).await? {
        return Ok(r);                    // ✅ already done, return it
    }
    self.repo.create(ctx.conn(), input).await
}
```

Verified by the kill-and-resume harness. A non-idempotent step passes unit tests and fails
production.

### 2.7 Never call a payment provider without an idempotency key

Every mutating call to a PSP goes through `idempotent_call()`. Never call the client directly.

```rust
self.stripe.capture(intent_id, amount).await?;                    // ❌ unprotected
idempotent_call(ctx, Operation::Capture, payment_id, |key| {      // ✅
    self.stripe.capture(intent_id, amount, key)
}).await?;
```

A timeout means **unknown**, never failure. Resolving it by retrying without a key double-charges
a real customer. Equally: money crossing a provider boundary uses `to_minor_units` /
`from_minor_units` and nothing else — a provider crate calling `.to_i64()` on a `Decimal` is a
review failure. See [PAYMENTS.md](docs/PAYMENTS.md) §5, §6.

### 2.8 Explicit connections

Every service method takes `&mut Conn<'_>`. There is no ambient transaction context.

```rust
pub async fn create(&self, conn: &mut Conn<'_>, input: CreateProduct) -> Result<Product>
```

The caller owns the transaction boundary, because a workflow step spanning two modules needs one
transaction across both and an implicit context makes that invisible.

### 2.9 Request DTOs are never entity types

An entity struct must not derive `Deserialize`. Ever.

```rust
#[derive(Deserialize)] pub struct Product { ... }        // ❌ mass assignment
async fn create(Json(p): Json<Product>) -> ...           // ❌ client sets id, status

#[derive(Deserialize, Validate)] pub struct CreateProductRequest {
    pub title: String, pub handle: Option<String>,       // ✅ explicit allowlist
}
```

Store and admin DTOs stay separate even for the same entity — the admin may set `unit_price`,
`status`, and `metadata`; the store must not. A handler whose body type is an entity fails CI.

### 2.10 Never look up a customer-owned resource by raw id

```rust
async fn get_order(Path(id): Path<OrderId>)              // ❌ IDOR
async fn get_order(Owned(order): Owned<Order, ByCustomer>)  // ✅
```

`Owned<T, Scope>` resolves and verifies ownership in one step. Return **404, not 403**, on
ownership failure — a 403 confirms the record exists. See [SECURITY §7](docs/SECURITY.md).

### 2.11 Store API is frozen

Anything under `crates/api/store/` **or** `crates/api/admin/` is a compatibility contract with Medusa
v2 — the forked storefront and dashboard both depend on it. Changing
a response shape — including adding a field that alters serialisation order or nullability —
requires a parity-suite update and a written exemption in the PR.

Fixture diffs are not flaky. If a parity test fails, you broke compatibility.

### 2.12 No index without a named query

Adding an index requires naming the query it serves, in the migration comment:

```sql
-- serves: GET /admin/orders?customer_id=&order=-created_at
CREATE INDEX idx_order_customer_created ON "order" (customer_id, created_at DESC)
    WHERE deleted_at IS NULL;
```

### 2.13 Destructive migrations are phased

`DROP COLUMN`, `ALTER TYPE`, and `NOT NULL` without a default must be marked `-- phase: contract`
and paired with an earlier `-- phase: expand`. CI rejects unpaired destructive statements.

---

## 3. Adding a module

```bash
stheno new-module discount-codes
```

Generates the crate, migration directory, service skeleton, events file, tests, and registry entry.
Then:

1. Fill in `MODULE.toml` — schema, owned tables, published events, links ([TRD §4.1.1](docs/TRD.md)).
2. Define entities in `models.rs` as `pub(crate)`; write the migration creating **its own Postgres
   schema**; run `diesel print-schema` and **commit** `schema.rs`.
3. Derive the service: `#[derive(SthenoService)] #[service(models(…))]`.
4. Define `dto.rs` — what crosses the boundary. Never re-export `models.rs`.
5. Declare `FilterSpec` for every list endpoint.
6. Define events in `events.rs`; emit them inside the transaction. **Subscribe to nothing.**
7. Write workflows in `crates/flows/stheno-flows-<domain>/`, never in the module.
8. Put any event handlers in `crates/subscribers/`.
9. Add routes in the matching `api/store/*` or `api/admin/*` crate.
10. Register in `AppContext` in `stheno-server`.
11. Work through the module checklist in
    [IMPLEMENTATION_PLAN.md §5](docs/IMPLEMENTATION_PLAN.md).

---

## 4. Adding a workflow

Workflows live in `crates/flows/`, grouped by domain. Before writing one, check
[WORKFLOWS.md §2.2](docs/WORKFLOWS.md) — the cross-cutting step library probably already has what
you need (rung 2).

```rust
workflow! {
    name: "apply-discount-code",
    input: ApplyDiscountInput,
    options: { store: true, idempotent: true },
    steps: {
        cart  = LoadCartStep(input.cart_id),
        promo = ValidateCodeStep(input.code)   [retries: 1],
        apply = ApplyAdjustmentStep { cart: cart, promotion: promo },
    },
    output: apply.cart,
}
```

Rules:

- Every step that mutates state has a `compensate`. If it genuinely needs none, mark
  `no_compensation` explicitly — never leave it implicit.
- Side effects that must not roll back the transaction (emails, analytics, index sync) go in a
  final parallel group marked `no_compensation`.
- Set `store: false` only for pure-read flows.
- Anything touching money or inventory sets `idempotent: true`.

---

## 5. Adding a provider

1. New crate under `crates/providers/`, depending only on the module that owns the trait.
2. Implement the trait fully. A `todo!()` in a provider is a runtime panic in a checkout.
3. Register in `AppContext` behind a cargo feature.
4. Test against a `wiremock` fixture, not the live service.
5. Provider traits are **semver-stable within a major version** — adding a method without a
   default implementation is a breaking change.

---

## 6. Testing requirements

| Change | Required tests |
|---|---|
| Any domain logic | Unit test |
| Money, totals, allocation, tax | **Property test** — sums must be exact |
| Any workflow | Compensation test at every step position |
| Any workflow touching money or inventory | Idempotency + concurrency test |
| Any payment provider call | Ambiguity test — response dropped after the PSP commits |
| Any new endpoint | Authorization matrix entry: every role × ownership case |
| Any new jsonb column that can hold request data | Added to the data inventory in [SECURITY §11.1](docs/SECURITY.md) — same PR |
| Any Store endpoint | Parity fixture |
| Any list endpoint | Filter, sort, and pagination tests |
| Any migration | Applies clean; schema-drift check green |

Non-trivial logic leaves **one runnable check** behind — the smallest thing that fails if the
logic breaks. No frameworks, no fixture pyramids, no per-function suites unless asked.

Trivial one-liners need no test. YAGNI applies to tests too.

---

## 7. Gate commands

Run these before claiming anything is done. Quote real output — a summary of a test run you did
not perform is worse than no claim at all.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
./scripts/check-module-isolation.sh     # no lateral or upward module edges
./scripts/check-sql-boundaries.sh       # no cross-schema raw SQL
./scripts/check-shared-kernel.sh        # stheno-types stays starved
./scripts/check-module-contracts.sh     # MODULE.toml matches reality
./scripts/check-no-domain-gates.sh      # no Entitlements inside crates/modules/
./scripts/check-schema-drift.sh
cargo run -p stheno-cli -- openapi --check      # SDK drift
k6 run tests/load/<phase>.js                    # at phase gates
```

Storefront — run these when a frontend module changes (see [§12](#12-frontends-adapting-a-design)):

```bash
npm --prefix apps/storefront run typecheck
./scripts/check-storefront-boundaries.sh   # a frontend may not reach into core
./scripts/check-frontend-purity.sh            # a frontend presents; it does not fetch
./scripts/check-frontend-pin.sh               # every frontend pins a core major
./scripts/check-frontend-resolution.sh        # core binds one frontend, in one place
./scripts/check-frontend-switch.sh            # the bundler binds the one it was asked for
./scripts/check-frontend-not-forked.sh        # an independent frontend is not frontend-default repainted
```

---

## 8. Code style

- Match the surrounding code. Its conventions win over your preferences.
- Comments explain **why**, never what. The code says what.
- Errors carry context: `NotFound { entity: "product", id }`, never a bare string.
- No `unwrap()` or `expect()` outside tests and `main`. `?` or an explicit match.
- No `async` on a function that never awaits.
- Public items in `lib.rs` get doc comments. Private ones get them only when non-obvious.
- Mark deliberate shortcuts with a `ponytail:` comment naming the ceiling and the upgrade path:

```rust
// ponytail: linear scan over rules, fine under ~50 per price set.
// Index by attribute if a catalog shows up with more.
```

Those markers are harvested into a debt ledger. An unmarked shortcut is just a bug someone will
find later.

---

## 9. Dependencies

Adding a crate to the workspace requires, in the PR description:

1. What it does that twenty lines of our own code would not.
2. Its transitive dependency count.
3. Its maintenance status — last release, open issue count.
4. Its license.

`cargo deny` enforces the license allowlist. **Default answer is no.** The value of this codebase
over a Node one is partly that its dependency surface stays small (PRD problem P5) — do not
casually give that away.

---

## 10. Scope discipline

- Change only what the task requires. Every changed line should trace to the request.
- Do not "improve" adjacent code, comments, or formatting while you are in the file.
- Notice unrelated dead code? Mention it. Do not delete it.
- Never log, store, or place in `metadata`: passwords, tokens, card numbers, CVV, or full PII.
  The redaction layer is a backstop, not permission to be careless.
- Remove imports and helpers that **your** change orphaned. Nothing else.
- Beyond-parity work is the closed list in [PRD §5.13](docs/PRD.md). Anything else is 1.1.

---

## 11. Reporting

- Say what you actually did and what you actually ran. If tests fail, show the output.
- If you skipped part of the task, say which part and why — do not let it be discovered later.
- Name the rung the ladder stopped at.
- If a spec document is wrong or ambiguous, say so and propose the fix. These documents are
  drafts, not scripture — but change them deliberately and in the open, not by quietly
  implementing something else.
- Do not claim a phase gate is met without running the gate commands.

---

## 12. Frontends: adapting a design

<a id="12-frontends-adapting-a-design"></a>

### 12.1 What is actually being built

The engine is one backend that many frontends plug into. `apps/storefront/packages/frontend-*` are
called frontends for historical reasons, and the word has caused three rebuilds. **Read them as
frontends.** A frontend is a complete storefront UI — its own components, its own DOM, its own
design system — that satisfies [the contract](apps/storefront/packages/storefront-core/src/frontend/contract.ts)
and plugs into the engine unchanged.

The product is: *hand over a design, get a working storefront on the same trusted backend.*
Nothing about that sentence is satisfied by recolouring the frontend that already exists.

`frontend-default` is the Medusa-derived one ([ADR-002](docs/DECISIONS.md#adr-002)). It is a
reference, **not a starting point.** Copying it and restyling is the failure this section exists
to prevent, and it has happened three times.

### 12.2 `ADAPT <design>` — the standing instruction

> `ADAPT The Matrix Hours into packages/frontend-matrix.`

Build a new frontend module from that design. Three rules, in order:

1. **The design has a component → port it.** Its JSX structure, its stylesheet, its interaction
   behaviour. Not its data layer: source-shop fetching (context providers, API clients, sheet
   readers) is **dropped, not adapted** — data arrives as props ([ADR-013](docs/DECISIONS.md#adr-013)
   decision 4).
2. **The design has no such component → derive it from the design.** This is most of the work and
   the part that gets skipped. A lookbook has no address book; a watch shop has no order-transfer
   screen. Expect **two thirds of the contract to have no counterpart.** Those components are
   drawn from the design's own rules — its palette, type scale, spacing, border and motion
   language, as evidenced by the components it *did* ship. Match the components already ported
   from that design, not the ones in `frontend-default`.
3. **Never inherit frontend-default's markup.** Not as a base, not as a starting point, not "just for
   the account pages nobody looks at". If rule 2 is hard, that is the work, not a licence to fall
   back to rule 3.

**The report is per slot** — every contract slot labelled `ported` / `derived` / `forked`.
`forked` is a failure state, not a status. Example: [FRONTEND_MATRIX_SLOTS.md](docs/FRONTEND_MATRIX_SLOTS.md).

### 12.3 This is enforced, not trusted

`./scripts/check-frontend-not-forked.sh` strips every `className` from the frontend's files and
compares the remaining structure to `frontend-default`. Identical structure fails, whatever the
palette says. It also fails on any `@medusajs/ui` component import, because borrowing Medusa's
primitives keeps the DOM Medusa's after the class names stop being.

Opt a frontend in with `"stheno": { "independent": true }` in its package.json. `frontend-default` is
exempt by definition.

It is a **ratchet**, not a cliff: a frontend mid-conversion declares its remaining debt as
`stheno.forkDebt: { files, medusaUi }`, and the build fails the moment either count goes **up**.
New forking is blocked today; existing debt is paid down by lowering the numbers, and the gate
prints the new figure to lower them to. A frontend with no `forkDebt` must have zero of both.

Lower the declared debt in the same commit that lowers the real count. Leaving it high "for
headroom" re-opens the hole.

Every other storefront gate — purity, boundaries, pin, resolution — passed on all three forks.
None of them look at markup. Do not read a green build as evidence that this rule was followed.

### 12.4 `SKIN` — when a recolour is genuinely what is wanted

> `SKIN frontend-default to the Matrix palette.`

Tokens only: palette, type, radii, spacing, easings. Markup, DOM and interactions unchanged. If
you are editing JSX to satisfy a `SKIN`, you have exceeded it — stop and say so.

A `SKIN` is a legitimate, cheap product. It is only a failure when it is delivered in answer to an
`ADAPT`.

### 12.5 Routes

A design's own pages (`/about`, `/policies/...`) have no core route, and a frontend may not route
— core owns every URL shape ([ADR-013](docs/DECISIONS.md#adr-013) decision 4,
`check-storefront-boundaries.sh`). Say so and stop; the fix is a core route plus a recorded
decision. [ADR-020](docs/DECISIONS.md#adr-020) is the open one.

A design does not get to rename existing URLs either. `/shop` does not become the store route
because a design called it that — SEO and sitemap contracts cover those, and the Store API is
frozen ([§2.11](#211-store-api-is-frozen)).

---

### Standing rule — no unsourced figures

This one needs no keyword because it is never optional.

**A number rendered as fact must trace to data or to a named source.** Stock counts, warranty
terms, delivery windows, percentages, review counts, "trusted by N merchants". If the component
cannot source it from props, it does not ship — remove the element rather than filling it with a
plausible value.

Placeholder *prose* reads as placeholder. A figure in a stat position reads as measured, and a
storefront that quietly states things nobody verified is the same class of defect as
[§2.5 floating-point money](#25-no-floating-point-in-money): confidently wrong, and invisible
until it costs someone.

A frontend-matrix hero shipped "120 references in stock · 24 month warranty · 48h authentication".
All three were invented to balance a layout. See
[FRONTEND_MATRIX.md §5](docs/FRONTEND_MATRIX.md).
