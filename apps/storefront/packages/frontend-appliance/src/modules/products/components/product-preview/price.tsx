import { clx } from "@medusajs/ui"
import { VariantPrice } from "types/global"

export default async function PreviewPrice({ price }: { price: VariantPrice }) {
  if (!price) {
    return null
  }

  const isSale = price.price_type === "sale"

  return (
    <div className="flex items-center gap-2 text-xs">
      {isSale && (
        <span
          className="line-through text-zinc-400 font-light"
          data-testid="original-price"
        >
          {price.original_price}
        </span>
      )}
      <span
        className={clx("font-semibold", {
          "text-rose-600": isSale,
          "text-zinc-900": !isSale,
        })}
        data-testid="price"
      >
        {price.calculated_price}
      </span>
    </div>
  )
}
