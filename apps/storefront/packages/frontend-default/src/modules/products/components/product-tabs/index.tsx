"use client"

import Back from "@modules/common/icons/back"
import FastDelivery from "@modules/common/icons/fast-delivery"
import Refresh from "@modules/common/icons/refresh"

import Accordion from "./accordion"
import { HttpTypes } from "@medusajs/types"

type ProductTabsProps = {
  product: HttpTypes.StoreProduct
}

const ProductTabs = ({ product }: ProductTabsProps) => {
  const tabs = [
    {
      label: "Technical Specifications",
      component: <ProductInfoTab product={product} />,
    },
    {
      label: "Delivery, Returns & Warranty",
      component: <ShippingInfoTab />,
    },
  ]

  return (
    <div className="w-full">
      <Accordion type="multiple">
        {tabs.map((tab, i) => (
          <Accordion.Item
            key={i}
            title={tab.label}
            headingSize="medium"
            value={tab.label}
          >
            {tab.component}
          </Accordion.Item>
        ))}
      </Accordion>
    </div>
  )
}

const ProductInfoTab = ({ product }: ProductTabsProps) => {
  return (
    <div className="text-small-regular py-6">
      <div className="grid grid-cols-2 gap-x-8 gap-y-4">
        <div className="flex flex-col gap-y-4">
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">Build & Materials</span>
            <p className="text-zinc-600 dark:text-zinc-400">{product.material ? product.material : "Aerospace-grade Aluminum / Glass"}</p>
          </div>
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">Origin & Certification</span>
            <p className="text-zinc-600 dark:text-zinc-400">{product.origin_country ? product.origin_country : "Global Standards Compliant"}</p>
          </div>
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">Device Category</span>
            <p className="text-zinc-600 dark:text-zinc-400">{product.type ? product.type.value : "Pro Hardware"}</p>
          </div>
        </div>
        <div className="flex flex-col gap-y-4">
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">Unit Weight</span>
            <p className="text-zinc-600 dark:text-zinc-400">{product.weight ? `${product.weight} g` : "Standard Weight"}</p>
          </div>
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">Dimensions</span>
            <p className="text-zinc-600 dark:text-zinc-400">
              {product.length && product.width && product.height
                ? `${product.length}mm x ${product.width}mm x ${product.height}mm`
                : "Ultra-compact form factor"}
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

const ShippingInfoTab = () => {
  return (
    <div className="text-small-regular py-6">
      <div className="grid grid-cols-1 gap-y-6">
        <div className="flex items-start gap-x-3">
          <FastDelivery />
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">Express Tracked Delivery</span>
            <p className="max-w-sm text-zinc-600 dark:text-zinc-400 text-xs">
              Fast insured courier dispatch with real-time tracking. Arrives in 1-3 business days.
            </p>
          </div>
        </div>
        <div className="flex items-start gap-x-3">
          <Refresh />
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">2-Year Direct Warranty</span>
            <p className="max-w-sm text-zinc-600 dark:text-zinc-400 text-xs">
              Full hardware replacement or certified repair coverage against all manufacturing defects.
            </p>
          </div>
        </div>
        <div className="flex items-start gap-x-3">
          <Back />
          <div>
            <span className="font-semibold text-zinc-950 dark:text-white">14-Day Effortless Returns</span>
            <p className="max-w-sm text-zinc-600 dark:text-zinc-400 text-xs">
              Not completely satisfied? Return in original condition within 14 days for a full refund.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ProductTabs
