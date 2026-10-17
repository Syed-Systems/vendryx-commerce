import React, { useState } from "react"
import { Badge, Button, Container, Heading, Table, Text } from "@medusajs/ui"

export interface ReconciliationBucketItem {
  consignmentId: string
  trackingCode?: string
  orderId?: string
  paymentId?: string
  expectedAmount: number
  collectedAmount: number
  remittedAmount: number
  feeAmount: number
  reason?: string
}

export interface ReconciliationBucketsProps {
  matched: ReconciliationBucketItem[]
  collectedNotRemitted: ReconciliationBucketItem[]
  remittedNotMatched: ReconciliationBucketItem[]
  discrepant: ReconciliationBucketItem[]
  currency?: string
}

type BucketTab = "matched" | "collected_not_remitted" | "remitted_not_matched" | "discrepant"

export const ReconciliationBucketsView: React.FC<ReconciliationBucketsProps> = ({
  matched,
  collectedNotRemitted,
  remittedNotMatched,
  discrepant,
  currency = "BDT",
}) => {
  const [activeTab, setActiveTab] = useState<BucketTab>("matched")

  const getItems = () => {
    switch (activeTab) {
      case "matched":
        return matched
      case "collected_not_remitted":
        return collectedNotRemitted
      case "remitted_not_matched":
        return remittedNotMatched
      case "discrepant":
        return discrepant
    }
  }

  const items = getItems()

  return (
    <Container className="p-6 space-y-4">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b pb-4">
        <div>
          <Heading level="h3" className="text-lg font-semibold">
            Reconciliation Buckets
          </Heading>
          <Text className="text-ui-fg-subtle text-xs">
            Review matched transactions, pending remittances, and discrepancies
          </Text>
        </div>

        <div className="flex flex-wrap gap-2">
          <Button
            size="small"
            variant={activeTab === "matched" ? "primary" : "secondary"}
            onClick={() => setActiveTab("matched")}
          >
            Matched ({matched.length})
          </Button>
          <Button
            size="small"
            variant={activeTab === "collected_not_remitted" ? "primary" : "secondary"}
            onClick={() => setActiveTab("collected_not_remitted")}
          >
            Collected Not Remitted ({collectedNotRemitted.length})
          </Button>
          <Button
            size="small"
            variant={activeTab === "remitted_not_matched" ? "primary" : "secondary"}
            onClick={() => setActiveTab("remitted_not_matched")}
          >
            Remitted Not Matched ({remittedNotMatched.length})
          </Button>
          <Button
            size="small"
            variant={activeTab === "discrepant" ? "primary" : "secondary"}
            onClick={() => setActiveTab("discrepant")}
          >
            Discrepant ({discrepant.length})
          </Button>
        </div>
      </div>

      {items.length === 0 ? (
        <div className="text-center py-12 text-ui-fg-muted">
          <Text className="text-sm">No items in this bucket</Text>
        </div>
      ) : (
        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Consignment / Tracking</Table.HeaderCell>
              <Table.HeaderCell>Order ID</Table.HeaderCell>
              <Table.HeaderCell>Expected</Table.HeaderCell>
              <Table.HeaderCell>Collected</Table.HeaderCell>
              <Table.HeaderCell>Remitted</Table.HeaderCell>
              <Table.HeaderCell>Courier Fees</Table.HeaderCell>
              <Table.HeaderCell>Status / Reason</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {items.map((item, idx) => (
              <Table.Row key={`${item.consignmentId}-${idx}`}>
                <Table.Cell className="font-mono text-xs">
                  <div>{item.consignmentId}</div>
                  {item.trackingCode && (
                    <div className="text-ui-fg-subtle text-xs">{item.trackingCode}</div>
                  )}
                </Table.Cell>
                <Table.Cell className="text-xs">
                  {item.orderId || <span className="text-ui-fg-muted">Unlinked</span>}
                </Table.Cell>
                <Table.Cell className="text-xs">
                  {currency} {item.expectedAmount.toFixed(2)}
                </Table.Cell>
                <Table.Cell className="text-xs font-medium">
                  {currency} {item.collectedAmount.toFixed(2)}
                </Table.Cell>
                <Table.Cell className="text-xs text-ui-fg-interactive font-medium">
                  {currency} {item.remittedAmount.toFixed(2)}
                </Table.Cell>
                <Table.Cell className="text-xs text-ui-fg-subtle">
                  {currency} {item.feeAmount.toFixed(2)}
                </Table.Cell>
                <Table.Cell>
                  {item.reason ? (
                    <span className="text-xs text-ui-fg-error" title={item.reason}>
                      {item.reason}
                    </span>
                  ) : (
                    <Badge color="green">MATCHED</Badge>
                  )}
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table>
      )}
    </Container>
  )
}
