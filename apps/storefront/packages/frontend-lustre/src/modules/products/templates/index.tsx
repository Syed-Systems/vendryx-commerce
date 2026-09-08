import React, { Suspense } from "react"
import { notFound } from "next/navigation"
import { HttpTypes } from "@medusajs/types"
import ImageGallery from "../components/image-gallery"
import ProductActions from "../components/product-actions"
import ProductTabs from "../components/product-tabs"
import ProductInfo from "./product-info"
import SkeletonRelatedProducts from "@modules/skeletons/templates/skeleton-related-products"

export type ProductTemplateProps = {
  product: HttpTypes.StoreProduct
  region: HttpTypes.StoreRegion
  countryCode: string
  images: HttpTypes.StoreProductImage[]
  /** Rendered by core: live pricing and the related-products grid. */
  productActions: React.ReactNode
  relatedProducts: React.ReactNode
}

/**
 * Lustre ProductTemplate (Slot 47).
 * Editorial product showcase with dual-column layout and sticky purchase drawer.
 */
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
    <main className="min-h-screen bg-[var(--bg-primary)] py-8 sm:py-16" data-testid="product-container">
      <article className="content-container">
        {/* Main 2-Column Product Grid */}
        <div className="grid grid-cols-1 gap-12 lg:grid-cols-12 lg:gap-16 items-start">
          {/* Gallery Viewport */}
          <section aria-label="Visual Gallery" className="lg:col-span-7">
            <ImageGallery images={images} />
          </section>

          {/* Purchasing & Product Spec Rail */}
          <aside aria-label="Product Details" className="lg:col-span-5 space-y-8 lg:sticky lg:top-24">
            <header className="border-b border-[var(--hairline)] pb-6">
              <ProductInfo product={product} />
            </header>

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

            <div className="border-t border-[var(--hairline)] pt-6">
              <ProductTabs product={product} />
            </div>
          </aside>
        </div>

        {/* Curated Recommendations */}
        <section
          className="mt-24 border-t border-[var(--hairline)] pt-16"
          data-testid="related-products-container"
          aria-label="Related Recommendations"
        >
          <div className="mb-10 text-center sm:text-left">
            <h2 className="text-2xl font-semibold tracking-tight text-[var(--text-primary)] sm:text-3xl">
              Related products
            </h2>
          </div>

          <Suspense fallback={<SkeletonRelatedProducts />}>
            {relatedProducts}
          </Suspense>
        </section>
      </article>
    </main>
  )
}

export default ProductTemplate
