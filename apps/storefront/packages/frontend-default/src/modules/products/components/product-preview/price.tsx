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
          className="line-through text-[color:var(--ink-faint)] font-light"
          data-testid="original-price"
        >
          {price.original_price}
        </span>
      )}
      <span
        className={clx("font-semibold", {
          "text-rose-600": isSale,
          "text-[color:var(--ink)]": !isSale,
        })}
        data-testid="price"
      >
        {price.calculated_price}
      </span>
    </div>
  )
}
