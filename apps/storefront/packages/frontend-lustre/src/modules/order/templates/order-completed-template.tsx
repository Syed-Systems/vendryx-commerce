import React from "react"
import { HttpTypes } from "@medusajs/types"
import CartTotals from "@modules/common/components/cart-totals"
import Help from "../components/help"
import Items from "../components/items"
import OnboardingCta from "../components/onboarding-cta"
import OrderDetails from "../components/order-details"
import ShippingDetails from "../components/shipping-details"
import PaymentDetails from "../components/payment-details"
import { CheckCircle } from "@phosphor-icons/react/dist/ssr"

export type OrderCompletedTemplateProps = {
  order: HttpTypes.StoreOrder
  isOnboarding: boolean
}

/**
 * Lustre OrderCompletedTemplate (Slot 45).
 * Confirmation screen presented after checkout completion.
 */
const OrderCompletedTemplate: React.FC<OrderCompletedTemplateProps> = ({
  order,
  isOnboarding,
}) => {
  return (
    <main className="min-h-screen bg-[var(--bg-primary)] py-12 sm:py-16">
      <div
        className="content-container max-w-2xl space-y-8"
        data-testid="order-complete-container"
      >
        {isOnboarding && <OnboardingCta orderId={order.id} />}

        {/* Confirmation Banner */}
        <header className="text-center">
          <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
            <CheckCircle size={28} aria-hidden="true" />
          </div>
          <span className="text-xs font-semibold uppercase tracking-[0.25em] text-[var(--accent-sapphire)]">
            Acquisition Confirmed
          </span>
          <h1 className="mt-1 lustre-section-title text-[var(--text-primary)]">
            Thank You for Your Order
          </h1>
          <p className="mt-2 text-xs text-[var(--text-secondary)]">
            Your order has been recorded.
          </p>
        </header>

        {/* Order Details */}
        <OrderDetails order={order} />

        {/* Purchased Items */}
        <Items order={order} />

        {/* Totals Breakdown */}
        <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6">
          <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
            Payment Summary
          </h3>
          <CartTotals totals={order} />
        </div>

        {/* Fulfillment & Payment */}
        <div className="grid grid-cols-1 gap-6">
          <ShippingDetails order={order} />
          <PaymentDetails order={order} />
        </div>

        {/* Client Support */}
        <Help />
      </div>
    </main>
  )
}

export default OrderCompletedTemplate
