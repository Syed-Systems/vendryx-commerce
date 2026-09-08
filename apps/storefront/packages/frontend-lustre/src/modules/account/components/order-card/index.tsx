import React, { useMemo } from "react"
import { HttpTypes } from "@medusajs/types"
import { convertToLocale } from "@stheno/storefront-core"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import Thumbnail from "@modules/products/components/thumbnail"

export type OrderCardProps = {
  order: HttpTypes.StoreOrder
}

const OrderCard: React.FC<OrderCardProps> = ({ order }) => {
  const numberOfLines = useMemo(() => {
    return (
      order.items?.reduce((acc, item) => acc + item.quantity, 0) ?? 0
    )
  }, [order])

  const numberOfProducts = useMemo(() => {
    return order.items?.length ?? 0
  }, [order])

  return (
    <article
      className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6 text-xs transition hover:border-[var(--accent-sapphire)]/50"
      data-testid="order-card"
    >
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 border-b border-[var(--hairline)] pb-3">
        <div className="flex items-center gap-3">
          <span className="text-sm font-semibold tabular-nums text-[var(--accent-sapphire)]">
            #<span data-testid="order-display-id">{order.display_id}</span>
          </span>
          <span className="text-[var(--hairline)]">|</span>
          <span className="text-[var(--text-secondary)]" data-testid="order-created-at">
            {new Date(order.created_at).toLocaleDateString(undefined, {
              year: "numeric",
              month: "short",
              day: "numeric",
            })}
          </span>
        </div>

        <div className="flex items-center gap-3 font-semibold">
          <span className="text-[var(--text-primary)]" data-testid="order-amount">
            {convertToLocale({
              amount: order.total,
              currency_code: order.currency_code,
            })}
          </span>
          <span className="text-[11px] font-normal text-[var(--text-secondary)]">
            ({numberOfLines} {numberOfLines === 1 ? "unit" : "units"})
          </span>
        </div>
      </div>

      {/* Items Preview Rail */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 my-5">
        {order.items?.slice(0, 3).map((item) => (
          <div key={item.id} className="space-y-1.5" data-testid="order-item">
            <div className="h-20 w-20 overflow-hidden rounded-[var(--radius-sm)] border border-[var(--hairline)] bg-[var(--bg-primary)]">
              <Thumbnail thumbnail={item.thumbnail} images={[]} size="square" />
            </div>
            <p className="font-medium text-[var(--text-primary)] truncate" data-testid="item-title">
              {item.title}
            </p>
            <p className="text-[11px] text-[var(--text-secondary)]">
              Qty: <span data-testid="item-quantity">{item.quantity}</span>
            </p>
          </div>
        ))}

        {numberOfProducts > 3 && (
          <div className="flex h-20 w-20 flex-col items-center justify-center rounded-[var(--radius-sm)] border border-[var(--hairline)] bg-[var(--bg-primary)] text-center">
            <span className="font-semibold text-[var(--accent-sapphire)]">
              +{numberOfProducts - 3}
            </span>
            <span className="text-[10px] text-[var(--text-secondary)]">more</span>
          </div>
        )}
      </div>

      <div className="flex justify-end border-t border-[var(--hairline)] pt-3">
        <LocalizedClientLink
          href={`/account/orders/details/${order.id}`}
          className="inline-flex items-center gap-1.5 rounded-full border border-[var(--hairline)] bg-[var(--bg-primary)] px-4 py-2 text-xs font-semibold text-[var(--text-primary)] transition hover:border-[var(--accent-sapphire)] hover:text-[var(--accent-sapphire)]"
          data-testid="order-details-link"
        >
          <span>Examine Order</span>
          <span>→</span>
        </LocalizedClientLink>
      </div>
    </article>
  )
}

export default OrderCard
