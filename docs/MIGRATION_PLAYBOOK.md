# Migration Playbook

**How to move a module from `RwLock<HashMap>` to Postgres.**

Written from the three modules migrated in P11 — `region` (P11-17), `currency` (P11-18) and
`product` (P11-19, P11-20). P12–P14 follow it **unamended**. If a module needs a step that is
not here, that is a signal the abstraction is wrong ([PHASE2_PLAN §4.2](PHASE2_PLAN.md)) —
fix the abstraction and amend this document in the same change, rather than working around it
in the module.

`./scripts/check-playbook.sh` verifies mechanically that every module declared migrated
actually follows the steps below. It is not a reading comprehension test: it checks the tree.

---

## 0. Before you start

A module is migrated when **all** of these hold ([PHASE2_PLAN §6](PHASE2_PLAN.md)):

- no `RwLock<HashMap>` in its `service.rs`;
- every entity has a migration, a `CrudEntity` impl, and an index for each filterable column;
- its tests run against Postgres in CI and pass;
- its list endpoints honour `FilterSpec`;
- **its rows survive a process restart**, proven by a test;
- its latency budget is re-measured and committed.

Anything less is decorated, not migrated.

---

## 1. Write the migration

One directory per change at the repository root:

```
migrations/<timestamp>_<module>_schema/{up.sql,down.sql}
```

**The timestamp must be globally unique.** Diesel keys applied migrations by the prefix before
the first `_`, so two directories sharing a version means the second is *silently never
applied*. `migration_runner::migration_versions_are_unique` fails the build if you get this
wrong.

Ordering is global, not per module: `order` references tables that `product` and `cart`
create. Pick a timestamp that sorts after the modules you depend on.

Every table gets:

| Column | Why |
|---|---|
| `id text PRIMARY KEY` | `<prefix>_<ULID>`, wire-identical to Medusa (ADR-010) |
| `created_at`, `updated_at` `timestamptz NOT NULL DEFAULT now()` | |
| `deleted_at timestamptz NULL` | The repository's soft delete reads and writes this by name |

**`deleted_at` is not optional.** `PgCrudRepository::retrieve` filters on it and
`soft_delete`/`restore` write it; a table without it fails at runtime, not compile time.

Then:

- an index for **every column a list endpoint filters on**, partial on `WHERE deleted_at IS NULL`
  where the filter always excludes deleted rows;
- a `UNIQUE` index for every uniqueness rule — *in the database*, never a read-then-write check
  in the service, which two concurrent writers can both pass;
- `CHECK` constraints for invariants the service would otherwise have to remember
  (`currency.currency` checks lowercase codes and an ISO decimal range);
- `down.sql` that actually reverses `up.sql`. The drift check reverts a migration to prove it
  can detect drift, so an unreversible migration breaks P11-06.

Reference data (ISO currencies, country lists) is seeded **by the migration**, not by the
service constructor. It then ships with the schema and survives a restart.

## 2. Make the entities persistable

In `models.rs`, add `Deserialize` alongside `Serialize` — the repository reads rows back as
JSON via `row_to_json`, so an entity that cannot deserialize cannot be persisted — then one
impl per entity:

```rust
impl CrudEntity for Region {
    const TABLE: &'static str = "region.region";   // schema-qualified
    const ENTITY: &'static str = "region";         // used in NotFound / Conflict errors

    fn id(&self) -> String { self.id.to_string() }
    fn is_deleted(&self) -> bool { self.deleted_at.is_some() }
    fn set_deleted(&mut self, deleted: bool) {
        self.deleted_at = if deleted { Some(Utc::now()) } else { None };
    }
}
```

Entities keyed on something other than `id` — reference tables with a natural key — override
`const ID_COLUMN`. `currency` keys on `code`. Getting this wrong produces
`column "id" does not exist` at runtime.

Declare the id prefixes in the same file via `stheno_types::prefixed_id!` (ADR-010).

For a module with many entities, write the impls as one generated block rather than by hand —
`stheno-product` has twelve.

## 3. Rewrite the service

Replace every `Arc<RwLock<HashMap<..>>>` field with a `PgCrudRepository<T>`, and take a pool:

```rust
pub struct RegionService {
    regions: PgCrudRepository<Region>,
    countries: PgCrudRepository<Country>,
}

impl RegionService {
    pub fn new(pool: DbPool) -> Self { /* one repository per entity, pool cloned */ }
}
```

Then, method by method:

- `map.get(id)` → `repo.retrieve(None, id).await` — which already excludes soft-deleted rows;
- `map.values().filter(..)` → `repo.list(None, &filter)` with a `FilterSpec` carrying
  `deleted_at IS NULL`, an explicit order, **and a page limit**. An unbounded list is a
  sequential scan waiting for production data;
- `map.insert` → `create` / `update` / `upsert`;
- marking `deleted_at` by hand → `soft_delete`;
- **delete every `id.ends_with(caller_id)`**. Ids carry their prefix now (ADR-010), so equality
  works; suffix matching is a full table scan on an indexed column.

Where uniqueness was enforced by a lookup before insert, **delete the check** and let the
unique index reject it — map the resulting `Error::Conflict` to the module's domain error:

```rust
match self.variants.create(None, variant).await {
    Err(Error::Conflict { .. }) => Err(Error::Conflict {
        code: "variant_option_combination_exists", /* ... */ }),
    other => other,
}
```

Work that must commit or roll back together goes inside `with_transaction`, passing
`Some(conn)` to each repository call so they join the caller's transaction rather than opening
their own.

## 4. Convert the tests

Every test gets its own database:

```rust
let db = TestDb::migrated().await;
let service = ProductService::new(create_pool(&db.url, 5).expect("pool"));
// ...
db.drop().await;                 // once per test, not once per file
```

**Every test that creates a database must drop it**, or a suite run leaves one abandoned
database behind per test. `db.drop()` is async and therefore cannot be a `Drop` impl.

Add the two tests that distinguish a migration from a decoration:

1. **Rows outlive the service instance** — write through one service, read through a second
   built over the same pool. `check-playbook.sh` requires this test by name pattern.
2. **A soft delete is hidden but retained** — the row disappears from reads and lists, and is
   still on disk.

Prefer asserting *why* something failed over asserting that it failed. `assert!(result.is_err())`
is also satisfied by "relation does not exist", which is how a missing table can look like a
working constraint.

## 5. Wire up the call sites

`Service::new()` → `Service::new(pool)` breaks every construction site. Expect:

- `crates/stheno-server/src/lib.rs` — the composition root;
- the parity tests in `crates/api/*/tests/`;
- workflow tests in `crates/flows/*/tests/`.

Each becomes a `TestDb::migrated()` plus a pool. There is no in-memory mode to fall back to,
by design ([PHASE2_PLAN §1.3](PHASE2_PLAN.md)).

## 6. Declare it migrated

Add the module to the `MIGRATED` list in `scripts/check-no-in-memory-stores.sh`, with its task
id in a comment. From then on, reintroducing a `HashMap` fails the universal gate.

Never remove an entry to make the check pass — that is rule L3.

## 7. Run the gates

```bash
export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno
cargo test -p <module>
./scripts/check-no-in-memory-stores.sh
./scripts/check-schema-drift.sh
./scripts/gate-p11.sh          # or the current phase gate
```

---

## Pitfalls, all of which cost real time in P11

**A new migration is not embedded until `stheno-db` recompiles.** `embed_migrations!` reads a
directory cargo does not track. `crates/core/stheno-db/build.rs` emits the `rerun-if-changed`
that fixes this; the symptom without it is `relation "x" does not exist` a long way from its
cause.

**Duplicate migration version numbers are silently ignored**, not an error. See §1.

**`diesel_async::RunQueryDsl` shadows inherent methods named `execute` and `load`.** With it in
scope, `db.execute(sql)` and `counter.load(Ordering)` resolve to diesel and fail with baffling
trait errors. Call the real one fully qualified: `TestDb::execute(&db, sql)`.

**`CREATE DATABASE` cannot run inside a transaction**, and `batch_execute` opens one. Issue
those statements one per `sql_query(..).execute(..)`.

**`stheno_types::Error` cannot implement `From<diesel::result::Error>`** — `stheno-types`
depends on no other crate. Use `with_transaction`, which owns that conversion, instead of
diesel's `transaction()` directly.

**A serialisation failure raised inside your closure is already a domain error** by the time
the retry helper sees it. `with_serializable_retry` matches Postgres' message as well as the
typed error for exactly this reason; do not "simplify" that to a single check.

**Comparisons happen in the column's type, not in text.** `FilterSpec` binds values through
`json_populate_record`, so `quantity > "9"` is correctly false for a quantity of 10. If you
ever bypass `FilterSpec` with hand-written SQL, you own that problem yourself.
