import React from "react"
import { getCurrencySymbol } from "../../../lib/data/currencies"
import { Badge, Container, Heading, Text } from "@medusajs/ui"

export interface DiscrepancyAlertItem {
  lineId: string
  cycleId: string
  consignmentId: string
  trackingCode?: string
  orderId?: string
  unremittedAmount: number
  ageDays: number
  alertLevel: string
  message: string
}

export interface DiscrepancyAlertsBannerProps {
  alerts: DiscrepancyAlertItem[]
  currency?: string
}

export const DiscrepancyAlertsBanner: React.FC<DiscrepancyAlertsBannerProps> = ({
  alerts,
  currency = "BDT",
}) => {
  if (!alerts || alerts.length === 0) {
    return null
  }

  return (
    <Container className="p-6 bg-ui-bg-subtle border border-ui-border-error rounded-lg space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Badge color="red">CRITICAL</Badge>
          <Heading level="h3" className="text-base font-semibold text-ui-fg-error">
            {alerts.length} Aged Unsettled Discrepanc{alerts.length === 1 ? "y" : "ies"} Detected
          </Heading>
        </div>
        <Text className="text-xs text-ui-fg-subtle">
          Transactions unremitted longer than cycle threshold
        </Text>
      </div>

      <div className="divide-y divide-ui-border-base">
        {alerts.map((alert) => (
          <div
            key={alert.lineId}
            className="py-2.5 flex flex-col md:flex-row md:items-center justify-between text-xs gap-2"
          >
            <div>
              <span className="font-mono font-medium">{alert.consignmentId}</span>
              {alert.trackingCode && (
                <span className="text-ui-fg-subtle ml-2">({alert.trackingCode})</span>
              )}
              {alert.orderId && (
                <span className="text-ui-fg-subtle ml-2">Order: {alert.orderId}</span>
              )}
              <div className="text-ui-fg-muted mt-0.5">{alert.message}</div>
            </div>
            <div className="flex items-center gap-4 shrink-0">
              <span className="text-ui-fg-error font-medium">
                Unremitted: {getCurrencySymbol(currency)} {alert.unremittedAmount.toFixed(2)}
              </span>
              <Badge color="orange">{alert.ageDays} days overdue</Badge>
            </div>
          </div>
        ))}
      </div>
    </Container>
  )
}
