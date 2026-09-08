"use client"

import React from "react"
import { HttpTypes } from "@medusajs/types"
import OrderCard from "../order-card"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { Archive } from "@phosphor-icons/react/dist/ssr"

export type OrderOverviewProps = {
  orders: HttpTypes.StoreOrder[]
}

/**
 * Lustre OrderOverview (Slot 9).
 * Client order registry presenting past transaction logs and delivery manifests.
 */
const OrderOverview: React.FC<OrderOverviewProps> = ({ orders }) => {
  if (orders?.length) {
    return (
      <div className="space-y-6 w-full">
        <div className="border-b border-[var(--hairline)] pb-4">
          <span className="text-xs font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
            Account History
          </span>
          <h1 className="text-xl font-semibold text-[var(--text-primary)]">
            Order Transactions
          </h1>
        </div>

        <div className="space-y-4">
          {orders.map((order) => (
            <OrderCard key={order.id} order={order} />
          ))}
        </div>
      </div>
    )
  }

  return (
    <div
      className="flex flex-col items-center justify-center py-24 text-center"
      data-testid="no-orders-container"
    >
      <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-[var(--hairline)] text-[var(--accent-sapphire)]">
        <Archive size={24} aria-hidden="true" />
      </div>

      <h2 className="text-xl font-semibold text-[var(--text-primary)]">
        No Transaction Records
      </h2>

      <p className="mt-2 max-w-sm text-xs text-[var(--text-secondary)]">
        Completed orders will appear here.
      </p>

      <div className="mt-6">
        <LocalizedClientLink
          href="/store"
          className="inline-flex h-11 items-center justify-center rounded-full bg-[var(--accent-sapphire)] px-6 text-xs font-semibold text-white shadow-sm hover:bg-[var(--accent-sapphire-hover)] transition lustre-btn-press"
          data-testid="continue-shopping-button"
        >
          Browse catalog
        </LocalizedClientLink>
      </div>
    </div>
  )
}

export default OrderOverview
