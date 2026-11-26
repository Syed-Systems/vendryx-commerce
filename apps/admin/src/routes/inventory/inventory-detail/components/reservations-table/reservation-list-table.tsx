import { HttpTypes } from "@medusajs/types"
import { Text } from "@medusajs/ui"

type ReservationItemTableProps = {
  inventoryItem: HttpTypes.AdminInventoryItemResponse["inventory_item"]
}

export const ReservationItemTable = ({
  inventoryItem,
}: ReservationItemTableProps) => {
  return (
    <div className="px-6 py-8 text-center">
      <Text size="small" className="text-ui-fg-subtle">
        No active reservations for {inventoryItem.title ?? inventoryItem.sku ?? inventoryItem.id}.
      </Text>
    </div>
  )
}
