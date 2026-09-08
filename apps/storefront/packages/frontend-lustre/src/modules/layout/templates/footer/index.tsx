import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import FooterNav from "./footer-nav"

export type FooterProps = {
  collections?: HttpTypes.StoreCollection[] | null
  productCategories?: HttpTypes.StoreProductCategory[] | null
}

/**
 * Lustre Footer.
 * Sourced presentation footer reflecting actual catalog structure.
 * Never invents claims, fake hours, concierge details, or unrouted policies.
 * Conforms to contract slot 41 and FRONTEND_LUSTRE_PLAN.md section 7.
 */
export default async function Footer({
  collections,
  productCategories,
}: FooterProps) {
  const currentYear = new Date().getFullYear()

  return (
    <footer className="w-full bg-[var(--bg-canvas)] border-t border-[var(--hairline)] pt-16 pb-12 mt-24 text-[var(--text-secondary)]">
      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-10 border-b border-[var(--hairline)] pb-12">
          {/* Identity */}
          <div className="md:col-span-1 space-y-3">
            <LocalizedClientLink href="/" className="inline-block">
              <span className="text-xl font-semibold tracking-[0.2em] uppercase text-[var(--text-primary)]">
                Store
              </span>
              <span className="block text-[9px] tracking-[0.25em] text-[var(--text-muted)] uppercase mt-0.5">
                Watches &amp; Objects
              </span>
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store"
              className="block text-xs text-[var(--text-secondary)] transition-colors hover:text-[var(--text-primary)]"
            >
              Browse the catalog
            </LocalizedClientLink>
          </div>

          {/* Navigation sections (Desktop columns + Mobile Disclosures) */}
          <div className="md:col-span-3">
            <FooterNav
              collections={collections}
              productCategories={productCategories}
            />
          </div>
        </div>

        {/* Footer bottom metadata & copyright */}
        <div className="pt-6 flex flex-col sm:flex-row items-center justify-between gap-4 text-[11px] text-[var(--text-muted)]">
          <div>
            Catalog, account, and order information are supplied by the store.
          </div>
          <div>
            &copy; {currentYear} Store. All rights reserved.
          </div>
        </div>
      </div>
    </footer>
  )
}
