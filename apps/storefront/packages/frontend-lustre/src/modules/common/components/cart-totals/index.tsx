"use client"

import React from "react"
import { convertToLocale } from "@stheno/storefront-core"

export type CartTotalsProps = {
  totals: {
    total?: number | null
    subtotal?: number | null
    tax_total?: number | null
    currency_code: string
    item_subtotal?: number | null
    shipping_subtotal?: number | null
    discount_subtotal?: number | null
  }
}

/**
 * Lustre CartTotals.
 * Clear financial summary with tabular numerals, discrete line items, and final total.
 */
const CartTotals: React.FC<CartTotalsProps> = ({ totals }) => {
  const {
    currency_code,
    total,
    tax_total,
    item_subtotal,
    shipping_subtotal,
    discount_subtotal,
  } = totals

  return (
    <div className="w-full text-sm text-[var(--text-secondary)]">
      <div className="space-y-2.5">
        <div className="flex items-center justify-between">
          <span>Subtotal</span>
          <span
            data-testid="cart-subtotal"
            data-value={item_subtotal || 0}
            className="tabular-nums font-medium text-[var(--text-primary)]"
          >
            {convertToLocale({ amount: item_subtotal ?? 0, currency_code })}
          </span>
        </div>

        <div className="flex items-center justify-between">
          <span>Shipping estimate</span>
          <span
            data-testid="cart-shipping"
            data-value={shipping_subtotal || 0}
            className="tabular-nums font-medium text-[var(--text-primary)]"
          >
            {convertToLocale({ amount: shipping_subtotal ?? 0, currency_code })}
          </span>
        </div>

        {Boolean(discount_subtotal) && (
          <div className="flex items-center justify-between text-[var(--accent-sapphire)] font-medium">
            <span>Promotion savings</span>
            <span
              data-testid="cart-discount"
              data-value={discount_subtotal || 0}
              className="tabular-nums"
            >
              - {convertToLocale({ amount: discount_subtotal ?? 0, currency_code })}
            </span>
          </div>
        )}

        <div className="flex items-center justify-between">
          <span>Estimated taxes</span>
          <span
            data-testid="cart-taxes"
            data-value={tax_total || 0}
            className="tabular-nums font-medium text-[var(--text-primary)]"
          >
            {convertToLocale({ amount: tax_total ?? 0, currency_code })}
          </span>
        </div>
      </div>

      <div className="my-4 border-t border-[var(--hairline)]" />

      <div className="flex items-center justify-between text-base font-semibold text-[var(--text-primary)]">
        <span>Total due</span>
        <span
          data-testid="cart-total"
          data-value={total || 0}
          className="text-lg tabular-nums tracking-tight"
        >
          {convertToLocale({ amount: total ?? 0, currency_code })}
        </span>
      </div>

      <div className="mt-4 border-t border-[var(--hairline)]" />
    </div>
  )
}

export default CartTotals
