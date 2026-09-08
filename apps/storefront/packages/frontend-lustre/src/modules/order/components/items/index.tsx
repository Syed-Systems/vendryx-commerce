import React from "react"
import { HttpTypes } from "@medusajs/types"
import { repeat } from "@stheno/storefront-core"
import Item from "@modules/order/components/item"
import SkeletonLineItem from "@modules/skeletons/components/skeleton-line-item"

export type ItemsProps = {
  order: HttpTypes.StoreOrder
}

const Items: React.FC<ItemsProps> = ({ order }) => {
  const items = order.items

  return (
    <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-6">
      <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-4">
        Ordered Items
      </h3>
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <tbody data-testid="products-table">
            {items?.length
              ? items
                  .sort((a, b) => ((a.created_at ?? "") > (b.created_at ?? "") ? -1 : 1))
                  .map((item) => (
                    <Item
                      key={item.id}
                      item={item}
                      currencyCode={order.currency_code}
                    />
                  ))
              : repeat(2).map((i) => <SkeletonLineItem key={i} />)}
          </tbody>
        </table>
      </div>
    </div>
  )
}

export default Items
