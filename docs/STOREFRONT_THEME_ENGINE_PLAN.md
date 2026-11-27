# Storefront Theme Engine — implementation plan (ADR-038)

This is the working plan for [ADR-038](DECISIONS.md#adr-038). It is written so that any agent
can pick up one task, finish it, prove it, and hand back, with no orchestrator present.

- **Why:** the owner wants one server (the Rust engine), no Node.js storefront, every visual and
  copy choice tokenized and managed from the admin center, data only from the engine, heavy edge
  caching, and AI design tools (Google AI Studio, Antigravity) that only design and tokenize.
- **What changes:** the engine renders storefront pages from `minijinja` template themes. Each
  theme is one folder with a `theme.json` of tokens. The Next.js storefront in `apps/storefront`
  keeps serving until task T4.7 deletes it.
- **Where it stands:** see the task board (section 4). Update it in the same PR as the work.

Read first, in order: [AGENTS.md](../AGENTS.md), [ADR-038](DECISIONS.md#adr-038),
[ADR-037](DECISIONS.md#adr-037) (edge and caching), [ADR-006](DECISIONS.md#adr-006) (money),
[ADR-021](DECISIONS.md#adr-021) (AI content is data), then this file.
[THEME_AUTHORING.md](THEME_AUTHORING.md) is the contract a theme author follows.

---

## 1. How to work this plan

### 1.1 Picking a task

1. Pull `main`. Read the task board (section 4).
2. Pick the **lowest-numbered task whose status is `todo` and whose dependencies are all
   `done`**. Tasks in the same phase with no dependency between them may run in parallel by
   different agents. Their "Files" lists do not overlap; that is what makes them parallel.
3. Create a branch from `main`: `feat/<task-id-lowercase>-<short-slug>`, for example
   `feat/t1-2-theme-schema`. Never use an AI product name in a branch, commit, comment or PR.
4. First commit on the branch: change the task's status on the board to
   `in progress (<branch>, <YYYY-MM-DD>)`. Push it and open a **draft PR** titled
   `<TASK-ID>: <task title>`, so other agents can see the claim.

### 1.2 Doing the task

- Touch only the files the task lists. If the task cannot be done without touching something
  else, stop and set the status to `blocked: <reason>` instead of widening the scope.
- Before writing any code, walk the ladder and stop at the first rung that answers the need:
  (1) does it need to exist at all, (2) already in this codebase, (3) standard library,
  (4) native platform feature, (5) an already-installed dependency, (6) one line,
  (7) only then the minimum implementation. Name the rung in the PR description for each
  change. Never cut validation, error handling, security or accessibility.
- For codebase questions run `graphify query "<question>"` before grep or broad file reads, and
  `graphify update .` after code changes.
- Keep the PR to one task. Small, verifiable, reversible.

### 1.3 Finishing the task

1. Run the task's **Verify** commands and the **global gates** for the areas you touched
   (section 1.4). All must pass.
2. In the PR description, copy the task's acceptance criteria as a checklist and give evidence
   for each one: the command run and its result, a test name, or a screenshot for visual work.
3. Set the task's status to `done (<PR link>)` on the board, mark the PR ready for review.
4. If you discover work that is not in any task, add it as a new task at the end of its phase
   (next free number, status `todo`, with dependencies, files, acceptance and verify). Do not
   do it in the current PR.

### 1.4 Global gates

Run these for every area your task touches. CI runs the same checks and is the final authority.

| Area | Commands |
|---|---|
| Rust | `cargo fmt -p <crate> -- --check` · `cargo clippy -p <crate> --all-targets -- -D warnings` · `cargo test -p <crate>` |
| Rust, DB-backed tests | Postgres running; `DATABASE_URL` set (see 1.5). Same `cargo test` |
| Migrations | `bash scripts/check-migration-phases.sh` · `bash scripts/check-playbook.sh` · `bash scripts/check-schema-drift.sh` |
| Admin app | `cd apps/admin && npm ci --strict-allow-scripts && npm run typecheck && npm test` |
| Themes | `bash scripts/check-theme-pack.sh` (from T1.9 onwards) |
| Docs | `bash scripts/check-docs-links.sh` |
| Shell scripts | New `.sh` files are committed executable: `git update-index --chmod=+x <file>` |

### 1.5 Local environment notes (Windows development machine)

- Rust: `cargo +stable-x86_64-pc-windows-gnullvm ...` with `CARGO_TARGET_DIR=C:\Users\Mahi\rust-target`
  and `CARGO_INCREMENTAL=0`. The msvc toolchain cannot link on this machine.
- Postgres for tests: start Docker Desktop, run `docker compose up -d postgres`, then
  `DATABASE_URL=postgres://stheno:stheno@localhost:5900/stheno` (host port 5900, because Windows
  reserves 5432 here; see the untracked `docker-compose.override.yml`).
- `cargo fmt --all` fails on Windows with "filename too long"; format per package instead.
- `core.filemode` is false on this checkout, so the executable bit must be set with
  `git update-index --chmod=+x`.

---

### 1.6 Rules that prevent guessing

These apply to every task. They exist because a plan is only as good as its least careful reader.

1. **Never invent a name.** Every function, type, file, route, table and command a task names was
   checked against the code. If one does not exist when you look, stop and set the task to
   `blocked: <name> not found`. Do not create a look-alike.
2. **Never invent data.** No product names, prices, ratings, review text, stock numbers,
   delivery promises or statistics in code, templates, fixtures shown to shoppers, or docs. Test
   fixtures may contain obviously fake values (`"Test Product"`, `"10.00"`) inside test files only.
3. **Never weaken a check to pass.** Do not delete or loosen an assertion, add `#[ignore]`, mark a
   test `skip`, widen an allow-list, or edit a gate script, unless the task says to. If a check
   fails for a reason the task does not cover, stop and report the exact output.
4. **Never claim a result you did not see.** The PR description quotes the real command output
   (the `test result:` line, the gate's last line). "Should pass" is not evidence.
5. **Stay in your files.** The task's Create/Edit lists are the only paths you may change. A needed
   change elsewhere is a new task, not part of this one.
6. **Ask by blocking, not by guessing.** A decision the task does not make (a colour, a wording, a
   behaviour) is the owner's. Set `blocked: needs owner decision: <question>` and stop.
7. **One task, one PR.** Do not start the next task in the same branch.
8. **Money is `Decimal`.** Never `f32`/`f64` for an amount (ADR-006); never compute a price, total
   or discount outside the engine's existing pricing and cart code.

## 2. Architecture

```
Shopper ─► Cloudflare (caches HTML, theme assets, catalog JSON; tenant-prefixed tags)
             └─► Cloudflare Tunnel ─► Rust engine (stheno-server)
                    ├─ storefront renderer (stheno-storefront): route ─► loader ─► template ─► HTML
                    ├─ Store API /store/*: carts, checkout, payments, account (JSON)
                    ├─ theme assets /theme-assets/<theme>/<hash>/...
                    └─ PgBouncer ─► Postgres (only on cache miss or live actions)
```

### 2.1 Theme package

```
themes/<theme-id>/
  theme.json            tokens, sections, defaults, metadata
  templates/
    layout.html.j2      <head>, header, footer, token <style> block
    index.html.j2       home
    collection.html.j2  listings: category, collection, /store
    product.html.j2     product detail
    cart.html.j2
    page.html.j2        about, policies
    account/*.html.j2   login, dashboard, profile, addresses, orders, order detail
    order/*.html.j2     confirmed, transfer, accept, decline
    partials/*.html.j2  product card, price, pagination, and so on (theme-owned)
  assets/
    theme.css           colour, font, radius, shadow and spacing only via var(--theme-*)
    islands/*.js        interaction only: cart drawer, gallery, WebGL hero
    images/*            theme art only (small); product photos never live here
```

Core-owned partials live in the engine, in `crates/storefront/stheno-storefront/templates/core/`:
checkout, payment, sign-in and account forms, and the Free-plan "Powered by Vendryx"
attribution. A theme includes them and cannot override them.

### 2.2 `theme.json`

```json
{
  "schema_version": 1,
  "id": "hydro-vessel",
  "name": "Hydro Vessel",
  "tokens": [
    { "id": "ink", "type": "color", "default": "#0E1116", "label": "Text" },
    { "id": "radius_card", "type": "range", "min": 0, "max": 32, "unit": "px", "default": 16, "label": "Card corner radius" },
    { "id": "font_heading", "type": "font", "options": ["outfit", "dm_sans"], "default": "outfit", "label": "Heading font" },
    { "id": "card_style", "type": "select", "options": ["minimal", "framed", "editorial"], "default": "minimal", "label": "Product card style" },
    { "id": "grid_columns", "type": "range", "min": 2, "max": 5, "default": 4, "label": "Products per row" }
  ],
  "sections": {
    "index": [
      { "type": "hero", "settings": [
        { "id": "heading", "type": "text", "max_length": 80, "default": "Built for altitude", "label": "Heading" },
        { "id": "image", "type": "image", "label": "Image" }
      ]}
    ]
  }
}
```

- Token types (closed set): `color`, `range`, `select`, `font`, `text`, `richtext` (sanitized
  subset), `image`, `url` (https or a same-origin path), `boolean`.
- `font` options are keys into the curated, licence-checked, self-hosted font set
  (`scripts/check-font-licences.sh`), never URLs.
- Defaults are the designer's values. Merchant overrides are stored per tenant (T2.1).
- The engine compiles colour, radius, font, shadow and spacing tokens into one `<style>` block of
  `--theme-<id>` custom properties, and exposes every token to templates as `settings.<id>` and
  every section setting as `section.settings.<id>`.

### 2.3 What templates receive

Read-only objects built by engine loaders, never by the theme: `store` (name, logo, domain,
plan, locale, currency), `settings`, `sections`, `page` (title, description, canonical URL,
Open Graph image), `product`, `collection` (products, filters, pagination), `cart` (only on the
cart page, never cached), `customer` (only on account pages, never cached), `policies`,
`regions`, `routes` (URL helpers, so templates never build URLs by hand).

Loaders call engine services in process. Field names follow the Store API DTOs, so templates
and interaction scripts see one shape. The Store API is frozen (AGENTS.md §2.11): add routes,
never change existing shapes.

Filters and functions, and nothing else: `money` (formats `stheno-types::Money`, a `Decimal`
plus currency, per ADR-006), `image_url(width)`, `asset_url`, `t` (locale strings), `date`.
Autoescaping is on; `|safe` on loader data is a gate failure.

### 2.4 Caching

- Public HTML (home, listings, product, pages): `Cache-Control: public, s-maxage=300,
  stale-while-revalidate=86400`; tenant-prefixed `Cache-Tag` (comma-separated) and
  `Surrogate-Key` (space-separated) per ADR-037 decision 3; no `Set-Cookie`. Extend
  `crates/api/stheno-api-store/src/cache.rs`; do not write a second helper.
- Never cached: cart, checkout, account and order pages, and any response that sets a cookie.
- Personalised fragments (cart count, signed-in state) load through interaction scripts calling
  `/store/*`, so public pages are identical for every shopper and cacheable at the edge.
- Purges follow ADR-037 decision 3: entity tags plus one listing tag per tenant; hostname purge
  for branding, appearance and other store-wide changes; written to the leased outbox in the same
  transaction; delivered by `crates/providers/stheno-cdn-cloudflare`.
- Correctness: a cached page may show a price up to `s-maxage` old. Cart and checkout always
  recompute from the database, so a stale page can never charge a stale price.

### 2.5 Security

Autoescape on; `richtext` sanitized at save and at render; strict Content-Security-Policy
(`script-src 'self'`, no inline scripts; the token style block is hashed); interaction scripts
call same-origin `/store/*` only; sessions in HttpOnly, Secure, SameSite=Lax cookies set by the
engine; `Host` is the only tenant identity (ADR-022); the cache key includes the host; no theme
file can reach data, money or checkout.

---

## 3. Storefront routes the engine must serve (parity list)

`/{country}` (home) · `/{country}/store` · `/{country}/categories/{...category}` ·
`/{country}/collections/{handle}` · `/{country}/products/{handle}` · `/{country}/cart` ·
`/{country}/checkout` · `/{country}/account` (login, dashboard, profile, addresses, orders,
orders/details/{id}) · `/{country}/order/{id}/confirmed` ·
`/{country}/order/{id}/transfer/{token}` with `/accept` and `/decline` · `/{country}/about` ·
`/{country}/policies/{handle}` · `/sitemap.xml` · `/robots.txt`.

The source of truth for today's behaviour is
`apps/storefront/packages/storefront-core/src/app/` until T4.7 deletes it.

---

## 4. Task board

Status values: `todo`, `in progress (<branch>, <date>)`, `blocked: <reason>`, `done (<PR>)`.

| ID | Task | Depends on | Status |
|---|---|---|---|
| T0.1 | Merge `main` into PR #4 and park the AI Studio storefront as design source | — | done (PR #4, merge commit `905a33d2`) |
| T0.2 | Land the docs for ADR-038 (this plan, AGENTS.md, ADR index, authoring contract) | — | in progress (docs/storefront-theme-engine, 2026-11-27) |
| T1.1 | `stheno-storefront` crate skeleton, mounted in the engine | T0.1 | todo |
| T1.2 | `theme.json` schema, Rust types and validator | T0.1 | todo |
| T1.3 | Theme loading from `STHENO_THEMES_DIR` and hashed asset serving | T1.1, T1.2 | todo |
| T1.4 | Token compiler and `settings` / `sections` objects | T1.2, T1.3 | todo |
| T1.5 | Loaders and filters for home, listing and product pages | T1.1 | todo |
| T1.6 | Cache headers and tenant-prefixed tags on rendered pages | T1.5 | todo |
| T1.7 | Core partials: attribution footer and the include mechanism | T1.3 | todo |
| T1.8 | Confirm Cloudflare plan facts (owner input) and record them | — | todo |
| T1.9 | `scripts/check-theme-pack.sh` gate, wired into CI | T1.2 | todo |
| T1.10 | `themes/_starter` reference theme, passing the gate | T1.4, T1.5, T1.7, T1.9 | todo |
| T2.1 | Migration: per-tenant theme selection and token values | T0.1 | todo |
| T2.2 | Admin endpoints for themes and appearance, with validation and purge | T1.2, T2.1 | todo |
| T2.3 | Admin Appearance screen generated from `theme.json` | T2.2 | todo |
| T3.1 | Convert `maison-cuir` (Maison Véla) | T1.10 | todo |
| T3.2 | Convert `hydro-vessel` (KOVA) | T1.10 | todo |
| T3.3 | Convert `homedine-eco` (Homedine) | T1.10 | todo |
| T3.4 | Port Default (Apple HIG baseline) from `frontend-default` | T1.10 | todo |
| T3.5 | Port Mahal | T1.10 | todo |
| T3.6 | Port Lustre | T1.10 | todo |
| T3.7 | Port Matrix | T1.10 | todo |
| T4.1 | Cart page and cart interaction script | T1.10 | todo |
| T4.2 | Checkout: COD, bKash, Nagad, SSLCommerz, Stripe | T4.1 | todo |
| T4.3 | Account pages and engine session cookies | T1.10 | todo |
| T4.4 | Order confirmed and order transfer pages | T4.3 | todo |
| T4.5 | About and policy pages, sitemap, robots, Open Graph, JSON-LD | T1.10 | todo |
| T4.6 | Performance acceptance (Lighthouse) on every theme | T3.1–T3.7, T4.5 | todo |
| T4.7 | Cut-over: engine serves all storefront hosts; delete `apps/storefront` | T2.3, T3.1–T3.7, T4.1–T4.6 | todo |
| T5.1 | Finalize THEME_AUTHORING.md and the AI Studio prompt | T1.10, T2.3 | todo |
| T5.2 | Decide ADR-037's reverse-proxy duties now that the engine serves the storefront | T4.7 | todo |
| T6.1 | Upgrade `diesel-async` past 0.5 and `diesel` to >= 2.3.8; remove the two ignores in `.cargo/audit.toml` | — | todo |

---

## 5. Tasks

Each task lists: **Goal**, **Files** (the only paths it may touch), **Steps**, **Acceptance**,
**Verify**. Paths under `crates/storefront/stheno-storefront/` are abbreviated `sf/`.

### Phase 0 — unblock

#### T0.1 Merge `main` into PR #4 and park the AI Studio storefront as design source

- **Goal:** PR #4 (`sync/main-2026-11-26`) merges cleanly into `main`, the current
  engine-connected storefront keeps building, and the AI Studio work from `12203039` is kept as
  design source without being part of any build.
- **Files:** `package.json`, `bun.lock`, `apps/storefront/**`, `src/assets/**`,
  `design-source/ai-studio-2026-11-27/**`.
- **Steps:**
  1. On `sync/main-2026-11-26`, `git fetch origin main` and merge `FETCH_HEAD` (no rebase, no
     force-push).
  2. Conflicts: keep PR #4's `package.json` (no root workspaces) and PR #4's
     `apps/storefront/package.json`. Delete `bun.lock`.
  3. For the files `12203039` **modified** in `apps/storefront` (`package.json`,
     `src/app/layout.tsx`, `src/middleware.ts`, `tsconfig.json`), keep PR #4's version.
  4. Move every file `12203039` **added** under `apps/storefront/` (`DESIGN_SYSTEM.md`,
     `next.config.mjs`, `postcss.config.mjs`, `tailwind.config.ts`, `src/app/page.tsx`,
     `src/app/globals.css`, `src/app/robots.ts`, `src/app/sitemap.ts`, `src/core/**`,
     `src/themes/**`, `public/media/**`, `public/brand/vendryx-mark.svg`) with `git mv` into
     `design-source/ai-studio-2026-11-27/`, keeping relative paths. Move the repo-root
     `src/assets/images/**` there too.
  5. Add `design-source/README.md`: what this folder is, that nothing builds from it, and that
     T3.1–T3.3 convert it.
- **Acceptance:** PR #4 shows no conflicts; `apps/storefront` builds with no backend running;
  no `next.config.mjs`, second Tailwind/PostCSS config, root `bun.lock` or root workspaces
  remain; CI on PR #4 passes every stage except the separate playbook task.
- **Verify:**
  `cd apps/storefront && npm ci --strict-allow-scripts && npx tsc --noEmit && STHENO_BACKEND_URL=http://localhost:9000 NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY=pk_test npm run build` ·
  `git ls-files | grep -E "bun.lock|next.config.mjs" ` returns nothing.

#### T0.2 Land the docs for ADR-038

- **Goal:** every agent finds the decision and this plan from `main`.
- **Files:** `docs/**`, `AGENTS.md`, `README.md`, `apps/storefront/README.md`,
  `docs-site/docs/frontend-authoring.md`.
- **Acceptance:** ADR-038 in the index; ADR-031 marked superseded; AGENTS.md §12 points here;
  `scripts/check-docs-links.sh` passes.
- **Verify:** `bash scripts/check-docs-links.sh`.

### Phase 1 — renderer and contract

Every name below exists in the codebase as of 2026-11-27 unless the task says "create". If a
name you are told to use does not exist, stop and set the task to `blocked: <name> not found`.
Do not invent a replacement.

Facts every Phase 1 task relies on (verified against the code):

| Need | Use exactly this | Where |
|---|---|---|
| Current tenant id | `stheno_db::tenant::current() -> Result<String>` | `crates/core/stheno-db/src/tenant.rs:63` |
| Run code as a tenant | `stheno_db::tenant::scope(tenant_id, async { ... }).await` | `crates/core/stheno-db/src/tenant.rs:81` |
| A full shopper-facing product (variants, options, images, prices) | `stheno_api_store::product_expand::expand_products(products, &product_service, &pricing_service, &inventory_service, currency_code, region_id) -> Result<Vec<StoreProduct>>` | `crates/api/stheno-api-store/src/product_expand.rs:103` |
| One product by id | `ProductService::get_product(&self, id: &str) -> Result<ProductDTO>` | `crates/modules/stheno-product/src/service.rs:124` |
| A page of products | `ProductService::query_products_paged(&self, handle, id_prefix, search, limit, offset) -> Result<(Vec<ProductDTO>, i64)>` | same file `:356` |
| Categories, collections | `ProductService::list_categories()`, `get_category(id)`, `list_collections()`, `get_collection(id)` | same file `:836-937` |
| Regions | `RegionService::list_regions() -> Result<Vec<RegionDTO>>` | `crates/modules/stheno-region/src/service.rs:129` |
| Store name, logo, colours | `StoreService::get_branding() -> Result<BrandingDTO>` | `crates/modules/stheno-store/src/service.rs:326` |
| Archetype, card style, image aspect | `StoreService::get_storefront_appearance() -> Result<StorefrontAppearanceDTO>` | same file `:422` |
| Public cache headers | `stheno_api_store::cache::public_cache_headers(tenant_id: &str, entities: &[(&str, &str)], include_listing: bool) -> HeaderMap` | `crates/api/stheno-api-store/src/cache.rs:12` |
| Which product a shopper may see | the private `is_public(&ProductDTO)` rule in `crates/api/stheno-api-store/src/products.rs:118` (status `published`, not deleted). Make it `pub` in T1.5 rather than copying it. | |
| Integration test harness | `stheno_server::TestApp::spawn().await` gives `.app` (axum `Router`), `.database_url`; end with `.teardown().await` | `crates/stheno-server/src/lib.rs:1690-1734` |
| Where tenant-scoped routes are assembled | `let tenant_routes = Router::new()` | `crates/stheno-server/src/lib.rs:1429` |

`StoreProduct` (in `product_expand.rs`) is the object templates call `product`. Its price lives at
`product.variants[i].calculated_price.calculated_amount` (a `Decimal`) with
`product.variants[i].calculated_price.currency_code` (a `String`). `calculated_price` is
`Option`: a variant with no price for the region has none, and templates must render
"Unavailable" for it, never `0`.

#### T1.1 `stheno-storefront` crate skeleton, mounted in the engine

- **Goal:** with `STHENO_ENGINE_STOREFRONT=1`, `GET /{country}` on a tenant host returns HTML
  rendered by `minijinja` containing the tenant's store name. With it unset, nothing changes.
- **Create:**
  - `crates/storefront/stheno-storefront/Cargo.toml`: `name = "stheno-storefront"`,
    `edition = "2021"`, dependencies `axum`, `minijinja = { version = "3", features = ["loader"] }`,
    `serde`, `serde_json`, `tokio`, `stheno-db`, `stheno-store`, `stheno-types` (all `workspace = true`
    except `minijinja`).
  - `crates/storefront/stheno-storefront/src/lib.rs` with exactly this public surface:

    ```rust
    pub struct StorefrontContext {
        pub store_service: std::sync::Arc<stheno_store::StoreService>,
    }

    /// Routes this crate serves. Mounted by stheno-server inside the tenant layer.
    pub fn storefront_router(ctx: StorefrontContext) -> axum::Router;
    ```

    The handler for `GET /{country}` calls `ctx.store_service.get_branding().await` and renders an
    inline template `"<!doctype html><title>{{ store.name }}</title><h1>{{ store.name }}</h1>"`
    with `store.name` = `branding.store_name.unwrap_or_default()`. On any error return HTTP 500
    with an empty body; never `unwrap()` in a handler.
  - `crates/stheno-server/tests/engine_storefront.rs` (see Acceptance). The router test lives in
    `stheno-server`, not in the new crate: it needs `stheno_server::TestApp`, and
    `stheno-storefront` cannot depend on `stheno-server` (that would be a dependency cycle).
- **Edit:**
  - root `Cargo.toml`: add `"crates/storefront/stheno-storefront",` to `[workspace] members` after
    `"crates/api/stheno-api-store",`; add `minijinja = { version = "3", features = ["loader"] }` and
    `stheno-storefront = { path = "crates/storefront/stheno-storefront", version = "0.1.0" }` to
    `[workspace.dependencies]`.
  - `crates/stheno-server/Cargo.toml`: add `stheno-storefront = { workspace = true }`.
  - `crates/stheno-server/src/lib.rs`: in the `tenant_routes` builder (line 1429), before the
    tenant layer is applied, add
    `.merge(if std::env::var("STHENO_ENGINE_STOREFRONT").as_deref() == Ok("1") { stheno_storefront::storefront_router(...) } else { Router::new() })`.
- **Do not:** touch `apps/storefront`; add any route outside `/{country}`; read the tenant from
  anything but the existing tenant layer; add a second template engine.
- **Acceptance (tests in `crates/stheno-server/tests/engine_storefront.rs`, each must exist with this name):**
  1. `renders_the_tenant_store_name`: with the flag on, a request with the test tenant's host to
     `/bd` returns 200, `content-type` starting `text/html`, and a body containing the branding's
     `store_name`.
  2. `flag_off_leaves_routes_unchanged`: with the flag unset, `/bd` returns the same status the
     engine returned before this change (404).
- **Verify:** `cargo test -p stheno-server --test engine_storefront` prints
  `test result: ok. 2 passed; 0 failed`. Plus the Rust global gates for `stheno-storefront` and
  `stheno-server`.

#### T1.2 `theme.json` schema, Rust types and validator

- **Goal:** one definition of a valid theme and of valid merchant values, used by the loader
  (T1.3), the admin (T2.2) and the gate (T1.9).
- **Create:**
  - `themes/theme.schema.json`: JSON Schema (draft 2020-12) for section 2.2. Required top-level
    keys: `schema_version` (integer, must equal `1`), `id` (`^[a-z][a-z0-9-]{1,40}$`), `name`
    (1–60 chars), `tokens` (array), `sections` (object). `additionalProperties: false` everywhere.
  - `crates/storefront/stheno-storefront/src/schema.rs`, public items exactly:

    ```rust
    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ThemeManifest { pub schema_version: u32, pub id: String, pub name: String,
                               pub tokens: Vec<TokenDef>, pub sections: std::collections::BTreeMap<String, Vec<SectionDef>> }

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
    pub enum TokenKind {
        Color { default: String },
        Range { min: i64, max: i64, default: i64, unit: Option<String> },
        Select { options: Vec<String>, default: String },
        Font { options: Vec<String>, default: String },
        Text { max_length: usize, default: String },
        Richtext { max_length: usize, default: String },
        Image,
        Url { default: Option<String> },
        Boolean { default: bool },
    }

    pub struct ValidationError { pub path: String, pub message: String }

    /// Parse and check a theme.json file. Every error, not just the first.
    pub fn parse_manifest(json: &str) -> Result<ThemeManifest, Vec<ValidationError>>;

    /// Check merchant-supplied values against a manifest. Unknown keys are errors.
    pub fn validate_values(manifest: &ThemeManifest, values: &serde_json::Value)
        -> Result<(), Vec<ValidationError>>;
    ```

    `TokenDef` and `SectionDef` carry `id` (`^[a-z][a-z0-9_]{0,40}$`), `label` (1–60 chars) and the
    flattened `TokenKind`. You may add private helpers; do not add public items.
  - Rules `parse_manifest` enforces beyond serde: `schema_version == 1`; ids unique within
    `tokens` and within each section; `Range`: `min <= default <= max`; `Select`/`Font`: non-empty
    `options` containing `default`; `Color`: `^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$`; `Font`
    options are keys present in the curated font list (read from wherever
    `scripts/check-font-licences.sh` reads it; if that is not machine-readable, set the task to
    blocked and say so).
  - Rules `validate_values` enforces: every key is a declared token id or
    `"<section_type>.<setting_id>"`; colour format; integer within range; option membership;
    text length in characters, not bytes; `Url` is `https://` or starts with a single `/`
    (never `//`, never `javascript:`); `Richtext` only `p`, `strong`, `em`, `a[href]` tags.
  - `crates/storefront/stheno-storefront/tests/schema.rs` and fixtures under
    `crates/storefront/stheno-storefront/tests/fixtures/`.
- **Acceptance:** one test per rule above, named `rejects_<rule>` (for example
  `rejects_range_default_outside_bounds`, `rejects_javascript_url`), each asserting the returned
  `ValidationError.path` equals the offending field path (for example `tokens[2].default`), plus
  `accepts_the_section_2_2_example`, which parses the `theme.json` example in section 2.2 of this
  plan verbatim.
- **Do not:** use a JSON Schema crate at runtime (the Rust types plus these rules are the
  validator; `theme.schema.json` is for editors and the gate); accept unknown keys anywhere.
- **Verify:** `cargo test -p stheno-storefront --test schema`; every test listed passes.

#### T1.3 Theme loading and hashed asset serving

- **Goal:** themes load from disk at runtime; their assets are served with immutable caching.
- **Create:** `src/themes.rs` (`pub struct ThemeRegistry`, `pub fn load(dir: &Path) -> ThemeRegistry`,
  `pub fn get(&self, id: &str) -> Option<&LoadedTheme>`, `pub fn ids(&self) -> Vec<String>`),
  `src/assets.rs`, `tests/themes.rs`.
- **Edit:** root `Cargo.toml` `tower-http` features: add `"fs"`.
- **Behaviour:** read `STHENO_THEMES_DIR` (default `themes`); for each subfolder with a
  `theme.json`, parse with `parse_manifest`; an invalid theme is logged with
  `tracing::error!` and skipped, the others still load; folders whose name starts with `_` load
  only when `STHENO_THEMES_INCLUDE_STARTER=1` (tests set it). Asset URL:
  `/theme-assets/<theme-id>/<first 12 hex chars of SHA-256 of the file>/<relative path>`,
  served with `Cache-Control: public, max-age=31536000, immutable`. A path containing `..`, a
  backslash, or resolving outside the theme's `assets/` folder returns 404.
- **Acceptance (tests):** `invalid_theme_is_skipped_not_fatal`, `asset_url_changes_when_content_changes`,
  `path_traversal_is_404`, `assets_are_immutable_cached`.
- **Do not:** read themes from the database; embed themes in the binary; serve anything outside
  `assets/`.
- **Verify:** `cargo test -p stheno-storefront --test themes`.

#### T1.4 Token compiler and `settings` / `sections` objects

- **Goal:** defaults plus merchant values become one CSS block and the template objects.
- **Create:** `src/tokens.rs` with
  `pub fn merge(manifest: &ThemeManifest, merchant: &serde_json::Value) -> serde_json::Map<String, serde_json::Value>`
  and `pub fn style_block(manifest: &ThemeManifest, merged: &serde_json::Map<String, serde_json::Value>) -> String`;
  `tests/tokens.rs`.
- **Rules:** merchant values win over defaults key by key; values are validated with
  `validate_values` before merging (invalid input never reaches CSS). `style_block` emits
  `:root{--theme-<id>:<value>;...}` for `Color`, `Range` (with `unit`, or bare number), `Font`
  (as `var(--font-<key>)`) and `Select` (as the option string), sorted by token id, nothing else.
  Text-like tokens never appear in CSS.
- **Acceptance (tests):** `merchant_value_overrides_default`, `invalid_merchant_value_is_rejected_before_css`,
  `style_block_is_sorted_and_stable` (a snapshot string written in the test), `text_tokens_are_not_emitted_as_css`.
- **Verify:** `cargo test -p stheno-storefront --test tokens`.

#### T1.5 Loaders and filters for home, listing and product pages

- **Goal:** `/{country}`, `/{country}/store`, `/{country}/categories/{...}`,
  `/{country}/collections/{handle}` and `/{country}/products/{handle}` render real engine data.
- **Create:** `src/loaders/{mod,home,listing,product}.rs`, `src/filters.rs`, `tests/loaders.rs`.
- **Edit:** `crates/api/stheno-api-store/src/products.rs`: change `fn is_public` to `pub fn is_public`
  (the only change allowed in that crate).
- **How:** build `StoreProduct`s with `expand_products` (see the facts table), never a second
  pricing path. Resolve the currency from the region whose countries include the `{country}`
  path segment (`RegionService::list_regions`); unknown country → the engine's 404 page.
  Product by handle: `query_products_paged(Some(handle), None, None, 1, 0)`, then
  `is_public`; not public or not found → 404. Listing: 24 per page, `?page=N` (1-based),
  `query_products_paged`, filtered by `is_public`.
- **Filters (register these and nothing else):**
  - `money(amount, currency_code)`: `Decimal` and ISO code → display string. Symbols: `USD $`,
    `EUR €`, `GBP £`, `BDT ৳`; any other code is suffixed (`120.00 JPY`). Two decimals except
    `BDT`, which shows whole taka. Thousands separator `,`. Examples the tests must assert
    exactly: `money(20, "USD") == "$20.00"`, `money(14400, "BDT") == "৳14,400"`,
    `money(1234.5, "EUR") == "€1,234.50"`, `money(7, "JPY") == "7.00 JPY"`.
  - `image_url(url, width)`: returns `url` with `?w=<width>` appended (or `&w=` if it already has
    a query). Width must be one of 160, 320, 640, 960, 1280, else the nearest larger one.
  - `asset_url(path)`: the T1.3 hashed URL for the current theme.
  - `t(key)`: looks up a locale string; a missing key renders the key itself and logs a warning.
  - `date(value, format)`: `format` is one of `"short"` (`27 Nov 2026`) or `"long"`
    (`27 November 2026`).
- **Acceptance (tests, against a migrated database seeded in the test):**
  `product_page_shows_engine_price`, `unpriced_variant_renders_unavailable_not_zero`,
  `draft_product_is_404`, `unknown_country_is_404`, `listing_paginates_24_per_page`,
  `money_filter_formats_exactly` (the four examples), `unregistered_filter_is_an_error`.
- **Do not:** compute prices, totals or discounts in a loader or template; call the Store API
  over HTTP from the renderer; return a product whose status is not published.
- **Verify:** `cargo test -p stheno-storefront --test loaders` with `DATABASE_URL` set.

#### T1.6 Cache headers and tenant-prefixed tags

- **Goal:** rendered pages are cacheable exactly as section 2.4 says.
- **Create:** `src/cache.rs`, `tests/cache.rs`.
- **How:** public pages return the headers from `public_cache_headers(tenant_id, entities,
  include_listing)` with the entity tags of everything they render (`("product", handle)` per
  product; listing pages pass `include_listing = true`). Do not write a second header helper.
  Pages that read `cart` or `customer` set `Cache-Control: private, no-store` and no tags.
- **Acceptance (tests):** `product_page_carries_product_tag`, `listing_page_carries_listing_tag`,
  `public_pages_never_set_cookie`, `cart_page_is_no_store`.
- **Verify:** `cargo test -p stheno-storefront --test cache`.

#### T1.7 Core partials and the include mechanism

- **Goal:** themes include core partials they cannot override.
- **Create:** `crates/storefront/stheno-storefront/templates/core/attribution.html.j2`,
  `src/render.rs`, `tests/core_partials.rs`.
- **How:** one `minijinja::Environment` per theme with a loader that resolves `core/...` names
  **only** from the engine's `templates/core/` (compiled in with `include_str!`) and every other
  name only from the theme's `templates/`. Set
  `env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html)` so every template escapes
  regardless of file extension. `attribution.html.j2` renders
  `<p class="vx-attribution">Powered by Vendryx</p>` when `store.plan == "free"`, else nothing.
- **Acceptance (tests):** `attribution_shows_on_free_only`, `theme_cannot_shadow_core_partial`
  (a theme file at `templates/core/attribution.html.j2` is ignored), `output_is_html_escaped`
  (a store name `<script>` renders as `&lt;script&gt;`).
- **Verify:** `cargo test -p stheno-storefront --test core_partials`.

#### T1.8 Confirm Cloudflare plan facts

- **Goal:** replace the "unverified" items in ADR-037 and ADR-038 with facts.
- **Files:** `docs/DECISIONS.md` (the "Unverified at decision time" paragraphs of ADR-037 and
  ADR-038 only).
- **Steps:** ask the owner which Cloudflare plan the zone is on. Then, from Cloudflare's own
  documentation pages, record for that plan: purge by tag available (yes/no) and its rate
  limit; custom hostnames included and price beyond; whether Tunnel ingress preserves `Host`.
  Each line gets the date and the documentation URL.
- **Do not:** guess a value or use a non-Cloudflare source. If the owner has not answered, set
  the task to `blocked: waiting for the owner's Cloudflare plan`.
- **Verify:** `bash scripts/check-docs-links.sh`.

#### T1.9 `scripts/check-theme-pack.sh` gate

- **Goal:** a theme that breaks ADR-038 decisions 3–6 fails CI.
- **Create:** `scripts/check-theme-pack.sh` (then `git update-index --chmod=+x`),
  `scripts/tests/theme-pack/run.sh` (also executable), one fixture folder per check under
  `scripts/tests/theme-pack/fixtures/<check-name>/`, and a `valid/` fixture that passes.
- **Edit:** `.github/workflows/ci.yml`: one step `- name: Theme packs` with
  `run: ./scripts/check-theme-pack.sh` in the `rust-static` job, after `Migration hygiene`.
- **Checks (each prints `FAIL: <theme>: <file>:<line>: <reason>` and the script exits 1 if any
  fail):** `theme.json` parses with the T1.2 rules (call
  `cargo run -q -p stheno-storefront --bin validate-theme -- <dir>`; add that small binary in this
  task); no `#[0-9a-fA-F]{3,8}\b`, `rgb(`, `rgba(`, `hsl(`, or CSS named colour in templates,
  CSS or scripts; no `font-family:` except `var(`; no `box-shadow:`/`border-radius:` value that is
  not `var(` or `0`; no `|safe`; no `fetch(` or `XMLHttpRequest` unless the argument starts with
  `'/store/` or `"/store/`; no `http://` or `https://` in `src=`/`href=`; no `innerHTML`; no file
  outside `themes/<id>/`; no file named `templates/core/...`; default colour pairs `ink`/`canvas`
  meet WCAG AA contrast 4.5:1.
- **Acceptance:** `scripts/tests/theme-pack/run.sh` runs the gate against every fixture and
  asserts each bad fixture fails with its expected reason and `valid/` passes.
- **Verify:** `bash scripts/tests/theme-pack/run.sh` prints `theme-pack fixtures: all passed`.

#### T1.10 `themes/_starter` reference theme

- **Goal:** the smallest complete theme; the template every conversion and AI design tool copies.
- **Create:** `themes/_starter/` with every file in section 2.1. Tokens: `canvas`, `ink`,
  `ink_muted`, `accent`, `surface` (colours), `font_body`, `font_heading` (fonts),
  `radius_card`, `radius_button`, `space_unit` (ranges), `card_style`, `grid_columns`,
  `image_aspect` (selects). Section `index.hero` with `heading`, `subheading`, `image`,
  `cta_label`.
- **Acceptance:** `bash scripts/check-theme-pack.sh` passes; with `STHENO_THEMES_INCLUDE_STARTER=1`
  and the tenant's active theme set to `_starter`, home, listing and product pages render with
  real data (a test in `crates/storefront/stheno-storefront/tests/starter.rs` named
  `starter_renders_home_listing_and_product`).
- **Verify:** both commands above.

### Phase 2 — admin Appearance and storage

#### T2.1 Migration: per-tenant theme selection and token values

- **Goal:** each tenant has an active theme and saved token values, under row-level security.
- **Create:** `migrations/<UTC timestamp, format YYYYMMDDHHMMSS>_storefront_theme_setting/up.sql`
  and `down.sql`.
- **up.sql must contain exactly these objects:**

  ```sql
  CREATE TABLE store.theme_setting (
      tenant_id     text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
      theme_id      text        NOT NULL CHECK (theme_id ~ '^[a-z_][a-z0-9-]{1,40}$'),
      theme_version integer     NOT NULL DEFAULT 1,
      is_active     boolean     NOT NULL DEFAULT false,
      values        jsonb       NOT NULL DEFAULT '{}'::jsonb,
      updated_by    text,
      updated_at    timestamptz NOT NULL DEFAULT now(),
      PRIMARY KEY (tenant_id, theme_id)
  );
  CREATE UNIQUE INDEX idx_theme_setting_one_active
      ON store.theme_setting (tenant_id) WHERE is_active;
  ALTER TABLE store.theme_setting ENABLE ROW LEVEL SECURITY;
  ALTER TABLE store.theme_setting FORCE ROW LEVEL SECURITY;
  CREATE POLICY tenant_isolation ON store.theme_setting
      USING (tenant_id = (SELECT current_setting('app.tenant_id', true)))
      WITH CHECK (tenant_id = (SELECT current_setting('app.tenant_id', true)));
  GRANT SELECT, INSERT, UPDATE, DELETE ON store.theme_setting TO stheno_app;
  ```

  `down.sql`: `DROP TABLE IF EXISTS store.theme_setting;`
- **Also:** add `ThemeSettingEntity` (`const TABLE = "store.theme_setting"`) and
  `get_active_theme_setting()` / `save_theme_setting(theme_id, values, actor)` /
  `set_active_theme(theme_id)` to `crates/modules/stheno-store/src/service.rs`, and a
  `tests/persistence.rs`-style test in `stheno-store` named `theme_settings_outlive_the_service_instance`.
- **Acceptance:** `cargo test -p stheno-db --test rls_audit` still passes (the table is scoped);
  `cargo test -p stheno-db --test migration_runner` passes; the persistence test passes.
- **Verify:** the migration gates in section 1.4 and the tests above.

#### T2.2 Admin endpoints for themes and appearance

- **Create:** `crates/api/stheno-api-admin/src/appearance.rs`, tests in
  `crates/api/stheno-api-admin/tests/appearance.rs`.
- **Routes (register in the existing admin router next to `admin_settings_routes`):**
  - `GET /admin/themes` → `{"themes": [{"id", "name", "schema": <ThemeManifest>}]}` from the
    `ThemeRegistry`.
  - `GET /admin/appearance` → `{"theme_id": "...", "values": {...}}` for the active theme.
  - `PUT /admin/appearance` with `{"theme_id", "values"}` → validate with
    `stheno_storefront::schema::validate_values`; on errors return 400
    `{"type":"invalid_data","errors":[{"path","message"}]}`; on success save, set active, enqueue a
    hostname purge for the tenant in the same transaction (ADR-037 decision 3), return 200 with the
    saved document.
- **Acceptance (tests):** `put_rejects_bad_colour_with_path`, `put_rejects_out_of_range`,
  `put_rejects_unknown_option`, `put_rejects_javascript_url`, `put_rejects_oversized_text`,
  `put_persists_and_enqueues_one_purge`, `staff_without_branding_permission_gets_403` (same
  permission the branding endpoints require).
- **Verify:** `cargo test -p stheno-api-admin --test appearance`.

#### T2.3 Admin Appearance screen generated from `theme.json`

- **Create:** `apps/admin/src/routes/appearance/appearance-view.tsx`,
  `apps/admin/src/routes/appearance/controls/` (one component per token type),
  `apps/admin/src/routes/appearance/__tests__/appearance.spec.tsx`; add the route as a child of
  the `/settings` route in `apps/admin/src/dashboard-app/routes/get-route.map.tsx` (see how
  `routes[1].children[0]` is handled around line 94).
- **How:** fetch `/admin/themes` and `/admin/appearance`; render one control per token in
  manifest order, labelled with `label`; colour → native `<input type="color">`, range →
  `<input type="range">` showing the value and unit, select/font → `<select>`, text →
  `<input maxLength>`, richtext → `<textarea>`, image → an `<input type="file">` handled the same way as the logo field in `branding-view.tsx` (around line 437), url →
  `<input type="url">`, boolean → checkbox. Show field errors from the 400 response next to the
  matching control by `path`. The live preview reuses the iframe preview in
  `apps/admin/src/routes/branding/branding-view.tsx`.
- **Acceptance (tests):** one test per control type rendering from a fixture manifest; one test
  that a 400 response puts the error message under the right control; one test that adding a
  token to the fixture manifest renders a new control with no code change.
- **Verify:** admin global gates (section 1.4).

### Phase 3 — themes

Every theme task has the same shape and the same procedure. Work in `themes/<id>/` only.

**Source of the design:**
- T3.1–T3.3: the React design in `design-source/ai-studio-2026-11-27/apps/storefront/src/themes/<id>/`.
- T3.4–T3.7: `apps/storefront/packages/frontend-default/src/modules/` plus the theme constants in
  `apps/storefront/packages/storefront-core/src/lib/branding.ts` (`DEFAULT_BRANDING`,
  `MAHAL_BRANDING`, `LUSTRE_BRANDING`, `MATRIX_BRANDING`) and the `mahal:` / `lustre:` /
  `matrix:` classes in `hero/index.tsx`, `product-preview/index.tsx` and `app/layout.tsx`.

**Procedure (do every step, in order):**
1. `cp -r themes/_starter themes/<id>` and set `id` and `name` in `theme.json`.
2. **Inventory the literals.** Run, against the source folder:
   `grep -rnoE "#[0-9a-fA-F]{3,8}\b|rgba?\([^)]*\)|font-family:[^;]+|box-shadow:[^;]+|border-radius:[^;]+" <source>`
   and paste the full output into the PR description. Every line must end up as a token, or be
   listed in the PR as "dropped: not used by the converted theme".
3. **Make tokens.** One `color` token per distinct colour (name it by role: `canvas`, `ink`,
   `accent`, ... never by value like `green_500`). One `range` per distinct radius/shadow/spacing
   scale. One `font` per typeface, using keys from the curated font list.
4. **Inventory the copy.** Every visible string in the source becomes a `text` or `richtext`
   section setting whose `default` is the source text, exactly. Store names, product names and
   prices in the source are data, not copy: delete them; they come from loaders.
5. **Delete hardcoded data.** Catalogs (`catalog.ts`), tenants (`tenant-resolver.ts`
   `TENANT_DIRECTORY`), reviews, ratings, "N sold"/"trusted by" figures: delete and render loader
   data. A section with no data source (for example reviews, which the engine does not store yet)
   is removed and listed in the PR as "removed: no engine data".
6. **Write templates.** One template per page in section 2.1; shared pieces as `partials/`.
   Include `core/attribution.html.j2` in the footer and the core checkout and sign-in partials
   where the design had those forms.
7. **Write interaction scripts** for drawers, galleries and WebGL in `assets/islands/`, plain
   browser JavaScript with no build step, calling only `/store/*`.
8. Run `bash scripts/check-theme-pack.sh` until it passes.
9. **Screenshots:** home, listing, product, cart at 390 px and 1440 px wide, side by side with the
   source design, attached to the PR. The owner approves the look; an agent does not.

**Acceptance:** gate passes; the literal and copy inventories are in the PR; every route in section
3 that exists by then renders with real data; the owner has approved the screenshots.

**Verify:** `bash scripts/check-theme-pack.sh` · `cargo test -p stheno-storefront`.

| Task | Theme id | Source notes |
|---|---|---|
| T3.1 | `maison-cuir` | Mostly tokenized already (92 `var(--theme-*)` references, 14 literals). |
| T3.2 | `hydro-vessel` | `webgl/heroCanvas.ts` and `webgl/specsCanvas.ts` become islands; 28 literals. |
| T3.3 | `homedine-eco` | 233 colour literals; `catalog.ts` is hardcoded data to delete. |
| T3.4 | `default` | Apple HIG baseline (AGENTS.md §12.2); `DEFAULT_BRANDING`. |
| T3.5 | `mahal` | `MAHAL_BRANDING` and every `mahal:` class. |
| T3.6 | `lustre` | `LUSTRE_BRANDING` and every `lustre:` class. |
| T3.7 | `matrix` | `MATRIX_BRANDING` and every `matrix:` class. |

### Phase 4 — parity and cut-over

#### T4.1 Cart page and cart interaction script

- **Files:** `sf/src/loaders/cart.rs`, `sf/templates/core/cart*.html.j2`, `themes/_starter/`
  cart template and island, tests.
- **Acceptance:** add, update and remove lines through `/store/carts/*`; totals always come from
  the engine response; the cart page is `no-store`; the cart count island works on cached pages.
- **Verify:** `cargo test -p stheno-storefront` (cart tests).

#### T4.2 Checkout

- **Files:** `sf/templates/core/checkout/**`, `sf/src/loaders/checkout.rs`, tests.
- **Acceptance:** COD, bKash, Nagad, SSLCommerz and Stripe complete end to end against the
  existing engine flows, including provider redirects and webhooks; the checkout page loads no
  third-party script except what the payment provider requires
  (`scripts/check-checkout-scripts.sh` passes); PCI SAQ-A checks pass
  (`scripts/check-pci-saq-a.sh`).
- **Verify:** the scripts above · checkout tests in `stheno-storefront` and `stheno-api-store`.

#### T4.3 Account pages and engine session cookies

- **Acceptance:** sign-in, profile, addresses, orders and order detail work with HttpOnly,
  Secure, SameSite=Lax cookies set by the engine; account pages are `no-store`.

#### T4.4 Order confirmed and order transfer pages

- **Acceptance:** confirmed, transfer, accept and decline behave as today's routes.

#### T4.5 Pages, sitemap, robots, Open Graph, JSON-LD

- **Acceptance:** about and policy pages render from engine data; `sitemap.xml` and `robots.txt`
  per tenant; every public page has title, description, canonical and Open Graph tags; product
  pages carry Product JSON-LD with the engine's price.

#### T4.6 Performance acceptance

- **Acceptance:** Lighthouse mobile performance at least 90 and accessibility at least 95 on
  home, listing and product pages for every theme; results attached to the PR.

#### T4.7 Cut-over and deletion

- **Goal:** the engine serves every storefront host; the Next.js storefront is gone.
- **Files:** `crates/stheno-server` (flag default on), `apps/storefront/**` (delete),
  `docker-compose.yml`, `deploy/**`, `.github/workflows/**` (remove storefront jobs, keep the
  theme gate), `scripts/check-frontend-*.sh` and `check-storefront-boundaries.sh` (delete or
  retarget), docs that describe the Next.js storefront.
- **Acceptance:** every route in section 3 is served by the engine in staging; CI is green with
  no storefront Node job; ADR-031's notice and ADR-038's status say Implemented.

### Phase 5 — kit and follow-up

#### T5.1 Finalize THEME_AUTHORING.md and the AI Studio prompt

- **Acceptance:** an agent given only THEME_AUTHORING.md and the starter theme produces a theme
  that passes the gate on the first try; the copy-paste prompt is in the document.

#### T5.2 ADR-037 reverse-proxy duties

- **Goal:** record whether stripping tenant-override headers, translating `CF-IPCountry`, and the
  admin/storefront host split move into the engine, and implement the decision.
- **Acceptance:** a new ADR, and the duties enforced with tests wherever they land.

---

## 6. Out of scope

Marketing site and admin app styling (they keep their own design systems); the 29 migration
playbook gaps (a separate task); multi-language content beyond existing locale strings; a theme
marketplace or merchant-uploaded themes.

## 7. Open questions for the owner

1. Cloudflare plan level (T1.8): decides tag purge and the number of merchant custom domains.
2. Trial label: the admin says 14-Day, the engine and ADR-033 say 7 days. The owner chose the
   14-Day label on 2026-11-26.

## 8. Change log

| Date | Change |
|---|---|
| 2026-11-27 | Plan written for ADR-038. Supersedes the draft in `.orchestration/task-storefront-theme-engine/spec.md`. |
| 2026-11-27 | Hardened for any agent: section 1.6 (rules that prevent guessing), verified API facts table, exact signatures, test names and expected outputs for Phases 1–2, a step-by-step conversion procedure for Phase 3, and T6.1. |
| 2026-11-27 | T0.1 done: `main` at `12203039` merged into PR #4; AI Studio files parked in `design-source/ai-studio-2026-11-27/`; `bun.lock` removed. |
