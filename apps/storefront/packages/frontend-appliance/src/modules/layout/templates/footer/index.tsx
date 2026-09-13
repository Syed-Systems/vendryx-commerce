import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

/**
 * Presentation only. Core resolves collections and categories, per ADR-013
 * decision 4.
 */
export default async function Footer({
  collections,
  productCategories,
  enableCustomerAccounts = false,
}: {
  collections: HttpTypes.StoreCollection[]
  productCategories: HttpTypes.StoreProductCategory[]
  enableCustomerAccounts?: boolean
}) {

  return (
    <footer className="bg-[#0B0C0E] text-zinc-300 border-t border-zinc-800/80 w-full pt-16 pb-12 mt-20">
      <div className="content-container flex flex-col w-full">
        {/* Top Newsletter & Brand Statement Bar */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 pb-14 border-b border-zinc-800/80 items-start">
          <div className="lg:col-span-6 space-y-4">
            <LocalizedClientLink
              href="/"
              className="inline-block"
            >
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold tracking-[0.18em] uppercase text-white">
                  VENDRYX
                </span>
                <span className="px-2 py-0.5 rounded text-[9px] font-bold uppercase bg-amber-500/15 text-amber-400 border border-amber-500/30">
                  APPLIANCES
                </span>
              </div>
              <span className="block text-[10px] tracking-[0.25em] text-zinc-400 uppercase font-sans font-medium mt-1">
                Electrical &amp; Living Infrastructure
              </span>
            </LocalizedClientLink>
            <p className="text-sm text-zinc-400 max-w-md leading-relaxed font-normal">
              Curating high-performance kitchen appliances, espresso systems, climate control, and precision electrical equipment.
            </p>
          </div>

          <div className="lg:col-span-6 space-y-3">
            <h4 className="text-xs uppercase tracking-widest text-zinc-200 font-semibold">
              Appliance &amp; Living Briefings
            </h4>
            <p className="text-xs text-zinc-400 leading-relaxed font-normal">
              Subscribe to receive updates on new culinary appliances, climate engineering releases, and trade consultations.
            </p>
            <form className="flex flex-col sm:flex-row gap-2 max-w-md pt-1" action="#" method="POST">
              <input
                type="email"
                placeholder="Enter your email address"
                required
                className="bg-zinc-900/90 border border-zinc-800 px-4 py-2.5 rounded-full text-xs text-white placeholder-zinc-500 focus:outline-none focus:border-amber-500/50 flex-1 transition-colors"
              />
              <button
                type="button"
                className="bg-white text-zinc-950 px-6 py-2.5 rounded-full text-xs font-semibold uppercase tracking-wider hover:bg-zinc-200 active:scale-95 transition-all"
              >
                Subscribe
              </button>
            </form>
          </div>
        </div>

        {/* Links Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 py-12 border-b border-zinc-800/80 text-xs">
          {/* Categories */}
          <div className="flex flex-col space-y-3">
            <span className="text-xs uppercase tracking-widest text-zinc-100 font-semibold">
              Collections
            </span>
            <ul className="space-y-2 text-zinc-400" data-testid="footer-categories">
              <li>
                <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                  All Collections
                </LocalizedClientLink>
              </li>
              <li>
                <LocalizedClientLink href="/store?sortBy=created_at" className="hover:text-white transition-colors">
                  New Arrivals
                </LocalizedClientLink>
              </li>
              {collections?.slice(0, 4).map((c) => (
                <li key={c.id}>
                  <LocalizedClientLink
                    className="hover:text-white transition-colors"
                    href={`/collections/${c.handle}`}
                  >
                    {c.title}
                  </LocalizedClientLink>
                </li>
              ))}
            </ul>
          </div>

          {/* Categories */}
          <div className="flex flex-col space-y-3">
            <span className="text-xs uppercase tracking-widest text-zinc-100 font-semibold">
              Categories
            </span>
            <ul className="space-y-2 text-zinc-400">
              {productCategories && productCategories.length > 0 ? (
                productCategories.slice(0, 5).map((c) => (
                  <li key={c.id}>
                    <LocalizedClientLink
                      className="hover:text-white transition-colors"
                      href={`/categories/${c.handle}`}
                      data-testid="category-link"
                    >
                      {c.name}
                    </LocalizedClientLink>
                  </li>
                ))
              ) : (
                <>
                  <li>
                    <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                      Smartphones & Foldables
                    </LocalizedClientLink>
                  </li>
                  <li>
                    <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                      Laptops & Ultrabooks
                    </LocalizedClientLink>
                  </li>
                  <li>
                    <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                      Spatial Audio & Studio
                    </LocalizedClientLink>
                  </li>
                  <li>
                    <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                      Precision Accessories
                    </LocalizedClientLink>
                  </li>
                </>
              )}
            </ul>
          </div>

          {/* Client Concierge & Care */}
          <div className="flex flex-col space-y-3">
            <span className="text-xs uppercase tracking-widest text-zinc-100 font-semibold">
              Hardware Support & Care
            </span>
            <ul className="space-y-2 text-zinc-400">
              {enableCustomerAccounts && (
                <li>
                  <LocalizedClientLink href="/account" className="hover:text-white transition-colors">
                    My Account & Orders
                  </LocalizedClientLink>
                </li>
              )}
              <li>
                <LocalizedClientLink href="/cart" className="hover:text-white transition-colors">
                  Shopping Bag
                </LocalizedClientLink>
              </li>
              <li>
                <span className="hover:text-white transition-colors cursor-pointer">
                  Express Tracked Shipping
                </span>
              </li>
              <li>
                <span className="hover:text-white transition-colors cursor-pointer">
                  Official Manufacturer Warranty
                </span>
              </li>
              <li>
                <span className="hover:text-white transition-colors cursor-pointer">
                  Technical Specifications & Care
                </span>
              </li>
            </ul>
          </div>

          {/* Contact & Social */}
          <div className="flex flex-col space-y-3">
            <span className="text-xs uppercase tracking-widest text-zinc-100 font-semibold">
              Support Engineering
            </span>
            <ul className="space-y-2 text-zinc-400">
              <li>
                <span className="text-zinc-500 block text-[11px]">Hardware Helpdesk</span>
                <span className="text-zinc-200">support@vendryxstore.com</span>
              </li>
              <li>
                <span className="text-zinc-500 block text-[11px]">Availability</span>
                <span className="text-zinc-300">Mon – Fri: 09:00 – 18:00 EST</span>
              </li>
              {/* Wraps. Three social names plus separators are wider than this column at
                  mobile widths, and an unwrapped flex row pushed the last one 33px past
                  the viewport — which made every page on the site scroll sideways, since
                  the footer is global. */}
              <li className="pt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-zinc-400">
                <span className="hover:text-white transition-colors cursor-pointer">GitHub</span>
                <span>•</span>
                <span className="hover:text-white transition-colors cursor-pointer">X / Twitter</span>
                <span>•</span>
                <span className="hover:text-white transition-colors cursor-pointer">YouTube</span>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Bar: Copyright & Payment badging */}
        <div className="flex flex-col sm:flex-row items-center justify-between pt-8 gap-4 text-[11px] text-zinc-500">
          <div>
            © {new Date().getFullYear()} VENDRYX Pro Tech Store. All rights reserved.
          </div>
          {/* Wraps: five badges on one unwrapped row are 33px wider than a 390px viewport,
              which made every page on the site scroll sideways — the footer is global. */}
          <div className="flex flex-wrap items-center justify-center gap-x-4 gap-y-1 text-zinc-400 sm:justify-end">
            <span>Visa</span>
            <span>Mastercard</span>
            <span>Apple Pay</span>
            <span>Cash on Delivery</span>
            <span>WhatsApp Order</span>
          </div>
        </div>
      </div>
    </footer>
  )
}
