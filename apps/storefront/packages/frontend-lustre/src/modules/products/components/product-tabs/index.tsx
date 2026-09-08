"use client"

import React from "react"
import { HttpTypes } from "@medusajs/types"
import Accordion from "./accordion"

export type ProductTabsProps = {
  product: HttpTypes.StoreProduct
}

const ProductTabs: React.FC<ProductTabsProps> = ({ product }) => {
  const details = [
    product.material && { label: "Material", value: product.material },
    product.origin_country && {
      label: "Country of origin",
      value: product.origin_country,
    },
    product.type?.value && { label: "Type", value: product.type.value },
    product.weight && { label: "Weight", value: `${product.weight} g` },
    product.length && product.width && product.height
      ? {
          label: "Dimensions",
          value: `${product.length} x ${product.width} x ${product.height}`,
        }
      : null,
  ].filter(Boolean) as Array<{ label: string; value: string }>

  if (details.length === 0) {
    return null
  }

  return (
    <div className="w-full">
      <Accordion type="multiple" defaultValue={["details"]}>
        <Accordion.Item value="details" title="Product details">
          <dl className="grid grid-cols-1 gap-4 text-xs sm:grid-cols-2">
            {details.map((detail) => (
              <div key={detail.label}>
                <dt className="font-medium text-[var(--text-primary)]">
                  {detail.label}
                </dt>
                <dd className="text-[var(--text-secondary)]">{detail.value}</dd>
              </div>
            ))}
          </dl>
        </Accordion.Item>
      </Accordion>
    </div>
  )
}

export default ProductTabs
