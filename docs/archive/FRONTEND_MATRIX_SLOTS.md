# frontend-matrix — slot ledger

**Status:** `ADAPT` in progress · **Date:** 2026-08-22 ·
**Required by:** [AGENTS.md §12](../../AGENTS.md#12-frontends-adapting-a-design)

`ADAPT` requires every contract slot to be labelled. This is that label, for all 45 slots in
[`contract.ts`](../../apps/storefront/packages/storefront-core/src/frontend/contract.ts). It is
deliberately blunt: **`forked` is a failure state, not a status.** Every row still marked `forked`
is frontend-default's markup wearing the Matrix palette, and the point of this file is that none of
them can hide.

| Label | Meaning |
|---|---|
| `ported` | The source component's markup, stylesheet and interactions, rebuilt on core's props |
| `derived` | No source counterpart. Drawn from the design's own rules (`design-system/MASTER.md`) and from the components already ported from it — never from frontend-default |
| `forked` | **Still frontend-default's markup.** Must be replaced before this is an `ADAPT` |

---

## 1. Where it stands

**Counted by `./scripts/check-frontend-not-forked.sh`, not by hand.** At last run: **79 of 127 files
still structurally frontend-default**, and **37 still render `@medusajs/ui` components**. Both
numbers must reach zero before this is an `ADAPT` rather than a `SKIN`.

Quote the gate's output when reporting progress here. A hand-maintained count in this file drifts
the moment someone forgets to update it, which is how "mostly done" survived three rounds.

The `SKIN` that shipped first satisfies all 45 slots, which is exactly why the count above is the
only honest way to describe progress. A build that compiles proves nothing here.

---

## 2. Ported — the design's own components

| Slot | Source component | What carried over |
|---|---|---|
| `Nav` | `Navbar.jsx` | Scroll-away header, shrink-on-scroll, full-screen mobile overlay, `useScrollDirection` verbatim. Cart is a core-rendered slot; `lucide-react` replaced by four inline paths |
| `Footer` | `Footer.jsx` | Four-column top section, brand block, bottom bar — wired to real collections and categories where the source had one hardcoded link |
| `StoreTemplate` | `ProductGrid.jsx` | Centred header and a chip rail **above** a full-bleed grid. Medusa's 256px filter sidebar is gone; the source stylesheet owns the breakpoints |
| `SubmitButton` | `CheckoutButton` | Full-width gold block; click-to-navigate became form submit + `useFormStatus`, because the action is core's |
| `BrandFeatures` | `EditorialSection.jsx` | Three-pillar band, numbered cards, gold rule |

Ported frontend modules that core imports but the contract does not name:

| Module | Source | Note |
|---|---|---|
| `products/.../product-preview` | `ProductCard.jsx` | Pointer-tracked spotlight, tilt, second-image crossfade, badge stack, gold price rail. Discount badge reads core's `percentage_diff` instead of recomputing from two numbers |
| `layout/.../cart-dropdown` | `CartDrawer` + `CartItem` | **An interaction change, not a restyle.** Medusa's was a `@headlessui/react` Popover opening on hover with a 5s auto-close; this is a click-opened drawer with overlay, Escape, focus trap and body-scroll lock |
| `store/.../refinement-list` | `FilterBar.jsx` | Chip row. Source filtered client-side from `ShopContext`; these chips write `sortBy` to the URL because core owns the query |
| `store/templates/paginated-products` | `ProductGrid.jsx` | The grid itself — the design's columns and gutters, not the Medusa preset's scale |

## 3. Derived — no counterpart in the design

Drawn from `MASTER.md` and from the components already ported above.

| Slot | Note |
|---|---|
| `Input` | Mono label above a square field, gold on focus. Dropped Medusa's floating-label trick, which needed a global rule in **core's** stylesheet to position a **frontend's** label |
| `Checkbox` | Square that fills gold. Medusa's hardcoded `id="checkbox"` on every instance, so two on a page shared one id |
| `RadioIndicator` | The one circle in the frontend — a square radio lies about exclusivity, and accessibility outranks the style rule |
| `Divider`, `ErrorMessage`, `InteractiveLink`, `Spinner`, `ChevronDown` | Built on the frontend's own `common/styles/ui.module.css` |
| `Hero` | In the Matrix language, but **not** the source `HeroSection` — see §5 |

## 4. Forked — the work remaining

**Has a source counterpart — port it:**

| Slot | Source | Staged stylesheet |
|---|---|---|
| `CollectionTemplate` | `CollectionGrid.jsx` | `collections/templates/collection-grid.module.css` |
| `CategoryTemplate` | `ProductGrid.jsx` | reuses `store/templates/product-grid.module.css` |
| `CategoryGrid` | `CollectionGrid.jsx` | reuses the collection sheet |
| `CartTemplate` | `CartDrawer` + `CartItem` | reuses the drawer and row sheets |
| `ItemsPreviewTemplate` | `CartDrawer.jsx` | reuses the drawer sheet |
| `CartTotals` | `CartDrawer` footer | reuses the drawer sheet |
| `FreeShippingPriceNudge` | `CartDrawer` shipping note | reuses the drawer sheet |
| `LookbookStory` | `AboutSection.jsx` | `home/components/lookbook-story/about.module.css` |
| `ProductTemplate` | `app/product/[id]/page.js` | — |

`ticker.module.css` is staged and wired to nothing: `BrandTicker` has no contract slot. **A staged
stylesheet is not progress** — it counts when a component renders it.

**No counterpart — must be derived** (the source shop had no accounts, orders or transfers):

`AccountLayout` · `LoginTemplate` · `Overview` · `OrderOverview` · `AddressBook` ·
`ProfileName` · `ProfileEmail` · `ProfilePhone` · `ProfilePassword` · `ProfileBillingAddress` ·
`TransferRequestForm` · `TransferActions` · `TransferImage` · `OrderCompletedTemplate` ·
`OrderDetailsTemplate` · `CartMismatchBanner` · `NewsletterSection` · `CountrySelect` ·
`DiscountCode` · `SkeletonCardDetails` · `SkeletonCartPage` · `SkeletonOrderConfirmed`

That is 22 slots the design never had an opinion about — and it is the normal case, not a
surprise. [AGENTS.md §12.2](../../AGENTS.md#12-frontends-adapting-a-design) rule 2 governs them:
derive from the design, never from `frontend-default`.

---

## 5. Decisions this adaptation is waiting on

**The hero — decided: `gsap` was added.** The source `HeroSection` is an exploded watch in 3D:
six `WatchParts` SVG layers inside a `perspective: 1400px` scene, pinned for 150% of scroll and
scrubbed through explode → labels → hold → reassemble, with pointer parallax weighted per layer.
It is not WebGL — the live site renders zero `<canvas>` — but it is genuinely three-dimensional.

CSS scroll-driven animation was built first and **cannot** do it: `animation-timeline` cannot pin
a section, and nothing in CSS tracks the pointer. So `gsap` and `@gsap/react` are storefront
dependencies now, matching the source shop's own. Logged in
[FRONTEND_MATRIX_CORE_DELTAS.md](FRONTEND_MATRIX_CORE_DELTAS.md).

Three bugs this uncovered, all worth knowing before pinning anything else:

- **`useGSAP`'s `scope` resolves selector strings inside the scope**, so an ancestor passed as a
  selector never matches. ScrollTrigger then falls back to the tween target and pins nothing,
  silently. Resolve ancestors from the DOM (`closest`), never by selector.
- **`overflow-x: hidden` on `<body>` forces the other axis to compute as `auto`**, making body a
  scroll container. ScrollTrigger measures against the window and pins against the wrong scroller.
  Use `overflow-x: clip`.
- **Pin spacing cannot be reserved when the pinned element's parent is a flex container.** Core's
  home page wraps sections in `flex flex-col` and `.heroSection` is itself a column flex box, so
  both candidates were flex children: the pin held but reserved no scroll distance, and the next
  section slid up over a fixed hero. `.pinWrap` in `hero.module.css` gives the pinned grid a block
  parent — a frontend-side fix, so core's page is untouched.

**The routes.** `/about` and `/policies/[id]` have no core equivalent, and a frontend may not
route. [ADR-020](../DECISIONS.md#adr-020) is **Proposed** and records the options. That half does not
start until it is decided.

**Quick add-to-cart.** The source `ProductCard`'s slide-up rail added to cart from the listing.
The rail is ported and navigates instead: a frontend does not mutate the cart, and the listing hands
it no add action. Making it real needs a core change — a cart action passed into the card — which
would be a contract addition, not a styling fix.
