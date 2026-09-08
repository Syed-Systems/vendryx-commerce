import React, { useState } from "react"
import { Button, Container, Heading, Text } from "@medusajs/ui"
import {
  CourierSettlementSummary,
  CourierSettlementSummaryProps,
} from "./components/reconciliation-summary"
import {
  ReconciliationBucketsView,
  ReconciliationBucketItem,
} from "./components/reconciliation-buckets-view"
import {
  DiscrepancyAlertsBanner,
  DiscrepancyAlertItem,
} from "./components/discrepancy-alerts-banner"

export interface ReconciliationViewProps {
  cycle?: CourierSettlementSummaryProps
  buckets?: {
    matched: ReconciliationBucketItem[]
    collectedNotRemitted: ReconciliationBucketItem[]
    remittedNotMatched: ReconciliationBucketItem[]
    discrepant: ReconciliationBucketItem[]
  }
  alerts?: DiscrepancyAlertItem[]
  onImportReport?: () => void
  onRunReconciliation?: (cycleId: string) => void
}

export const ReconciliationView: React.FC<ReconciliationViewProps> = ({
  cycle,
  buckets = {
    matched: [],
    collectedNotRemitted: [],
    remittedNotMatched: [],
    discrepant: [],
  },
  alerts = [],
  onImportReport,
  onRunReconciliation,
}) => {
  const [reconciling, setReconciling] = useState(false)

  const handleReconcile = async () => {
    if (!cycle) return
    setReconciling(true)
    try {
      await onRunReconciliation?.(cycle.cycleId)
    } finally {
      setReconciling(false)
    }
  }

  return (
    <div className="space-y-6 max-w-7xl mx-auto p-4 md:p-8">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <Heading level="h1" className="text-2xl font-bold">
            Courier COD Settlement & Reconciliation
          </Heading>
          <Text className="text-ui-fg-subtle text-sm">
            Reconcile cash collected by delivery partners against bank remittances and courier fees
          </Text>
        </div>

        <div className="flex items-center gap-3">
          <Button variant="secondary" onClick={onImportReport}>
            Import Settlement Report
          </Button>
          {cycle && (
            <Button
              variant="primary"
              isLoading={reconciling}
              onClick={handleReconcile}
            >
              Reconcile Cycle
            </Button>
          )}
        </div>
      </div>

      {alerts.length > 0 && <DiscrepancyAlertsBanner alerts={alerts} />}

      {cycle ? (
        <>
          <CourierSettlementSummary {...cycle} />
          <ReconciliationBucketsView
            matched={buckets.matched}
            collectedNotRemitted={buckets.collectedNotRemitted}
            remittedNotMatched={buckets.remittedNotMatched}
            discrepant={buckets.discrepant}
            currency={cycle.currency}
          />
        </>
      ) : (
        <Container className="p-12 text-center text-ui-fg-subtle">
          <Heading level="h3" className="text-lg font-semibold">
            No Settlement Cycle Selected
          </Heading>
          <Text className="text-sm mt-1">
            Import a courier payout statement or select a cycle to view reconciliation status.
          </Text>
        </Container>
      )}
    </div>
  )
}
