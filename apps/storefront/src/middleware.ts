// The middleware *logic* is core's, per ADR-013 decision 4. Next requires
// middleware to exist at the app root, so the app shell re-exports core's
// implementation rather than owning one.
export { middleware } from "@stheno/storefront-core/middleware"

/**
 * The matcher must be a literal in this file.
 *
 * It used to be re-exported from core alongside the handler. Next reads the
 * matcher by static analysis of this module, so a re-exported one is invisible
 * to it and the matcher is ignored entirely — which meant the middleware ran on
 * every request. Among the casualties was Next's image optimizer:
 * `/_next/image` was caught, region-resolved, and 307-redirected to
 * `/{countryCode}/_next/image`, which is not a route. Every `<Image>` on the
 * site therefore 404'd and rendered a broken-image glyph.
 *
 * `Thumbnail` had been using `next/image` long before this surfaced, so product
 * images were broken too. Nothing asserted on them, so it read as a design
 * choice rather than a fault.
 *
 * The old list read `_next/static|_next/image`, which would have been correct if
 * it had ever been applied. `_next` is excluded as a whole tree here, and
 * `.*\..*` covers asset paths the explicit extension list used to miss.
 *
 * The logic stays in core; only this routing table is local, because only this
 * file can be read statically.
 */
export const config = {
  matcher: ["/((?!api|_next|_vercel|.*\\..*).*)"],
}
