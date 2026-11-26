import { getProductPrice } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import Thumbnail from "../thumbnail"
import PreviewPrice from "./price"

export default async function ProductPreview({
  product,
  isFeatured,
  region,
}: {
  product: HttpTypes.StoreProduct
  isFeatured?: boolean
  region: HttpTypes.StoreRegion
}) {
  const { cheapestPrice } = getProductPrice({
    product,
  })

  const isSale = cheapestPrice?.price_type === "sale"
  const variantCount = product.variants?.length || 0

  return (
    <LocalizedClientLink href={`/products/${product.handle}`} className="group block h-full">
      <div 
        data-testid="product-wrapper"
        data-product-card="true"
        className="
          flex flex-col h-full transition-all duration-300
          
          /* Mahal: Glassy floating card */
          group-mahal:bg-brand-surface/60 group-mahal:backdrop-blur-md
          group-mahal:border group-mahal:border-brand-hairline
          group-mahal:rounded-card group-mahal:overflow-hidden
          group-mahal:p-3 group-mahal:shadow-sm
          group-mahal:hover:-translate-y-1 group-mahal:hover:shadow-md

          /* Lustre: Stark minimalist block */
          group-lustre:bg-brand-surface
          group-lustre:border-b group-lustre:border-brand-hairline
          group-lustre:pb-4

          /* Matrix: Cyberpunk border and glow */
          group-matrix:bg-brand-background group-matrix:border group-matrix:border-brand-primary/30
          group-matrix:p-4 group-matrix:hover:border-brand-primary group-matrix:hover:shadow-[0_0_15px_rgba(0,255,0,0.2)]
        "
      >
        {/* Image Container with Badges */}
        <div data-product-image="true" className="
          relative 
          group-mahal:rounded-[12px] group-mahal:overflow-hidden
          group-lustre:grayscale group-lustre:hover:grayscale-0 group-lustre:transition-all
          group-matrix:border group-matrix:border-brand-primary/20
        ">
          <Thumbnail
            thumbnail={product.thumbnail}
            images={product.images}
            size="full"
            isFeatured={isFeatured}
          />

          {/* Floating Badges */}
          <div className="absolute top-3 left-3 flex flex-col gap-1 pointer-events-none">
            {isSale && (
              <span className="
                badge-tag shadow-sm
                group-mahal:bg-brand-error/90 group-mahal:text-brand-surface group-mahal:backdrop-blur-lg
                group-lustre:bg-brand-text group-lustre:text-brand-surface group-lustre:rounded-none group-lustre:uppercase
                group-matrix:bg-brand-error group-matrix:text-brand-background group-matrix:font-mono group-matrix:uppercase group-matrix:border group-matrix:border-brand-error
                bg-brand-error text-brand-surface
              ">
                Sale
              </span>
            )}
            {isFeatured && !isSale && (
              <span className="
                badge-tag shadow-sm
                group-mahal:bg-brand-accent/90 group-mahal:text-brand-primary group-mahal:backdrop-blur-lg
                group-lustre:bg-transparent group-lustre:border group-lustre:border-brand-text group-lustre:text-brand-text group-lustre:rounded-none
                group-matrix:bg-transparent group-matrix:border group-matrix:border-brand-primary group-matrix:text-brand-primary group-matrix:font-mono group-matrix:uppercase
                bg-brand-primary text-brand-surface
              ">
                Atelier Edit
              </span>
            )}
          </div>

          {/* Quick View Pill on Hover - only shown on Default/Mahal, hidden on Lustre/Matrix */}
          <div className="
            absolute bottom-3 inset-x-3 opacity-0 group-hover:opacity-100 transition-all duration-200 
            transform translate-y-1 group-hover:translate-y-0 hidden sm:block
            group-lustre:!hidden group-matrix:!hidden
          ">
            <div className="w-full py-2 bg-brand-surface/95 backdrop-blur-md text-brand-text text-center text-xs font-heading font-semibold uppercase tracking-wider rounded-button shadow-md">
              View Garment
            </div>
          </div>
        </div>

        {/* Product Details */}
        <div className="
          flex flex-col space-y-1
          group-mahal:mt-4
          group-lustre:mt-5
          group-matrix:mt-4
          mt-3.5
        ">
          <div className="flex items-baseline justify-between gap-2">
            <h3
              className="
                text-sm font-heading font-medium text-brand-text transition-colors line-clamp-1
                group-mahal:text-base
                group-lustre:text-xs group-lustre:uppercase group-lustre:tracking-widest group-lustre:font-mono
                group-matrix:text-xs group-matrix:uppercase group-matrix:tracking-widest group-matrix:font-mono group-matrix:text-brand-primary
              "
              data-testid="product-title"
            >
              {product.title}
            </h3>
            <div className="
              group-lustre:text-xs group-lustre:font-mono
              group-matrix:text-xs group-matrix:font-mono group-matrix:text-brand-primary
            ">
              {cheapestPrice && <PreviewPrice price={cheapestPrice} />}
            </div>
          </div>

          {/* Subtitle / Department / Variant indicator */}
          <div className="
            flex items-center justify-between text-[11px] text-brand-secondary font-light
            group-lustre:font-mono group-lustre:tracking-wider
            group-matrix:font-mono group-matrix:tracking-widest group-matrix:text-brand-secondary/70 group-matrix:uppercase
          ">
            <span>
              {product.collection?.title || (product.categories && product.categories[0]?.name) || "Ready to Wear"}
            </span>
            {variantCount > 1 && (
              <span>{variantCount} options</span>
            )}
          </div>
        </div>
      </div>
    </LocalizedClientLink>
  )
}
