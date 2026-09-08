"use client"

import React, { useState } from "react"
import { HttpTypes } from "@medusajs/types"
import { updateLineItem } from "@stheno/storefront-core"
import CartItemSelect from "@modules/cart/components/cart-item-select"
import DeleteButton from "@modules/common/components/delete-button"
import LineItemOptions from "@modules/common/components/line-item-options"
import LineItemPrice from "@modules/common/components/line-item-price"
import LineItemUnitPrice from "@modules/common/components/line-item-unit-price"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import Spinner from "@modules/common/icons/spinner"
import Thumbnail from "@modules/products/components/thumbnail"
import ErrorMessage from "@modules/common/components/error-message"

export type ItemProps = {
  item: HttpTypes.StoreCartLineItem
  type?: "full" | "preview"
  currencyCode: string
}

const Item: React.FC<ItemProps> = ({ item, type = "full", currencyCode }) => {
  const [updating, setUpdating] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const changeQuantity = async (quantity: number) => {
    setError(null)
    setUpdating(true)

    try {
      await updateLineItem({
        lineId: item.id,
        quantity,
      })
    } catch (err: any) {
      setError(err.message || "Failed to update item quantity.")
    } finally {
      setUpdating(false)
    }
  }

  const maxQuantity = item.variant?.manage_inventory ? 10 : 10

  return (
    <tr
      className="border-b border-[var(--hairline)] transition-colors hover:bg-[var(--bg-surface)]/40"
      data-testid="product-row"
    >
      {/* Thumbnail */}
      <td className="py-4 pl-0 pr-4 align-top w-20 sm:w-24">
        <LocalizedClientLink
          href={`/products/${item.product_handle}`}
          className={`block overflow-hidden rounded-[var(--radius-sm)] border border-[var(--hairline)] bg-[var(--bg-surface)] ${
            type === "preview" ? "w-14 h-14" : "w-20 h-20"
          }`}
        >
          <Thumbnail
            thumbnail={item.thumbnail}
            images={item.variant?.product?.images}
            size="square"
          />
        </LocalizedClientLink>
      </td>

      {/* Description & Options */}
      <td className="py-4 px-2 align-top text-left">
        <LocalizedClientLink
          href={`/products/${item.product_handle}`}
          className="text-sm font-semibold text-[var(--text-primary)] hover:text-[var(--accent-sapphire)] transition"
          data-testid="product-title"
        >
          {item.product_title}
        </LocalizedClientLink>

        <div className="mt-1">
          <LineItemOptions
            variant={item.variant}
            title={item.variant_title}
            data-testid="product-variant"
          />
        </div>

        {type === "full" && (
          <div className="mt-2 text-xs text-[var(--text-secondary)]">
            <LineItemUnitPrice item={item} style="tight" currencyCode={currencyCode} />
          </div>
        )}

        {error && (
          <div className="mt-2">
            <ErrorMessage error={error} data-testid="product-error-message" />
          </div>
        )}
      </td>

      {/* Full Mode: Quantity & Delete */}
      {type === "full" && (
        <td className="py-4 px-3 align-top">
          <div className="flex items-center gap-2">
            <DeleteButton id={item.id} data-testid="product-delete-button" />
            <div className="relative">
              <CartItemSelect
                value={item.quantity}
                onChange={(e) => changeQuantity(parseInt(e.target.value, 10))}
                disabled={updating}
                data-testid="product-select-button"
              >
                {Array.from({ length: Math.min(maxQuantity, 10) }, (_, i) => (
                  <option value={i + 1} key={i}>
                    {i + 1}
                  </option>
                ))}
              </CartItemSelect>
              {updating && (
                <span className="absolute -top-1 -right-1 flex h-3 w-3 items-center justify-center">
                  <Spinner className="h-3 w-3 animate-spin text-[var(--accent-sapphire)]" />
                </span>
              )}
            </div>
          </div>
        </td>
      )}

      {/* Item Price */}
      <td className="py-4 pl-4 pr-0 align-top text-right">
        <div className="font-semibold text-xs text-[var(--text-primary)]">
          <LineItemPrice item={item} style="tight" currencyCode={currencyCode} />
        </div>
      </td>
    </tr>
  )
}

export default Item
