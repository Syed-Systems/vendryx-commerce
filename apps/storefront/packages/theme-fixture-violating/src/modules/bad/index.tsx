// Every line below is a boundary or purity violation, on purpose. This theme is
// never built and never shipped: it exists so check-storefront-boundaries.sh and
// check-theme-purity.sh are proven to fail on a real violation rather than
// trusted to.

// Boundary: core internals and the SDK.
import { sdk } from "@lib/config"
import { retrieveCart } from "@lib/data/cart"
// Boundary: core's checkout module.
import { SubmitButton } from "@modules/checkout/components/submit-button"

// Purity: a theme reading through core's server entry instead of taking props.
import { listProducts } from "@stheno/storefront-core/server"

export default async function Bad() {
  const cart = await retrieveCart()
  const products = await listProducts({ countryCode: "us" })
  await sdk.client.fetch("/store/regions")
  return (
    <div>
      {cart?.id} {products.response.products.length}
      <SubmitButton>go</SubmitButton>
    </div>
  )
}
