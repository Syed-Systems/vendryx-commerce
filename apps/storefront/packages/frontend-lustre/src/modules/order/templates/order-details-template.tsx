"use client"

import React from "react"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { ArrowLeft } from "@phosphor-icons/react/dist/ssr"
import Help from "../components/help"
import Items from "../components/items"
import OrderDetails from "../components/order-details"
import OrderSummary from "../components/order-summary"
import ShippingDetails from "../components/shipping-details"

export type OrderDetailsTemplateProps = {
  order: HttpTypes.StoreOrder
}

/**
 * Lustre OrderDetailsTemplate (Slot 46).
 * Detailed order view accessed via user account history.
 */
const OrderDetailsTemplate: React.FC<OrderDetailsTemplateProps> = ({ order }) => {
  return (
    <div className="space-y-6" data-testid="order-details-container">
      {/* Header with return link */}
      <div className="flex items-center justify-between border-b border-[var(--hairline)] pb-4">
        <div>
          <span className="text-xs font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
            Account Management
          </span>
          <h1 className="text-xl font-semibold text-[var(--text-primary)]">
            Order Record
          </h1>
        </div>

        <LocalizedClientLink
          href="/account/orders"
          className="inline-flex items-center gap-2 rounded-full border border-[var(--hairline)] bg-[var(--bg-surface)] px-4 py-2 text-xs font-semibold text-[var(--text-primary)] transition hover:border-[var(--text-secondary)] lustre-btn-press"
          data-testid="back-to-overview-button"
        >
          <ArrowLeft size={14} />
          <span>Back to History</span>
        </LocalizedClientLink>
      </div>

      <OrderDetails order={order} showStatus />
      <Items order={order} />
      <ShippingDetails order={order} />
      <OrderSummary order={order} />
      <Help />
    </div>
  )
}

export default OrderDetailsTemplate
