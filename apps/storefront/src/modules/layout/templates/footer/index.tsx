import { listCategories } from "@lib/data/categories"
import { listCollections } from "@lib/data/collections"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

export default async function Footer() {
  const { collections } = await listCollections({
    fields: "*products",
  })
  const productCategories = await listCategories()

  return (
    <footer className="bg-zinc-950 text-zinc-300 border-t border-zinc-900 w-full pt-16 pb-12 mt-20">
      <div className="content-container flex flex-col w-full">
        {/* Top Newsletter & Brand Statement Bar */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 pb-14 border-b border-zinc-800/80 items-start">
          <div className="lg:col-span-6 space-y-4">
            <LocalizedClientLink
              href="/"
              className="inline-block"
            >
              <span className="text-2xl font-serif tracking-[0.25em] uppercase font-bold text-white">
                STHENO
              </span>
              <span className="block text-[10px] tracking-[0.35em] text-zinc-400 uppercase font-sans">
                Atelier • Modern Apparel
              </span>
            </LocalizedClientLink>
            <p className="text-sm text-zinc-400 max-w-md leading-relaxed font-light">
              Crafting conscious, enduring wardrobe essentials through refined silhouettes, exceptional fabrications, and ethical craftsmanship.
            </p>
          </div>

          <div className="lg:col-span-6 space-y-3">
            <h4 className="text-xs uppercase tracking-widest text-zinc-200 font-semibold">
              Join the Atelier Circle
            </h4>
            <p className="text-xs text-zinc-400 leading-relaxed font-light">
              Subscribe to receive preview access to private sales, seasonal lookbooks, and 10% off your first acquisition.
            </p>
            <form className="flex flex-col sm:flex-row gap-2 max-w-md pt-1" action="#" method="POST">
              <input
                type="email"
                placeholder="Enter your email address"
                required
                className="bg-zinc-900/90 border border-zinc-800 px-4 py-2.5 rounded-full text-xs text-white placeholder-zinc-500 focus:outline-none focus:border-zinc-500 flex-1 transition-colors"
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
                      Outerwear & Coats
                    </LocalizedClientLink>
                  </li>
                  <li>
                    <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                      Tops & Knitwear
                    </LocalizedClientLink>
                  </li>
                  <li>
                    <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                      Tailored Pants
                    </LocalizedClientLink>
                  </li>
                  <li>
                    <LocalizedClientLink href="/store" className="hover:text-white transition-colors">
                      Accessories
                    </LocalizedClientLink>
                  </li>
                </>
              )}
            </ul>
          </div>

          {/* Client Concierge & Care */}
          <div className="flex flex-col space-y-3">
            <span className="text-xs uppercase tracking-widest text-zinc-100 font-semibold">
              Client Concierge
            </span>
            <ul className="space-y-2 text-zinc-400">
              <li>
                <LocalizedClientLink href="/account" className="hover:text-white transition-colors">
                  My Account & Orders
                </LocalizedClientLink>
              </li>
              <li>
                <LocalizedClientLink href="/cart" className="hover:text-white transition-colors">
                  Shopping Bag
                </LocalizedClientLink>
              </li>
              <li>
                <span className="hover:text-white transition-colors cursor-pointer">
                  Complimentary Shipping & Returns
                </span>
              </li>
              <li>
                <span className="hover:text-white transition-colors cursor-pointer">
                  Garment Care & Sizing Guide
                </span>
              </li>
              <li>
                <span className="hover:text-white transition-colors cursor-pointer">
                  Sustainability & Sourcing
                </span>
              </li>
            </ul>
          </div>

          {/* Contact & Social */}
          <div className="flex flex-col space-y-3">
            <span className="text-xs uppercase tracking-widest text-zinc-100 font-semibold">
              Direct Assistance
            </span>
            <ul className="space-y-2 text-zinc-400">
              <li>
                <span className="text-zinc-500 block text-[11px]">Concierge Desk</span>
                <span className="text-zinc-200">concierge@sthenostore.com</span>
              </li>
              <li>
                <span className="text-zinc-500 block text-[11px]">Hours</span>
                <span className="text-zinc-300">Mon – Fri: 09:00 – 18:00 EST</span>
              </li>
              {/* Wraps. Three social names plus separators are wider than this column at
                  mobile widths, and an unwrapped flex row pushed the last one 33px past
                  the viewport — which made every page on the site scroll sideways, since
                  the footer is global. */}
              <li className="pt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-zinc-400">
                <span className="hover:text-white transition-colors cursor-pointer">Instagram</span>
                <span>•</span>
                <span className="hover:text-white transition-colors cursor-pointer">Pinterest</span>
                <span>•</span>
                <span className="hover:text-white transition-colors cursor-pointer">WhatsApp</span>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Bar: Copyright & Payment badging */}
        <div className="flex flex-col sm:flex-row items-center justify-between pt-8 gap-4 text-[11px] text-zinc-500">
          <div>
            © {new Date().getFullYear()} STHENO Storefront Atelier. All rights reserved. Powered by Medusa v2.
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
