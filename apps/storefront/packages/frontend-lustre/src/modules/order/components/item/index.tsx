import React from "react"
import { HttpTypes } from "@medusajs/types"
import LineItemOptions from "@modules/common/components/line-item-options"
import LineItemPrice from "@modules/common/components/line-item-price"
import LineItemUnitPrice from "@modules/common/components/line-item-unit-price"
import Thumbnail from "@modules/products/components/thumbnail"

export type ItemProps = {
  item: HttpTypes.StoreCartLineItem | HttpTypes.StoreOrderLineItem
  currencyCode: string
}

const Item: React.FC<ItemProps> = ({ item, currencyCode }) => {
  return (
    <tr
      className="border-b border-[var(--hairline)]"
      data-testid="product-row"
    >
      <td className="py-4 pl-0 pr-4 w-16 align-top">
        <div className="h-16 w-16 overflow-hidden rounded-[var(--radius-sm)] border border-[var(--hairline)] bg-[var(--bg-surface)]">
          <Thumbnail thumbnail={item.thumbnail} size="square" />
        </div>
      </td>

      <td className="py-4 px-2 align-top text-left">
        <span
          className="text-xs font-semibold text-[var(--text-primary)]"
          data-testid="product-name"
        >
          {item.product_title}
        </span>
        <div className="mt-0.5">
          <LineItemOptions
            variant={item.variant}
            title={item.variant_title}
            data-testid="product-variant"
          />
        </div>
      </td>

      <td className="py-4 pl-4 pr-0 align-top text-right">
        <div className="flex flex-col items-end text-xs">
          <span className="text-[var(--text-secondary)]">
            <span data-testid="product-quantity">{item.quantity}</span> ×{" "}
            <LineItemUnitPrice
              item={item}
              style="tight"
              currencyCode={currencyCode}
            />
          </span>
          <span className="font-semibold text-[var(--text-primary)] mt-0.5">
            <LineItemPrice
              item={item}
              style="tight"
              currencyCode={currencyCode}
            />
          </span>
        </div>
      </td>
    </tr>
  )
}

export default Item
