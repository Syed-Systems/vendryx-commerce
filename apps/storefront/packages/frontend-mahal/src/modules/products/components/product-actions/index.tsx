"use client"

import { useMemo, useState } from "react"
import { useParams } from "next/navigation"
import { addToCart, getProductPrice } from "@stheno/storefront-core"

type RecordLike = Record<string, any>

const optionMap = (variant: RecordLike) =>
  (variant?.options || []).reduce((result: Record<string, string>, option: RecordLike) => {
    result[option.option_id] = option.value
    return result
  }, {})

export default function ProductActions({ product, disabled }: RecordLike) {
  const params = useParams()
  const countryCode = String(params.countryCode || "")
  const [selected, setSelected] = useState<Record<string, string>>(() =>
    product?.variants?.length === 1 ? optionMap(product.variants[0]) : {}
  )
  const [adding, setAdding] = useState(false)

  const variant = useMemo(
    () => product?.variants?.find((candidate: RecordLike) => {
      const candidateOptions = optionMap(candidate)
      return Object.keys(candidateOptions).every((key) => candidateOptions[key] === selected[key])
    }),
    [product?.variants, selected]
  )

  const inStock = Boolean(
    variant && (!variant.manage_inventory || variant.allow_backorder || (variant.inventory_quantity || 0) > 0)
  )
  const price = getProductPrice({ product, variantId: variant?.id })
  const displayedPrice = price.variantPrice?.calculated_price || price.cheapestPrice?.calculated_price

  const add = async () => {
    if (!variant?.id || !countryCode) return
    setAdding(true)
    try {
      await addToCart({ variantId: variant.id, quantity: 1, countryCode })
    } finally {
      setAdding(false)
    }
  }

  return (
    <div className="mahal-product-actions">
      {displayedPrice ? <strong className="action-price">{displayedPrice}</strong> : null}
      {(product?.options || []).map((option: RecordLike) => (
        <fieldset className="variant-group" key={option.id} data-testid="product-options">
          <legend>{option.title}</legend>
          <div className="size-options">
            {(option.values || []).map((value: RecordLike) => {
              const active = selected[option.id] === value.value
              return (
                <button
                  className={active ? "selected" : ""}
                  type="button"
                  aria-pressed={active}
                  key={value.id || value.value}
                  disabled={Boolean(disabled || adding)}
                  onClick={() => setSelected((current) => ({ ...current, [option.id]: value.value }))}
                >
                  {value.value}
                </button>
              )
            })}
          </div>
        </fieldset>
      ))}
      <button
        className="solid-button"
        type="button"
        data-testid="add-product-button"
        disabled={Boolean(disabled || adding || !variant || !inStock)}
        onClick={add}
      >
        {adding ? "ADDING…" : !variant ? "SELECT AN OPTION" : inStock ? "ADD TO BAG" : "UNAVAILABLE"}
      </button>
    </div>
  )
}
