import { useMutation, useQuery } from "@tanstack/react-query"
import { sdk } from "../../lib/client"
import { queryClient } from "../../lib/query-client"

export interface SettlementCycle {
  id: string
  provider_id: string
  courier_cycle_id: string
  cycle_start?: string | null
  cycle_end?: string | null
  total_collected: number | string
  total_fees: number | string
  total_remitted: number | string
  currency: string
  status: "pending" | "reconciled" | "discrepant" | string
  raw_report?: any
  created_at: string
  updated_at: string
}

export interface SettlementLine {
  id: string
  cycle_id: string
  consignment_id: string
  tracking_code?: string | null
  order_id?: string | null
  collected_amount: number | string
  courier_fee: number | string
  cod_fee: number | string
  remitted_amount: number | string
  reconciliation_status: string
  discrepancy_reason?: string | null
  created_at: string
  updated_at: string
}

export interface SettlementAlert {
  lineId: string
  cycleId: string
  consignmentId: string
  trackingCode?: string
  orderId?: string
  unremittedAmount: number
  ageDays: number
  alertLevel: "warning" | "critical"
  message: string
}

export interface ReconciliationReport {
  run_id: string
  cycle_id: string
  provider_id: string
  total_lines_checked: number
  matched_count: number
  collected_not_remitted_count: number
  remitted_not_matched_count: number
  discrepancy_count: number
  total_collected: number | string
  total_fees: number | string
  total_remitted: number | string
  state: string
  buckets: {
    matched: any[]
    collected_not_remitted: any[]
    remitted_not_matched: any[]
    discrepant: any[]
  }
  generated_at: string
}

export const RECONCILIATION_QUERY_KEY = "reconciliation" as const

export const useSettlementCycles = (providerId?: string) => {
  return useQuery({
    queryKey: [RECONCILIATION_QUERY_KEY, "cycles", providerId],
    queryFn: async () => {
      const url = providerId
        ? `/admin/reconciliation/cycles?provider_id=${encodeURIComponent(providerId)}`
        : "/admin/reconciliation/cycles"
      const res = await sdk.client.fetch<{ cycles: SettlementCycle[] }>(url, {
        method: "GET",
      })
      return res.cycles ?? []
    },
  })
}

export const useSettlementCycle = (id?: string) => {
  return useQuery({
    queryKey: [RECONCILIATION_QUERY_KEY, "cycle", id],
    queryFn: async () => {
      if (!id) return null
      const res = await sdk.client.fetch<{ cycle: SettlementCycle }>(
        `/admin/reconciliation/cycles/${id}`,
        { method: "GET" }
      )
      return res.cycle
    },
    enabled: !!id,
  })
}

export const useSettlementCycleLines = (id?: string) => {
  return useQuery({
    queryKey: [RECONCILIATION_QUERY_KEY, "cycle-lines", id],
    queryFn: async () => {
      if (!id) return []
      const res = await sdk.client.fetch<{ lines: SettlementLine[] }>(
        `/admin/reconciliation/cycles/${id}/lines`,
        { method: "GET" }
      )
      return res.lines ?? []
    },
    enabled: !!id,
  })
}

export const useReconciliationAlerts = (id?: string, cycleDays: number = 7) => {
  return useQuery({
    queryKey: [RECONCILIATION_QUERY_KEY, "alerts", id, cycleDays],
    queryFn: async () => {
      if (!id) return []
      const res = await sdk.client.fetch<{ alerts: any[] }>(
        `/admin/reconciliation/cycles/${id}/alerts?cycle_days=${cycleDays}`,
        { method: "GET" }
      )
      return (res.alerts ?? []).map((a: any) => ({
        lineId: a.line_id || a.lineId,
        cycleId: a.cycle_id || a.cycleId,
        consignmentId: a.consignment_id || a.consignmentId,
        trackingCode: a.tracking_code || a.trackingCode,
        orderId: a.order_id || a.orderId,
        unremittedAmount: Number(a.unremitted_amount || a.unremittedAmount || 0),
        ageDays: Number(a.age_days || a.ageDays || 0),
        alertLevel: (a.alert_level || a.alertLevel || "warning") as "warning" | "critical",
        message: a.message,
      })) as SettlementAlert[]
    },
    enabled: !!id,
  })
}

export const useRunReconciliation = () => {
  return useMutation({
    mutationFn: async (cycleId: string) => {
      const res = await sdk.client.fetch<{ report: ReconciliationReport }>(
        `/admin/reconciliation/cycles/${cycleId}/reconcile`,
        { method: "POST" }
      )
      return res.report
    },
    onSuccess: (_, cycleId) => {
      queryClient.invalidateQueries({
        queryKey: [RECONCILIATION_QUERY_KEY, "cycle", cycleId],
      })
      queryClient.invalidateQueries({
        queryKey: [RECONCILIATION_QUERY_KEY, "cycles"],
      })
      queryClient.invalidateQueries({
        queryKey: [RECONCILIATION_QUERY_KEY, "alerts", cycleId],
      })
    },
  })
}

export const useImportSettlementReport = () => {
  return useMutation({
    mutationFn: async (payload: any) => {
      const res = await sdk.client.fetch<{ import: any }>(
        "/admin/reconciliation/import",
        {
          method: "POST",
          body: payload,
        }
      )
      return res.import
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: [RECONCILIATION_QUERY_KEY],
      })
    },
  })
}
