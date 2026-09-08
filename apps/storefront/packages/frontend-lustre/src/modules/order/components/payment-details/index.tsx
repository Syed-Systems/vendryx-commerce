import React from "react"
import { HttpTypes } from "@medusajs/types"
import { convertToLocale, isStripeLike, paymentInfoMap } from "@stheno/storefront-core"

export type PaymentDetailsProps = {
  order: HttpTypes.StoreOrder
}

const PaymentDetails: React.FC<PaymentDetailsProps> = ({ order }) => {
  const payment = order.payment_collections?.[0]?.payments?.[0]
  const info = payment?.provider_id ? paymentInfoMap[payment.provider_id] : null

  return (
    <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs">
      <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
        Payment details
      </h3>

      {payment ? (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block mb-1.5">
              Provider
            </span>
            <div className="flex items-center gap-2" data-testid="payment-method">
              {info?.icon && <span className="text-[var(--accent-sapphire)]">{info.icon}</span>}
              <span className="font-medium text-[var(--text-primary)]">
                {info?.title || payment.provider_id}
              </span>
            </div>
          </div>

          <div>
            <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block mb-1.5">
              Payment record
            </span>
            <p className="text-[var(--text-primary)]" data-testid="payment-amount">
              {isStripeLike(payment.provider_id) && payment.data?.card_last4
                ? `Card ending in ${payment.data.card_last4}`
                : convertToLocale({
                    amount: payment.amount,
                    currency_code: order.currency_code,
                  })}
            </p>
            {payment.created_at && (
              <p className="text-[var(--text-secondary)] mt-0.5">
                Timestamp: {new Date(payment.created_at).toLocaleString()}
              </p>
            )}
          </div>
        </div>
      ) : (
        <p className="text-[var(--text-secondary)]">No payment record provided.</p>
      )}
    </div>
  )
}

export default PaymentDetails
