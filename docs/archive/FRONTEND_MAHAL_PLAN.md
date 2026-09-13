# Frontend Mahal: Image-to-Code Storefront Plan

**Status:** Implemented and verified  
**Target:** `apps/storefront/packages/frontend-mahal`  
**Development harness:** Vite  
**Production integration:** Stheno `@stheno/storefront-core` frontend contract  
**References:** five user-supplied desktop screenshots dated 2026-09-08

`mahal` is an internal frontend key. The Vite fixture harness uses approved Vendryx identity assets;
production receives the same identity through Stheno's merchant-branding data. The Mahal and Le Reve
marks are replaced, while their photographed page composition remains the reconstruction target.
Prices, availability, shipping terms, policies, and contact channels still come from approved data.

## 1. Design read

Reading this as: an image-led South Asian fashion marketplace for fashion-conscious shoppers, with a
bright editorial retail language, disciplined catalog density, square product media, and restrained
interaction.

This is an independent frontend reconstruction. It is not a skin of `frontend-default`,
`frontend-matrix`, or `frontend-lustre`.

| Dial | Value | Reason |
|---|---:|---|
| `DESIGN_VARIANCE` | 5 | The references use strong campaign composition but a regular commerce grid |
| `MOTION_INTENSITY` | 3 | Sticky chrome, drawers, menus, hover swaps, and direct feedback only |
| `VISUAL_DENSITY` | 6 | Product lists and home merchandising are intentionally information-rich |
| `ART_DIRECTION` | 7 | Photography and seasonal edits carry the identity |
| `IMPLEMENTATION_CLARITY` | 10 | Every source and derived reference must expose buildable hierarchy and spacing |
| `IMAGE_USAGE_PRIORITY` | 10 | Real campaign and product imagery is the primary design material |

### Visual direction

- Theme lock: bright light mode across the public storefront.
- Interface palette, borders, shadows, and neutral wells are sampled from the supplied pixels. Only
  identity colors that belong to the replaced brand may resolve through Vendryx branding tokens.
- Shape rule: square or 2px surfaces; pills only for compact status and utility controls where the
  reference uses them. Product cards are not rounded containers.
- Typography: clean grotesk for commerce copy and a merchant-supplied display or wordmark treatment
  for identity. No substitute logo is synthesized.
- Image system: fixed portrait product ratios, wide campaign banners, controlled editorial crops,
  and no invented product imagery in production data.

## 2. Ladder decision and architecture

The ladder stops at **rung 2** for application structure: Stheno already has the storefront route,
data, checkout, localization, SEO, and 45-slot presentation contract. The existing Vite apps provide
the repository's Vite and Tailwind setup pattern. Neither an existing frontend's JSX nor its DOM will
be reused.

The implementation uses one visual system with two consumers:

```text
shared CSS, tokens, and media rules
        |                         |
        | deterministic fixtures | real props and core children
        v                         v
Vite visual harness         Stheno storefront-core adapters
fast screen matching        production routes/data/SEO/checkout
```

The Vite harness is a local and CI visual-reconstruction surface. It renders deterministic fixtures
for each screen and state, and it is not the production router or data client. Production remains in
the existing Next.js host because moving it to a standalone Vite SPA would duplicate route ownership,
Store API orchestration, cookies, checkout, metadata, structured data, and no-JavaScript behavior.

If a separately deployed Vite SPA is required, implementation pauses first for a new ADR and explicit
scope covering routing, SSR or prerendering, SEO, Store API access, session cookies, checkout PCI
boundaries, and parity with the current host.

## 3. Reference evidence map

| Supplied reference | What it governs |
|---|---|
| Jewelry mega-menu screenshot, 1178 x 812 | Announcement bar, centered identity, utility icons, primary nav, mega-menu, campaign tiles |
| Black Wallflower product page, 2476 x 6806 | Product gallery, thumbnail rail, sticky purchase panel, accordion, related rail, newsletter, trust strip, footer |
| Kurta Sets men's listing, 2476 x 16384 | Breadcrumb and title, featured products, sticky catalog chrome, sidebar filters, dense three-column grid, pagination |
| Clothing home, 2476 x 16384 | Campaign hero, designer spotlight, style and occasion edits, bridal story, accessories, lower navigation |
| Clothing listing, 2476 x 12760 | Four-column product rhythm, neutral media wells, minimal tile metadata, long catalog continuation |

The supplied screenshots are the canonical visual specification for every covered screen. The
implementation mirrors their visible hierarchy point by point: landmark order, wrappers, columns,
alignment, dimensions, gaps, typography, media crops, borders, controls, sticky layers, and section
sequence. It does not reinterpret, modernize, simplify, or improve them.

A bitmap cannot reveal the original website's private source tree, element names, scripts, or
off-screen responsive DOM. Therefore the deliverable is an original DOM that reproduces the DOM
structure visible in the pixels, not an unsupported claim that its hidden node tree is byte-for-byte
identical to the source website.

### Fidelity contract

- Every visible object in a supplied screenshot receives a corresponding element or image asset.
- Elements stay in the same visual and reading order, including repeated merchandising sections.
- Reference widths, heights, gutters, grid tracks, whitespace, and image object positions are measured
  from the source instead of selected by taste.
- Branding substitution is limited to logo, wordmark, favicon, document identity, copyright identity,
  and genuine brand-color tokens. It does not authorize a layout or component redesign.
- The supplied photography is used where the user has provided it. Store API media replaces fixture
  imagery only in production and must preserve the same frame and crop behavior.
- Missing mobile and transactional screens are derived conservatively from the measured desktop DOM;
  fresh generated references may resolve only those missing surfaces.
- Visual QA uses same-size captures, 50 percent overlay inspection, and automated screenshot diffs.
  Geometry and content-order differences have zero tolerance. Font rasterization and photographic
  encoding differences are reviewed separately from layout mismatch.

## 4. Source-first image-to-code execution set

The user's clarification overrides concept generation for the five supplied screens. Those bitmaps
are analyzed directly and remain the source of truth. They are never replaced with AI-generated
reinterpretations or cropped into altered compositions.

Create high-resolution analysis records for:

1. Desktop announcement bar, header, primary nav, and jewelry mega-menu.
2. Complete home page, with a measurement record for every merchandising section.
3. Complete men's catalog page, including featured row, sticky chrome, filters, product repetitions,
   pagination, and empty image positions visible in the source.
4. Complete alternate clothing catalog, including its four-column rhythm and footer.
5. Complete product detail, including its captured sticky-header position, media rail, purchase panel,
   related products, newsletter, trust strip, and footer.

Fresh standalone images are generated only for surfaces absent from the provided evidence:

6. Mobile header and full-height navigation.
7. Mobile catalog with filter and sort drawers.
8. Product tile states not visible in the references: hover image, unavailable, loading, and error.
9. Mobile product detail with sticky purchase action.
10. Cart drawer, cart page, and their empty/error states.
11. Checkout presentation surfaces.
12. Sign-in, account, order detail, transfer, and confirmation surfaces, each rendered separately when
    a combined image would make text or controls unreadable.

Each source or derived image gets an extraction record for text hierarchy, DOM regions, grid,
gutters, spacing, media ratios, colors, borders, controls, states, and responsive collapse before code
is written. A derived image may fill missing evidence but may not change screenshot-covered geometry.

## 5. Screen and DOM matrix

### Global chrome

- Slim announcement region rendered only from sourced content.
- Two-level desktop header: centered merchant identity with account, search, bag, and region tools;
  one-line category navigation below.
- Keyboard-safe mega-menu with category rail plus two or more real campaign links when data exists.
- Mobile full-height menu, search surface, cart drawer, region selector, and focus restoration.
- Sticky behavior changes position or border, not element order, so layout does not jump.

### Home

- Full-width campaign hero with real art, short approved copy, and one primary action.
- Designer spotlight using one narrative feature and a finite product strip.
- Ready-to-ship and occasion edits using real collection/category links.
- Shop-by-style and shop-by-occasion grids with mixed but repeatable media ratios.
- Bridal and accessory editorial banners only when approved content and valid routes exist.
- Newsletter only when a working action exists.

### Store, category, collection, and search

- Breadcrumb, concise title, optional sourced description, and optional finite featured row.
- Desktop filter rail with accessible disclosure groups; mobile filter and sort drawers.
- Three columns beside the filter rail or four columns without it; three on tablet and two on mobile.
- Tiles expose only image, brand or collection when sourced, title, core-formatted price, compare-at
  state, favorite affordance when functional, and valid quick-add behavior.
- Loading, empty, no-results, error, long-title, one-image, and unavailable variants preserve layout.

### Product detail

- Desktop thumbnail rail and large portrait media gallery beside a sticky purchase column.
- Title, brand when sourced, formatted price, option states, size guide only when routed, purchase
  action, and factual disclosures only.
- Description and material/care sections use accessible accordions.
- Related products use a finite responsive rail without autoplay.
- Mobile uses a swipeable gallery and sticky purchase action with reduced-transparency fallback.

### Cart, checkout, account, and orders

- Cart drawer and cart page keep editing, totals, errors, pending state, and checkout action explicit.
- Checkout remains core-owned; Mahal supplies tokens and allowed presentation surfaces only.
- Auth, profile, addresses, history, order detail, transfer, and confirmation derive from the same
  sharp editorial language rather than borrowing another frontend's markup.
- All 45 entries in `FrontendContract` receive original Mahal DOM and a `ported`, `derived`, or
  `forked` ledger entry. `forked` is a release failure; expected total is 45 `derived` and zero
  `forked` unless a supplied reference directly governs a slot.

## 6. Content and truth constraints

- Copy the five supplied files into a frontend-owned reference directory only after preserving their
  provenance. Production product media still comes from Store API props.
- No hardcoded stock count, delivery time, warranty, review count, discount, support availability,
  shipping promise, payment mark, or social contact.
- BDT and all other money stay core-formatted. The frontend performs no decimal or locale math.
- WhatsApp and chat controls render only when the core contract provides a configured destination and
  permitted action. Otherwise they are omitted.
- Existing URL labels and localized route shapes stay unchanged. New editorial routes require a
  core-owned route and recorded decision before use.

## 7. Implementation phases

### Phase 0: approve architecture and content seams

- Confirm that Vite is the development/visual harness and Stheno remains the production host.
- Confirm internal key `mahal`, approved Vendryx assets, and the identity-only substitution boundary.
- Inventory which marketing copy, campaign images, chat endpoints, and policy routes are real.

Exit: no production component requires a fabricated claim or frontend-owned fetch.

### Phase 1: measure supplied references and derive missing screens

- Analyze the five complete supplied images at original resolution and create per-section measurement
  overlays.
- Generate only the missing responsive, state, and transactional references named in section 4.
- Record per-screen DOM regions, measurements, sticky layers, and responsive rules.
- Freeze tokens, type scale, shape rule, media ratios, and interaction states.

Exit: every planned surface has a readable source image or an explicit derivation rule.

### Phase 2: package and Vite harness

- Create `frontend-mahal` from an empty directory.
- Add `stheno.independent: true` with no fork debt.
- Add a package-local Vite fixture harness that imports the same components used by production.
- Reuse repository Vite, React, CSS, Headless UI, Phosphor, and test capabilities where already
  declared; add no dependency until the AGENTS.md dependency review is recorded.

Exit: fixture screens build in Vite and the empty production binding resolves without touching core
route or data ownership.

### Phase 3: tokens, primitives, and chrome

- Build CSS tokens, typography, focus, buttons, fields, alerts, skeletons, header, mega-menu, mobile
  menu, cart drawer, and footer.
- Validate keyboard, focus trap/restoration, Escape, outside click, zoom, reduced motion, and reduced
  transparency.

### Phase 4: home, catalog, and product detail

- Implement the reference-led public commerce screens in visual priority order.
- Compare Vite fixture captures to approved references at 1440 x 900, 1280 x 800, 768 x 1024, and
  390 x 844 after each screen family.
- Use Playwright screenshot assertions and DOM landmark assertions; tune tokens and layout rather
  than inserting viewport-specific markup duplicates.

### Phase 5: transactional and long-tail contract surfaces

- Implement cart, account, order, transfer, confirmation, error, empty, loading, and pending states.
- Skin checkout only through the existing seam.
- Complete the 45-slot ledger and prove zero frontend-default structural reuse.

### Phase 6: production integration and release QA

- Add the Mahal frontend binding, typecheck config, development/build commands, and conformance tests.
- Run real API-backed route tests without changing Store API response shapes.
- Capture desktop, tablet, and mobile route-family evidence.
- Audit page copy for sourced facts, accessibility, and the image-to-code pre-flight checklist.

## 8. Required gates

At minimum, run and quote real output before completion:

```bash
npm --prefix apps/storefront run typecheck:mahal
npm --prefix apps/storefront run build:mahal
npm --prefix apps/storefront run test
npm --prefix apps/storefront run test:e2e
npm --prefix apps/storefront run test:lighthouse
./scripts/check-storefront-boundaries.sh
./scripts/check-frontend-purity.sh
./scripts/check-frontend-pin.sh
./scripts/check-frontend-resolution.sh
./scripts/check-frontend-switch.sh
./scripts/check-frontend-not-forked.sh
./scripts/check-frontend-conformance.sh
python scripts/doc-sync-workflow.py check
```

The Vite harness also needs its own typecheck, production build, fixture route smoke test, and
Playwright visual suite. Lighthouse targets remain at least 95 in each category on product detail,
with LCP under 2.0 seconds, INP under 150 milliseconds, CLS under 0.05, and usable no-JavaScript
catalog/product pages through the production host.

## 9. Definition of done

- The Vite harness and production frontend render the same Mahal visual hierarchy, tokens, media,
  and responsive rules; production adapters replace fixture values with core props.
- Home, catalog, product detail, cart, checkout skin, account, order, and state screens are covered.
- All 45 contract slots use original DOM authored for Mahal and have zero fork debt.
- Screenshot-covered screens reproduce every visible region in the same order and measured geometry.
- Same-size visual captures pass overlay review and automated screenshot comparison; only documented
  font-rasterization or image-encoding noise may remain.
- No frontend owns routes, fetches, money calculation, cookies, checkout flow, or payment SDK logic.
- Vendryx replaces source-site identity without changing the photographed layout system.
- No unsourced fact, policy, contact channel, product state, or brand asset is rendered.
- Keyboard, focus, zoom, reduced motion, reduced transparency, empty, loading, error, pending, and
  unavailable paths are verified.
- The storefront gates, Vite gates, visual tests, Lighthouse checks, and image-to-code pre-flight all
  pass with recorded output.
