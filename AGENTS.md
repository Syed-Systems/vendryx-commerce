# AGENTS.md — Build rules for agents working on Stheno

Read this before writing code. It is not background reading; the rules below are enforced by CI,
and a change that violates them fails the build rather than earning a review comment.

**Spec package:** [docs/PRD.md](docs/PRD.md) · [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [docs/TRD.md](docs/TRD.md) ·
[docs/DATA_MODEL.md](docs/DATA_MODEL.md) · [docs/API_SPEC.md](docs/API_SPEC.md) ·
[docs/WORKFLOWS.md](docs/WORKFLOWS.md) · [docs/PAYMENTS.md](docs/PAYMENTS.md) · [docs/ENTITLEMENTS.md](docs/ENTITLEMENTS.md) · [docs/SECURITY.md](docs/SECURITY.md) · [docs/SITEMAP.md](docs/SITEMAP.md) ·
[docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) · [docs/TASKS.md](docs/TASKS.md) · [docs/DECISIONS.md](docs/DECISIONS.md) · [docs/DOCS_WORKFLOW.md](docs/DOCS_WORKFLOW.md) · [docs/WORK_LOG.md](docs/WORK_LOG.md) ·
[docs/STOREFRONT_THEME_ENGINE_PLAN.md](docs/STOREFRONT_THEME_ENGINE_PLAN.md) · [docs/THEME_AUTHORING.md](docs/THEME_AUTHORING.md)

**Working a task?** [TASKS.md §1](docs/TASKS.md) is the loop protocol. Rule L3 — never edit an exit
command to make it pass — is the one that keeps the loop honest.

**Handed a storefront design to build?** Read [§12](#12-frontends-visual-theming) and
[docs/THEME_AUTHORING.md](docs/THEME_AUTHORING.md) first. Since [ADR-038](docs/DECISIONS.md#adr-038)
a design becomes one theme folder, `themes/<theme-id>/`: templates plus a `theme.json` of tokens,
rendered by the engine. It is not a new frontend package and not a React app. The work is tracked
task by task in [docs/STOREFRONT_THEME_ENGINE_PLAN.md](docs/STOREFRONT_THEME_ENGINE_PLAN.md).

---

## 0. Before you touch anything

**Branch from `main`, one task per branch.** Name it `feat/<task-id>-<slug>`, `fix/...` or
`docs/...`, never after an AI product. Plan-driven work follows the pickup protocol in
[STOREFRONT_THEME_ENGINE_PLAN.md §1](docs/STOREFRONT_THEME_ENGINE_PLAN.md): claim the task on the
board in your first commit, open a draft PR, prove every acceptance criterion, then mark it done.
`origin/frontend` is historical: it has been fully merged into `main` and has no commits after
2026-10-20.

Check remote branches with `git fetch --all --prune && git branch -r` before concluding a branch
does not exist — `git branch -a` alone reads stale remote-tracking refs and will lie to you.

All of §1–§11 below is CI-enforced repo-wide; §2 matters when you touch `crates/`, §0 and §12 when
you touch `apps/`.

### Running it

`run-dev.bat` (local, uncommitted) — double-click it, or `run-dev.bat` / `run-dev.bat stop`.
It is a thin wrapper: it shells out to `scripts/dev-tenants.sh` (the same script CI runs,
`.github/workflows/ci.yml`) and then starts the four frontends. Do not re-implement tenant
creation, migrations, or owner registration in batch — that script owns them.

### This Windows box breaks four documented defaults

Verified by running them. Each costs an afternoon if rediscovered.

| Thing | What breaks | Why |
|---|---|---|
| Postgres `5432` | `bind: access to the socket is forbidden` | Windows reserves TCP `5429–5528`. Published on host **`5900`** by `docker-compose.override.yml` (untracked, local only). |
| Vite `5173/5174/5175` | `listen EACCES: permission denied` | Windows reserves TCP `5141–5240`. |
| Node `25.x` | every storefront page 500s: `localStorage.getItem is not a function` | Node 25 exposes a global `localStorage` that throws without `--localstorage-file`. Needs `NODE_OPTIONS=--no-experimental-webstorage` (set by the launcher). **Real fix is Node 22; CI pins 20.** |
| `.cargo/config.toml` | — | **Fixed.** It pinned `target-dir` to `C:/Users/Mahi/rust-target`. Builds now go to the repo-local `target/`, which `.gitignore` already covers. Set `CARGO_TARGET_DIR` in your shell if you want them elsewhere. |

Live ports: engine `9000`, storefront `8000`, admin `6000` (local) / `6100` (Docker), platform `6001` / `6201` (Docker), marketing `6002` / `6202` (Docker),
Postgres `5900`, Redis `6379`. Use Docker ports if Windows blocks the local ones.

### The build tree is large, and that is not the project's size

`target/` reached **24.7 GB** here. It was not the code. Every integration test file links its
own executable (~240 of them), and on Windows MSVC writes a full `.pdb` per binary even under
`debug = "line-tables-only"` — 25.8 MB each, so 6.2 GB for a partial build and ~15 GB for the
whole suite. `Cargo.toml` now sets `[profile.test] debug = "none"` so the throwaway test
binaries carry no symbols while `cargo build` / `cargo run` keep their line numbers.

Two consequences worth knowing:

- **A panic inside a test reports a function name, not a file and line.** That is the tradeoff,
  not a bug. A failing test already names its file.
- **The disk will fill again.** `cargo test --workspace` needs roughly 7 GB of real code on top
  of whatever is there. If a build dies with `os error 112` / "not enough space on the disk",
  that is it — not a code error. `scripts/check-disk-space.sh` exists for exactly this, and CI
  runs it before the test stage. Delete `target/debug/incremental` first; it is a regenerable
  cache and is the single largest thing you can reclaim without losing compiled artifacts.

### Frontend facts that are not in the docs

- **Tenant comes from the `Host` header**, not from login. `*.localhost` resolves inside browsers
  but **not** in Node, so storefront SSR must use `127.0.0.1`, not `localhost`. `127.0.0.1` and
  `localhost` are registered as secondary domains of the Pro tenant.
- **Storefront env file is `apps/storefront/.env.local`; admin and marketing use `.env`.**
- `apps/admin/.env.example` ships `VITE_CLERK_PUBLISHABLE_KEY=pk_test_...`. That placeholder is
  truthy and **overrides** the working dev key hardcoded in
  `apps/admin/src/providers/clerk-provider/clerk-auth-provider.tsx`. Comment it out locally.
- `apps/storefront/package.json` **was broken**: all 7 dev/build/start scripts called
  `scripts/frontend.js`, deleted in commit `70f5d81`, and 12 more referenced `matrix`/`mahal`/
  `appliance` variants whose `tsconfig.*.json` do not exist. Fixed locally to call `next`
  directly. Re-check this before assuming `npm run dev` works on a fresh clone.
- `scripts/dev-tenants.sh status` **never connects**. Its `db=` value and tenant table are
  hardcoded strings. Do not cite it as evidence about the live stack.
- npm's `allowScripts` gating blocks postinstall for `sharp`, `esbuild`, `unrs-resolver`, and
  `@clerk/shared`. Pages render anyway, but `sharp` means storefront image optimization may fail.

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

### Focused feature loop

For feature work, build and test the changed package(s) and the directly affected consumers only.
Cargo will compile the selected package's dependency closure; it must not compile unrelated
workspace packages or test targets just because they share a workspace. Run every test that covers
the changed behavior and its affected contracts, including the edge cases required by the table
above. A passing focused set is the feature gate.

Start by mapping changed files to Cargo packages, frontend apps, API contracts, and migration
checks. Typical Rust commands are:

```bash
cargo check --locked -p <changed-package>
cargo test --locked -p <changed-package> --lib
cargo test --locked -p <changed-package> --test <relevant-test-target>
cargo clippy --locked -p <changed-package> --all-targets -- -D warnings
```

Use only the commands needed for the changed behavior. Add directly affected downstream packages
and their contract/parity tests when a public DTO, event, workflow, schema, or API changes. Run
frontend checks only for the app that changed. Run repository-wide static boundary checks only when
the change can affect the boundary they enforce. Do not add or run unrelated test targets.

Use the persistent Space build cache and keep Cargo concurrency bounded to the available host
capacity. Do not run duplicate builds concurrently or clear `target`/dependency caches as a routine
step. Record each focused command and its real result; skipped checks must name the reason.

The workspace-wide commands in §7 are **integration/release gates**, not the feature loop. Run them
at an explicitly scheduled integration/release gate, or when a change to shared infrastructure,
workspace configuration, or a cross-cutting public contract makes broad validation necessary. Do
not use a workspace-wide run to compensate for not identifying the affected dependency closure.

---

## 7. Gate commands

These are broad integration/release gates. They are not required for every feature change; use §6's
focused feature loop during implementation. When a broad gate is scheduled or required by a
cross-cutting change, run it on Space and report the real output. Never claim a gate passed if it
was not run.

Order matters: format and boundaries before the long compile, `clippy` before `test` so a
warning does not cost you a test run.

```bash
# ── Rust, in CI order (ci.yml stage 1 then 2) ──
cargo fmt --all -- --check
./scripts/check-module-isolation.sh     # no lateral or upward module edges
./scripts/check-sql-boundaries.sh       # no cross-schema raw SQL
./scripts/check-shared-kernel.sh        # stheno-types stays starved
./scripts/check-module-contracts.sh     # MODULE.toml matches reality
./scripts/check-dto-separation.sh       # request DTOs are not entity types (§2.9)
./scripts/check-no-domain-gates.sh      # no Entitlements inside crates/modules/
./scripts/check-no-in-memory-stores.sh  # no fake persistence (§2.8)
./scripts/check-no-enterprise-paths.sh  # no bypass of the public surface
./scripts/check-migration-phases.sh     # expand/contract pairing (§2.13)
./scripts/check-ci-postgres.sh
./scripts/check-upstream-pin.sh
./scripts/check-disk-space.sh           # before a full test run
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-schema-drift.sh

# ── Frontend, in CI order (ci.yml stage 1 then 2) ──
npm --prefix apps/admin      run typecheck
npm --prefix apps/storefront run typecheck      # == npx tsc --noEmit
# Per-theme typecheck, only when you add a frontend under packages/:
cd apps/storefront && for cfg in tsconfig.*.json; do [ "$cfg" = "tsconfig.json" ] && continue; npx tsc --noEmit -p "$cfg"; done
./scripts/check-frontend-not-forked.sh   # an independent frontend is not frontend-default repainted
./scripts/check-frontend-switch.sh       # the bundler binds the one it was asked for
./scripts/check-storefront-boundaries.sh # a frontend may not reach into core
./scripts/check-frontend-purity.sh       # a frontend presents; it does not fetch
./scripts/check-frontend-pin.sh          # every frontend pins a core major
./scripts/check-frontend-resolution.sh   # core binds one frontend, in one place
npm --prefix apps/admin      test && npm --prefix apps/admin      run build
npm --prefix apps/storefront run build

# ── Only when these are touched ──
cargo deny check
cargo run -p stheno-cli -- openapi --check       # SDK drift
python scripts/doc-sync-workflow.py check        # doc sync & pricing invariants
./scripts/check-image.sh                         # distroless size + cold-boot budgets
./scripts/check-disclosure.sh                    # security disclosure + SBOM
./scripts/check-pci-saq-a.sh                     # checkout script integrity
./scripts/check-provider-conformance.sh          # provider fixtures / conformance
k6 run tests/load/<phase>.js                     # at phase gates
```

**On Windows: not every gate has a `.ps1` twin.** 20 of the 70 `check-*`/`gate-*` scripts are
bash-only, and they include **every frontend gate** — `check-frontend-not-forked`,
`check-frontend-switch`, `check-storefront-boundaries`, `check-frontend-purity`,
`check-frontend-pin`, `check-frontend-resolution`, plus `check-ci-postgres`, `check-disk-space`,
`check-docs-links`, `check-upstream-pin`, `check-extraction-parity`, `check-pool-checkout`,
`check-playbook`, `check-control-plane-audit`. Run them under Git Bash
(`C:\Program Files\Git\bin\bash.exe`) or WSL. Do not assume a Windows-native equivalent exists.

`check-frontend-switch.sh` needs `STHENO_FRONTEND` set and **fails under `--turbopack`**
(`next.config.js` rewrites through the webpack resolver, which Turbopack bypasses).

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

## 12. Frontends: Visual Theming (ADR-038, replacing ADR-031)

<a id="12-frontends-visual-theming"></a>

### 12.0 Where this is going (ADR-038, accepted 2026-11-27)

[ADR-038](docs/DECISIONS.md#adr-038) replaces the Next.js storefront with pages rendered by the
Rust engine from `minijinja` template themes. The migration is in progress; the task board in
[docs/STOREFRONT_THEME_ENGINE_PLAN.md](docs/STOREFRONT_THEME_ENGINE_PLAN.md) says what is done.
These rules apply to all new storefront work now:

1. **A design is one theme folder, `themes/<theme-id>/`**: templates, one stylesheet, interaction
   scripts, theme art, and a `theme.json`. It touches nothing outside its folder.
2. **Everything visual and every piece of copy is a token in `theme.json`.** No colour, font,
   radius, shadow or spacing literal anywhere else. Merchants change values in the admin
   Appearance screen; the engine validates them.
3. **A theme contains no data.** Products, prices, stock, reviews, policies and store identity come
   from engine loaders only. No invented figure renders (see "no unsourced figures" below).
4. **Templates and scripts are sandboxed.** No fetch except same-origin `/store/*`, no money maths,
   no `innerHTML`, no `|safe` on loader data, no third-party origins.
5. **Checkout, payment, sign-in, account forms and the Free-plan attribution are core partials**
   a theme includes and cannot override.
6. **No Node.js storefront, no Cloudflare Workers.** New storefront features are built in the
   engine (`stheno-storefront`), not in `apps/storefront`.

`scripts/check-theme-pack.sh` enforces rules 1–5 once task T1.9 lands. The full authoring contract
is [docs/THEME_AUTHORING.md](docs/THEME_AUTHORING.md).

**Until the cut-over (task T4.7)** the Next.js storefront described in §12.1–12.4 keeps serving
production. Fix bugs in it; do not add features or themes to it.

### 12.1 The Unified Runtime (transitional, ADR-031)

The current production storefront is a single, unified Next.js runtime ([ADR-031](docs/DECISIONS.md#adr-031), superseded by ADR-038) that serves all multi-tenant branded storefronts. **We do not scaffold or deploy separate `frontend-*` packages for different tenants.** The battle-tested, accessible DOM structure of `frontend-default` is the canonical chassis for every store until the cut-over.

Scaffolding a new frontend package (`apps/storefront/packages/frontend-<name>`) for purely visual differentiation is an architectural violation. The CI pipeline and architectural guidelines reject it. The concept of "ADAPT vs SKIN" is deprecated.

### 12.2 Dynamic Theming and Variant Modifiers (transitional, ADR-031)

All visual variations must be structured through design tokens and data attributes on `frontend-default`. Do not fork or rewrite JSX to achieve a new design.

1. **Runtime Design Token Injection**: Tenant-specific design tokens are injected dynamically on SSR. These govern colors (`--brand-primary`, etc.), typography (`--brand-font-family`), corner radii (`--brand-radius-card`), and materials (e.g., frosted glass).
2. **Config-Driven Visual Variant Modifiers**: If a design requires structural layout shifts (e.g., Apple sticky 2-column layout vs inline, square vs portrait media), these are implemented using semantic data attributes (e.g., `data-card-style="apple-minimal"`, `data-image-aspect="portrait"`) on mature `frontend-default` components.
3. **Apple HIG Baseline**: The default aesthetic baseline strictly follows Apple Human Interface Guidelines: 8pt spatial grid, generous whitespace, tabular pricing numerals, smooth 16px squircles, pill buttons, translucent frosted materials, and fluid spring transitions.

### 12.3 Routes

A design's own pages (`/about`, `/policies/...`) have no core route, and the storefront may not route
— core owns every URL shape ([ADR-013](docs/DECISIONS.md#adr-013) decision 4,
`check-storefront-boundaries.sh`). Say so and stop; the fix is a core route plus a recorded
decision. [ADR-020](docs/DECISIONS.md#adr-020) is the open one.

A design does not get to rename existing URLs either. `/shop` does not become the store route
because a design called it that — SEO and sitemap contracts cover those, and the Store API is
frozen ([§2.11](#211-store-api-is-frozen)).

### 12.4 How it is actually laid out

Only three packages exist under `apps/storefront/packages/`:

- `storefront-core` — data loaders, middleware, region handling, shared UI. A frontend may consume
  this but must never reach past it.
- `frontend-default` — the canonical chassis and the only supported storefront.
- `frontend-fixture-violating` — a deliberate violator used by the frontend gates.
  `check-frontend-not-forked.sh` explicitly skips `frontend-fixture-*`, so do not "fix" it and
  do not count it when you reason about how many frontends ship.

The Next app itself is `apps/storefront` root: `src/app/[countryCode]/`, `src/middleware.ts`,
`next.config.js`. `next.config.js` reads `STHENO_FRONTEND` (default `default`) and rewrites
through the webpack resolver, so **`next dev --turbopack` silently serves the wrong frontend** —
`check-frontend-switch.sh` exists because a build once reported "Compiled successfully" while
shipping `frontend-default` markup. Use the plain webpack dev server.

`docs/VENDRYX_SCREEN_MATRIX.md` is the UI catalogue.
[docs/archive/FRONTEND_MATRIX.md](docs/archive/FRONTEND_MATRIX.md) is the historical matrix.

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

A theme's hero shipped "120 references in stock · 24 month warranty · 48h authentication".
All three were invented to balance a layout. See
[FRONTEND_MATRIX.md §5](docs/archive/FRONTEND_MATRIX.md).


<!-- BEGIN UNIVERSAL AGENT GUARDRAILS -->
## Universal build, test, and artifact rules

Apply these rules to every project under `D:\Syed Systems`, alongside the nearest project-specific `AGENTS.md` and documented workflow.

1. **Inspect before running commands.** Check the repository instructions, package/build scripts, current Git status, and whether a relevant dev server or build is already running. Use the repository's actual package manager and commands; do not assume one stack's commands apply everywhere.
2. **Use the shortest reliable feedback loop.** Start with formatting, lint, type checking, and the narrowest test that covers the change. Then check/test the changed package or module and its direct consumers as needed. UI changes can use the existing dev server, hot reload, or preview. Tests may compile the changed code and required dependencies; that is normal and does not mean the whole app must be packaged again.
3. **Reuse incremental work.** Keep using the existing checkout, dependency installation, build directory, and supported local cache. Do not run `clean`, clear caches, reinstall dependencies, or rebuild unrelated packages/platforms as a reflex. Do not launch duplicate builds for the same checkout.
4. **Run full builds when they answer a real requirement.** Use a full app/package build for packaging or deployment validation, release readiness, build-system/toolchain/dependency changes, or when the project's required CI/gate calls for it. A clean reproducibility build belongs at the appropriate CI/release boundary, not after every small edit. If a user or project gate explicitly requires a full build, follow it and report its disk/time implications where material.
5. **Keep artifacts contained.** Put generated outputs, logs, reports, and packages in the project's documented build/output directories. Keep generated files and caches out of Git. Do not make extra project copies to test; use the current checkout, or a worktree only when isolation is required. Do not store redundant logs or packaged builds in source directories.
6. **Treat caches as useful, disposable performance data.** Do not delete caches automatically or claim a check has zero disk cost. If disk pressure warrants cleanup, identify the exact generated/cache paths and their sizes, avoid paths used by active processes, and use the build tool's supported cleanup mechanism. Preserve source, user data, dependency stores, and release artifacts unless the user explicitly asks to remove them.
7. **Keep CI reproducible.** CI should run the required checks from a clean checkout and may restore/save bounded caches keyed by relevant inputs. Avoid rebuilding the same target multiple times in one workflow when a prior job artifact or cache can serve the required validation.
8. **Report what actually ran.** State the focused checks and broader gates run, skipped checks and why, and any build/cache cleanup performed. Never imply that a packaged app, another platform, or the full repository was validated when it was not.
<!-- END UNIVERSAL AGENT GUARDRAILS -->

## graphify

Graphify links: [Interactive graph](graphify-out/graph.html) | [Architecture report](graphify-out/GRAPH_REPORT.md).
- For source exploration, do not use grep/rg to locate files. Query the graph first, then open the exact paths it returns.

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or instructions before doing anything else.

Rules:
- When graphify-out/graph.json exists, run `graphify query "<question>"` before your first grep, search or source-file read for a codebase question, and put this rule in every subagent prompt that explores code. Read raw files only for lines the graph or an earlier read already located. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
- A committed graphify-out/graph.json is rewritten whole on every build, so a large diff to it in a pull request is expected. Do not flag it as churn or suggest skipping the graph rebuild or its hooks.
