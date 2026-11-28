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
 * Versioned per docs/FRONTEND_VERSIONING.md. Reaching past this into @lib/* or
 * @core/* is what scripts/check-storefront-boundaries.sh fails on.
 */

// Formatting. Money lives in core on purpose - FRONTEND_CONTRACT_INPUTS.md §6.3.
export { convertToLocale } from "./lib/util/money"
export { getPercentageDiff } from "./lib/util/get-percentage-diff"
export { getProductPrice } from "./lib/util/get-product-price"
export { isSimpleProduct } from "./lib/util/product"
export { default as repeat } from "./lib/util/repeat"
export { REF_PARAM_KEY } from "./lib/attribution"
// A type is erased at build time, so exposing one is not a read path.
export type { Locale } from "./lib/data/locales"
export { isManual, isStripeLike, paymentInfoMap } from "./lib/constants"

// Branding & runtime tokens (P26-05)
export {
  resolveBrandingTokens,
  resolveBrandingCssVars,
  emitBrandingCss,
  DEFAULT_BRANDING,
  FONT_STACKS,
  // Named brand presets. DEFAULT_BRANDING was already public; these are its siblings and
  // belong on the same surface, because a frontend that may read the default preset may
  // read the others. Exporting them is what lets dev-theme-switcher stay on the public
  // entry point instead of reaching past it into @lib/.
  MAHAL_BRANDING,
  LUSTRE_BRANDING,
  MATRIX_BRANDING,
} from "./lib/branding"
export type { TenantBranding, ResolvedBranding } from "./lib/branding"

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
  revokeCustomerWhatsAppConsent,
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
  retrieveOrderPaymentProof,
  submitManualOrderPaymentProof,
} from "./lib/data/orders"
export type { OrderPaymentProof } from "./lib/data/orders"
export { updateLocale } from "./lib/data/locale-actions"
export { resetOnboardingState } from "./lib/data/onboarding"
