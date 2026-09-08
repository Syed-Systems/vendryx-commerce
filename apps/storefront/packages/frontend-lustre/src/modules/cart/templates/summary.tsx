"use client"

import React from "react"
import { HttpTypes } from "@medusajs/types"
import CartTotals from "@modules/common/components/cart-totals"
import DiscountCode from "@modules/common/components/discount-code"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

export type SummaryProps = {
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

const Summary: React.FC<SummaryProps> = ({ cart }) => {
  const step = getCheckoutStep(cart)

  return (
    <aside
      aria-label="Order Summary"
      className="space-y-6 rounded-[var(--radius-lg)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 shadow-sm backdrop-blur-md"
    >
      <h2 className="text-lg font-semibold text-[var(--text-primary)]">
        Order Summary
      </h2>

      <div className="space-y-4">
        <DiscountCode cart={cart} />
        <div className="border-t border-[var(--hairline)] pt-4">
          <CartTotals totals={cart} />
        </div>
      </div>

      <div className="pt-2">
        <LocalizedClientLink
          href={`/checkout?step=${step}`}
          data-testid="checkout-button"
          className="group flex h-12 w-full items-center justify-center rounded-full bg-[var(--accent-sapphire)] px-6 text-xs font-semibold uppercase tracking-widest text-white shadow-md transition hover:bg-[var(--accent-sapphire-hover)] hover:shadow-lg lustre-btn-press"
        >
          <span>Proceed to Checkout</span>
          <span className="ml-2 transition-transform duration-150 group-hover:translate-x-1">
            →
          </span>
        </LocalizedClientLink>
      </div>

      <div className="border-t border-[var(--hairline)] pt-4 text-center">
        <p className="text-[11px] text-[var(--text-secondary)]">
          Encrypted checkout with real-time tax and courier calculation at final review.
        </p>
      </div>
    </aside>
  )
}

export default Summary
