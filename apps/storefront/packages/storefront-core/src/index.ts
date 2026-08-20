/**
 * @stheno/storefront-core — the public API a theme may use.
 *
 * ADR-013 decision 4: a theme supplies components against a typed contract and
 * receives props. It does not fetch and it does not route. So this entry point
 * carries formatting helpers, UI hooks, and the mutations a component fires from
 * an event handler. Read paths live in ./server and are the P25-10 debt.
 *
 * This entry must stay client-safe: theme client components import it, so every
 * module reachable from here is either pure or marked "use server". Pulling in a
 * plain server module (lib/data/categories.ts, lib/data/cookies.ts) would drag
 * next/headers into the client graph and fail the build outright.
 *
 * Versioned per docs/THEME_VERSIONING.md. Reaching past this into @lib/* or
 * @core/* is what scripts/check-storefront-boundaries.sh fails on.
 */

// Formatting. Money lives in core on purpose - THEME_CONTRACT_INPUTS.md §6.3.
export { convertToLocale } from "./lib/util/money"
export { getPercentageDiff } from "./lib/util/get-percentage-diff"
export { getProductPrice } from "./lib/util/get-product-price"
export { isSimpleProduct } from "./lib/util/product"
export { default as repeat } from "./lib/util/repeat"
export { REF_PARAM_KEY } from "./lib/attribution"
// A type is erased at build time, so exposing one is not a read path.
export type { Locale } from "./lib/data/locales"
export { isStripeLike, paymentInfoMap } from "./lib/constants"

// UI state. No data access.
export { default as useToggleState } from "./lib/hooks/use-toggle-state"
export type { StateType } from "./lib/hooks/use-toggle-state"
export { useIntersection } from "./lib/hooks/use-in-view"
export { ModalProvider, useModal } from "./lib/context/modal-context"

// Mutations a themed control fires from an event handler.
export {
  addToCart,
  applyPromotions,
  deleteLineItem,
  updateLineItem,
  updateRegion,
} from "./lib/data/cart"
export {
  addCustomerAddress,
  deleteCustomerAddress,
  login,
  signout,
  signup,
  transferCart,
  updateCustomer,
  updateCustomerAddress,
} from "./lib/data/customer"
export {
  acceptTransferRequest,
  createTransferRequest,
  declineTransferRequest,
} from "./lib/data/orders"
export { updateLocale } from "./lib/data/locale-actions"
export { resetOnboardingState } from "./lib/data/onboarding"
