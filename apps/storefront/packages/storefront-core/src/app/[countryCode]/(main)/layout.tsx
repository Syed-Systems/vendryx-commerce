import { Metadata } from "next"

import { listCartOptions, retrieveCart } from "@lib/data/cart"
import { listCategories } from "@lib/data/categories"
import { listCollections } from "@lib/data/collections"
import { getLocale } from "@lib/data/locale-actions"
import { listLocales } from "@lib/data/locales"
import { listRegions } from "@lib/data/regions"
import CartButtonLoader from "@core/loaders/cart-button"
import { retrieveCustomer } from "@lib/data/customer"
import { getBaseURL } from "@lib/util/env"
import { StoreCartShippingOption, StoreRegion } from "@medusajs/types"
import CartMismatchBanner from "@frontend/layout/components/cart-mismatch-banner"
import Footer from "@frontend/layout/templates/footer"
import Nav from "@frontend/layout/templates/nav"
import FreeShippingPriceNudge from "@frontend/shipping/components/free-shipping-price-nudge"

export const metadata: Metadata = {
  metadataBase: new URL(getBaseURL()),
}

export default async function PageLayout(props: { children: React.ReactNode }) {
  const customer = await retrieveCustomer()
  const cart = await retrieveCart()
  const [regions, locales, currentLocale] = await Promise.all([
    listRegions().then((regions: StoreRegion[]) => regions),
    listLocales(),
    getLocale(),
  ])
  const { collections } = await listCollections({ fields: "*products" })
  const productCategories = await listCategories()
  let shippingOptions: StoreCartShippingOption[] = []

  if (cart) {
    const { shipping_options } = await listCartOptions()

    shippingOptions = shipping_options
  }

  return (
    <>
      <Nav
        regions={regions}
        locales={locales}
        currentLocale={currentLocale}
        cartButton={<CartButtonLoader />}
      />
      {customer && cart && (
        <CartMismatchBanner customer={customer} cart={cart} />
      )}

      {cart && (
        <FreeShippingPriceNudge
          variant="popup"
          cart={cart}
          shippingOptions={shippingOptions}
        />
      )}
      {props.children}
      <Footer collections={collections} productCategories={productCategories} />
    </>
  )
}
