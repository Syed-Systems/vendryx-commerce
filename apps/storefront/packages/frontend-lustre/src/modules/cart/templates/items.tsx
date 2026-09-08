import React from "react"
import { HttpTypes } from "@medusajs/types"
import { repeat } from "@stheno/storefront-core"
import Item from "@modules/cart/components/item"
import SkeletonLineItem from "@modules/skeletons/components/skeleton-line-item"

export type ItemsTemplateProps = {
  cart?: HttpTypes.StoreCart
}

const ItemsTemplate: React.FC<ItemsTemplateProps> = ({ cart }) => {
  const items = cart?.items

  return (
    <div className="w-full">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="border-b border-[var(--hairline)] text-[10px] font-semibold uppercase tracking-widest text-[var(--text-secondary)]">
              <th className="py-3 pl-0 pr-4">Selection</th>
              <th className="py-3 px-2">Specification</th>
              <th className="py-3 px-3">Quantity</th>
              <th className="py-3 pl-4 pr-0 text-right">Subtotal</th>
            </tr>
          </thead>
          <tbody data-testid="items-table">
            {items
              ? items
                  .sort((a, b) => ((a.created_at ?? "") > (b.created_at ?? "") ? -1 : 1))
                  .map((item) => (
                    <Item
                      key={item.id}
                      item={item}
                      type="full"
                      currencyCode={cart?.currency_code || "USD"}
                    />
                  ))
              : repeat(3).map((i) => <SkeletonLineItem key={i} />)}
          </tbody>
        </table>
      </div>
    </div>
  )
}

export default ItemsTemplate
