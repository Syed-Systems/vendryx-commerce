import { getProductPrice } from "@stheno/storefront-core"
import { HttpTypes } from "@medusajs/types"
import MatrixProductCard from "./card"

/**
 * Server half of the ported card: core resolves the price, the client half owns
 * the markup and the pointer interactions. Money arrives already formatted —
 * FRONTEND_CONTRACT_INPUTS §6.3 — so nothing here does arithmetic on a total.
 */
export default async function ProductPreview({
  product,
  isFeatured,
  region,
}: {
  product: HttpTypes.StoreProduct
  isFeatured?: boolean
  region: HttpTypes.StoreRegion
}) {
  const { cheapestPrice } = getProductPrice({ product })

  const isSale = cheapestPrice?.price_type === "sale"
  const images = product.images ?? []
  const initialImage = product.thumbnail || images[0]?.url
  const hoverImage = images.length > 1 ? images[1].url : null

  // The source card computed its own discount from two numbers. Core already
  // carries the difference, so the badge reads real data instead of recomputing
  // it — and never touches a decimal (ADR-006).
  const discountLabel =
    isSale && cheapestPrice?.percentage_diff
      ? `-${cheapestPrice.percentage_diff}%`
      : null

  return (
    <MatrixProductCard
      href={`/products/${product.handle}`}
      title={product.title}
      collection={
        product.collection?.title ||
        product.categories?.[0]?.name ||
        "Reference"
      }
      initialImage={initialImage}
      hoverImage={hoverImage}
      price={cheapestPrice?.calculated_price}
      originalPrice={isSale ? cheapestPrice?.original_price : undefined}
      discountLabel={discountLabel}
      isFeatured={isFeatured}
    />
  )
}
