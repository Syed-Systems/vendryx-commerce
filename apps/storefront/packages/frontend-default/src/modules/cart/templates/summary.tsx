"use client"

import CartTotals from "@modules/common/components/cart-totals"
import Divider from "@modules/common/components/divider"
import DiscountCode from "@modules/common/components/discount-code"
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
    <div className="flex flex-col gap-y-4 p-6 bg-[color:var(--band-dark)]/80 rounded-[var(--brand-radius-card)] border text-[color:var(--on-dark-muted)]/80">
      <h2 className="text-xl font-heading font-bold text-[color:var(--on-dark)]">
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
        <button className="w-full py-3.5 px-6 rounded-[var(--brand-radius-button)] bg-[color:var(--band-dark)] text-[color:var(--on-dark)] font-semibold text-xs uppercase tracking-widest hover:bg-[color:var(--band-dark)] active:scale-[0.99] transition-all shadow-md flex items-center justify-center gap-2">
          <span>Proceed to Checkout</span>
          <span>→</span>
        </button>
      </LocalizedClientLink>

      <div className="text-[11px] text-[color:var(--on-dark-muted)] text-center font-light pt-2 flex items-center justify-center gap-2">
        <span>🔒 Secure 256-Bit SSL Encryption</span>
      </div>
    </div>
  )
}

export default Summary
