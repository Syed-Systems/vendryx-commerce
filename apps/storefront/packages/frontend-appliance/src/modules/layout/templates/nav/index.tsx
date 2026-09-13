import type { ReactNode } from "react"
import type { Locale } from "@stheno/storefront-core"
import { HttpTypes, StoreRegion } from "@medusajs/types"
import AppleNavbar from "@modules/layout/components/apple-nav-bar"

/**
 * Apple-grade Architectural Navigation for Electrical & Home Appliances.
 * Ultra-minimalist frosted glass, horizontal category typography,
 * full-width flyout curtains, and zero hamburger menus.
 */
export default async function Nav({
  regions,
  locales,
  currentLocale,
  cartButton,
  categories,
  enableCustomerAccounts = true,
  customer,
}: {
  regions: StoreRegion[]
  locales: Locale[] | null
  currentLocale: string | null
  cartButton: ReactNode
  categories?: HttpTypes.StoreProductCategory[] | null
  enableCustomerAccounts?: boolean
  customer?: HttpTypes.StoreCustomer | null
}) {
  return (
    <div className="sticky top-0 inset-x-0 z-50 group">
      {/* Service & Delivery Announcement Bar (Zero unsourced figures) */}
      <div className="bg-[#0D0E11] text-[#A1A1A6] text-[11px] font-normal tracking-tight py-1.5 px-4 text-center flex items-center justify-center gap-2 border-b border-white/[0.06]">
        <span className="inline-block w-1.5 h-1.5 rounded-full bg-emerald-400" />
        <span>White-Glove Delivery &amp; Expert Installation Available</span>
        <span className="hidden sm:inline text-zinc-600">•</span>
        <span className="hidden sm:inline text-zinc-400">Authorized Manufacturer Warranty on All Equipment</span>
      </div>

      {/* Apple Flagship Global Navigation (Zero Hamburger Menu) */}
      <AppleNavbar
        cartButton={cartButton}
        categories={categories}
        enableCustomerAccounts={enableCustomerAccounts}
        customer={customer}
      />
    </div>
  )
}
