/**
 * @stheno/storefront-core/server — read paths, server components only.
 *
 * This entry exists to make a debt visible rather than to bless it. ADR-013
 * decision 4 says a theme does not fetch: every theme component importing from
 * here is a P25-10 violation waiting to be lifted into core and passed down as
 * props. scripts/check-frontend-purity.sh counts them.
 *
 * It is separate from the root entry because these modules reach for
 * next/headers, which must never enter a client bundle.
 */
export { listCategories } from "./lib/data/categories"
export { listCollections } from "./lib/data/collections"
export { listProducts, listProductsWithSort } from "./lib/data/products"
export { getRegion, listRegions } from "./lib/data/regions"
export { listLocales } from "./lib/data/locales"
export { getLocale } from "./lib/data/locale-actions"
export { retrieveCart } from "./lib/data/cart"
