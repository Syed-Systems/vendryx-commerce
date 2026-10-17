import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

/**
 * Presentation only. Core resolves collections and categories, per ADR-013
 * decision 4.
 */
export default async function Footer({
  collections,
  productCategories,
  enableCustomerAccounts = true,
  storeName,
}: {
  collections: HttpTypes.StoreCollection[]
  productCategories: HttpTypes.StoreProductCategory[]
  enableCustomerAccounts?: boolean
  /** Tenant's resolved display name. Core supplies it (ADR-013 decision 4);
   *  the frontend must not invent a wordmark, or every theme says "Stheno". */
  storeName: string
}) {

  return (
    <footer className="bg-[color:var(--band-dark)] text-[color:var(--on-dark)] border-t text-[color:var(--on-dark-muted)] w-full pt-16 pb-8 safe-bottom">
      <div className="content-container flex flex-col w-full">
        {/* Top Atelier statement bar */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 pb-14 border-b text-[color:var(--on-dark-muted)]/80 items-start">
          <div className="lg:col-span-7 space-y-4">
            <LocalizedClientLink href="/" className="inline-block">
              <span className="text-2xl font-heading tracking-[0.25em] indent-[0.25em] uppercase font-bold text-[color:var(--on-dark)]">
                {storeName}
              </span>
              <span className="block text-[10px] tracking-[0.35em] indent-[0.35em] text-[color:var(--on-dark-muted)] uppercase font-sans">
                Atelier • Modern Apparel
              </span>
            </LocalizedClientLink>
            <p className="body-copy measure text-[color:var(--on-dark-muted)] font-light">
              Considered wardrobe essentials — refined silhouettes, exceptional
              fabrications, and craftsmanship we can name.
            </p>
          </div>

          {/* The audit flagged two problems here. The form could not submit
              (`action="#"`, `type="button"`), and it promised "10% off your
              first acquisition" — an invented offer. There is no newsletter
              endpoint in the engine, and AGENTS.md forbids the storefront
              routing, so there is nowhere honest for this to post. It now routes
              to the one place a customer can actually register. */}
          <div className="lg:col-span-5 space-y-3">
            <h2 className="label-eyebrow text-[color:var(--on-dark)]">Join the atelier</h2>
            <p className="body-copy text-[color:var(--on-dark-muted)] font-light">
              Create an account to track orders, save pieces you are considering,
              and check out faster.
            </p>
            <LocalizedClientLink
              href="/account"
              className="glass-chip inline-flex min-h-[44px] w-full items-center justify-center gap-2 bg-[color:var(--brand-surface)] px-6 py-3 text-[color:var(--on-dark)] text-xs font-semibold uppercase tracking-[0.18em] hover:bg-[color:var(--band-dark)] transition-colors sm:w-auto"
            >
              Create an account
            </LocalizedClientLink>
          </div>
        </div>

        {/* Links Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-8 py-12 border-b text-[color:var(--on-dark-muted)]/80 text-xs">
          <nav aria-label="Collections" className="flex flex-col space-y-3">
            <h2 className="label-eyebrow text-[color:var(--on-dark)]">Collections</h2>
            <ul className="space-y-1 text-[color:var(--on-dark-muted)]" data-testid="footer-categories">
              <li>
                <LocalizedClientLink href="/store" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  All Collections
                </LocalizedClientLink>
              </li>
              <li>
                <LocalizedClientLink href="/store?sortBy=created_at" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  New Arrivals
                </LocalizedClientLink>
              </li>
              {collections?.slice(0, 4).map((c) => (
                <li key={c.id}>
                  <LocalizedClientLink
                    className="tap hover:text-[color:var(--on-dark)] transition-colors"
                    href={`/collections/${c.handle}`}
                  >
                    {c.title}
                  </LocalizedClientLink>
                </li>
              ))}
            </ul>
          </nav>

          <nav aria-label="Categories" className="flex flex-col space-y-3">
            <h2 className="label-eyebrow text-[color:var(--on-dark)]">Categories</h2>
            <ul className="space-y-1 text-[color:var(--on-dark-muted)]">
              {/* No fabricated category list.

                  This used to render four hardcoded names — "Outerwear &
                  Coats", "Tops & Knitwear", "Tailored Pants", "Accessories" —
                  every one of them a link to `/store`, on any tenant with no
                  categories. That is worse than an empty column: the customer
                  sees four plausible-sounding categories, clicks, and lands on
                  the full catalogue with no relationship to what they chose. On
                  the sari themes it also advertised "Tailored Pants".

                  A merchant with no categories has no categories. Say so. */}
              {productCategories && productCategories.length > 0 ? (
                productCategories.slice(0, 5).map((c) => (
                  <li key={c.id}>
                    <LocalizedClientLink
                      className="tap hover:text-[color:var(--on-dark)] transition-colors"
                      href={`/categories/${c.handle}`}
                      data-testid="category-link"
                    >
                      {c.name}
                    </LocalizedClientLink>
                  </li>
                ))
              ) : (
                <li className="text-[color:var(--on-dark-faint)]">
                  No categories yet.
                </li>
              )}
            </ul>
          </nav>

          {/* The audit flagged three items here rendered as
              `<span className="cursor-pointer">` — not links, not focusable,
              leading nowhere. Inventing routes for them is not available: core
              owns every URL shape and `check-storefront-boundaries.sh` fails the
              build if a frontend routes. So the dead affordances go rather than
              ship a link that lies. */}
          <nav aria-label="Your account" className="flex flex-col space-y-3">
            <h2 className="label-eyebrow text-[color:var(--on-dark)]">Your Account</h2>
            <ul className="space-y-1 text-[color:var(--on-dark-muted)]">
              <li>
                <LocalizedClientLink href="/account" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  Orders &amp; Addresses
                </LocalizedClientLink>
              </li>
              <li>
                <LocalizedClientLink href="/account#profile" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  Profile
                </LocalizedClientLink>
              </li>
              <li>
                <LocalizedClientLink href="/cart" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  Shopping Bag
                </LocalizedClientLink>
              </li>
            </ul>
          </nav>

          <nav aria-label="Browse" className="flex flex-col space-y-3">
            <h2 className="label-eyebrow text-[color:var(--on-dark)]">Browse</h2>
            <ul className="space-y-1 text-[color:var(--on-dark-muted)]">
              <li>
                <LocalizedClientLink href="/collections/summer-collection" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  Summer Collection
                </LocalizedClientLink>
              </li>
              <li>
                <LocalizedClientLink href="/store" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  Everything
                </LocalizedClientLink>
              </li>
              <li>
                <LocalizedClientLink href="/cart" className="tap hover:text-[color:var(--on-dark)] transition-colors">
                  Your Bag
                </LocalizedClientLink>
              </li>
            </ul>
          </nav>
        </div>

        {/* Bottom bar.

            The payment badge row went: "Visa / Mastercard / Apple Pay / Cash on
            Delivery / WhatsApp Order" is a claim about which processors this
            merchant has enabled, rendered unconditionally. It is configured per
            tenant and was never read from anywhere. The contact column likewise
            asserted an address and opening hours that do not exist. */}
        <div className="flex flex-col sm:flex-row items-center justify-between pt-8 gap-4 text-[0.6875rem] text-[color:var(--on-dark-muted)]">
          <p>© {new Date().getFullYear()} {storeName}. All rights reserved.</p>
          <nav aria-label="Legal" className="flex flex-wrap items-center justify-center gap-x-5 gap-y-1 sm:justify-end">
            <LocalizedClientLink href="/policies/privacy" className="tap hover:text-[color:var(--on-dark)] transition-colors">
              Privacy
            </LocalizedClientLink>
            <LocalizedClientLink href="/policies/terms" className="tap hover:text-[color:var(--on-dark)] transition-colors">
              Terms
            </LocalizedClientLink>
          </nav>
        </div>
      </div>
    </footer>
  )
}
