import type { ComponentType } from "react"

/**
 * P25-05 — the theme contract.
 *
 * Every export a theme must supply for core to render. Derived from what core
 * actually imports, not from a wish-list: see docs/THEME_CONTRACT_INPUTS.md for
 * the surface a designer should argue with before this is treated as final.
 *
 * Presence is enforced here, by theme/conformance.ts. Props are enforced at
 * core's call sites, where TypeScript already checks them precisely — typing
 * them twice would only let the two drift apart.
 */
export type ThemeSlot =
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

export type ThemeContract = { readonly [K in ThemeSlot]: ComponentType<any> }
