"use client"

import React from "react"
import { HttpTypes } from "@medusajs/types"
import { repeat } from "@stheno/storefront-core"
import Item from "@modules/cart/components/item"
import SkeletonLineItem from "@modules/skeletons/components/skeleton-line-item"

export type ItemsPreviewTemplateProps = {
  cart: HttpTypes.StoreCart
}

const ItemsPreviewTemplate: React.FC<ItemsPreviewTemplateProps> = ({ cart }) => {
  const items = cart.items
  const hasOverflow = items && items.length > 4

  return (
    <div className={`overflow-x-auto ${hasOverflow ? "max-h-[420px] overflow-y-auto" : ""}`}>
      <table className="w-full text-left border-collapse">
        <tbody data-testid="items-table">
          {items
            ? items
                .sort((a, b) => ((a.created_at ?? "") > (b.created_at ?? "") ? -1 : 1))
                .map((item) => (
                  <Item
                    key={item.id}
                    item={item}
                    type="preview"
                    currencyCode={cart.currency_code}
                  />
                ))
            : repeat(3).map((i) => <SkeletonLineItem key={i} />)}
        </tbody>
      </table>
    </div>
  )
}

export default ItemsPreviewTemplate
