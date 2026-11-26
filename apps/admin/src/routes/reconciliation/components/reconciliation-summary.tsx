import React from "react"
import { Badge, Container, Heading, Text } from "@medusajs/ui"
import { getCurrencySymbol } from "../../../lib/data/currencies"

export interface CourierSettlementSummaryProps {
  cycleId: string
  providerId: string
  courierCycleId: string
  totalCollected: number
  totalFees: number
  totalRemitted: number
  currency?: string
  status: "open" | "reconciled" | "discrepant" | "closed"
}

export const CourierSettlementSummary: React.FC<CourierSettlementSummaryProps> = ({
  cycleId,
  providerId,
  courierCycleId,
  totalCollected,
  totalFees,
  totalRemitted,
  currency = "BDT",
  status,
}) => {
  const isBalanced = Math.abs(totalRemitted + totalFees - totalCollected) < 0.001

  return (
    <Container className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Heading level="h2" className="text-xl font-semibold capitalize">
            {providerId} Settlement Cycle
          </Heading>
          <Text className="text-ui-fg-subtle text-sm">
            Invoice / Ref: {courierCycleId} (ID: {cycleId})
          </Text>
        </div>
        <Badge
          color={
            status === "reconciled"
              ? "green"
              : status === "discrepant"
              ? "red"
              : "orange"
          }
        >
          {status.toUpperCase()}
        </Badge>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="p-4 bg-ui-bg-subtle rounded-lg border border-ui-border-base">
          <Text className="text-xs uppercase text-ui-fg-muted font-medium">
            Total Collected at Doorstep
          </Text>
          <Text className="text-2xl font-bold mt-1">
            {getCurrencySymbol(currency)} {totalCollected.toFixed(2)}
          </Text>
          <Text className="text-xs text-ui-fg-subtle mt-1">
            Gross cash collected by courier
          </Text>
        </div>

        <div className="p-4 bg-ui-bg-subtle rounded-lg border border-ui-border-base">
          <Text className="text-xs uppercase text-ui-fg-muted font-medium">
            Courier & COD Fees
          </Text>
          <Text className="text-2xl font-bold mt-1 text-ui-fg-error">
            - {getCurrencySymbol(currency)} {totalFees.toFixed(2)}
          </Text>
          <Text className="text-xs text-ui-fg-subtle mt-1">
            Delivery charges + COD handling fees
          </Text>
        </div>

        <div className="p-4 bg-ui-bg-subtle rounded-lg border border-ui-border-base">
          <Text className="text-xs uppercase text-ui-fg-muted font-medium">
            Net Remittance (Owed to Merchant)
          </Text>
          <Text className="text-2xl font-bold mt-1 text-ui-fg-interactive">
            {getCurrencySymbol(currency)} {totalRemitted.toFixed(2)}
          </Text>
          <Text className="text-xs text-ui-fg-subtle mt-1">
            Net payout owed to merchant account
          </Text>
        </div>
      </div>

      <div
        className={`p-3 rounded-md text-xs flex items-center justify-between border ${
          isBalanced
            ? "bg-ui-bg-base border-ui-border-base text-ui-fg-subtle"
            : "bg-ui-bg-subtle border-ui-border-error text-ui-fg-error font-medium"
        }`}
      >
        <span>
          Accounting Balance Invariant: Remitted ({getCurrencySymbol(currency)} {totalRemitted.toFixed(2)}) + Fees ({getCurrencySymbol(currency)} {totalFees.toFixed(2)}) = Collected ({getCurrencySymbol(currency)} {totalCollected.toFixed(2)})
        </span>
        <Badge color={isBalanced ? "green" : "red"}>
          {isBalanced ? "BALANCED" : "FEE MISMATCH"}
        </Badge>
      </div>
    </Container>
  )
}
