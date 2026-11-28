# Storefront Theme Engine — implementation plan (ADR-038)

This is the working plan for [ADR-038](DECISIONS.md#adr-038). It is written so that any agent
can pick up one task, finish it, prove it, and hand back, with no orchestrator present.

- **Why:** the owner wants one server (the Rust engine), no Node.js storefront, every visual and
  copy choice tokenized and managed from the admin center, data only from the engine, heavy edge
  caching, and AI design tools (Google AI Studio, Antigravity) that only design and tokenize.
- **What changes:** the engine renders storefront pages from `minijinja` template themes. Each
  theme is one folder with a `theme.json` of tokens. The Next.js storefront in `apps/storefront`
  keeps serving until task T4.14 deletes it.
- **Where it stands:** see the task board (section 4). Update it in the same PR as the work.

Read first, in order: [AGENTS.md](../AGENTS.md), [ADR-038](DECISIONS.md#adr-038),
[ADR-037](DECISIONS.md#adr-037) (edge and caching), [ADR-006](DECISIONS.md#adr-006) (money),
[ADR-021](DECISIONS.md#adr-021) (AI content is data), then this file.
[THEME_AUTHORING.md](THEME_AUTHORING.md) is the contract a theme author follows.
[AGENT_START_PROMPTS.md](AGENT_START_PROMPTS.md) has copy-paste prompts for starting agents.

---

## 1. How to work this plan

### 1.1 Picking a task

1. Pull `main`. Read the task board (section 4).
2. Pick the **lowest-numbered task whose status is `todo` and whose dependencies are all
   `done`**. Tasks in the same phase with no dependency between them may run in parallel by
   different agents. Their "Files" lists do not overlap; that is what makes them parallel.
3. **One branch and one PR per phase, not per task.** The phase branch is
   `feat/phase-<n>-<short-slug>` (Phase 1: `feat/phase-1-renderer`, Phase 2:
   `feat/phase-2-appearance`, Phase 3: `feat/phase-3-themes`, Phase 4: `feat/phase-4-cutover`,
   Phase 5: `feat/phase-5-kit`, Phase 6: `feat/phase-6-deps`, Phase 7: `feat/phase-7-go-live`).
   If it does not exist yet, create it from `main`, push it, and open a **draft PR** titled
   `Phase <n>: <phase title from section 5>`. If it exists, check it out and pull it.
   Never use an AI product name in a branch, commit, comment or PR.
4. First commit for your task, on the phase branch: change the task's status on the board to
   `in progress (<YYYY-MM-DD>)`. Push it, so other agents can see the claim. Two agents working
   on the same phase pull before every push and never force-push.

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
   (section 1.4), **on Space** (section 1.5). All must pass.
2. In the phase PR description, under a heading for your task, copy its acceptance criteria as
   a checklist and give evidence for each one: the command run and its real output line, a test
   name, or a screenshot for visual work.
3. Commit with the message `<TASK-ID>: <task title>`, set the task's status to
   `done (<commit sha>)` on the board, push. Do not start the next task on the same commit.
4. Mark the phase PR **ready for review only when every task in the phase is `done`** and the
   full CI run on the PR is green. The owner merges it.
5. If you discover work that is not in any task, add it as a new task at the end of its phase
   (next free number, status `todo`, with dependencies, files, acceptance and verify). Do not
   do it in the current PR.

### 1.4 Global gates

Run these for every area your task touches, **on Space, never on the Windows machine** (section
1.5). CI runs the same checks and is the final authority. The commands below are what Space runs;
the script-only checks (bash `check-*.sh`, docs links) are light and may run locally.

| Area | Commands |
|---|---|
| Rust | `cargo fmt -p <crate> -- --check` (local is fine) · then on Space: `.\scripts\build-on-space.ps1 -Mode rust-package -CargoPackage <crate> -CargoClippy -CargoTestServices` (clippy with `-D warnings`, plus every test with Postgres and Redis) |
| Migrations | `bash scripts/check-migration-phases.sh` · `bash scripts/check-playbook.sh` · `bash scripts/check-schema-drift.sh` |
| Admin app | on Space: `.\scripts\build-on-space.ps1 -Mode web-admin` |
| Themes | `bash scripts/check-theme-pack.sh` (from T1.9 onwards) |
| Docs | `bash scripts/check-docs-links.sh` |
| Shell scripts | New `.sh` files are committed executable: `git update-index --chmod=+x <file>` |

### 1.5 Where code is written and where it is tested

**Write code locally; build and test only on Space** (the company build server). The owner's
machine has no free disk for Rust or web builds. Do not run `cargo build`, `cargo test`,
`cargo clippy`, `npm run build` or a Playwright suite locally.

```powershell
# Rust: one crate, with clippy, and the DB/Redis services for integration tests
.\scripts\build-on-space.ps1 -Mode rust-package -CargoPackage <crate> -CargoClippy -CargoTestServices
# Rust: one test target only
.\scripts\build-on-space.ps1 -Mode rust-package -CargoPackage <crate> -CargoTestTarget <test-file-name> -CargoTestServices
# Rust: what CI runs
.\scripts\build-on-space.ps1 -Mode rust-ci
# Admin app / web tests / static web build
.\scripts\build-on-space.ps1 -Mode web-admin
.\scripts\build-on-space.ps1 -Mode web-test
.\scripts\build-on-space.ps1 -Mode web-static
```

If Space is unreachable (for example Tailscale asks for a browser re-approval, or another job is
using it), stop and tell the owner. Do not fall back to a local build.

Other notes for this checkout:
- `cargo fmt -p <crate>` is fine locally (no build). `cargo fmt --all` fails on Windows with
  "filename too long"; format per package.
- `core.filemode` is false, so the executable bit must be set with
  `git update-index --chmod=+x <file>`.

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
7. **One task per commit, one PR per phase.** Finish and commit one task before claiming the
   next. Every task of a phase lands in that phase's single PR (section 1.1).
8. **Money is `Decimal`.** Never `f32`/`f64` for an amount (ADR-006); never compute a price, total
   or discount outside the engine's existing pricing and cart code.

### 1.7 Owner tasks, and the order of the whole project

Tasks numbered `O1`–`O7` on the board are the owner's: they need accounts, money, servers or
approval. An agent never does them. When an agent reaches a task that depends on an open `O`
task, it sets its own task to `blocked: waiting for O<n>` and stops.

The whole project, in order. Phases run one after another unless marked parallel.

| Step | What | Who |
|---|---|---|
| 1 | Merge PR #4 (sync with main, CI green) and PR #5 (this plan) into `main` | owner merges |
| 2 | Phase 1, tasks T1.1–T1.10 (T1.8 needs O1) | agents, one task per session |
| 3 | Phase 2, T2.1–T2.3 | agents |
| 4 | Phase 3, T3.1–T3.7, one theme per session; screenshots approved by the owner (O7) | agents, owner approves |
| 5 | Phase 4, T4.1–T4.14 (T4.13 needs staging from T7.2, so do O2, O3 and T7.2 before it) | agents, owner for O-tasks |
| 6 | Phase 5, T5.1–T5.2 | agents |
| 7 | Phase 6, T6.1 (parallel: any time, in its own session) | agent |
| 8 | Phase 7, T7.1–T7.6, plus T7.5 for the rest of the platform | agents and owner |

Each phase ends with one PR that someone who did not write it reviews (prompt R in
[AGENT_START_PROMPTS.md](AGENT_START_PROMPTS.md)) before the owner merges it.

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
`apps/storefront/packages/storefront-core/src/app/` until T4.14 deletes it.

---

## 4. Task board

Status values: `todo`, `in progress (<date>)`, `blocked: <reason>`, `done (<commit sha>)`.

| ID | Task | Depends on | Status |
|---|---|---|---|
| T0.1 | Merge `main` into PR #4 and park the AI Studio storefront as design source | — | done (PR #4, merge commit `905a33d2`) |
| T0.2 | Land the docs for ADR-038 (this plan, AGENTS.md, ADR index, authoring contract) | — | in progress (docs/storefront-theme-engine, 2026-11-27) |
| T1.1 | `stheno-storefront` crate skeleton, mounted in the engine | T0.1 | blocked: `cargo deny check` fails on the baseline and this branch |
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
| T4.1 | Country prefix and region resolution | T1.5 | todo |
| T4.2 | Internal Store API client | T1.1 | todo |
| T4.3 | Shopper session cookie, sign-in, register, sign-out | T4.2 | todo |
| T4.4 | Cart: forms, cookie, page, count island | T4.1, T4.2, T1.10 | todo |
| T4.5 | Checkout steps, COD and manual transfer | T4.4 | todo |
| T4.6 | Checkout: Stripe | T4.5 | todo |
| T4.7 | Checkout: bKash, Nagad, SSLCommerz (sandbox run needs O4) | T4.5 | todo |
| T4.8 | Account pages | T4.3, T1.10 | todo |
| T4.9 | Order confirmed, payment proof, order transfer pages | T4.5, T4.8 | todo |
| T4.10 | Store pages (about, policies): migration, service, routes | T1.5 | todo |
| T4.11 | Admin screen for store pages | T4.10, T2.3 | todo |
| T4.12 | SEO: head tags, JSON-LD, sitemap, robots | T1.5, T4.10 | todo |
| T4.13 | Performance and accessibility acceptance | T3.1–T3.7, T4.12, T7.2 | todo |
| T4.14 | Cut-over: engine serves all storefront hosts; delete `apps/storefront` | T2.3, T3.1–T3.7, T4.1–T4.13 | todo |
| T5.1 | Finalize THEME_AUTHORING.md and the AI Studio prompt | T1.10, T2.3 | todo |
| T5.2 | Origin proxy duties after cut-over (ADR-039) | T4.14 | todo |
| T6.1 | Upgrade `diesel-async` past 0.5 and `diesel` to >= 2.3.8; remove the two ignores in `.cargo/audit.toml` | — | todo |
| T7.1 | Edge acceptance checklist | T1.8, O1, T7.2 | todo |
| T7.2 | Staging environment | O2, O3 | todo |
| T7.3 | Payments in sandbox on staging | T4.5–T4.7, T7.2, O4 | todo |
| T7.4 | Backups and a restore drill | T7.2 | todo |
| T7.5 | Platform backlog outside the storefront (pointer) | — | todo |
| T7.6 | Production launch | T4.14, T5.2, T7.1–T7.4, O5, O6 | todo |
| O1 | **Owner:** tell agents the Cloudflare plan level (feeds T1.8) | — | todo |
| O2 | **Owner:** choose the staging and production server | — | todo |
| O3 | **Owner:** create the Cloudflare Tunnel and put its token on the server | O2 | todo |
| O4 | **Owner:** enter sandbox credentials for Stripe, bKash, Nagad, SSLCommerz in the staging admin | T7.2 | todo |
| O5 | **Owner:** enter live payment credentials in the production admin | T7.3 | todo |
| O6 | **Owner:** point production DNS at Cloudflare | T7.2 | todo |
| O7 | **Owner:** approve each theme's screenshots (Phase 3) and merge each phase PR | — | ongoing |

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
    `edition = "2021"`, dependencies `axum`, `minijinja = "3"`,
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
    `"crates/api/stheno-api-store",`; add `minijinja = "3"` and
    `stheno-storefront = { path = "crates/storefront/stheno-storefront", version = "0.1.0" }` to
    `[workspace.dependencies]`. MiniJinja 3 includes template loading by default.
  - `Cargo.lock`: regenerate for the new workspace crate and MiniJinja dependency.
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

Phase 4 makes the engine do everything the Next.js storefront does today, then deletes it.
Branch and PR: `feat/phase-4-cutover`, `Phase 4: parity and cut-over`.

**The one design rule of this phase: the storefront never re-implements commerce.** Every cart,
checkout, account and order action goes through the existing Store API handlers, called in process
through the internal client from T4.2. No price, total, tax, discount, stock or ownership check is
written in `stheno-storefront`. If a task seems to need one, stop and set it to `blocked`.

**Pages work without JavaScript.** Every action is a plain HTML `<form method="post">` to an
engine route that answers with `303 See Other`. Interaction scripts only enhance those forms
(a drawer instead of a page load). This is how Shopify's `/cart/add` works, and it means a page
is correct even before or without its script.

Facts Phase 4 relies on (verified against the code on 2026-11-28; line numbers may drift a few
lines, the names do not):

| Need | Use exactly this | Where |
|---|---|---|
| Services the Store API uses | `StoreApiContext` (`cart_service`, `payment_service`, `order_service`, `auth_service`, `region_service`, ...) | `crates/api/stheno-api-store/src/lib.rs:50-79`, built at `crates/stheno-server/src/lib.rs:1484` |
| Cart routes | `store_cart_routes`: `POST /store/carts`, `GET`/`POST /store/carts/{id}`, `POST /store/carts/{id}/line-items`, `POST`/`DELETE /store/carts/{id}/line-items/{line_id}`, `POST /store/carts/{id}/shipping-methods`, `POST`/`DELETE /store/carts/{id}/promotions`, `POST /store/carts/{id}/customer`, `POST /store/carts/{id}/complete` | `crates/api/stheno-api-store/src/lib.rs:146-172`, handlers in `carts.rs` |
| Payment routes | `store_payment_routes`: `POST /store/payment-collections`, `POST /store/payment-collections/{id}/payment-sessions`, `POST /store/payment-collections/{id}/payment-sessions/{session_id}`, `GET /store/payment-providers` | `crates/api/stheno-api-store/src/lib.rs:175-194`, `payment_collections.rs` |
| Shipping options | `GET /store/shipping-options`, `POST /store/shipping-options/{id}/calculate` | `crates/api/stheno-api-store/src/lib.rs:238-252` |
| Payment provider ids | COD `pp_system_default`, manual bank transfer `pp_manual`, Stripe `pp_stripe_stripe`, bKash `pp_bkash_bkash`, Nagad `pp_nagad_nagad`, SSLCommerz `pp_sslcommerz_sslcommerz` | provider crates under `crates/providers/stheno-payment-*/src/lib.rs` |
| Provider return URLs | read from the payment session's `data`: bKash `callback_url` (`stheno-payment-bkash/src/lib.rs:221`), Nagad `callback_url` (`stheno-payment-nagad/src/lib.rs:144`), SSLCommerz `success_url`, `fail_url`, `cancel_url` (`stheno-payment-sslcommerz/src/lib.rs:153-162`) | |
| Provider webhooks | `POST /hooks/payment/{provider_id}`, handler `receive_payment_webhook` | `crates/api/stheno-api-store/src/hooks.rs:37-61`, mounted `crates/stheno-server/src/lib.rs:1563` |
| Shopper sign-in | `POST /auth/customer/emailpass` (handler `authenticate`), `POST /auth/customer/emailpass/register` (`register`) | `crates/api/stheno-api-auth/src/routes.rs:58`, `:108`, route table `:425-472` |
| Shopper token today | JWT sent as `Authorization: Bearer`, read by the private `bearer_token(&HeaderMap)` | `crates/api/stheno-api-store/src/customers.rs:47` |
| Existing cookie pattern to copy | admin cookie `__Host-stheno-admin-session` set by `session_cookie` (`Path=/; Secure; HttpOnly; SameSite=Lax`) | `crates/api/stheno-api-auth/src/routes.rs:794-814` |
| Customer routes | `GET`/`POST /store/customers/me`, `GET`/`POST /store/customers/me/addresses`, `GET`/`POST`/`DELETE /store/customers/me/addresses/{address_id}`, `POST /store/customers` | `crates/api/stheno-api-store/src/lib.rs:281-311`, `customers.rs` |
| Order routes | `GET /store/orders`, `GET /store/orders/{id}`, `GET`/`POST /store/orders/{id}/payment-proof`, `POST /store/orders/{id}/transfer/request`, `.../accept`, `.../decline`, `.../cancel` | `crates/api/stheno-api-store/src/lib.rs:197-235`, `orders.rs` |
| Store config (buyer accounts on/off) | `GET /store/config` | `crates/stheno-server/src/lib.rs:1474` |
| Checkout script allow-list | `docs/payment-page-scripts.md` (the CSP for checkout pages is written there) | checked by `scripts/check-checkout-scripts.sh`, `scripts/check-pci-saq-a.sh` |
| Today's storefront behaviour to match | pages in `apps/storefront/packages/storefront-core/src/app/[countryCode]/`, data calls in `.../src/lib/data/*.ts`, checkout steps in `.../src/modules/checkout/` | until T4.13 deletes it |

Not in the engine today (tasks below create them): a shopper cookie, policy and about page
content, `sitemap.xml`, `robots.txt`, JSON-LD, and a browser return page for bKash, Nagad and
SSLCommerz.

#### T4.1 Country prefix and region resolution

- **Goal:** `/{country}/...` resolves to the region that contains that country, exactly as
  `apps/storefront/packages/storefront-core/src/middleware.ts` does today.
- **Create:** `crates/storefront/stheno-storefront/src/region.rs` with
  `pub async fn resolve_region(region_service: &stheno_region::RegionService, country: &str) -> Result<Option<RegionDTO>, stheno_types::Error>`
  (match `country` case-insensitively against each region's countries' `iso_2`).
- **Edit:** `src/lib.rs` (call it from every page handler); `crates/stheno-server/tests/engine_storefront.rs`.
- **Behaviour:**
  1. `GET /` redirects `307` to `/{country}` where `country` is, in order: the `CF-IPCountry`
     request header if it maps to a region, else the first country of the first region from
     `RegionService::list_regions()`.
  2. A `{country}` that maps to no region answers `404` with the theme's 404 page if it has
     `templates/404.html.j2`, else a plain 404.
  3. No regions at all: `503`, body empty.
  4. Paths containing a `.` (files) are never treated as a country.
- **Acceptance tests (exact names):** `root_redirects_to_ip_country`,
  `root_redirects_to_first_region_without_ip_header`, `unknown_country_is_404`,
  `no_regions_is_503`.
- **Verify (Space):** `.\scripts\build-on-space.ps1 -Mode rust-package -CargoPackage stheno-server -CargoTestTarget engine_storefront -CargoTestServices`.

#### T4.2 Internal Store API client

- **Goal:** storefront handlers call the existing Store API handlers in process, with the same
  validation, pricing, flows and errors as an HTTP call, and no network hop.
- **How:** build the same store routers that `crates/stheno-server/src/lib.rs` builds around line
  1484-1563 (read that block; reuse the exact constructor calls, do not write new ones) and pass the
  merged `axum::Router` into `StorefrontContext` as `pub store_api: axum::Router`. Call it with
  `tower::ServiceExt::oneshot`. The request runs inside the current tenant scope, so tenant
  isolation is unchanged.
- **Create:** `crates/storefront/stheno-storefront/src/store_api.rs`:

  ```rust
  pub struct StoreApi { router: axum::Router }
  pub struct StoreApiResponse { pub status: http::StatusCode, pub body: serde_json::Value }
  impl StoreApi {
      pub fn new(router: axum::Router) -> Self;
      /// `token` is the shopper JWT from the cookie (T4.3), sent as `Authorization: Bearer`.
      pub async fn call(&self, method: http::Method, path: &str,
                        body: Option<serde_json::Value>, token: Option<&str>)
                        -> Result<StoreApiResponse, stheno_types::Error>;
  }
  ```
- **Edit:** `src/lib.rs` (`StorefrontContext` gains `store_api`), `crates/stheno-server/src/lib.rs`
  (pass the router where `storefront_router` is mounted).
- **Do not:** call any `*_service` method directly for carts, checkout, payments, customers or
  orders. Catalogue loaders from Phase 1 keep their direct service calls.
- **Acceptance tests:** `internal_client_matches_http_for_regions` (same JSON as
  `GET /store/regions` over HTTP), `internal_client_passes_bearer_token`
  (`GET /store/customers/me` with a valid token returns 200, without one 401).
- **Verify (Space):** as T4.1.

#### T4.3 Shopper session cookie

- **Goal:** a shopper signs in through an HTML form; the engine keeps the JWT in an HttpOnly
  cookie; the Store API accepts that cookie as well as the existing bearer header.
- **Cookie:** name `__Host-stheno-shopper`, value the access token returned by the auth service,
  attributes copied from `session_cookie` in `crates/api/stheno-api-auth/src/routes.rs:814`
  (`Path=/; Secure; HttpOnly; SameSite=Lax`), `Max-Age` equal to the access token lifetime.
- **Edit `crates/api/stheno-api-store/src/customers.rs`:** replace the private
  `bearer_token(&HeaderMap) -> Option<&str>` with
  `pub(crate) fn shopper_token(headers: &HeaderMap) -> Option<String>` that returns the bearer
  token if present, otherwise the `__Host-stheno-shopper` cookie value. Update every caller
  (find them with `grep -rn "bearer_token" crates/api/stheno-api-store/src`). The bearer header
  keeps working exactly as before (the Store API is frozen: add, never change).
- **Create engine routes in `stheno-storefront`:**
  - `POST /{country}/account/login` (form `email`, `password`): calls
    `POST /auth/customer/emailpass` through the internal client pattern (build the auth router the
    same way T4.2 builds the store router). Success: set the cookie, `303` to
    `/{country}/account`. Failure: `303` to `/{country}/account?error=login` (never say whether the
    email exists). A `429` from the auth route is passed through as `429`.
  - `POST /{country}/account/register` (form `email`, `password`, `first_name`, `last_name`):
    `POST /auth/customer/emailpass/register`, then `POST /store/customers`, then sign in as above.
  - `POST /{country}/account/logout`: clear the cookie (`Max-Age=0`), `303` to `/{country}`.
- **CSRF:** every engine `POST` rejects with `403` when the `Origin` header is present and its host
  differs from the request `Host`, or when both `Origin` and `Referer` are absent. `SameSite=Lax`
  already keeps the cookie off cross-site POSTs; this is the second lock.
- **Acceptance tests:** `login_sets_httponly_cookie`, `bad_login_does_not_reveal_account`,
  `store_api_accepts_cookie_token`, `bearer_header_still_works`, `logout_clears_cookie`,
  `cross_origin_post_is_rejected`.
- **Verify (Space):** `-CargoPackage stheno-api-store -CargoClippy -CargoTestServices`, then as T4.1.

#### T4.4 Cart

- **Goal:** add to cart, change quantity, remove, and see the cart, with and without JavaScript.
- **Cookie:** `__Host-stheno-cart`, value the cart id, same attributes as T4.3, `Max-Age` 30 days.
- **Engine routes:**
  - `POST /{country}/cart/add` (form `variant_id`, `quantity`): if no valid cart cookie, create a
    cart with `POST /store/carts` and `{"region_id": <region from T4.1>}` and set the cookie; then
    `POST /store/carts/{id}/line-items`. `303` to `/{country}/cart`.
  - `POST /{country}/cart/update` (form `line_id`, `quantity`): `POST /store/carts/{id}/line-items/{line_id}`.
  - `POST /{country}/cart/remove` (form `line_id`): `DELETE /store/carts/{id}/line-items/{line_id}`.
  - `GET /{country}/cart`: renders `cart.html.j2` with `cart` = the body of `GET /store/carts/{id}`.
  - `GET /_sf/cart-count`: `{"count": <sum of line quantities>}` or `{"count": 0}`.
  - With header `Accept: application/json`, the three POST routes answer `200` with the cart JSON
    instead of `303`, so a script can update a drawer.
- **Caching:** cart page and `/_sf/cart-count` send `Cache-Control: private, no-store` and never a
  `Cache-Tag`. Public pages never read the cart cookie.
- **Signed-in shopper:** after sign-in (T4.3), if a cart cookie exists call
  `POST /store/carts/{id}/customer` so the cart follows the account.
- **Island:** `themes/_starter/assets/islands/cart.js`: on load, fetch `/_sf/cart-count` and fill
  every `[data-cart-count]`; intercept `form[data-cart-form]` submits with `fetch` and
  `Accept: application/json`. No price arithmetic: display what the response contains.
- **Templates:** `themes/_starter/templates/cart.html.j2`, `partials/cart-line.html.j2`;
  product card and product page get a `<form method="post" action="{{ routes.cart_add }}" data-cart-form>`.
- **Acceptance tests:** `add_to_cart_creates_cart_and_sets_cookie`, `update_and_remove_line`,
  `cart_page_is_no_store`, `cart_count_endpoint`, `json_accept_returns_cart`,
  `totals_come_from_store_api` (assert the rendered total equals the Store API's `total`).
- **Verify (Space):** as T4.1, plus `bash scripts/check-theme-pack.sh`.

#### T4.5 Checkout: steps, COD and manual transfer

- **Goal:** the four-step checkout of today (address, delivery, payment, review), rendered by the
  engine from core partials, completing orders with COD and manual bank transfer.
- **Routes:** `GET /{country}/checkout?step=address|delivery|payment|review` (default `address`)
  and one `POST` per step:
  - `POST /{country}/checkout/address` → `POST /store/carts/{id}` with `email`,
    `shipping_address`, `billing_address`.
  - `POST /{country}/checkout/delivery` (form `option_id`) → `POST /store/carts/{id}/shipping-methods`.
  - `POST /{country}/checkout/payment` (form `provider_id`) → create the payment collection and
    session (`POST /store/payment-collections`, then `.../payment-sessions`).
  - `POST /{country}/checkout/complete` → `POST /store/carts/{id}/complete`; on success clear the
    cart cookie and `303` to `/{country}/order/{order_id}/confirmed`.
  Delivery options come from `GET /store/shipping-options?cart_id={id}`, providers from
  `GET /store/payment-providers?region_id={region}`.
- **Templates (core, not theme):** `crates/storefront/stheno-storefront/templates/core/checkout.html.j2`
  and `core/checkout/{address,delivery,payment,review}.html.j2`. Themes only wrap them.
- **Providers in this task:** `pp_system_default` (shown as "Cash on delivery") and `pp_manual`
  (shown as "Manual bank transfer", with the transfer instructions the manual provider returns).
  Others are listed but disabled until T4.6 and T4.7.
- **Headers:** checkout pages are `private, no-store` and carry the CSP from
  `docs/payment-page-scripts.md` §3 exactly.
- **Acceptance tests:** `checkout_requires_cart`, `address_step_saves_address`,
  `delivery_lists_engine_options`, `cod_order_completes`, `manual_transfer_order_completes`,
  `checkout_has_payment_page_csp`. Plus `bash scripts/check-checkout-scripts.sh` and
  `bash scripts/check-pci-saq-a.sh` pass.
- **Verify (Space):** as T4.1.

#### T4.6 Checkout: Stripe

- **Goal:** card payments through Stripe Elements, as today
  (`apps/storefront/packages/storefront-core/src/modules/checkout/components/payment-wrapper/index.tsx`
  and `payment-button/index.tsx`).
- **How:** a core island `templates/core/islands/stripe.js`, loaded only on the payment step when
  `pp_stripe_stripe` is chosen. It loads `https://js.stripe.com/v3/` (the only third-party script
  allowed, per `docs/payment-page-scripts.md`), mounts the card element, calls
  `stripe.confirmCardPayment(client_secret)` with the `client_secret` from the payment session, then
  submits the `complete` form.
- **Publishable key:** read where the Stripe provider keeps its configuration in
  `crates/providers/stheno-payment-stripe/src/lib.rs`. If it has a publishable key, pass it to the
  template. If it does not, set the task to `blocked: needs owner decision: where the Stripe
  publishable key is configured` and stop.
- **Acceptance:** test `stripe_step_renders_client_secret_and_key` (mocked provider); manual
  check on staging with a Stripe test key and card `4242 4242 4242 4242`, screenshot in the PR.
- **Verify (Space):** as T4.1, plus the two checkout scripts.

#### T4.7 Checkout: bKash, Nagad, SSLCommerz

- **Goal:** the three hosted-redirect gateways work end to end. Today's storefront never had them
  (no client code exists), so this is new behaviour, built on engine flows that already exist.
- **How:**
  1. When creating the payment session for one of these providers, put the return URLs in the
     session `data`: bKash and Nagad `callback_url`, SSLCommerz `success_url`, `fail_url`,
     `cancel_url`, each set to
     `https://{request host}/{country}/checkout/return/{provider_id}?cart={cart_id}`.
  2. Redirect the browser to the gateway URL the session returns.
  3. New route `GET` and `POST /{country}/checkout/return/{provider_id}`: re-read the payment
     session through the Store API. If authorized, call `POST /store/carts/{id}/complete`
     (it is idempotent: the webhook at `/hooks/payment/{provider_id}` may have completed it
     first) and `303` to the confirmed page. Otherwise `303` to
     `/{country}/checkout?step=payment&error=payment`.
  4. Never trust query parameters from the gateway as proof of payment; only the session state
     read from the engine counts.
- **Acceptance tests:** `return_url_is_tenant_host`, `return_completes_authorized_cart`,
  `return_with_unpaid_session_goes_back_to_payment`, `return_ignores_forged_query_status`.
  Sandbox runs for all three on staging need the owner's sandbox credentials (task O4); until
  then, record "sandbox run pending O4" in the PR, do not mark the sandbox criterion done.
- **Verify (Space):** as T4.1.

#### T4.8 Account pages

- **Goal:** sign-in, register, dashboard, profile, addresses, orders and order detail, matching
  `apps/storefront/packages/storefront-core/src/app/[countryCode]/(main)/account/`.
- **Routes:** `GET /{country}/account` (login form if signed out, dashboard if signed in),
  `/account/profile`, `/account/addresses`, `/account/orders`, `/account/orders/{id}`; form posts
  `POST /account/profile` → `POST /store/customers/me`; `POST /account/addresses` →
  `POST /store/customers/me/addresses`; `POST /account/addresses/{id}` and
  `POST /account/addresses/{id}/delete` → the address routes.
- **Rules:** if `GET /store/config` says buyer accounts are off, every account route is `404`.
  All account pages are `private, no-store`. A `401` from the Store API clears the cookie and
  redirects to `/{country}/account`.
- **Templates:** theme `account/*.html.j2`; forms come from `core/login.html.j2` and
  `core/account-form.html.j2`.
- **Acceptance tests:** `signed_out_sees_login`, `dashboard_lists_own_orders`,
  `cannot_open_another_customers_order` (expect 404), `profile_update_saves`,
  `address_crud`, `accounts_off_hides_routes`.
- **Verify (Space):** as T4.1.

#### T4.9 Order confirmed, payment proof and order transfer pages

- **Routes:** `GET /{country}/order/{id}/confirmed` (order from `GET /store/orders/{id}`; for
  `pp_manual` also `GET /store/orders/{id}/payment-proof` and an upload form posting to
  `POST /store/orders/{id}/payment-proof`); `GET /{country}/order/{id}/transfer/{token}` with
  accept and decline forms posting to `POST /store/orders/{id}/transfer/accept` and `.../decline`.
- **Rules:** `private, no-store`; an order the shopper may not see is `404`, never `403`.
- **Acceptance tests:** `confirmed_page_shows_engine_totals`, `manual_payment_proof_upload`,
  `transfer_accept`, `transfer_decline`, `foreign_order_is_404`.
- **Verify (Space):** as T4.1.

#### T4.10 Store pages: about and policies (engine data)

- **Goal:** about and policy text is merchant data stored in the engine. Today it is written into
  the Next.js frontend; that is the kind of hardcoding ADR-038 removes.
- **Create:** migration `migrations/<timestamp>_store_page/up.sql` and `down.sql`:

  ```sql
  CREATE TABLE store.store_page (
      id          text        PRIMARY KEY,
      tenant_id   text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
      handle      text        NOT NULL CHECK (handle ~ '^[a-z0-9][a-z0-9-]{0,63}$'),
      title       text        NOT NULL CHECK (length(title) BETWEEN 1 AND 120),
      body        text        NOT NULL CHECK (length(body) <= 60000),
      published   boolean     NOT NULL DEFAULT false,
      created_at  timestamptz NOT NULL DEFAULT now(),
      updated_at  timestamptz NOT NULL DEFAULT now(),
      deleted_at  timestamptz
  );
  CREATE UNIQUE INDEX idx_store_page_handle
      ON store.store_page (tenant_id, handle) WHERE deleted_at IS NULL;
  ALTER TABLE store.store_page ENABLE ROW LEVEL SECURITY;
  ALTER TABLE store.store_page FORCE ROW LEVEL SECURITY;
  CREATE POLICY tenant_isolation ON store.store_page
      USING (tenant_id = (SELECT current_setting('app.tenant_id', true)))
      WITH CHECK (tenant_id = (SELECT current_setting('app.tenant_id', true)));
  GRANT SELECT, INSERT, UPDATE, DELETE ON store.store_page TO stheno_app;
  ```
  `down.sql`: `DROP TABLE IF EXISTS store.store_page;`. This follows the T2.1 migration exactly
  (same RLS policy form and grant). Entity id prefix `spage` (ADR-010); add it wherever ADR-010's
  prefix list lives (find it with `graphify query "Id prefix list ADR-010"`).
- **Service and routes:** a `StorePage` entity, repository and service methods in
  `crates/modules/stheno-store` following the pattern of the existing branding methods
  (`get_branding` in `service.rs:326`); admin routes `GET`/`POST /admin/store-pages`,
  `GET`/`POST`/`DELETE /admin/store-pages/{id}`; store route `GET /store/pages/{handle}`
  (published only). `body` is sanitized with the same richtext rule as theme `richtext` tokens
  (paragraphs, bold, italic, links) on save and on render. Saving purges the page's cache tag.
- **Engine pages:** `GET /{country}/about` (handle `about`) and `GET /{country}/policies/{handle}`
  render `page.html.j2`; unpublished or missing is `404`. Public cache headers with tag
  `page:<handle>`.
- **Acceptance tests:** `rls_isolates_store_pages` (add the table to the RLS audit if the audit
  needs it), `unpublished_page_is_404`, `richtext_is_sanitized`, `saving_purges_page_tag`.
- **Verify (Space):** `-CargoPackage stheno-store -CargoClippy -CargoTestServices`, plus
  `bash scripts/check-migration-phases.sh`, `bash scripts/check-playbook.sh`,
  `bash scripts/check-schema-drift.sh`.

#### T4.11 Admin screen for store pages

- **Goal:** the merchant creates and edits about and policy pages in the admin.
- **Files:** a route under `apps/admin/src/routes/settings/store-pages/` following the structure
  of the Appearance screen from T2.3; list, create, edit, publish toggle, delete; body edited
  with the admin's existing rich text input if one exists (search `apps/admin/src/components/inputs`),
  else a plain textarea. Every string through the admin's i18n files.
- **Acceptance:** typecheck and tests pass; a test `store-pages.spec.tsx` covers create, edit,
  publish and delete against mocked endpoints; screenshot in the PR.
- **Verify (Space):** `.\scripts\build-on-space.ps1 -Mode web-admin`.

#### T4.12 SEO: titles, Open Graph, JSON-LD, sitemap and robots

- **Head tags:** every public page renders `<title>`, `<meta name="description">`,
  `<link rel="canonical">` (the request host plus path, no query string), and `og:title`,
  `og:description`, `og:url`, `og:image` from the `page` object. The `page` object is filled by
  each loader: product → title, description, first image; collection → title, description;
  home → store name and the store description if branding has one.
- **JSON-LD:** product pages render one `<script type="application/ld+json">` with schema.org
  `Product`: `name`, `image`, `description`, `sku` (variant), `offers` with `price` (the engine's
  `calculated_amount` as a decimal string), `priceCurrency`, and `availability`. Built in Rust with
  `serde_json`, never by string concatenation in a template, and allowed by the CSP via a hash or
  because `application/ld+json` is not executed (check the CSP spec; record which).
- **`/sitemap.xml`:** per tenant host: home, `/store`, every published product, category,
  collection and published store page, for every region country; `Cache-Control: public,
  s-maxage=3600`; tag `listing`. Over 50,000 URLs: a sitemap index.
- **`/robots.txt`:** `Allow: /`, `Disallow: /*/checkout`, `/*/account`, `/*/cart`, `/_sf/`, and the
  sitemap URL.
- **Acceptance tests:** `product_page_has_canonical_and_og`, `product_json_ld_uses_engine_price`,
  `sitemap_lists_published_products_only`, `robots_disallows_private_paths`.
- **Verify (Space):** as T4.1.

#### T4.13 Performance and accessibility acceptance

- **Goal:** every theme is fast and accessible on a phone.
- **How:** on staging (task T7.2), run Lighthouse in mobile mode against home, one listing and one
  product page per theme: `npx lighthouse <url> --preset=perf --form-factor=mobile --output=json`
  and the accessibility category. Lighthouse runs a browser against a deployed URL; it builds
  nothing, so it may run on any machine.
- **Acceptance:** performance ≥ 90 and accessibility ≥ 95 on every page tested; the JSON reports
  attached to the PR. A theme that misses gets a fix commit in its own theme folder only.

#### T4.14 Cut-over and deletion of the Next.js storefront

- **Goal:** the engine serves every storefront host; `apps/storefront` is gone.
- **Steps, in order:**
  1. Make `STHENO_ENGINE_STOREFRONT` default to on (the flag stays for one release as a
     kill switch, then T4.14's follow-up removes it).
  2. On staging, walk every route in section 3 for every theme; record pass or fail in the PR.
  3. Delete `apps/storefront/**`; remove the `storefront` service from `docker-compose.yml`
     (line ~152) and any storefront entry in `deploy/chart/` and `deploy/proxy/Caddyfile`
     (route those hosts to the engine instead).
  4. Remove storefront jobs from `.github/workflows/ci.yml`; keep the theme gate.
  5. Delete `scripts/check-frontend*.sh`, `scripts/check-storefront-boundaries.sh` and their
     `.ps1` twins, and every reference to them (find with
     `grep -rn "check-frontend\|check-storefront-boundaries" --include=*.yml --include=*.sh --include=*.md .`).
  6. Update docs that describe the Next.js storefront (`grep -rln "apps/storefront" docs AGENTS.md README.md`).
  7. ADR-031's notice and ADR-038's status: Implemented.
- **Acceptance:** CI green with no storefront Node job; staging walk-through all pass;
  `bash scripts/check-docs-links.sh` passes.
- **Verify (Space):** `.\scripts\build-on-space.ps1 -Mode rust-ci`.

### Phase 5 — kit and follow-up

Branch and PR: `feat/phase-5-kit`, `Phase 5: theme kit and edge duties`.

#### T5.1 Finalize THEME_AUTHORING.md and the AI Studio prompt

- **Goal:** a design tool given only `docs/THEME_AUTHORING.md` and `themes/_starter/` produces a
  theme that passes the gate on the first try.
- **Steps:**
  1. Replace every "draft" and "expect small corrections" in THEME_AUTHORING.md with the final
     contract; the data table in §4 must list exactly the fields the loaders now provide (read
     them from the loader code, do not copy from the old table).
  2. Update §8 and `docs/AGENT_START_PROMPTS.md` prompt C to match.
  3. Test it: give a fresh agent session only those two inputs and the brief "a minimal
     black-and-white theme called `trial-mono`"; run `bash scripts/check-theme-pack.sh` on the
     result. Record the gate output in the PR. Do not commit `trial-mono`.
- **Acceptance:** the trial theme passes the gate without hand edits.

#### T5.2 Origin proxy duties after cut-over

- **Goal:** ADR-037 left three duties with the origin proxy (Caddy, `deploy/proxy/Caddyfile`):
  strip client-sent tenant-override headers, translate `CF-IPCountry`, split admin and storefront
  hosts. Confirm they still hold now that the engine serves storefront hosts.
- **Decision already made:** keep Caddy as the origin proxy. It is already deployed and checked
  (`deploy/proxy/check.sh`); moving its duties into the engine adds code for no gain.
- **Steps:** update the Caddyfile so storefront hosts route to the engine; extend
  `deploy/proxy/check.py` with one case per duty for a storefront host; run `bash deploy/proxy/check.sh`
  (it starts containers: run it on Space or staging, not on the owner's machine). Record the
  decision as ADR-039 "Caddy stays the origin proxy" in `docs/DECISIONS.md`.
- **Acceptance:** `deploy/proxy/check.sh` passes with the new cases; ADR-039 in the index.

### Phase 6 — dependency upkeep

Branch and PR: `feat/phase-6-deps`, `Phase 6: dependency upkeep`. It can run at any time,
in parallel with any other phase: it touches no storefront or theme code.

#### T6.1 Upgrade `diesel-async` and `diesel`; remove the two audit ignores

- **Goal:** `.cargo/audit.toml` ignores RUSTSEC-2026-0136 and RUSTSEC-2026-0137 because
  `diesel-async = "0.5"` pins `diesel ~2.2`. Upgrade so both ignores can go.
- **Edit:** root `Cargo.toml` lines ~210-212: `diesel` to the latest `2.3.x` (at least `2.3.8`),
  `diesel-async` to the first release that accepts that `diesel` (read its changelog on
  crates.io; write the version you chose and why in the PR), `diesel_migrations` to the matching
  `2.3.x`. Keep every existing feature flag. Then `Cargo.lock` via `cargo update -p diesel
  -p diesel-async -p diesel_migrations` (this resolves only, it builds nothing).
- **Then:** fix every compile error the upgrade causes, changing call sites only, never
  behaviour. If a fix needs a behaviour change, stop and set `blocked: <the API change>`.
- **Remove:** both entries from `.cargo/audit.toml`. Leave the file's header comment.
- **Acceptance:** `.\scripts\build-on-space.ps1 -Mode rust-ci` passes (every crate builds, clippy
  clean, every test passes); the CI "Supply chain" stage passes with no ignores.
- **Verify (Space):** `.\scripts\build-on-space.ps1 -Mode rust-ci`.

### Phase 7 — go live

Branch and PR: `feat/phase-7-go-live` for repository changes. Several tasks here are owner tasks:
they need accounts, money or decisions only the owner has. They are listed in section 1.7 and
marked **Owner** on the board. An agent never creates accounts, enters credentials or buys plans.

#### T7.1 Edge acceptance checklist

- **Goal:** fill the seven blank result cells of the edge acceptance checklist in
  `docs/DEPLOYMENT.md` §8 (tag purge limits, `Cache-Tag` limits, stale-while-revalidate
  behaviour, custom-hostname pricing, wildcard certificate depth, Tunnel `Host` preservation,
  `stheno-file-s3` against R2) with measured results.
- **Depends on:** T1.8 and O1 (Cloudflare plan known), T7.2 (staging exists).
- **How:** each cell gets the command run against staging and its output, or the Cloudflare
  documentation URL plus the plan name for pure plan facts.
- **Acceptance:** no blank cell; any result that contradicts ADR-037 opens a new task.

#### T7.2 Staging environment

- **Depends on:** O2 (owner picks the server) and O3 (owner creates the Cloudflare Tunnel token).
- **Goal:** the full `docker-compose.yml` stack (proxy, cloudflared, postgres, redis, pgbouncer,
  engine, admin, platform, marketing) running on the staging server behind its own hostnames.
- **Steps:** write `deploy/staging/README.md` with the exact commands; secrets only in the
  server's environment file, never in the repository; run migrations with the engine's
  `--migrate`; seed with `scripts/dev-tenants.sh` equivalents for three test tenants.
- **Acceptance:** every route in section 3 answers on staging for each test tenant; `/health`
  is green; the README lets someone with server access rebuild staging from nothing.

#### T7.3 Payments in sandbox, on staging

- **Depends on:** O4 (owner enters sandbox credentials in the admin of the staging test tenant).
- **Goal:** one real sandbox order per provider (COD, manual transfer, Stripe, bKash, Nagad,
  SSLCommerz), each reaching the confirmed page and the admin order list, with the webhook seen
  in the engine log.
- **Acceptance:** a table in the PR: provider, order id, webhook received yes/no, screenshot.

#### T7.4 Backups and a restore drill

- **Goal:** prove `docs/DEPLOYMENT.md` §7 works: take a backup of staging Postgres, restore it into
  an empty database, start the engine against it, and open one order created in T7.3.
- **Acceptance:** commands and timings recorded in `docs/DEPLOYMENT.md` §7; restore succeeds.

#### T7.5 Platform backlog outside the storefront

- **Goal:** the parts of the product this plan does not cover are finished under their own plans,
  with the same pickup rules (section 1). They are: `docs/PLATFORM_COMPLETION_TASKS.md` groups A–I,
  and `docs/STATUS.md` phases P30–P36 (subscription billing first, `docs/STATUS.md:66-70`).
- **How:** this task is a pointer, not work. An agent with prompt A that is told "T7.5" opens those
  documents, picks the first open item there and follows the same protocol.
- **Acceptance:** every item in those documents is done or explicitly deferred by the owner.

#### T7.6 Production launch

- **Depends on:** everything above, plus O5 (live payment keys) and O6 (production DNS).
- **Steps:** deploy production from the staging README; switch DNS; run the T7.3 table again with
  live COD and one low-value live card payment the owner makes; watch logs for one hour.
- **Rollback:** keep the previous engine image tag; rollback is redeploying it. Migrations in this
  plan are additive (ADR-safe phases), so no data rollback is needed.
- **Acceptance:** the owner signs off in the PR.

---

## 6. Out of scope

Marketing site and admin app styling (they keep their own design systems); the 29 migration
playbook gaps (a separate task); multi-language content beyond existing locale strings; a theme
marketplace or merchant-uploaded themes.

## 7. Open questions for the owner

1. Cloudflare plan level: now task O1 (feeds T1.8 and T7.1).
2. Server for staging and production: task O2.
3. Trial label: the admin says 14-Day, the engine and ADR-033 say 7 days. The owner chose the
   14-Day label on 2026-11-26.

## 8. Change log

| Date | Change |
|---|---|
| 2026-11-27 | Plan written for ADR-038. Supersedes the draft in `.orchestration/task-storefront-theme-engine/spec.md`. |
| 2026-11-27 | Hardened for any agent: section 1.6 (rules that prevent guessing), verified API facts table, exact signatures, test names and expected outputs for Phases 1–2, a step-by-step conversion procedure for Phase 3, and T6.1. |
| 2026-11-27 | T0.1 done: `main` at `12203039` merged into PR #4; AI Studio files parked in `design-source/ai-studio-2026-11-27/`; `bun.lock` removed. |
| 2026-11-28 | One PR per phase; every build and test on Space. Phases 4–7 written in full: internal Store API client, shopper cookie, cart, checkout per provider, account, order pages, store pages, SEO, cut-over, theme kit, Caddy decision, diesel upgrade, go-live, and owner tasks O1–O7. |
