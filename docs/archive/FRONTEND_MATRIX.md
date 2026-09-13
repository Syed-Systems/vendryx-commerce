# frontend-matrix — The Matrix Hours

**Status:** Working · **Date:** 2026-08-22 · **Relates to:** [P27-03..P27-05, P27-08](../PHASE3_TASKS.md) ·
**Governed by:** [ADR-013](../DECISIONS.md#adr-013)

A second storefront frontend, ported from the standalone **The Matrix Hours** shop
(`../The Matrix Hours`) — a dark, flat, luxury-watch storefront. It exists to be
sold and to be run next to `frontend-default` on the same source tree.

---

## 1. Running it

```bash
npm --prefix apps/storefront run dev:matrix      # matrix alone, :8001
npm --prefix apps/storefront run dev:parallel    # default :8000 + matrix :8001
npm --prefix apps/storefront run build:matrix    # production build → .next-matrix
npm --prefix apps/storefront run typecheck:matrix
./scripts/check-frontend-switch.sh                  # the binding actually switched
```

Both processes read the same `.env.local` and talk to the same backend. They are
two renderings of one catalog, which is the point — the comparison is the
product demo.

**Turbopack is off for per-frontend processes.** The binding is a webpack request
rewrite (§3) and Turbopack does not read the `webpack` hook, so it would serve
`frontend-default` out of a matrix build without saying so. `next.config.js` throws
rather than let that happen; the default `npm run dev` keeps `--turbopack`.

---

## 2. What was ported, and what was not

The source shop is JavaScript, Next 16, its own `ShopContext`, and a
Google-Sheets catalog. None of that crosses over — what crosses over is the
design language in `design-system/MASTER.md`: jet black, gold accent, 1px
borders instead of shadows, no radii, no glassmorphism, Space Mono for anything
technical, Plus Jakarta for headings.

| From the source shop | Status |
|---|---|
| Token palette and easings (`globals.css`) | Ported verbatim into `modules/styles/tokens.css` |
| `ProductCard.module.css` | Ported, squared off, spotlight made CSS-only |
| Navbar, Hero, footer language | Rebuilt against core's props |
| GSAP exploded-watch hero scene | **Dropped.** Needs the shop's part artwork and a client timeline; would also add `gsap` as a storefront dependency |
| `CartContext`, `useShopData`, `lib/sheets.js` | **Dropped.** A frontend does not fetch — ADR-013 decision 4, enforced by `check-frontend-purity.sh` |
| Quick add-to-cart on the product card | **Dropped**, same reason. The card's slide-up rail is now a link affordance |
| Custom cursor, grain overlay, scroll progress | Not ported. Skipped deliberately, not blocked — add them if the demo wants them |

**Being converted from a `SKIN` into a real frontend module.** Per-slot status
is in [FRONTEND_MATRIX_SLOTS.md](FRONTEND_MATRIX_SLOTS.md); the live count comes from
`./scripts/check-frontend-not-forked.sh`, which fails while any file is still
frontend-default's markup. §2 below describes how the original `SKIN` was built; it
is history, not the target.

**As first shipped, this was a `SKIN`, not a `PORT`** — see [AGENTS.md §12](../../AGENTS.md#12-frontends-adapting-a-design).
frontend-matrix started as a copy of `frontend-default`, so it satisfies all 45
contract slots by construction, and none of The Matrix Hours' own components
(`Navbar`, `CartDrawer`, `CollectionGrid`, `FilterBar`, `EditorialSection`,
`BrandTicker`, `GrainOverlay`, `ScrollProgress`, the `WatchParts` hero) came
across. Three passes took the copy to the Matrix look:

1. **Token layer** — `modules/styles/tokens.css` re-points the Medusa UI preset's
   own `--fg-*` / `--bg-*` / `--border-*` / `--button-*` variables at the dark
   palette. That re-skins every `text-ui-*` and `bg-ui-*` utility across all 124
   components without touching their markup, which is most of the surface.
2. **Literal pass** — the remaining hardcoded light Tailwind literals
   (`bg-white`, `text-zinc-900`, `border-gray-200`, …) were mapped to the Matrix
   palette; radii and drop shadows were removed per MASTER.md §1.
3. **Identity components** — `home/components/hero`, `layout/templates/nav`,
   `products/components/product-preview` (+ its CSS module),
   `products/components/thumbnail`, `common/components/submit-button` and
   `common/components/input` were rewritten by hand.

Passes 1 and 2 are why this took hours instead of the phase P27 budgets. The
honest caveat is in §5.

---

## 3. How the binding switches

`tsconfig.json` keeps exactly one `@frontend/*` mapping, at `frontend-default` — that
is still the single binding `check-frontend-resolution.sh` enforces, and it is what
`npm run typecheck` checks. `tsconfig.matrix.json` is the same file pointed at
`frontend-matrix` for `typecheck:matrix`.

At bundle time `next.config.js` reads `STHENO_FRONTEND` and installs a
`NormalModuleReplacementPlugin` that rewrites `@frontend/*` and `@modules/*` to the
bound frontend's directory, with `@modules/checkout/*` and
`@modules/common/components/localized-client-link` held back to core.

**Why a request rewrite and not `resolve.alias`:** it was `resolve.alias` first.
Next registers its own tsconfig-paths resolver on the same resolution hook and
wins, so `STHENO_FRONTEND=matrix next build` reported *"Compiled successfully"* and
emitted `frontend-default`'s markup. Every existing gate passed. Only grepping the
emitted chunks found it. `scripts/check-frontend-switch.sh` is that grep, made
cheap and mandatory.

Each frontend also gets its own `distDir` (`.next` / `.next-matrix`), without which
two parallel `next dev` processes overwrite each other's build output.

---

## 4. Core changes this required

Two, logged in [FRONTEND_MATRIX_CORE_DELTAS.md](FRONTEND_MATRIX_CORE_DELTAS.md), which is within
the P27-07 budget of two.

---

## 5. What is not done

- **Renders, but nobody has looked at it.** The engine is up (`./scripts/dev-tenants.sh up`)
  and both frontends return 200 with their own markup on `/us`. That proves the
  binding and the data path, not the design. No page has been opened in a
  browser and no screenshot taken — do that before showing it to anyone.
- **Pass 2 was mechanical.** A regex mapped light utilities to dark ones across
  ~56 files. It cannot tell a card background from a button background, so
  expect some contrast to be wrong on the pages nobody has opened yet — account,
  order detail, transfer, and the skeletons are the likely offenders.
- **Checkout is unskinned.** The 18 checkout components are core's (ADR-013
  decision 2) and pick up the token layer, so they go dark, but the deliberate
  skinning pass of P27-06 has not happened.
- **No conformance pack.** P27-01/02 defines one and it does not exist yet, so
  "frontend-matrix satisfies the contract" currently means "it typechecks against
  it" — presence, not behaviour.
- **The hero has no figures, deliberately.** A spec strip of invented numbers
  (references in stock, warranty months, authentication turnaround) was removed:
  the contract hands `Hero` no props, so a frontend cannot source any of them, and
  numbers in that position read as measured. Restoring it needs real inputs —
  which means either new `Hero` props from core (a third core delta, over the
  P27-07 budget) or a merchant-editable content surface, which does not exist
  yet. The same applies to the promo bar in `nav`.
- **Marketing copy is still frontend-default's, and it is apparel copy.**
  `brand-features` offers "carbon-neutral courier delivery on all orders over
  $150" and `newsletter-section` offers "10% off"; neither belongs on a watch
  store, and both are claims nobody has approved. Left alone rather than
  silently rewritten — that copy is the merchant's to write.
- **Fonts load from Google Fonts** via `@import` in the token layer rather than
  `next/font`, because the font choice belongs to the frontend and the root layout
  belongs to core. Fine for a demo; revisit if it shows up in LCP.
