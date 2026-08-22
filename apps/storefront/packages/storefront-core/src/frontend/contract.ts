import type { ComponentType } from "react"

/**
 * P25-05 — the frontend contract.
 *
 * "Frontend" here is historical and has misled three attempts. What this type
 * describes is a **frontend**: a complete storefront UI with its own components,
 * its own DOM and its own design system, which plugs into this engine. It is not
 * a palette, and it is not frontend-default with different colours — satisfying
 * every slot below by copying frontend-default and restyling it is the specific
 * failure `scripts/check-frontend-not-forked.sh` exists to fail the build over.
 *
 * Read AGENTS.md §12 before implementing one.
 *
 * Every export a frontend must supply for core to render. Derived from what core
 * actually imports, not from a wish-list: see docs/FRONTEND_CONTRACT_INPUTS.md for
 * the surface a designer should argue with before this is treated as final.
 *
 * Presence is enforced here, by frontend/conformance.ts. Props are enforced at
 * core's call sites, where TypeScript already checks them precisely — typing
 * them twice would only let the two drift apart.
 */
export type FrontendSlot =
  "AccountLayout" |
  "AddressBook" |
  "BrandFeatures" |
  "CartMismatchBanner" |
  "CartTemplate" |
  "CartTotals" |
  "CategoryGrid" |
  "CategoryTemplate" |
  "Checkbox" |
  "ChevronDown" |
  "CollectionTemplate" |
  "CountrySelect" |
  "DiscountCode" |
  "Divider" |
  "ErrorMessage" |
  "Footer" |
  "FreeShippingPriceNudge" |
  "Hero" |
  "Input" |
  "InteractiveLink" |
  "ItemsPreviewTemplate" |
  "LoginTemplate" |
  "LookbookStory" |
  "Nav" |
  "NewsletterSection" |
  "OrderCompletedTemplate" |
  "OrderDetailsTemplate" |
  "OrderOverview" |
  "Overview" |
  "ProductTemplate" |
  "ProfileBillingAddress" |
  "ProfileEmail" |
  "ProfileName" |
  "ProfilePassword" |
  "ProfilePhone" |
  "RadioIndicator" |
  "SkeletonCardDetails" |
  "SkeletonCartPage" |
  "SkeletonOrderConfirmed" |
  "Spinner" |
  "StoreTemplate" |
  "SubmitButton" |
  "TransferActions" |
  "TransferImage" |
  "TransferRequestForm"

export type FrontendContract = { readonly [K in FrontendSlot]: ComponentType<any> }
