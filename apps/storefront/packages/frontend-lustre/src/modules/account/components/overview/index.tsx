import React from "react"
import { HttpTypes } from "@medusajs/types"
import { convertToLocale } from "@stheno/storefront-core"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { CaretDown } from "@phosphor-icons/react/dist/ssr"

export type OverviewProps = {
  customer: HttpTypes.StoreCustomer | null
  orders: HttpTypes.StoreOrder[] | null
}

const getProfileCompletion = (customer: HttpTypes.StoreCustomer | null) => {
  let count = 0
  if (!customer) return 0
  if (customer.email) count++
  if (customer.first_name && customer.last_name) count++
  if (customer.phone) count++
  const billingAddress = customer.addresses?.find((addr) => addr.is_default_billing)
  if (billingAddress) count++
  return (count / 4) * 100
}

/**
 * Lustre Overview (Slot 10).
 * Dashboard summary displaying profile completion, saved destinations, and recent activity.
 */
const Overview: React.FC<OverviewProps> = ({ customer, orders }) => {
  const completion = getProfileCompletion(customer)
  const addressCount = customer?.addresses?.length || 0

  return (
    <div className="space-y-8" data-testid="overview-page-wrapper">
      {/* Header Banner */}
      <div className="flex flex-col sm:flex-row sm:items-baseline sm:justify-between gap-2 border-b border-[var(--hairline)] pb-4">
        <div>
          <span className="text-[10px] font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
            Account Dashboard
          </span>
          <h1
            className="text-2xl font-semibold text-[var(--text-primary)] mt-0.5"
            data-testid="welcome-message"
            data-value={customer?.first_name}
          >
            Welcome, {customer?.first_name || "Client"}
          </h1>
        </div>

        <div className="text-xs text-[var(--text-secondary)]">
          Signed in as:{" "}
          <span
            className="font-semibold text-[var(--text-primary)]"
            data-testid="customer-email"
            data-value={customer?.email}
          >
            {customer?.email}
          </span>
        </div>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-5 text-xs">
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block">
            Profile Completion
          </span>
          <div className="mt-2 flex items-baseline gap-2">
            <span
              className="text-3xl font-semibold text-[var(--accent-sapphire)]"
              data-testid="customer-profile-completion"
              data-value={completion}
            >
              {completion}%
            </span>
            <span className="text-[var(--text-secondary)] uppercase text-[10px]">
              Fields completed
            </span>
          </div>
        </div>

        <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-5 text-xs">
          <span className="font-semibold uppercase tracking-wider text-[var(--text-secondary)] block">
            Saved Destinations
          </span>
          <div className="mt-2 flex items-baseline gap-2">
            <span
              className="text-3xl font-semibold text-[var(--text-primary)]"
              data-testid="addresses-count"
              data-value={addressCount}
            >
              {addressCount}
            </span>
            <span className="text-[var(--text-secondary)] uppercase text-[10px]">
              Saved addresses
            </span>
          </div>
        </div>
      </div>

      {/* Recent Orders List */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold text-[var(--text-primary)]">
            Recent orders
          </h2>
          <LocalizedClientLink
            href="/account/orders"
            className="text-xs font-semibold text-[var(--accent-sapphire)] hover:underline"
          >
            View all
          </LocalizedClientLink>
        </div>

        <ul className="space-y-3" data-testid="orders-wrapper">
          {orders && orders.length > 0 ? (
            orders.slice(0, 5).map((order) => (
              <li
                key={order.id}
                className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-4 text-xs transition hover:border-[var(--accent-sapphire)]/40"
                data-testid="order-wrapper"
                data-value={order.id}
              >
                <LocalizedClientLink
                  href={`/account/orders/details/${order.id}`}
                  className="flex items-center justify-between"
                >
                  <div className="grid grid-cols-3 gap-4 flex-1">
                    <div>
                      <span className="text-[10px] text-[var(--text-secondary)] uppercase tracking-wider block">
                        Date
                      </span>
                      <span className="font-medium text-[var(--text-primary)]" data-testid="order-created-date">
                        {new Date(order.created_at).toLocaleDateString(undefined, {
                          month: "short",
                          day: "numeric",
                          year: "numeric",
                        })}
                      </span>
                    </div>

                    <div>
                      <span className="text-[10px] text-[var(--text-secondary)] uppercase tracking-wider block">
                        Order #
                      </span>
                      <span
                        className="font-medium text-[var(--accent-sapphire)]"
                        data-testid="order-id"
                        data-value={order.display_id}
                      >
                        #{order.display_id}
                      </span>
                    </div>

                    <div>
                      <span className="text-[10px] text-[var(--text-secondary)] uppercase tracking-wider block">
                        Total
                      </span>
                      <span className="font-semibold text-[var(--text-primary)]" data-testid="order-amount">
                        {convertToLocale({
                          amount: order.total,
                          currency_code: order.currency_code,
                        })}
                      </span>
                    </div>
                  </div>

                  <button
                    type="button"
                    className="ml-4 flex h-8 w-8 items-center justify-center rounded-full border border-[var(--hairline)] text-[var(--text-secondary)] hover:text-[var(--accent-sapphire)] hover:border-[var(--accent-sapphire)] transition lustre-btn-press"
                    data-testid="open-order-button"
                  >
                    <span className="sr-only">Go to order #{order.display_id}</span>
                    <CaretDown size={14} className="-rotate-90" />
                  </button>
                </LocalizedClientLink>
              </li>
            ))
          ) : (
            <div
              className="rounded-[var(--radius-md)] border border-dashed border-[var(--hairline)] p-8 text-center text-xs text-[var(--text-secondary)]"
              data-testid="no-orders-message"
            >
              No past transactions logged to this account profile.
            </div>
          )}
        </ul>
      </div>
    </div>
  )
}

export default Overview
