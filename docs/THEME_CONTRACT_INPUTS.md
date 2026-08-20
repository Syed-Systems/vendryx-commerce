# Theme Contract Inputs — what theme B needs

**Status:** Draft, **awaiting designer sign-off** · **Date:** 2026-08-20 · **Task:** P25-02 ·
**Governed by:** [ADR-013](DECISIONS.md#adr-013)

This document exists to be argued with, and it must be argued with **before**
[P25-05](PHASE3_TASKS.md) freezes the theme contract.

[PHASE3_PLAN.md §P27](PHASE3_PLAN.md) states the risk this list answers:

> A second theme built by the same people who built core will not exercise the contract honestly.
> They will unconsciously avoid the awkward parts.

**Read that warning before reading the inventory.** The list below was derived by the authors of
core from the 143 components in `apps/storefront/src/modules`. That is precisely the bias P25-02
was written to defeat. It is a *starting position for the designer to cut into*, not a
requirements document. Every row a designer deletes, merges, or splits is the task working; a
designer who reads it and agrees with all of it has not read it.

---

## 1. How to use this document

| Section | Who owns it |
|---|---|
| §3 Theme surface | The designer. Cut it, rename it, restructure it |
| §4 Core-owned | Nobody. Not negotiable — [ADR-013](DECISIONS.md#adr-013) decisions 2 and 4 |
| §5 Skinnable core | The designer owns the *look*; core owns the flow |
| §6 Open questions | **Must be answered before P25-05.** These are the contract's real shape |
| §7 Known pressure | Read before freezing. Three of these are already in motion |

---

## 2. The line, restated

[ADR-013](DECISIONS.md#adr-013) decision 4: *a theme supplies components against a typed contract
and receives props. It does not fetch, and it does not route.*

That single sentence decides every ambiguous row below. If a component in §3 turns out to need a
`fetch`, a server action, or knowledge of a URL, it is **not a theme component** and belongs in
core with a token-styled surface instead. `check-theme-purity.sh` (P25-10) enforces this
mechanically, so getting it wrong here costs a phase, not a review comment.

---

## 3. Theme surface — what a theme must supply

Grouped by shopper journey, not by the current folder layout, deliberately: the folder layout is
theme-default's opinion and inheriting it is the failure mode this task exists to prevent.

### 3.1 Chrome — present on every page

| Surface | Renders | Notes for the designer |
|---|---|---|
| `Header` | Store name/logo, navigation, cart affordance, region + language pickers | theme-default splits this into `nav`, `side-menu`, `cart-button`, `cart-dropdown`, `country-select`, `language-select`. **That split is not required.** A theme may ship one component |
| `Footer` | Navigation, legal, store identity | |
| `CartMismatchBanner` | Warning when the signed-in customer's cart differs from the session cart | Rare, easy to forget, breaks trust when missing |
| `FreeShippingNudge` | Progress toward a free-shipping threshold | Currently rendered in cart and checkout only |

### 3.2 Home

| Surface | Renders |
|---|---|
| `Hero` | Above-the-fold marketing slot |
| `CategoryGrid` | Category entry points |
| `FeaturedProducts` / `ProductRail` | Horizontally grouped product collections |
| `LookbookStory`, `BrandFeatures`, `NewsletterSection` | Marketing sections |

**Open to the designer:** home is the surface with the least commerce logic and the most design
variance. It is a strong candidate for being a *composition slot* rather than a fixed component
list — see §6.1.

### 3.3 Catalog — store, category, collection

| Surface | Renders | Notes |
|---|---|---|
| `ProductListing` | The grid/list of products for store, category and collection pages | theme-default uses three near-identical templates. One contract entry, three call sites, is likely correct |
| `ProductCard` | One product in a listing: thumbnail, title, price | The single most-rendered theme component |
| `RefinementList` / `SortControl` | Filtering and sort | Core owns the query params; the theme owns the control |
| `Pagination` | Page navigation | Core owns the URL shape |
| `CategoryHeader` / `CollectionHeader` | Title, description, breadcrumbs | |

### 3.4 Product detail

| Surface | Renders | Notes |
|---|---|---|
| `ProductGallery` | Images | |
| `ProductInfo` | Title, subtitle, description, collection link | |
| `ProductOptions` | Variant option selection | **Receives selection state from core.** It does not own which variant is selected |
| `ProductPrice` | Price, including sale/compare-at presentation | Money formatting comes from core — see §6.3 |
| `AddToCart` | The button and its pending/disabled/out-of-stock states | Behaviour is core's; every visual state is the theme's |
| `ProductTabs` | Product details, shipping info, returns | |
| `RelatedProducts` | | |
| `MobileProductActions` | Sticky mobile purchase bar | theme-default ships this; a theme may legitimately not want it. **Optional export** |

### 3.5 Cart

| Surface | Renders |
|---|---|
| `CartTemplate` | The cart page layout |
| `CartLineItem` | One line: thumbnail, title, options, quantity control, unit price, line total, remove |
| `CartSummary` | Totals block and the checkout CTA |
| `CartPreview` | The dropdown/drawer contents |
| `EmptyCart` | |
| `SignInPrompt` | Guest prompt shown in cart |

### 3.6 Account

| Surface | Renders |
|---|---|
| `LoginForm`, `RegisterForm` | Credentials, validation display |
| `AccountLayout`, `AccountNav` | Account shell |
| `AccountOverview` | Dashboard |
| `OrderCard`, `OrderHistory` | Past orders |
| `AddressBook`, `AddressCard`, `AddressForm` | Saved addresses |
| `ProfileField` | theme-default ships six near-identical variants (name, email, phone, password, billing address, account info). **One generic component is probably right** |
| `TransferRequestForm` | Order transfer |

### 3.7 Order confirmation and detail

| Surface | Renders | Notes |
|---|---|---|
| `OrderConfirmedTemplate` | Post-purchase page | The highest-trust page in the journey |
| `OrderDetailsTemplate` | Historical order | |
| `OrderItems` / `OrderLineItem` | | |
| `OrderSummary`, `PaymentDetails`, `ShippingDetails`, `OrderHelp` | | |
| `TransferActions` | Accept/decline order transfer | |

### 3.8 Primitives

theme-default ships 32 components under `common/` and 15 skeletons. They are the theme's own
internals, **not contract entries** — with three exceptions that core renders directly and
therefore must be able to name:

| Surface | Why core needs it |
|---|---|
| `LoadingSkeleton` | Core owns the `<Suspense>` boundaries. It must be able to render *something* while data resolves |
| `ErrorDisplay` | Core owns the error boundaries |
| `Money` | Core formats; the theme presents. See §6.3 |

Everything else under `common/` and every other skeleton is theme-internal. **A theme is free to
ship zero of them.** Do not let 47 internal components leak into the contract as required exports
— that is the single most likely way this contract ends up unimplementable by anyone but us.

---

## 4. Core-owned — a theme never supplies these

Not a design position. [ADR-013](DECISIONS.md#adr-013) decision 4, and
`check-storefront-boundaries.sh` (P25-09) fails the build on a violation.

| Concern | Where |
|---|---|
| Every route and URL shape | `app/` tree → core in P25-04 |
| Region resolution and the `[countryCode]` segment | `middleware.ts` → core in P25-03 |
| All data fetching and server actions | `src/lib/data/*` (15 files) → core in P25-03 |
| Cart, customer and region context | `src/lib/context` → core |
| SEO: metadata, sitemap, OG images | core |
| `LocalizedClientLink` | Routing. A theme uses it; it never reimplements it |
| Payment SDK wrappers (`stripe-wrapper`) | Compliance surface |

---

## 5. Skinnable core — checkout

[ADR-013](DECISIONS.md#adr-013) decision 2: **skinnable, never rewritable.** The 18 components in
`src/modules/checkout` stay in core (P25-08). A Pro theme restyles them; it cannot alter the flow,
the step order, the validation, or the script manifest.

What a theme may change: type, colour, spacing, button treatment, field styling, and the visual
arrangement *within* a step.

What it may not change: that there are steps, which steps, their order, what validates when, and
what loads on the page. `check-checkout-scripts.sh` and the P25-08 `checkout-ownership` test both
fail on an attempt.

**Designer input needed:** the styling seam for checkout is not yet designed. If it turns out a
theme needs to move a field between steps to achieve its design, that is an
[ADR-013](DECISIONS.md#adr-013) reversal condition and must be raised now, not in P27-06.

---

## 6. Open questions — answer before P25-05

These are the contract. Everything above is inventory.

### 6.1 Is home a component list or a slot?

A fixed list of seven home components bakes theme-default's marketing structure into every future
theme. A single `HomePage` export with core supplying data is more honest and less useful.
**Designer decides.**

### 6.2 How much layout does core impose?

Does core render a page shell (header, `<main>`, footer) into which theme components drop, or does
the theme own the entire page? The first is safer and constrains design; the second is free and
means every theme reimplements the shell. **This is the highest-consequence question in this
document** — it decides whether theme B is a restyle or a rebuild.

### 6.3 Who formats money?

Currency formatting is commerce logic with a presentation surface. Proposal: core formats and
passes a formatted string *plus* the raw decimal; the theme chooses which to render. This matters
more than it sounds — [ADR-006](DECISIONS.md#adr-006) money is decimal, and a theme doing its own
`toFixed(2)` on a BDT total is exactly the silent wrongness Phase 3 is full of. **Confirm the
proposal or replace it.**

### 6.4 Does a theme own its own responsive breakpoints?

theme-default uses `@medusajs/ui-preset`'s scale. A theme with its own design system will not.
If breakpoints are theme-owned, core cannot render anything responsive — including the checkout
shell in §5.

### 6.5 What is optional?

`MobileProductActions`, `FreeShippingNudge`, `CartMismatchBanner`, `SignInPrompt`,
`RelatedProducts`, `ProductTabs` — each is plausibly absent from a given design. Every one made
*required* is a theme author shipping an empty component to satisfy a type. Every one made
*optional* is a surface that can silently vanish. **Draw this line explicitly per component; do
not let it default.**

### 6.6 Does the theme supply icons?

theme-default ships 17 icons, three of which are payment-brand marks (`ideal`, `bancontact`,
`paypal`) rendered by checkout — which is core. Payment-method iconography is arguably core's, and
it grows with every [P22](PHASE3_TASKS.md) gateway: bKash, Nagad and SSLCommerz all have brand
marks with usage rules attached.

---

## 7. Known pressure on this contract

Three things will push on the contract **while it is being written**. All three are known now, and
that is the only reason they are cheap.

1. **Branding tokens (P26) land after the contract is frozen.** Theme components must consume
   colour, font and store identity as CSS custom properties resolved per request — *not* as props,
   and never as build-time Tailwind constants. A contract that passes branding as props forces a
   re-render per tenant and breaks P26-05. **Design every §3 surface against `var(--...)` from the
   start.**

2. **Subscriptions are in flight and are a core feature, not a theme.**
   [ADR-013](DECISIONS.md#adr-013) says so explicitly in its consequences: *"A merchant wanting a
   genuinely different purchase flow — subscription, quote-to-order, wholesale approval — is not
   served by any of the three tiers... answering it with a theme would be the failure this record
   exists to prevent."* Work on subscriptions is happening concurrently with this phase. If it
   adds a purchase-flow surface — a plan selector on the product page, a recurring-total line in
   the cart, a subscription section in the account area — **those are §3 rows that do not exist in
   this draft**, and freezing the contract without them means every theme needs a core change to
   sell a subscription. Reconcile the two before P25-05.

3. **COD changes the order surface (P24).** Bangladesh COD orders carry three distinct amounts —
   ordered, collected, remitted. `OrderSummary` and `PaymentDetails` in §3.7 currently assume one.

---

## 8. What happens next

| Step | Owner | Blocks |
|---|---|---|
| Designer reviews §3 and answers §6 | Designer | P25-05 |
| §7.2 reconciled with the subscriptions work | Both tracks | P25-05 |
| Contract typed from the agreed list | Core | P25-06 |
| Theme B built from §3 without core changes | Theme author | P27-07 |

P27-07 budgets **two** core changes for theme B. More than two means this document was drawn
wrong, and `check-core-delta-budget.sh` will say so.
