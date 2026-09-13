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
        className="
          flex flex-col h-full transition-all duration-300
          
          /* Mahal: Glassy floating card */
          group-data-[theme=mahal]:bg-brand-surface/60 group-data-[theme=mahal]:backdrop-blur-md 
          group-data-[theme=mahal]:border group-data-[theme=mahal]:border-brand-hairline 
          group-data-[theme=mahal]:rounded-card group-data-[theme=mahal]:overflow-hidden
          group-data-[theme=mahal]:p-3 group-data-[theme=mahal]:shadow-sm
          group-data-[theme=mahal]:hover:-translate-y-1 group-data-[theme=mahal]:hover:shadow-md

          /* Lustre: Stark minimalist block */
          group-data-[theme=lustre]:bg-brand-surface
          group-data-[theme=lustre]:border-b group-data-[theme=lustre]:border-brand-hairline
          group-data-[theme=lustre]:pb-4

          /* Matrix: Cyberpunk border and glow */
          group-data-[theme=matrix]:bg-brand-background group-data-[theme=matrix]:border group-data-[theme=matrix]:border-brand-primary/30
          group-data-[theme=matrix]:p-4 group-data-[theme=matrix]:hover:border-brand-primary group-data-[theme=matrix]:hover:shadow-[0_0_15px_rgba(0,255,0,0.2)]
        "
      >
        {/* Image Container with Badges */}
        <div className="
          relative 
          group-data-[theme=mahal]:rounded-[12px] group-data-[theme=mahal]:overflow-hidden
          group-data-[theme=lustre]:grayscale group-data-[theme=lustre]:hover:grayscale-0 group-data-[theme=lustre]:transition-all
          group-data-[theme=matrix]:border group-data-[theme=matrix]:border-brand-primary/20
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
                group-data-[theme=mahal]:bg-brand-error/90 group-data-[theme=mahal]:text-brand-surface group-data-[theme=mahal]:backdrop-blur-lg
                group-data-[theme=lustre]:bg-brand-text group-data-[theme=lustre]:text-brand-surface group-data-[theme=lustre]:rounded-none group-data-[theme=lustre]:uppercase
                group-data-[theme=matrix]:bg-brand-error group-data-[theme=matrix]:text-brand-background group-data-[theme=matrix]:font-mono group-data-[theme=matrix]:uppercase group-data-[theme=matrix]:border group-data-[theme=matrix]:border-brand-error
                bg-brand-error text-brand-surface
              ">
                Sale
              </span>
            )}
            {isFeatured && !isSale && (
              <span className="
                badge-tag shadow-sm
                group-data-[theme=mahal]:bg-brand-accent/90 group-data-[theme=mahal]:text-brand-primary group-data-[theme=mahal]:backdrop-blur-lg
                group-data-[theme=lustre]:bg-transparent group-data-[theme=lustre]:border group-data-[theme=lustre]:border-brand-text group-data-[theme=lustre]:text-brand-text group-data-[theme=lustre]:rounded-none
                group-data-[theme=matrix]:bg-transparent group-data-[theme=matrix]:border group-data-[theme=matrix]:border-brand-primary group-data-[theme=matrix]:text-brand-primary group-data-[theme=matrix]:font-mono group-data-[theme=matrix]:uppercase
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
            group-data-[theme=lustre]:!hidden group-data-[theme=matrix]:!hidden
          ">
            <div className="w-full py-2 bg-brand-surface/95 backdrop-blur-md text-brand-text text-center text-xs font-heading font-semibold uppercase tracking-wider rounded-button shadow-md">
              View Garment
            </div>
          </div>
        </div>

        {/* Product Details */}
        <div className="
          flex flex-col space-y-1
          group-data-[theme=mahal]:mt-4
          group-data-[theme=lustre]:mt-5
          group-data-[theme=matrix]:mt-4
          mt-3.5
        ">
          <div className="flex items-baseline justify-between gap-2">
            <h3
              className="
                text-sm font-heading font-medium text-brand-text transition-colors line-clamp-1
                group-data-[theme=mahal]:text-base
                group-data-[theme=lustre]:text-xs group-data-[theme=lustre]:uppercase group-data-[theme=lustre]:tracking-widest group-data-[theme=lustre]:font-mono
                group-data-[theme=matrix]:text-xs group-data-[theme=matrix]:uppercase group-data-[theme=matrix]:tracking-widest group-data-[theme=matrix]:font-mono group-data-[theme=matrix]:text-brand-primary
              "
              data-testid="product-title"
            >
              {product.title}
            </h3>
            <div className="
              group-data-[theme=lustre]:text-xs group-data-[theme=lustre]:font-mono
              group-data-[theme=matrix]:text-xs group-data-[theme=matrix]:font-mono group-data-[theme=matrix]:text-brand-primary
            ">
              {cheapestPrice && <PreviewPrice price={cheapestPrice} />}
            </div>
          </div>

          {/* Subtitle / Department / Variant indicator */}
          <div className="
            flex items-center justify-between text-[11px] text-brand-secondary font-light
            group-data-[theme=lustre]:font-mono group-data-[theme=lustre]:tracking-wider
            group-data-[theme=matrix]:font-mono group-data-[theme=matrix]:tracking-widest group-data-[theme=matrix]:text-brand-secondary/70 group-data-[theme=matrix]:uppercase
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
