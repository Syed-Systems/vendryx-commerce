"use client"

import CartTotals from "@modules/common/components/cart-totals"
import Divider from "@modules/common/components/divider"
import DiscountCode from "@modules/checkout/components/discount-code"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { HttpTypes } from "@medusajs/types"

type SummaryProps = {
  cart: HttpTypes.StoreCart & {
    promotions: HttpTypes.StorePromotion[]
  }
}

function getCheckoutStep(cart: HttpTypes.StoreCart) {
  if (!cart?.shipping_address?.address_1 || !cart.email) {
    return "address"
  } else if (cart?.shipping_methods?.length === 0) {
    return "delivery"
  } else {
    return "payment"
  }
}

const Summary = ({ cart }: SummaryProps) => {
  const step = getCheckoutStep(cart)

  return (
    <div className="flex flex-col gap-y-4 p-6 bg-zinc-50/80 rounded-2xl border border-zinc-200/80">
      <h2 className="text-xl font-serif font-bold text-zinc-950">
        Order Summary
      </h2>
      <DiscountCode cart={cart} />
      <Divider />
      <CartTotals totals={cart} />
      <LocalizedClientLink
        href={"/checkout?step=" + step}
        data-testid="checkout-button"
        className="block mt-2"
      >
        <button className="w-full py-3.5 px-6 rounded-full bg-zinc-950 text-white font-semibold text-xs uppercase tracking-widest hover:bg-zinc-800 active:scale-[0.99] transition-all shadow-md flex items-center justify-center gap-2">
          <span>Proceed to Checkout</span>
          <span>→</span>
        </button>
      </LocalizedClientLink>

      <div className="text-[11px] text-zinc-500 text-center font-light pt-2 flex items-center justify-center gap-2">
        <span>🔒 Secure 256-Bit SSL Encryption</span>
      </div>
    </div>
  )
}

export default Summary
