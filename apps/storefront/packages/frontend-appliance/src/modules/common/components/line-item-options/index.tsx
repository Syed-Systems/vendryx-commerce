import { HttpTypes } from "@medusajs/types"
import { Text } from "@medusajs/ui"

type LineItemOptionsProps = {
  variant: HttpTypes.StoreProductVariant | undefined
  /**
   * The variant name snapshotted onto the line item.
   *
   * Medusa records `variant_title` on the line item precisely so a receipt does not depend
   * on the variant still existing, or on the caller having expanded it. Reading only the
   * nested `variant` meant every cart and checkout row rendered a bare "Variant:" with
   * nothing after it whenever that expansion was absent.
   */
  title?: string | null
  "data-testid"?: string
  "data-value"?: HttpTypes.StoreProductVariant
}

const LineItemOptions = ({
  variant,
  title,
  "data-testid": dataTestid,
  "data-value": dataValue,
}: LineItemOptionsProps) => {
  const label = variant?.title ?? title

  if (!label) {
    return null
  }

  return (
    <Text
      data-testid={dataTestid}
      data-value={dataValue}
      className="inline-block txt-medium text-ui-fg-subtle w-full overflow-hidden text-ellipsis"
    >
      Variant: {label}
    </Text>
  )
}

export default LineItemOptions
