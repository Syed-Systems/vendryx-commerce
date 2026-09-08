# frontend-lustre — slot ledger

**Status:** `ADAPT` Complete · **Date:** 2026-09-08 ·
**Required by:** [AGENTS.md §12](../AGENTS.md#12-frontends-adapting-a-design)

`ADAPT` requires every contract slot to be labelled. This is that label, for all 45 slots in
[`contract.ts`](../apps/storefront/packages/storefront-core/src/frontend/contract.ts).

| Label | Meaning | Count |
|---|---|---|
| `ported` | Rebuilt directly from primary design specifications | 0 |
| `derived` | Drawn from Lustre cold luxury design rules (`docs/FRONTEND_LUSTRE_PLAN.md`) — never from frontend-default | 45 |
| `forked` | Still frontend-default's markup | 0 |

---

## 1. Compliance Ledger (All 45 Slots)

Every slot below is cleanly derived with custom DOM hierarchy, cold luxury styling tokens (sapphire `#12558A`/`#6EA6D1`, 16px surface radius, frosted glass), zero `@medusajs/ui` imports, and strictly no unsourced figures.

| # | Contract Slot | Implementation Path | Status | Design Derivation Note |
|---|---|---|---|---|
| 1 | `AccountLayout` | `@frontend/account/templates/account-layout` | `derived` | Framed luxury client portal with sticky nav rail and concierge contact |
| 2 | `AddressBook` | `@frontend/account/components/address-book` | `derived` | Grid with dashed new-address trigger and custom edit modal |
| 3 | `BrandFeatures` | `@frontend/home/components/brand-features` | `derived` | Three Roman-numeral design tenets (Purity, Material, Permanence) |
| 4 | `CartMismatchBanner` | `@frontend/layout/components/cart-mismatch-banner` | `derived` | Frosted warning ribbon with currency harmonization button |
| 5 | `CartTemplate` | `@frontend/cart/templates` | `derived` | Editorial bag view with 2-column layout and sticky summary |
| 6 | `CartTotals` | `@frontend/common/components/cart-totals` | `derived` | Clean breakdown with `convertToLocale` and subtle hairline separators |
| 7 | `CategoryGrid` | `@frontend/home/components/category-grid` | `derived` | Architectural 3-column department grid with gradient scrims |
| 8 | `CategoryTemplate` | `@frontend/categories/templates` | `derived` | High-contrast catalog view with pill filters and parent breadcrumb |
| 9 | `Checkbox` | `@frontend/common/components/checkbox` | `derived` | Accessible custom checkbox with sapphire fill and check glyph |
| 10 | `ChevronDown` | `@frontend/common/icons/chevron-down` | `derived` | Crisp inline SVG geometric chevron |
| 11 | `CollectionTemplate` | `@frontend/collections/templates` | `derived` | Capsule exhibition view with refined serif title and product grid |
| 12 | `CountrySelect` | `@frontend/common/components/country-select` | `derived` | Pill-shaped native select with custom chevron indicator |
| 13 | `DiscountCode` | `@frontend/common/components/discount-code` | `derived` | Expandable voucher input with clean chip tag upon application |
| 14 | `Divider` | `@frontend/common/components/divider` | `derived` | Hairline separator matching cold luxury border token |
| 15 | `ErrorMessage` | `@frontend/common/components/error-message` | `derived` | Rounded alert badge with semantic error state |
| 16 | `Footer` | `@frontend/layout/templates/footer` | `derived` | 4-column structured directory with localized country/currency select |
| 17 | `FreeShippingPriceNudge` | `@frontend/shipping/components/free-shipping-price-nudge` | `derived` | Sapphire progress meter and threshold calculator |
| 18 | `Hero` | `@frontend/home/components/hero` | `derived` | 2-column hero with frosted luxury visual plate and sapphire ambient glow |
| 19 | `Input` | `@frontend/common/components/input` | `derived` | Pill-curved text input with focus ring and password reveal toggle |
| 20 | `InteractiveLink` | `@frontend/common/components/interactive-link` | `derived` | Micro-animated link with sliding arrow indicator |
| 21 | `ItemsPreviewTemplate` | `@frontend/cart/templates/preview` | `derived` | Streamlined item table for fast checkout drawer review |
| 22 | `LoginTemplate` | `@frontend/account/templates/login-template` | `derived` | Frosted authentication and registration card with tab switching |
| 23 | `LookbookStory` | `@frontend/home/components/lookbook-story` | `derived` | Dual visual gallery paired with qualitative atelier standard |
| 24 | `Nav` | `@frontend/layout/templates/nav` | `derived` | 72px floating frosted glass navigation bar with pill badges |
| 25 | `NewsletterSection` | `@frontend/home/components/newsletter-section` | `derived` | Private atelier register card with pill input and submit button |
| 26 | `OrderCompletedTemplate` | `@frontend/order/templates/order-completed-template` | `derived` | Confirmation screen with check icon, receipt breakdown, and next steps |
| 27 | `OrderDetailsTemplate` | `@frontend/order/templates/order-details-template` | `derived` | Comprehensive historical order record with return-to-overview link |
| 28 | `OrderOverview` | `@frontend/account/components/order-overview` | `derived` | Order transaction list with empty state and explore catalog CTA |
| 29 | `Overview` | `@frontend/account/components/overview` | `derived` | Client dashboard with completion percentage and recent orders rail |
| 30 | `ProductTemplate` | `@frontend/products/templates` | `derived` | Dual-column product detail showcase with sticky actions and accordion specs |
| 31 | `ProfileBillingAddress` | `@frontend/account/components/profile-billing-address` | `derived` | Inline address editor with action state and country selector |
| 32 | `ProfileEmail` | `@frontend/account/components/profile-email` | `derived` | Verified email address display and edit modal |
| 33 | `ProfileName` | `@frontend/account/components/profile-name` | `derived` | Customer name display and modification drawer |
| 34 | `ProfilePassword` | `@frontend/account/components/profile-password` | `derived` | Secure password change form with security masking |
| 35 | `ProfilePhone` | `@frontend/account/components/profile-phone` | `derived` | Telephone contact management form |
| 36 | `RadioIndicator` | `@frontend/common/components/radio` | `derived` | Sapphire-accented radio indicator for shipping and payment options |
| 37 | `SkeletonCardDetails` | `@frontend/skeletons/components/skeleton-card-details` | `derived` | Smooth shimmering placeholder for credit card input field |
| 38 | `SkeletonCartPage` | `@frontend/skeletons/templates/skeleton-cart-page` | `derived` | Two-column skeleton layout for cart review |
| 39 | `SkeletonOrderConfirmed` | `@frontend/skeletons/templates/skeleton-order-confirmed` | `derived` | Post-checkout receipt skeleton |
| 40 | `Spinner` | `@frontend/common/icons/spinner` | `derived` | Accessible SVG animation indicator with sapphire accents |
| 41 | `StoreTemplate` | `@frontend/store/templates` | `derived` | Catalog layout with search refinement, sorting chips, and paginated grid |
| 42 | `SubmitButton` | `@frontend/common/components/submit-button` | `derived` | Pill button with loading spinner state and sapphire hover animation |
| 43 | `TransferActions` | `@frontend/order/components/transfer-actions` | `derived` | Action buttons to accept or decline order custody transfer |
| 44 | `TransferImage` | `@frontend/order/components/transfer-image` | `derived` | Custom vector illustration of order reconciliation |
| 45 | `TransferRequestForm` | `@frontend/account/components/transfer-request-form` | `derived` | Pill form allowing clients to claim guest orders |

---

## 2. Supporting Primitives & Subcomponents

Beyond the 45 contract slots, all supporting primitives were derived from scratch without copying frontend-default:
- `layout/components/cart-dropdown` (Slide-out drawer with Escape, click-outside, and scroll-lock)
- `layout/components/side-menu` (Accessible slide-in mobile navigation)
- `products/components/product-preview` (Curated product cards with price formatting)
- `products/components/image-gallery` (Native CSS scroll snap gallery)
- `products/components/product-actions` (Variant option pills and add-to-bag CTA)
- `products/components/product-tabs` (Radix Accordion tabs with sourced specifications)
- `order-now` & `social-checkout` (Instant COD checkout and direct messaging order draft)
- Curated SVG icon library (17 custom SVG components, zero `@medusajs/icons` or `@medusajs/ui`)
