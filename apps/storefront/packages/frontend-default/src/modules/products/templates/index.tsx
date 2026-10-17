import React, { Suspense } from "react"
import ImageGallery from "@modules/products/components/image-gallery"
import ProductActions from "@modules/products/components/product-actions"
import ProductTabs from "@modules/products/components/product-tabs"
import ProductInfo from "@modules/products/templates/product-info"
import SkeletonRelatedProducts from "@modules/skeletons/templates/skeleton-related-products"
import { notFound } from "next/navigation"
import { HttpTypes } from "@medusajs/types"


type ProductTemplateProps = {
  product: HttpTypes.StoreProduct
  region: HttpTypes.StoreRegion
  countryCode: string
  images: HttpTypes.StoreProductImage[]
  /** Rendered by core: live pricing and the related-products grid. */
  productActions: React.ReactNode
  relatedProducts: React.ReactNode
}

const ProductTemplate: React.FC<ProductTemplateProps> = ({
  product,
  region,
  countryCode,
  images,
  productActions,
  relatedProducts,
}) => {
  if (!product || !product.id) {
    return notFound()
  }

  return (
    <>
      <div
        className="content-container py-8 sm:py-12"
        data-testid="product-container"
      >
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-14 items-start">
          {/* Left Column: Image Gallery */}
          <div className="lg:col-span-7 w-full">
            <ImageGallery images={images} />
          </div>

          {/* Right Column: Sticky Product Purchase Panel */}
          <div className="lg:col-span-5 flex flex-col gap-y-6 lg:sticky lg:top-28">
            {/* Header info: Collection, Title, Description */}
            <ProductInfo product={product} />

            {/* Actions: Variants, Price, Add to Bag, Mobile Drawer */}
            <Suspense
              fallback={
                <ProductActions
                  disabled={true}
                  product={product}
                  region={region}
                />
              }
            >
              {productActions}
            </Suspense>

            {/* Accordion Tabs: Specs, Materials, Shipping */}
            <div className="pt-4 border-t border-[color:var(--brand-hairline)]">
              <ProductTabs product={product} />
            </div>
          </div>
        </div>
      </div>

      {/* Related Products Rail */}
      <div
        className="content-container my-16 sm:my-28 border-t border-[color:var(--brand-hairline)] pt-16"
        data-testid="related-products-container"
      >
        <div className="mb-8">
          <span className="text-xs uppercase tracking-[0.2em] text-[color:var(--ink-faint)] font-semibold block mb-1">
            Complete the Look
          </span>
          <h2 className="text-2xl sm:text-3xl font-heading font-bold text-[color:var(--ink)]">
            You May Also Admire
          </h2>
        </div>
        <Suspense fallback={<SkeletonRelatedProducts />}>
          {relatedProducts}
        </Suspense>
      </div>
    </>
  )
}

export default ProductTemplate
