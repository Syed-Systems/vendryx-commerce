import React from "react"
import { HttpTypes } from "@medusajs/types"
import { convertToLocale } from "@stheno/storefront-core"

export type ShippingDetailsProps = {
  order: HttpTypes.StoreOrder
}

const ShippingDetails: React.FC<ShippingDetailsProps> = ({ order }) => {
  const shippingMethod = (order as any).shipping_methods?.[0]

  return (
    <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs">
      <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
        Fulfillment & Destination
      </h3>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div data-testid="shipping-address-summary">
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block mb-1.5">
            Recipient Address
          </span>
          <p className="font-medium text-[var(--text-primary)]">
            {order.shipping_address?.first_name} {order.shipping_address?.last_name}
          </p>
          <p className="text-[var(--text-secondary)]">
            {order.shipping_address?.address_1}
            {order.shipping_address?.address_2 ? `, ${order.shipping_address.address_2}` : ""}
          </p>
          <p className="text-[var(--text-secondary)]">
            {order.shipping_address?.city}, {order.shipping_address?.postal_code}
          </p>
          <p className="text-[var(--text-secondary)] uppercase">
            {order.shipping_address?.country_code}
          </p>
        </div>

        <div data-testid="shipping-contact-summary">
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block mb-1.5">
            Contact Channels
          </span>
          <p className="text-[var(--text-primary)]">
            {order.shipping_address?.phone || "No telephone provided"}
          </p>
          <p className="text-[var(--text-secondary)]">{order.email}</p>
        </div>

        <div data-testid="shipping-method-summary">
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block mb-1.5">
            Selected Carrier Method
          </span>
          <p className="font-medium text-[var(--text-primary)]">
            {shippingMethod?.name || "Not provided"}
          </p>
          {shippingMethod?.amount != null && (
            <p className="text-[var(--text-secondary)]">
              {convertToLocale({
                amount: shippingMethod.amount,
                currency_code: order.currency_code,
              })}
            </p>
          )}
        </div>
      </div>
    </div>
  )
}

export default ShippingDetails
