import React, { useMemo } from "react"
import {
  useSettlementCycles,
  useSettlementCycleLines,
  useReconciliationAlerts,
  useRunReconciliation,
} from "../../hooks/api/reconciliation"
import { ReconciliationView } from "./reconciliation-view"
import { ReconciliationBucketItem } from "./components/reconciliation-buckets-view"
import { CourierSettlementSummaryProps } from "./components/reconciliation-summary"

export const ReconciliationPage: React.FC = () => {
  const { data: cycles = [] } = useSettlementCycles()
  const activeCycle = cycles[0]

  const { data: lines = [] } = useSettlementCycleLines(activeCycle?.id)
  const { data: alerts = [] } = useReconciliationAlerts(activeCycle?.id)
  const runReconcileMutation = useRunReconciliation()

  const summaryProps: CourierSettlementSummaryProps | undefined = useMemo(() => {
    if (!activeCycle) return undefined
    return {
      cycleId: activeCycle.id,
      providerId: activeCycle.provider_id,
      courierCycleId: activeCycle.courier_cycle_id,
      totalCollected: Number(activeCycle.total_collected || 0),
      totalFees: Number(activeCycle.total_fees || 0),
      totalRemitted: Number(activeCycle.total_remitted || 0),
      currency: activeCycle.currency || "BDT",
      status: (activeCycle.status as any) || "pending",
    }
  }, [activeCycle])

  const buckets = useMemo(() => {
    const matched: ReconciliationBucketItem[] = []
    const collectedNotRemitted: ReconciliationBucketItem[] = []
    const remittedNotMatched: ReconciliationBucketItem[] = []
    const discrepant: ReconciliationBucketItem[] = []

    for (const line of lines) {
      const item: ReconciliationBucketItem = {
        consignmentId: line.consignment_id,
        trackingCode: line.tracking_code || undefined,
        orderId: line.order_id || undefined,
        expectedAmount: Number(line.collected_amount || 0),
        collectedAmount: Number(line.collected_amount || 0),
        remittedAmount: Number(line.remitted_amount || 0),
        feeAmount: Number(line.courier_fee || 0) + Number(line.cod_fee || 0),
        reason: line.discrepancy_reason || undefined,
      }

      if (line.reconciliation_status === "matched") {
        matched.push(item)
      } else if (line.reconciliation_status === "unremitted" || (item.collectedAmount > 0 && item.remittedAmount === 0)) {
        collectedNotRemitted.push(item)
      } else if (line.reconciliation_status === "unmatched" || !line.order_id) {
        remittedNotMatched.push(item)
      } else {
        discrepant.push(item)
      }
    }

    return { matched, collectedNotRemitted, remittedNotMatched, discrepant }
  }, [lines])

  const handleRunReconciliation = async (cycleId: string) => {
    await runReconcileMutation.mutateAsync(cycleId)
  }

  return (
    <ReconciliationView
      cycle={summaryProps}
      buckets={buckets}
      alerts={alerts}
      onRunReconciliation={handleRunReconciliation}
    />
  )
}
