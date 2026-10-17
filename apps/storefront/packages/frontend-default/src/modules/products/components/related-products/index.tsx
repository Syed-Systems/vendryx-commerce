import { HttpTypes } from "@medusajs/types"
import Product from "../product-preview"

/**
 * Presentation only. Core resolves the region and the related products, per
 * ADR-013 decision 4.
 */
export default async function RelatedProducts({
  region,
  products,
}: {
  region: HttpTypes.StoreRegion
  products: HttpTypes.StoreProduct[]
}) {
  if (!products.length) {
    return null
  }

  return (
    <div className="product-page-constraint">
      <div className="flex flex-col items-center text-center mb-16">
        <span className="text-base-regular text-[color:var(--ink-faint)] mb-6">
          Related products
        </span>
        <p className="text-2xl-regular text-ui-fg-base max-w-lg">
          You might also want to check out these products.
        </p>
      </div>

      <ul className="grid grid-cols-2 small:grid-cols-3 medium:grid-cols-4 gap-x-6 gap-y-8">
        {products.map((product) => (
          <li key={product.id}>
            <Product region={region} product={product} />
          </li>
        ))}
      </ul>
    </div>
  )
}
