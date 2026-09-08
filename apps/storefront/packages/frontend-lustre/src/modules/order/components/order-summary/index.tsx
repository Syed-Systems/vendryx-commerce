import React from "react"
import { HttpTypes } from "@medusajs/types"
import { convertToLocale } from "@stheno/storefront-core"

export type OrderSummaryProps = {
  order: HttpTypes.StoreOrder
}

const OrderSummary: React.FC<OrderSummaryProps> = ({ order }) => {
  const formatMoney = (amount?: number | null) => {
    if (amount == null) return "-"
    return convertToLocale({
      amount,
      currency_code: order.currency_code,
    })
  }

  return (
    <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs">
      <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
        Financial Breakdown
      </h3>

      <div className="space-y-2.5">
        <div className="flex justify-between text-[var(--text-secondary)]">
          <span>Subtotal</span>
          <span className="font-medium text-[var(--text-primary)]">{formatMoney(order.subtotal)}</span>
        </div>

        {order.discount_total > 0 && (
          <div className="flex justify-between text-emerald-600 dark:text-emerald-400">
            <span>Promotion Discount</span>
            <span>- {formatMoney(order.discount_total)}</span>
          </div>
        )}

        {order.gift_card_total > 0 && (
          <div className="flex justify-between text-emerald-600 dark:text-emerald-400">
            <span>Gift Card Credit</span>
            <span>- {formatMoney(order.gift_card_total)}</span>
          </div>
        )}

        <div className="flex justify-between text-[var(--text-secondary)]">
          <span>Courier Shipping</span>
          <span className="font-medium text-[var(--text-primary)]">{formatMoney(order.shipping_total)}</span>
        </div>

        <div className="flex justify-between text-[var(--text-secondary)]">
          <span>Taxes</span>
          <span className="font-medium text-[var(--text-primary)]">{formatMoney(order.tax_total)}</span>
        </div>

        <div className="border-t border-[var(--hairline)] pt-3 mt-3 flex justify-between text-sm font-semibold text-[var(--text-primary)]">
          <span>Total Settled</span>
          <span className="text-[var(--accent-sapphire)]">{formatMoney(order.total)}</span>
        </div>
      </div>
    </div>
  )
}

export default OrderSummary
