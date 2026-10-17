import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

type ProductInfoProps = {
  product: HttpTypes.StoreProduct
}

const ProductInfo = ({ product }: ProductInfoProps) => {
  return (
    <div id="product-info" className="flex flex-col gap-y-3">
      {/* Category / Collection Breadcrumb Badge */}
      <div className="flex items-center gap-2 text-xs uppercase tracking-widest text-[color:var(--ink-faint)] font-medium">
        {product.collection ? (
          <LocalizedClientLink
            href={`/collections/${product.collection.handle}`}
            className="hover:text-[color:var(--ink)] transition-colors text-amber-800/90 font-semibold"
          >
            {product.collection.title}
          </LocalizedClientLink>
        ) : (
          <span>Atelier Collection</span>
        )}
        <span>•</span>
        <span>In Stock</span>
      </div>

      {/* Product Heading */}
      <h1
        className="text-2xl sm:text-3xl lg:text-4xl font-heading font-bold text-[color:var(--ink)] tracking-tight leading-tight"
        data-testid="product-title"
      >
        {product.title}
      </h1>

      {/* Description */}
      {product.description && (
        <p
          className="text-sm text-[color:var(--ink-muted)] font-light leading-relaxed whitespace-pre-line pt-1"
          data-testid="product-description"
        >
          {product.description}
        </p>
      )}
    </div>
  )
}

export default ProductInfo
