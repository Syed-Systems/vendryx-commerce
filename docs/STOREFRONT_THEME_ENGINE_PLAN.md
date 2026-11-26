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
| T0.1 | Merge `main` into PR #4 and park the AI Studio storefront as design source | — | todo |
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

#### T1.1 `stheno-storefront` crate skeleton, mounted in the engine

- **Goal:** the engine answers a storefront route with HTML rendered by `minijinja`.
- **Files:** `crates/storefront/stheno-storefront/**`, `Cargo.toml` (workspace member and the
  `minijinja` dependency), `crates/stheno-server/src/lib.rs` (router mount only),
  `crates/stheno-server/Cargo.toml`.
- **Steps:** create the crate; add `minijinja` 3.x; expose `storefront_router(ctx)`; mount it in
  `stheno-server` behind a config flag `STHENO_ENGINE_STOREFRONT` (default off, so the Next.js
  storefront stays authoritative until T4.7), after `/store/*`, `/admin/*` and `/auth/*`; resolve
  the tenant with the existing tenant layer; render a fixture template with `store.name`.
- **Acceptance:** with the flag on, `GET /{country}` on a tenant host returns 200 HTML containing
  that tenant's name; with the flag off, behaviour is unchanged; an unknown host gets the same
  response the engine gives today.
- **Verify:** Rust gates for `stheno-storefront` and `stheno-server`; a router test using the
  existing `TestApp`.

#### T1.2 `theme.json` schema, Rust types and validator

- **Goal:** one definition of a valid theme, used by the loader, the admin and the gate.
- **Files:** `themes/theme.schema.json`, `sf/src/schema.rs`, `sf/tests/schema.rs`,
  `sf/tests/fixtures/**`.
- **Steps:** write the JSON Schema for section 2.2; Rust types with `serde`; a validator for
  theme files and for merchant values (types, ranges, `select` membership, colour format,
  `url` scheme, text length, richtext allowlist).
- **Acceptance:** valid fixtures load; invalid fixtures fail with a field path for each rule;
  unknown token types and unknown keys are rejected.
- **Verify:** `cargo test -p stheno-storefront --test schema`.

#### T1.3 Theme loading and hashed asset serving

- **Goal:** themes load from disk at runtime and their assets are served with immutable caching.
- **Files:** `sf/src/themes.rs`, `sf/src/assets.rs`, `sf/tests/themes.rs`, workspace
  `Cargo.toml` (enable the `tower-http` `fs` feature).
- **Steps:** read `STHENO_THEMES_DIR` (default `themes/`); validate each `theme.json` with T1.2
  and refuse to load an invalid theme (log it, keep serving the others); content-hash asset
  paths; serve `/theme-assets/<theme>/<hash>/<path>` with
  `Cache-Control: public, max-age=31536000, immutable`; auto-reload in development only.
- **Acceptance:** an invalid theme never crashes the engine; asset URLs change when content
  changes; a request outside the theme folder is rejected.
- **Verify:** `cargo test -p stheno-storefront --test themes`.

#### T1.4 Token compiler and `settings` / `sections` objects

- **Goal:** defaults plus merchant values become CSS custom properties and template objects.
- **Files:** `sf/src/tokens.rs`, `sf/tests/tokens.rs`.
- **Acceptance:** merged values (merchant over default) produce one style block of
  `--theme-<id>` properties with escaped values; `settings` and `sections` match the merged
  values; snapshot test of the style block.
- **Verify:** `cargo test -p stheno-storefront --test tokens`.

#### T1.5 Loaders and filters for home, listing and product pages

- **Goal:** templates receive real engine data for `/{country}`, `/store`, categories,
  collections and products.
- **Files:** `sf/src/loaders/**`, `sf/src/filters.rs`, `sf/tests/loaders.rs`.
- **Steps:** build the objects in section 2.3 from existing services in process; register only
  the filters in section 2.3; `money` formats `stheno-types::Money`.
- **Acceptance:** rendering against a migrated test database shows the seeded products, prices
  and collections; a missing product returns the engine's 404 page, not a 500; no template can
  call anything that is not registered.
- **Verify:** `cargo test -p stheno-storefront --test loaders` with `DATABASE_URL` set.

#### T1.6 Cache headers and tenant-prefixed tags

- **Goal:** rendered pages are cacheable at the edge exactly as section 2.4 says.
- **Files:** `crates/api/stheno-api-store/src/cache.rs` (extend), `sf/src/cache.rs`,
  `sf/tests/cache.rs`.
- **Acceptance:** public pages carry the headers and tenant-prefixed tags, with the same tags as
  the data they render; personalised pages carry `Cache-Control: no-store` and no tags; no
  public page sets a cookie.
- **Verify:** `cargo test -p stheno-storefront --test cache` · `cargo test -p stheno-api-store`.

#### T1.7 Core partials and the include mechanism

- **Goal:** themes include core partials they cannot override.
- **Files:** `sf/templates/core/**`, `sf/src/render.rs`, `sf/tests/core_partials.rs`.
- **Acceptance:** `{% include "core/attribution.html.j2" %}` renders "Powered by Vendryx" on the
  Free plan only; a theme file named like a core partial does not replace it.
- **Verify:** `cargo test -p stheno-storefront --test core_partials`.

#### T1.8 Confirm Cloudflare plan facts

- **Goal:** replace the "unverified" items in ADR-037 and ADR-038 with facts.
- **Files:** `docs/DECISIONS.md` (ADR-037/038 "Unverified" paragraphs only).
- **Steps:** ask the owner for the Cloudflare plan; confirm tag purge availability and limits,
  custom-hostname allowance, and Tunnel wildcard routing preserving `Host`; record each with the
  date and source.
- **Acceptance:** each item is marked confirmed or not available, with the fallback named.
- **Verify:** `bash scripts/check-docs-links.sh`.

#### T1.9 `scripts/check-theme-pack.sh` gate

- **Goal:** a theme that breaks ADR-038 decisions 3–6 fails CI.
- **Files:** `scripts/check-theme-pack.sh` (executable), `scripts/tests/theme-pack/**`
  (fixtures), `.github/workflows/ci.yml` (one step in the Rust static or Web static job).
- **Checks:** schema-valid `theme.json`; no colour literal (`#hex`, `rgb(`, `hsl(`, named
  colours), font family, `box-shadow` or radius literal in templates, CSS or scripts; CSS uses
  only `var(--theme-*)` for those properties; no `|safe`; no `fetch(` or `XMLHttpRequest` except
  to `/store/`; no external `src`/`href` origins; no `innerHTML`; no numeric figure in templates
  that is not a template expression; no file outside `themes/<id>/`; no file named like a core
  partial; default colour pairs meet WCAG 2.2 AA contrast.
- **Acceptance:** each check has a failing fixture and the starter theme passes; the step runs
  in CI.
- **Verify:** `bash scripts/check-theme-pack.sh` · `bash scripts/tests/theme-pack/run.sh`.

#### T1.10 `themes/_starter` reference theme

- **Goal:** the smallest complete theme, used as the template for every conversion and for AI
  Studio.
- **Files:** `themes/_starter/**`.
- **Acceptance:** renders home, listing and product pages from real data; passes the gate; every
  template in section 2.1 exists (cart, account and order templates may include a core partial
  and little else until phase 4).
- **Verify:** `bash scripts/check-theme-pack.sh` · `cargo test -p stheno-storefront`.

### Phase 2 — admin Appearance and storage

#### T2.1 Migration: per-tenant theme selection and token values

- **Goal:** each tenant has an active theme and its saved token values, under RLS.
- **Files:** `migrations/<timestamp>_storefront_theme_setting/{up,down}.sql`, the module that
  owns storefront appearance (extend the existing `storefront_appearance` work), its tests.
- **Steps:** follow the migration playbook; add the active `theme_id` and a
  `storefront.theme_setting` table (tenant_id, theme_id, theme_version, values jsonb, updated_by,
  updated_at) with tenant RLS; reconcile with the ADR-021 onboarding document (theme key,
  tokens, font, home sections) so onboarding writes the same table.
- **Acceptance:** rows survive a service restart; one tenant cannot read another's values;
  existing appearance data is preserved.
- **Verify:** migration gates (section 1.4) · `cargo test -p <owning crate>`.

#### T2.2 Admin endpoints for themes and appearance

- **Goal:** the admin can list themes and save validated values.
- **Files:** `crates/api/stheno-api-admin/src/appearance.rs` (new), its router registration,
  `crates/api/stheno-api-admin/tests/appearance.rs`.
- **Steps:** `GET /admin/themes` (installed themes and schemas); `GET /admin/appearance`;
  `PUT /admin/appearance` validated with T1.2, all-or-nothing, writes the hostname purge to the
  outbox in the same transaction (ADR-037).
- **Acceptance:** bad colour, out-of-range value, unknown option, unsafe URL and oversized text
  each return 400 with the field path; a valid save persists and enqueues one purge; RBAC
  matches the existing branding endpoints.
- **Verify:** `cargo test -p stheno-api-admin --test appearance`.

#### T2.3 Admin Appearance screen generated from `theme.json`

- **Goal:** merchants edit tokens with no admin code per token.
- **Files:** `apps/admin/src/routes/appearance/**`, the admin route map entry, its tests; reuse
  `apps/admin/src/routes/branding/branding-view.tsx` patterns (live preview frame).
- **Acceptance:** each token type renders its control (colour picker, slider, select, font
  select, text, richtext, image picker, URL, toggle); saving shows field errors from the API;
  the preview updates live; adding a token to the starter theme shows it with no admin change.
- **Verify:** admin gates (section 1.4).

### Phase 3 — themes

Every theme task has the same shape. Work in `themes/<id>/` only.

- **Source:** the React design in `design-source/ai-studio-2026-11-27/` for T3.1–T3.3;
  `apps/storefront/packages/frontend-default` and `storefront-core/src/lib/branding.ts` for
  T3.4–T3.7.
- **Steps:** copy `themes/_starter`; move every colour, font, radius, shadow and spacing literal
  into a token; move every piece of copy into a section setting with the source text as its
  default; delete hardcoded catalogs, tenants, reviews and figures and render loader data
  instead; rewrite components as templates and partials; rewrite drawers, galleries and WebGL as
  interaction scripts; theme art goes in `assets/images` only if small, otherwise through the
  file provider.
- **Acceptance:** passes the gate; renders every route in section 3 that exists by then;
  screenshots at 390 px and 1440 px of home, listing, product and cart match the source design
  and are attached to the PR for the owner to approve.
- **Verify:** `bash scripts/check-theme-pack.sh` · `cargo test -p stheno-storefront`.

| Task | Theme id | Notes |
|---|---|---|
| T3.1 | `maison-cuir` | Mostly tokenized already (92 token references, 14 literals). |
| T3.2 | `hydro-vessel` | WebGL hero and specs canvases become islands; 28 literals. |
| T3.3 | `homedine-eco` | 233 colour literals and a hardcoded catalog to remove. |
| T3.4 | `default` | Apple HIG baseline from AGENTS.md §12.2. |
| T3.5 | `mahal` | Retire the `mahal:` classes and `MAHAL_BRANDING`. |
| T3.6 | `lustre` | Retire the `lustre:` classes and `LUSTRE_BRANDING`. |
| T3.7 | `matrix` | Retire the `matrix:` classes and `MATRIX_BRANDING`. |

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
