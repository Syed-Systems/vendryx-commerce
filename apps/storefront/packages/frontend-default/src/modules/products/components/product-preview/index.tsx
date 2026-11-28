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
          mahal:bg-[color:color-mix(in_oklab,var(--brand-surface)_60%,transparent)] mahal:backdrop-blur-md
          mahal:border mahal:border-brand-hairline
          mahal:rounded-card mahal:overflow-hidden
          mahal:p-3 mahal:shadow-sm
          mahal:hover:-translate-y-1 mahal:hover:shadow-md

          /* Lustre: Stark minimalist block */
          lustre:bg-brand-surface
          lustre:border-b lustre:border-brand-hairline
          lustre:pb-4

          /* Matrix: Cyberpunk border and glow */
          matrix:bg-brand-background matrix:border matrix:border-[color:color-mix(in_oklab,var(--brand-primary)_30%,transparent)]
          matrix:p-4 matrix:hover:border-brand-primary matrix:hover:shadow-[0_0_15px_rgba(0,255,0,0.2)]
        "
      >
        {/* Image Container with Badges */}
        <div data-product-image="true" className="
          relative
          mahal:rounded-[12px] mahal:overflow-hidden
          lustre:grayscale lustre:hover:grayscale-0 lustre:transition-all
          matrix:border matrix:border-[color:color-mix(in_oklab,var(--brand-primary)_20%,transparent)]
        ">
          <Thumbnail
            thumbnail={product.thumbnail}
            images={product.images}
            size="full"
            isFeatured={isFeatured}
            alt={product.title}
          />

          {/* Floating Badges */}
          <div className="absolute top-3 left-3 flex flex-col gap-1 pointer-events-none">
            {isSale && (
              <span
                className="
                badge-tag shadow-sm
                mahal:bg-[color:color-mix(in_oklab,var(--brand-error)_90%,transparent)] mahal:text-brand-surface mahal:backdrop-blur-lg
                lustre:bg-brand-text lustre:text-brand-surface lustre:rounded-none lustre:uppercase
                matrix:bg-brand-error matrix:text-brand-background matrix:font-mono matrix:uppercase matrix:border matrix:border-brand-error
                bg-brand-error text-brand-surface
              "
              >
                Sale
              </span>
            )}
            {isFeatured && !isSale && (
              <span
                className="
                badge-tag shadow-sm
                mahal:bg-[color:color-mix(in_oklab,var(--brand-accent)_90%,transparent)] mahal:text-brand-primary mahal:backdrop-blur-lg
                lustre:bg-transparent lustre:border lustre:border-brand-text lustre:text-brand-text lustre:rounded-none
                matrix:bg-transparent matrix:border matrix:border-brand-primary matrix:text-brand-primary matrix:font-mono matrix:uppercase
                bg-brand-primary text-brand-surface
              "
              >
                Atelier Edit
              </span>
            )}
          </div>

          {/* Quick-view.

              Three defects fixed here. It was `opacity-0 group-hover:opacity-100`,
              which is unreachable by keyboard and invisible on touch, where
              `:hover` never fires. It invented its own `bg-[color:var(--brand-surface)]/95
              backdrop-blur-md` instead of using the shared glass material. And
              it said "View Garment" while everything around it says "product".

              `.hover-reveal` forces it visible on `(hover: none)` devices;
              `group-focus-visible` reveals it for keyboard users, who land on the
              card link itself. */}
          <div
            className="
              glass-chip hover-reveal
              absolute bottom-3 inset-x-3
              opacity-0 translate-y-1
              group-hover:opacity-100 group-hover:translate-y-0
              group-focus-visible:opacity-100 group-focus-visible:translate-y-0
              transition-all duration-200
              hidden sm:block
              lustre:!hidden matrix:!hidden
            "
          >
            <span className="w-full py-2 block text-brand-text text-center text-xs font-heading font-semibold uppercase tracking-wider">
              View Details
            </span>
          </div>
        </div>

        {/* Product Details */}
        <div className="
          flex flex-col gap-1.5
          mahal:mt-4
          lustre:mt-5
          matrix:mt-4
          mt-3.5
        ">
          <div className="flex items-baseline justify-between gap-2">
            <h3
              className="
                text-sm font-heading font-medium text-brand-text transition-colors line-clamp-1
                mahal:text-base
                lustre:text-xs lustre:uppercase lustre:tracking-widest lustre:font-mono
                matrix:text-xs matrix:uppercase matrix:tracking-widest matrix:font-mono matrix:text-brand-primary
              "
              data-testid="product-title"
            >
              {product.title}
            </h3>
            <div
              className="
                lustre:text-xs lustre:font-mono
                matrix:text-xs matrix:font-mono matrix:text-brand-primary
              "
            >
              {cheapestPrice && <PreviewPrice price={cheapestPrice} />}
            </div>
          </div>

          {/* Subtitle / Department / Variant indicator */}
          <div
            className="
            flex items-center justify-between text-xs text-[color:var(--ink-muted)] font-light
            lustre:font-mono lustre:tracking-wider
            matrix:font-mono matrix:tracking-widest matrix:text-brand-secondary matrix:uppercase
          "
          >
            <span className="line-clamp-1">
              {product.collection?.title || (product.categories && product.categories[0]?.name) || "Ready to Wear"}
            </span>
            {variantCount > 1 && (
              <span className="shrink-0">{variantCount} options</span>
            )}
          </div>
        </div>
      </div>
    </LocalizedClientLink>
  )
}
