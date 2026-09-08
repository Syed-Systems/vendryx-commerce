import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

export type ProductInfoProps = {
  product: HttpTypes.StoreProduct
}

/**
 * Lustre ProductInfo.
 * Sourced product heading, collection link, and editorial overview.
 * Complies with the standing rule: zero fabricated claims.
 */
const ProductInfo = ({ product }: ProductInfoProps) => {
  return (
    <div id="product-info" className="flex flex-col space-y-3">
      {product.collection && (
        <div>
          <LocalizedClientLink
            href={`/collections/${product.collection.handle}`}
            className="text-xs uppercase tracking-[0.2em] font-semibold text-[var(--accent-sapphire)] hover:underline"
          >
            {product.collection.title}
          </LocalizedClientLink>
        </div>
      )}

      <h1
        data-testid="product-title"
        className="text-2xl sm:text-3xl lg:text-4xl font-semibold tracking-tight text-[var(--text-primary)] leading-tight"
      >
        {product.title}
      </h1>

      {product.description && (
        <p
          data-testid="product-description"
          className="text-sm text-[var(--text-secondary)] leading-relaxed whitespace-pre-line pt-1"
        >
          {product.description}
        </p>
      )}
    </div>
  )
}

export default ProductInfo
