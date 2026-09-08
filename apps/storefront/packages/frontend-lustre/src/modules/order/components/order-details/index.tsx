import React from "react"
import { HttpTypes } from "@medusajs/types"

export type OrderDetailsProps = {
  order: HttpTypes.StoreOrder
  showStatus?: boolean
}

const formatStatus = (str?: string) => {
  if (!str) return "Unknown"
  const formatted = str.split("_").join(" ")
  return formatted.slice(0, 1).toUpperCase() + formatted.slice(1)
}

const OrderDetails: React.FC<OrderDetailsProps> = ({ order, showStatus }) => {
  return (
    <div className="space-y-4 rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-[var(--hairline)] pb-4">
        <div>
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block">
            Order Reference
          </span>
          <span className="text-lg font-semibold tabular-nums text-[var(--accent-sapphire)]" data-testid="order-id">
            #{order.display_id}
          </span>
        </div>

        <div className="sm:text-right">
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block">
            Date Placed
          </span>
          <span className="text-[var(--text-primary)]" data-testid="order-date">
            {new Date(order.created_at).toLocaleDateString(undefined, {
              year: "numeric",
              month: "short",
              day: "numeric",
            })}
          </span>
        </div>
      </div>

      <p className="text-[var(--text-secondary)]">
        Email:{" "}
        <strong className="text-[var(--text-primary)]" data-testid="order-email">
          {order.email}
        </strong>
      </p>

      {showStatus && (
        <div className="flex flex-wrap gap-4 pt-2 border-t border-[var(--hairline)]">
          <div>
            <span className="text-[var(--text-secondary)]">Fulfillment Status: </span>
            <span
              className="inline-flex items-center rounded-full bg-[var(--hairline)] px-2.5 py-0.5 font-semibold text-[var(--text-primary)]"
              data-testid="order-status"
            >
              {formatStatus(order.fulfillment_status)}
            </span>
          </div>

          <div>
            <span className="text-[var(--text-secondary)]">Payment Status: </span>
            <span
              className="inline-flex items-center rounded-full bg-[var(--hairline)] px-2.5 py-0.5 font-semibold text-[var(--text-primary)]"
              data-testid="order-payment-status"
            >
              {formatStatus(order.payment_status)}
            </span>
          </div>
        </div>
      )}
    </div>
  )
}

export default OrderDetails
