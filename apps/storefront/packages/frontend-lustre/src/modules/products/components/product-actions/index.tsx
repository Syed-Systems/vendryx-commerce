"use client"

import React, { useEffect, useMemo, useRef, useState } from "react"
import { useParams, usePathname, useRouter, useSearchParams } from "next/navigation"
import { HttpTypes } from "@medusajs/types"
import { addToCart, useIntersection } from "@stheno/storefront-core"
import { isEqual } from "lodash"
import OptionSelect from "./option-select"
import ProductPrice from "../product-price"
import MobileActions from "./mobile-actions"
import Spinner from "@modules/common/icons/spinner"

export type ProductActionsProps = {
  product: HttpTypes.StoreProduct
  region: HttpTypes.StoreRegion
  disabled?: boolean
}

const optionsAsKeymap = (
  variantOptions: HttpTypes.StoreProductVariant["options"]
) => {
  return variantOptions?.reduce((acc: Record<string, string>, varopt: any) => {
    acc[varopt.option_id] = varopt.value
    return acc
  }, {})
}

export default function ProductActions({
  product,
  disabled,
}: ProductActionsProps) {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()

  const [options, setOptions] = useState<Record<string, string | undefined>>({})
  const [isAdding, setIsAdding] = useState(false)
  const countryCode = useParams().countryCode as string

  useEffect(() => {
    if (product.variants?.length === 1) {
      const variantOptions = optionsAsKeymap(product.variants[0].options)
      setOptions(variantOptions ?? {})
    }
  }, [product.variants])

  const selectedVariant = useMemo(() => {
    if (!product.variants || product.variants.length === 0) {
      return undefined
    }

    return product.variants.find((v) => {
      const variantOptions = optionsAsKeymap(v.options)
      return isEqual(variantOptions, options)
    })
  }, [product.variants, options])

  const setOptionValue = (optionId: string, value: string) => {
    setOptions((prev) => ({
      ...prev,
      [optionId]: value,
    }))
  }

  const isValidVariant = useMemo(() => {
    return product.variants?.some((v) => {
      const variantOptions = optionsAsKeymap(v.options)
      return isEqual(variantOptions, options)
    })
  }, [product.variants, options])

  useEffect(() => {
    const params = new URLSearchParams(searchParams.toString())
    const value = isValidVariant ? selectedVariant?.id : null

    if (params.get("v_id") === value) {
      return
    }

    if (value) {
      params.set("v_id", value)
    } else {
      params.delete("v_id")
    }

    router.replace(pathname + "?" + params.toString())
  }, [selectedVariant, isValidVariant])

  const inStock = useMemo(() => {
    if (selectedVariant && !selectedVariant.manage_inventory) {
      return true
    }
    if (selectedVariant?.allow_backorder) {
      return true
    }
    if (
      selectedVariant?.manage_inventory &&
      (selectedVariant?.inventory_quantity || 0) > 0
    ) {
      return true
    }
    return false
  }, [selectedVariant])

  const actionsRef = useRef<HTMLDivElement>(null)
  const inView = useIntersection(actionsRef, "0px")

  const handleAddToCart = async () => {
    if (!selectedVariant?.id) return

    setIsAdding(true)
    try {
      await addToCart({
        variantId: selectedVariant.id,
        quantity: 1,
        countryCode,
      })
    } finally {
      setIsAdding(false)
    }
  }

  const hasOptions = (product.variants?.length ?? 0) > 1

  return (
    <section aria-label="Purchase Options" className="space-y-6" ref={actionsRef}>
      {/* Price header container with frosted glass accent */}
      <div className="rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-4 shadow-sm backdrop-blur-md">
        <ProductPrice product={product} variant={selectedVariant} />
      </div>

      {/* Variant selectors */}
      {hasOptions && (
        <fieldset className="space-y-4 rounded-[var(--radius-md)] border border-[var(--hairline)] bg-[var(--bg-surface)] p-5">
          <legend className="sr-only">Product Variants</legend>
          {(product.options || []).map((option) => (
            <OptionSelect
              key={option.id}
              option={option}
              current={options[option.id]}
              updateOption={setOptionValue}
              title={option.title ?? ""}
              data-testid="product-options"
              disabled={!!disabled || isAdding}
            />
          ))}
        </fieldset>
      )}

      {/* Main purchase CTA */}
      <div className="space-y-3 pt-1">
        <button
          type="button"
          onClick={handleAddToCart}
          disabled={!inStock || !selectedVariant || !!disabled || isAdding || !isValidVariant}
          className="group relative flex h-14 w-full items-center justify-center rounded-full bg-[var(--accent-sapphire)] px-8 text-sm font-semibold tracking-wide text-white shadow-md transition-all duration-200 hover:bg-[var(--accent-sapphire-hover)] hover:shadow-lg disabled:cursor-not-allowed disabled:opacity-40 lustre-btn-press"
          data-testid="add-product-button"
        >
          {isAdding ? (
            <span className="flex items-center gap-2">
              <Spinner className="h-4 w-4 animate-spin text-white" />
              <span>Updating Cart...</span>
            </span>
          ) : !selectedVariant && hasOptions ? (
            "Select Your Specifications"
          ) : !inStock || !isValidVariant ? (
            "Currently Unavailable"
          ) : (
            "Add to Shopping Bag"
          )}
        </button>

      </div>

      {/* Sticky Mobile Bar */}
      <MobileActions
        product={product}
        variant={selectedVariant}
        options={options}
        updateOptions={setOptionValue}
        inStock={inStock}
        handleAddToCart={handleAddToCart}
        isAdding={isAdding}
        show={!inView}
        optionsDisabled={!!disabled || isAdding}
      />
    </section>
  )
}
