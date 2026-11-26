import { CORE_LAYOUT_IDS } from "@medusajs/admin-shared"

import { OrderListTable } from "./components/order-list-table"

import { LayoutComposer } from "../../../components/layout-composer"
import { useFeatureFlag } from "../../../providers/feature-flag-provider"
import { ConfigurableOrderListTable } from "./components/order-list-table/configurable-order-list-table"
import { useLocation } from "react-router-dom"

export const OrderList = () => {
  const isViewConfigEnabled = useFeatureFlag("view_configurations")
  const isSocial = useLocation().pathname.includes("/social")

  return (
    <LayoutComposer
      widgetsZonePrefix="order.list"
      preferredLayoutId={CORE_LAYOUT_IDS.SINGLE_COLUMN}
      hasOutlet={false}
      sections={{
        main: (
          <LayoutComposer.Entry id="OrderListTable">
            {isViewConfigEnabled && !isSocial ? (
              <ConfigurableOrderListTable />
            ) : (
              <OrderListTable />
            )}
          </LayoutComposer.Entry>
        ),
      }}
    />
  )
}
