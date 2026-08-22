# Core changes frontend B required

**Status:** Open · **Date:** 2026-08-22 · **Task:** [P27-07](PHASE3_TASKS.md) ·
**Governed by:** [ADR-013](DECISIONS.md#adr-013)

[P27-07](PHASE3_TASKS.md) budgets **two** core changes for frontend B:

> More than two means the P25 contract was drawn wrong and is fixed here.

Frontend B is [frontend-matrix](FRONTEND_MATRIX.md). It needed two.

---

## Delta 1 — a frontend supplies a token layer

**File:** `apps/storefront/packages/storefront-core/src/app/layout.tsx`

```diff
 import "../styles/globals.css"
+import "@frontend/styles/tokens.css"
```

**Why:** [FRONTEND_CONTRACT_INPUTS §7.1](FRONTEND_CONTRACT_INPUTS.md) said frontend
components must consume colour and font as CSS custom properties resolved per
request, never as props or build-time Tailwind constants. It did not say where
those properties get *declared*. Nothing in the contract let a frontend put a
stylesheet on the page, so a frontend could only re-skin by editing markup in all
124 of its components.

**Shape of the fix:** one alias-resolved global stylesheet, loaded after
`globals.css` so a frontend's `:root` block lands after the Medusa UI preset's and
wins. Every frontend now supplies `modules/styles/tokens.css`; `frontend-default`'s is
a comment, because the preset's light palette is already its palette.

**Contract impact:** this is a new required file per frontend, so strictly it is a
contract addition. It is not in `FrontendSlot` — that type enumerates components,
and a stylesheet is not a component. **Open question for P25's owner:** should
the token layer be named in the contract type, or stay a filesystem convention
enforced by nothing? Today it is enforced by the build failing, which is late.

---

## Delta 2 — the bundler binding became switchable

**File:** `apps/storefront/next.config.js`

`STHENO_FRONTEND` selects the frontend; a `NormalModuleReplacementPlugin` rewrites
`@frontend/*` and `@modules/*` to the bound frontend, holding `@modules/checkout/*`
and `@modules/common/components/localized-client-link` back to core; `distDir`
is per-frontend so two frontends can run at once.

**Why:** the P25-07 binding is a `tsconfig.json` path, which is one frontend per
tree. Serving two frontends at once — the whole ask — needs the binding to be a
process input rather than a file edit. This is [P27-08](PHASE3_TASKS.md)'s own
subject matter, so calling it a *delta* is arguably generous to the count; it
changes no core module and no contract.

**Cost incurred:** Turbopack does not read the `webpack` hook, so per-frontend
processes run on webpack and `next.config.js` throws if `--turbopack` is passed
with a non-default frontend. Reverting that costs a Turbopack-native binding
(`turbopack.resolveAlias` has no prefix matching, so it would need one entry per
slot, generated).

---

## Reading of the budget

Two of two, and only Delta 1 touches the contract. That is a real signal the
P25 contract was drawn close to right — but read it with §5 of
[FRONTEND_MATRIX.md](FRONTEND_MATRIX.md) next to it. frontend-matrix started as a **copy
of frontend-default**, which is exactly the bias
[FRONTEND_CONTRACT_INPUTS §1](FRONTEND_CONTRACT_INPUTS.md) warned about:

> A second frontend built by the same people who built core will not exercise the
> contract honestly. They will unconsciously avoid the awkward parts.

A frontend that inherits frontend-default's component tree cannot discover a missing
slot, because it already has every slot. **The delta count above is a floor, not
a measurement.** The number that would mean something comes from a frontend built
from `docs/FRONTEND_CONTRACT_INPUTS.md` §3 alone, by someone who has not read
`frontend-default` — which is what P27-03 actually asks for and what this is not.

---

## Delta 3 — `gsap` and `@gsap/react` are storefront dependencies

**File:** `apps/storefront/package.json`

**Why:** the hero timepiece is the design. It is six SVG layers moved in a 3D scene, pinned for
150% of scroll and scrubbed, with pointer parallax — see
[FRONTEND_MATRIX_SLOTS.md §5](FRONTEND_MATRIX_SLOTS.md). A CSS scroll-driven implementation was
written first and removed: `animation-timeline` cannot pin, and CSS cannot follow the pointer.

**Not a contract change.** No core module, route or type moved; the dependency lands in the app
shell and is imported by exactly one client component in one frontend. `frontend-default` does not
import it and is unaffected.

**Cost, stated plainly:** ~50KB gzipped of JavaScript on the highest-traffic page of any store
built on this frontend, for a decorative scene. It is gated to `min-width: 992px` and
`prefers-reduced-motion: no-preference`, so phones and reduced-motion users download it and never
run it — which is the wrong trade if this frontend ever sells at scale. The fix, if it matters, is
a dynamic import of `watch-scene.tsx` behind the same media query, not removing the scene.

**Reversal condition:** if a second frontend needs a different animation library, the dependency
should move out of the app shell and into per-frontend dependencies — which the current workspace
layout already supports, since every frontend is its own package.
