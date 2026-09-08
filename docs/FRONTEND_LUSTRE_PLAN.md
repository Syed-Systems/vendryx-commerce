# Frontend Lustre: Luxury Watch and Ornament Storefront Plan

**Status:** Design and implementation plan  
**Package target:** `apps/storefront/packages/frontend-lustre`  
**Architecture:** Independent frontend on `@stheno/storefront-core`  
**Governed by:** [AGENTS.md](../AGENTS.md), [ADR-013](DECISIONS.md#adr-013), and the [frontend contract](../apps/storefront/packages/storefront-core/src/frontend/contract.ts)

`lustre` is an internal frontend key, not a customer-facing brand. Visible identity must come from
merchant-approved data. The package must never render `Lustre` as the shop name.

## 1. Design read

Reading this as: a premium commerce frontend for watch collectors, ornament buyers, and gift
shoppers, with a cold-luxury, product-first language informed by Apple design principles.

This is a greenfield frontend, not a skin and not an adaptation of
`frontend-default` or `frontend-matrix`.

| Dial | Value | Effect |
|---|---:|---|
| `DESIGN_VARIANCE` | 7 | Asymmetric editorial compositions with predictable commerce controls |
| `MOTION_INTENSITY` | 6 | One cinematic scroll moment, restrained reveals, and immediate controls |
| `VISUAL_DENSITY` | 3 | Gallery-like spacing with compact buying information where decisions happen |

### Experience principles

1. Product first. Photography, material, price, variant, and availability outrank decoration.
2. Quiet confidence. No black-and-gold luxury cliche, glow effects, fake scarcity, or ornamental copy.
3. Familiar control behavior. Apple guidance is used for clarity, hierarchy, comfort, accessibility,
   and natural feedback. The storefront does not imitate Apple product pages.
4. One action at a time. Every view has a clear primary action and no duplicate CTA intent.
5. Honest commerce. Every figure, delivery promise, warranty term, review count, and stock message
   comes from props or approved content.

## 2. Architectural position

The frontend supplies presentation components to the existing 45-slot contract. It does not own
routes, data fetching, money formatting, checkout sequencing, payment SDKs, or URL state.

```text
storefront-core
  routes, data, actions, money, checkout, SEO
          |
          | typed props and core-rendered children
          v
frontend-lustre
  markup, layout, states, tokens, imagery, interaction presentation
```

Non-negotiable implementation rules:

- Create new markup and styles for every slot.
- Set `stheno.independent: true` with no `forkDebt` allowance.
- Import no component from `@medusajs/ui`.
- Import no SDK, core internals, checkout module, request state, or server action.
- Keep existing routes and localized links. Do not add `/shop`, `/about`, or policy routes in the
  frontend.
- Keep checkout flow in core. The frontend supplies tokens and permitted presentation surfaces only.
- Use semantic CSS custom properties so merchant branding can replace approved identity tokens.

## 3. Visual system

### 3.1 Palette

The base direction is cold luxury. Precious-metal colors belong to product photography, not to the
interface chrome.

| Role | Light mode | Dark mode | Use |
|---|---|---|---|
| Canvas | `#F4F6F7` | `#121416` | Whole-page theme |
| Raised surface | `#E9EDF0` | `#1C2023` | Drawers, product media, form groups |
| Primary text | `#171A1D` | `#F1F3F4` | Headings and body |
| Secondary text | `#596168` | `#AEB5BA` | Supporting copy |
| Hairline | `#C9D0D5` | `#363C41` | Separation without card clutter |
| Accent | `#12558A` | `#6EA6D1` | Focus, selected state, primary action |
| Error | `#A52A34` | `#FF8C96` | Inline failure state only |

The page follows system color preference and locks one mode across all sections. The accent stays
sapphire throughout. Product photography may contain gold, silver, gemstone, enamel, or leather
without changing the UI accent.

### 3.2 Typography

- One family: a licensed, self-hosted `Geist Sans` variable font or the existing system stack if the
  font cannot pass the dependency and license review.
- Use Regular, Medium, and Semibold. Avoid thin weights for small text.
- Display: `clamp(2.75rem, 6vw, 6.5rem)`, tight tracking, maximum two lines.
- Section title: `clamp(2rem, 3.5vw, 4rem)`.
- Body: `clamp(1rem, 0.25vw + 0.95rem, 1.125rem)`, maximum 65 characters per line.
- Commerce labels and prices: tabular numerals, medium weight, no decorative monospace.
- Text remains useful at 200 percent zoom. Constrained horizontal groups stack before truncating.

### 3.3 Shape, spacing, and material

- Use an 8px spacing grid.
- Page width: maximum 1440px, with 24px mobile and 48px desktop gutters.
- Surface radius: 16px. Input radius: 12px. Buttons and compact chips: full pill. This is the complete
  radius rule.
- Prefer whitespace and one hairline over containers around every group.
- Use web frosted glass only for the sticky navigation and mobile purchase bar. It is a web
  approximation, not Apple Liquid Glass.
- Provide an opaque fallback when transparency or backdrop filtering is unavailable or reduced.
- Shadows are cool-tinted and used only for floating layers.

### 3.4 Imagery

Required asset language:

- Hero: one high-resolution still life combining a watch and an ornament on mineral glass or brushed
  steel. No product label, price, or claim is baked into the image.
- Catalog: source only product media supplied by the backend.
- Product detail: packshot, macro material view, side profile, and worn or scale context when those
  images exist in product data.
- Editorial section: one approved brand image. If none is supplied, omit the visual section rather
  than use unrelated stock.
- Reserve intrinsic aspect ratios and use `next/image` through the core-compatible image path.

Generated or commissioned marketing assets must be recorded as design assets. Generated imagery
must not depict a real catalog item unless the result is approved as an accurate representation.

## 4. Core page compositions

### 4.1 Global navigation

- 72px single-line desktop bar inside a 16px floating surface.
- Left: approved merchant wordmark or store name.
- Center: store, collections, and category entry points received from core.
- Right: locale, region, account, and bag.
- Mobile: wordmark, search if supplied by core, and bag remain visible. Navigation opens as a
  full-height Headless UI dialog.
- The bar becomes slightly more opaque after content passes beneath it. Its size does not animate.

### 4.2 Home

Use six distinct layout families:

```text
+---------------------------------------------------------------+
| floating navigation                                           |
+------------------------+--------------------------------------+
| approved copy          | watch and ornament still life       |
| primary action         |                                      |
+------------------------+------------------+-------------------+
| featured category      | category         | category          |
+------------------------+------------------+-------------------+
|             finite featured-product rail                      |
+-------------------------------+-------------------------------+
| pinned material image         | approved story passages       |
+-------------------------------+-------------------------------+
| curated collection mosaic                                     |
+---------------------------------------------------------------+
| working signup or appointment surface, otherwise omitted      |
+---------------------------------------------------------------+
| sourced footer                                                |
+---------------------------------------------------------------+
```

1. Asymmetric hero. Copy occupies five grid columns and the still life occupies seven. Maximum one
   eyebrow, one headline, one short paragraph, and one primary CTA.
2. Category gallery. One large feature tile and two smaller tiles, populated only from real category
   data.
3. Featured product rail. Drag and arrow navigation with visible next-item affordance.
4. Material story. A single pinned macro image with three short, approved passages. This is the only
   GSAP scroll narrative.
5. Curated collection mosaic. Mixed portrait and square product media with no duplicate card layout.
6. Newsletter or appointment surface. Render only when core supplies a working action and approved
   copy. Otherwise the slot returns `null`.

### 4.3 Catalog, category, and collection

- Editorial header above controls, never a centered title followed by generic equal cards.
- Desktop uses a four-column visual grid when media supports it. Tablet uses three and mobile uses two.
- Sort is a Headless UI listbox. Filters use a Headless UI dialog on mobile and Radix Accordion groups
  on larger screens.
- Product tile presents image, title, formatted price, and sourced availability or compare-at state.
- Hover may crossfade to the second real image. It must not tilt, glow, or expose a fake quick-add.
- Loading skeletons preserve the exact final media ratio and text rhythm.

### 4.4 Product detail

```text
+--------------------------------------+------------------------+
| product media                        | title                  |
| portrait and macro views             | formatted price        |
|                                      | variant options         |
|                                      | primary purchase action |
|                                      | sourced disclosures     |
+--------------------------------------+------------------------+
| finite related-product rail                                   |
+---------------------------------------------------------------+
```

- Desktop: seven-column media gallery and five-column sticky purchase panel.
- Mobile: Embla media carousel followed by title, price, options, and purchase action.
- Variant selection stays core-owned. Selected, unavailable, pending, and error states are visually
  distinct and do not rely on color alone.
- Product details use Radix Accordion. Only sourced product information is shown.
- The mobile purchase bar uses restrained frosted material with an opaque accessibility fallback.
- Related products use a finite Embla rail with manual controls and no autoplay.

### 4.5 Cart and checkout

- Bag preview is a right-side Headless UI dialog with focus trap, Escape dismissal, scroll lock, and
  focus restoration.
- Cart page keeps product editing on the left and a sticky total summary on the right.
- Error messages remain adjacent to the control or section that failed.
- Checkout is visually aligned through the approved token seam only. Step order, validation timing,
  payment wrappers, and completion actions remain untouched.
- No animation delays payment feedback, validation, bag totals, or the final order action.

### 4.6 Account, orders, and transfer

- Use a calm two-column account shell on desktop and a Headless UI disclosure menu on mobile.
- Order cards use whitespace and grouped facts. Avoid dense row-by-row divider tables.
- Addresses are clear blocks with explicit edit and remove actions.
- Transfer and confirmation screens prioritize identity, order reference, next action, and recovery.
- Destructive or irreversible actions use an accessible confirmation dialog supplied by Headless UI.

### 4.7 Error, empty, and loading states

The existing Matrix `ErrorMessage` provides the behavioral reference: render nothing without an
error, expose `role="alert"`, and preserve `data-testid`. Lustre uses new markup with a cool-red edge,
short title when context exists, message text, and an optional recovery action supplied by the caller.

- Inline form error: directly under the field, linked with `aria-describedby`.
- Section error: inside the failed section with a retry action only when core provides one.
- Page error: focused heading, explanation, and one valid route back.
- Empty state: explain what is absent and provide the next valid action.
- Loading: shape-matched skeleton, no generic centered spinner unless the contract exposes only
  `Spinner`.

## 5. Component source policy

The storefront has one design system, the Lustre tokens above. The libraries below provide behavior,
accessibility, or specialist mechanics. Their demo styling is never copied.

| Need | Source | Dependency position | Reason |
|---|---|---|---|
| Dialog, mobile menu, bag drawer, listbox, menu, checkbox, radio, form field | `@headlessui/react` | Already declared | Focus, keyboard, dismissal, portal, and state behavior |
| Product detail disclosure and desktop filter groups | `@radix-ui/react-accordion` | Already declared | WAI-ARIA accordion behavior and keyboard navigation |
| Hero and one material-story scroll sequence | `gsap`, `@gsap/react`, `ScrollTrigger` | Already declared | Scoped, scrubbed storytelling with reliable cleanup |
| Product media and product rails | `embla-carousel-react` | Proposed | Drag, snap, controls, responsive rails, and lifecycle cleanup |
| Interface icons | `@phosphor-icons/react` | Proposed | One coherent icon family with accessible React components |
| Images and links | Next.js `Image` and core `LocalizedClientLink` | Already present | Optimization, localization, and route ownership |
| Divider, error text, price text, skeleton surface | Semantic HTML and CSS | Platform | A dependency would not improve the simple semantics |

Before either proposed package is added, its transitive dependency count, current release activity,
open issue position, and license must be recorded as required by AGENTS.md section 9. If the review
rejects Embla, use native CSS scroll snap with explicit controls. Do not build a custom carousel
engine.

Do not add another full design system, toast system, scroll-smoothing library, or animation library.
Do not hand-draw SVG icons.

Official implementation references:

- [Apple design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles)
- [Apple accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility)
- [Apple motion](https://developer.apple.com/design/human-interface-guidelines/motion)
- [Headless UI dialog](https://headlessui.com/react/dialog)
- [Radix Accordion](https://www.radix-ui.com/primitives/docs/components/accordion)
- [Embla React](https://www.embla-carousel.com/docs/v8/get-started/react)
- [GSAP ScrollTrigger](https://gsap.com/docs/v3/Plugins/ScrollTrigger/)
- [Phosphor Icons](https://phosphoricons.com/)

## 6. Motion system

Every motion must communicate feedback, spatial consistency, state, or story.

| Surface | Purpose | Tool | Specification |
|---|---|---|---|
| Button press | Feedback | CSS | `transform: scale(0.98)`, 120ms, strong ease-out |
| Menu and listbox | State and spatial consistency | Headless UI plus CSS | Opacity and 8px translate, 180ms enter, 140ms exit |
| Bag drawer | Spatial consistency | Headless UI plus CSS | Translate from the same edge on enter and exit, 260ms |
| Product image hover | Feedback | CSS | Image opacity or scale to 1.02, 220ms, fine pointers only |
| Hero entrance | Hierarchy | GSAP | Copy and image opacity with small y offset, 500ms total |
| Material story | Explanation | GSAP ScrollTrigger | `start: "top top"`, pinned image, scrubbed detail changes |
| Section reveal | Prevent jarring change | CSS or IntersectionObserver | Opacity and 16px y offset, 360ms, 50ms stagger |
| Skeleton | State indication | CSS | Low-contrast shimmer only while loading |

Motion constraints:

- Animate only transform and opacity.
- Use `@gsap/react` or `gsap.context()` so every timeline and ScrollTrigger is reverted on cleanup.
- Do not attach raw window scroll listeners.
- Do not smooth-scroll or hijack catalog browsing.
- No autoplay carousel, perpetual floating product, cursor follower, or decorative marquee.
- Under `prefers-reduced-motion`, remove pinning, parallax, scaling, and translation. Retain short
  opacity changes that preserve state comprehension.
- Gate hover transforms behind `(hover: hover) and (pointer: fine)`.
- Frequently used controls remain immediate. Keyboard-triggered actions do not wait for animation.

## 7. Footer design

Reuse the existing Footer data contract and localized-link behavior, not either existing footer's
markup or hardcoded copy.

Desktop composition:

```text
merchant identity       catalog links       account links
approved short line     real collections    region and locale
-----------------------------------------------------------
copyright               real policy links only when routed
```

Mobile collapses to a vertical identity block followed by two Headless UI disclosures. Link groups
must wrap without horizontal overflow.

The footer must not invent:

- concierge email, phone number, hours, or location;
- free shipping, returns, care, warranty, or authentication promises;
- discounts, social handles, payment methods, or policy routes;
- category fallbacks when category data is empty.

If core supplies no approved value, omit the element.

## 8. Contract slot plan

Every target is `derived`. No slot may begin by copying the matching file from another frontend.
`ErrorMessage` and `Footer` may reuse behavior and prop contracts only.

| Slot | Planned Lustre treatment | Source |
|---|---|---|
| `AccountLayout` | Editorial two-column account shell | Derived, Headless UI mobile disclosure |
| `AddressBook` | Spacious address blocks with explicit actions | Derived |
| `BrandFeatures` | Approved materials or service facts only | Derived, content-gated |
| `CartMismatchBanner` | Persistent contextual warning below navigation | Derived from core state |
| `CartTemplate` | Product list plus sticky summary | Derived |
| `CartTotals` | Grouped money hierarchy with one checkout CTA | Derived |
| `CategoryGrid` | One-large, two-small category gallery | Derived, data-gated |
| `CategoryTemplate` | Editorial heading, controls, product grid | Derived |
| `Checkbox` | High-contrast Headless UI control | Derived, Headless UI |
| `ChevronDown` | Phosphor caret with rotation state | Derived, Phosphor |
| `CollectionTemplate` | Collection image lead plus product grid | Derived |
| `CountrySelect` | Compact labeled listbox | Derived, Headless UI |
| `DiscountCode` | Disclosure form with inline result | Derived, Headless UI |
| `Divider` | Sparse semantic separator | Derived, platform |
| `ErrorMessage` | Inline alert with recovery space | Derived from existing behavior |
| `Footer` | Identity and real dynamic links | Derived from existing data contract |
| `FreeShippingPriceNudge` | Sourced progress state only | Derived from core calculation |
| `Hero` | Asymmetric still life and approved copy | Derived, content-gated |
| `Input` | Labeled Headless UI field with helper and error slots | Derived, Headless UI |
| `InteractiveLink` | Text link with directional icon | Derived, Phosphor |
| `ItemsPreviewTemplate` | Compact bag items for drawer | Derived |
| `LoginTemplate` | Focused sign-in panel with recovery path | Derived |
| `LookbookStory` | One approved editorial story and macro image | Derived, content-gated |
| `Nav` | Floating chrome with accessible mobile dialog | Derived, Headless UI |
| `NewsletterSection` | Working approved signup or `null` | Derived, action-gated |
| `OrderCompletedTemplate` | Calm confirmation with next actions | Derived |
| `OrderDetailsTemplate` | Grouped receipt and fulfillment details | Derived |
| `OrderOverview` | Compact order cards with status text | Derived |
| `Overview` | Account summary without dashboard card clutter | Derived |
| `ProductTemplate` | Media gallery and sticky purchase panel | Derived, Embla and Radix |
| `ProfileBillingAddress` | Labeled editable address group | Derived, Headless UI |
| `ProfileEmail` | Labeled editable email group | Derived, Headless UI |
| `ProfileName` | Labeled editable name group | Derived, Headless UI |
| `ProfilePassword` | Password update with visibility control | Derived, Headless UI and Phosphor |
| `ProfilePhone` | Labeled editable phone group | Derived, Headless UI |
| `RadioIndicator` | High-contrast Headless UI radio state | Derived, Headless UI |
| `SkeletonCardDetails` | Product-card-shaped neutral skeleton | Derived, CSS |
| `SkeletonCartPage` | Cart-layout-shaped neutral skeleton | Derived, CSS |
| `SkeletonOrderConfirmed` | Confirmation-layout-shaped skeleton | Derived, CSS |
| `Spinner` | Small status-only indicator | Derived, CSS |
| `StoreTemplate` | Catalog heading, filters, sort, grid | Derived, Headless UI and Radix |
| `SubmitButton` | Full-width action with pending feedback | Derived, platform button |
| `TransferActions` | Clear accept and decline decision group | Derived, Headless UI confirmation |
| `TransferImage` | Aspect-ratio-safe order media | Derived, Next.js Image |
| `TransferRequestForm` | Focused request form with inline errors | Derived, Headless UI |

The implementation ledger must track `planned`, `in progress`, and `verified` separately from origin.
`derived` describes design origin and must never be used to imply that the slot is complete.

## 9. Required core decisions before implementation

The current home slots `Hero`, `BrandFeatures`, `CategoryGrid`, `LookbookStory`, and
`NewsletterSection` receive little or no merchant content. The current core metadata is also
apparel-specific. A production watch and ornament frontend cannot solve those gaps by fetching or
by inventing content.

Resolve these items before visual implementation:

1. Store identity seam. Pass approved store name, wordmark reference, and short descriptor to Nav,
   Hero, and Footer, or expose an equivalent core-owned presentation object.
2. Home composition data. Pass real categories, selected collections, approved editorial content,
   and working newsletter action state to the relevant slots.
3. Metadata ownership. Replace apparel-specific page metadata in core with store-configured title,
   description, and social image data.
4. Policy links. Do not render policy or about links until ADR-020 is accepted and core owns the
   routes.
5. Branding tokens. Confirm the CSS custom property names supplied per tenant. Lustre must consume
   those names rather than create a second tenant-branding channel.

Recommended shape: one core-owned, typed `StorefrontPresentation` object assembled from approved
store configuration and catalog data, then passed to presentation slots. This keeps fetching and
validation in core and prevents each new frontend from requesting another ad hoc prop.

If the team chooses not to make this core change, the storefront can still be implemented for one
fixed merchant. That merchant must supply all visible identity, copy, asset, and policy values before
the package is considered complete.

## 10. Responsive and accessibility acceptance

- Minimum pointer target is 44 by 44 CSS pixels for primary touch controls.
- Body text and controls meet WCAG 2.2 AA contrast. Primary reading text targets AAA where practical.
- All functions work with keyboard only and show a visible 2px focus ring with 2px offset.
- Icon-only controls have accessible names.
- Dialog focus is trapped while open and restored to the trigger on close.
- Errors are announced, connected to their fields, and never communicated by color alone.
- Product option state, sale state, and availability have text equivalents.
- At widths below 768px, asymmetric layouts collapse to one column except the two-column catalog
  grid.
- Large text does not obscure price, options, totals, or actions.
- Reduced motion and reduced transparency paths are tested, not inferred.
- Catalog browsing and product detail remain useful without JavaScript. Enhanced drawers and rails
  have link or document-flow fallbacks.

## 11. Performance budget

- LCP under 2.0 seconds, INP under 150ms, and CLS under 0.05 at p75, matching the storefront sitemap
  requirements.
- Lighthouse scores at least 95 for Performance, Accessibility, Best Practices, and SEO on product
  detail.
- Hero media is responsive, preloaded only when it is the LCP candidate, and never exceeds its display
  need.
- GSAP is isolated to client leaves and loaded only on pages that use it.
- Embla is used only for the product gallery and finite rails.
- No animated blur, full-page noise filter, custom cursor, WebGL, autoplay video, or autoplay carousel.
- Reserve media dimensions and use font fallback metrics to prevent layout shift.

## 12. Delivery sequence

### Phase 0: Contract and content decision

- Approve merchant identity and marketing-content inputs.
- Decide the metadata seam and ADR-020 policy-link behavior.
- Record any core deltas before frontend work.
- Obtain approved wordmark, product media, hero image, copy, and policy content.

Exit: no Lustre component depends on unsourced visible content or a frontend-owned fetch.

### Phase 1: Package and tokens

- Create `frontend-lustre` from an empty package, not from another frontend directory.
- Add independent package metadata with zero fork debt.
- Add the semantic token layer, font assets, theme lock, focus system, and base layout helpers.
- Add a Lustre TypeScript path config and dev, build, start, and typecheck commands.
- Run dependency review for Embla and Phosphor before installation.

Exit: the empty frontend binding resolves to Lustre and the boundary, purity, pin, resolution, switch,
and not-forked checks pass.

### Phase 2: Commerce primitives and chrome

- Implement input, checkbox, radio, submit, divider, error, spinner, links, country select, Nav,
  Footer, cart mismatch, and free-shipping state.
- Verify Headless UI focus, keyboard, portal, Escape, and outside-click behavior.

Exit: primitives pass interaction tests in both color modes, reduced motion, and 200 percent zoom.

### Phase 3: Catalog and product

- Implement Store, Category, Collection, product cards, gallery, product detail, product actions shell,
  related rail, and skeletons.
- Add Embla only after dependency approval.
- Verify empty, loading, unavailable, sale, long-title, single-image, and many-image cases.

Exit: product selection and cart behavior remain core-controlled, and catalog/product E2E tests pass.

### Phase 4: Cart, account, orders, and checkout skin

- Implement cart page and drawer, account shell and profile forms, order detail and confirmation,
  address book, and transfer surfaces.
- Apply the token seam to core checkout without moving fields or altering flow.
- Test errors, pending states, cancellation, focus restoration, and mobile sticky actions.

Exit: checkout parity remains green and the authorization and ownership behavior is unchanged.

### Phase 5: Home composition and GSAP

- Implement the six home layouts against approved content.
- Add the single pinned material story and restrained entry sequence.
- Provide static reduced-motion and reduced-transparency variants.
- Do not ship any empty marketing slot with placeholder claims.

Exit: every animation has a named purpose, cleanup test, reduced-motion path, and no scroll listener.

### Phase 6: Visual QA and release

- Capture desktop, tablet, and mobile screenshots for every route family.
- Test light and dark mode, keyboard, screen reader landmarks, zoom, reduced motion, slow network,
  empty catalog, and long localized copy.
- Compare structure against both existing frontends and remove any accidental match.
- Run Lighthouse and the complete storefront gate suite.

Exit: all 45 slots are verified, fork debt is zero, no `@medusajs/ui` component renders, and no
unsourced visible figure or claim remains.

## 13. Required gates

Run and quote real output before claiming implementation is complete:

```bash
npm --prefix apps/storefront run typecheck:lustre
npm --prefix apps/storefront run build:lustre
./scripts/check-storefront-boundaries.sh
./scripts/check-frontend-purity.sh
./scripts/check-frontend-pin.sh
./scripts/check-frontend-resolution.sh
./scripts/check-frontend-switch.sh
./scripts/check-frontend-not-forked.sh
npm --prefix apps/storefront run test
npm --prefix apps/storefront run test:e2e
npm --prefix apps/storefront run test:lighthouse
python scripts/doc-sync-workflow.py check
```

The implementation must also include a per-slot ledger, browser screenshots, and a component-source
ledger proving where each behavior-heavy primitive came from.

## 14. Definition of done

- The new package connects to the same backend through storefront-core.
- All 45 contract slots exist and have original Lustre structure.
- No route, fetch, money calculation, checkout flow, or merchant secret moves into the frontend.
- Store identity and marketing content are approved and sourced.
- Error, empty, pending, disabled, unavailable, and success states are designed and tested.
- Footer content contains only real data and valid core-owned routes.
- Apple-informed hierarchy, control size, accessibility, and motion restraint are visible in every
  route family.
- Headless UI, Radix, GSAP, Embla, Phosphor, Next.js primitives, and native HTML are used only for
  the responsibilities named in this plan.
- The design is recognizably different from the dark, flat, gold-accented Matrix frontend and the
  Medusa-derived default frontend.
- Performance, accessibility, SEO, no-JavaScript browsing, and frontend isolation gates pass.
