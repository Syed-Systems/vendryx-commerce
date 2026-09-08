import { useQuery, UseQueryOptions } from "@tanstack/react-query"
import { FetchError } from "@medusajs/js-sdk"
import { sdk } from "../../lib/client"
import { queryKeysFactory } from "../../lib/query-key-factory"

// ── Types ────────────────────────────────────────────────────────────────────

export interface PendingOrderItem {
  id: string
  display_id: number
  currency_code: string
  /** null when the order summary has not been computed yet */
  total: string | null
  payment_status: string
  fulfillment_status: string
  created_at: string
}

export interface PendingOrdersSummary {
  count: number
  items: PendingOrderItem[]
}

export interface LowStockItem {
  inventory_item_id: string
  location_id: string
  stocked_quantity: number
  reserved_quantity: number
  available_quantity: number
}

export interface LowStockSummary {
  count: number
  items: LowStockItem[]
  threshold: number
}

export interface HomeSummaryResponse {
  pending_orders: PendingOrdersSummary | null
  low_stock: LowStockSummary | null
  /** Any non-fatal sub-query errors. The client shows partial data + a warning. */
  errors: string[]
}

// ── Query key ─────────────────────────────────────────────────────────────────

const HOME_QUERY_KEY = "home" as const
export const homeQueryKeys = queryKeysFactory(HOME_QUERY_KEY)

// ── Hook ──────────────────────────────────────────────────────────────────────

/**
 * Fetches the merchant owner home summary: pending orders and low-stock alerts.
 *
 * Every value in the response comes from a real service call on the backend.
 * If a sub-query fails the backend still returns whatever succeeded, with the
 * error surfaced in `response.errors`. The hook always shows real data or an
 * explicit loading/error state — never hardcoded or fabricated values.
 */
export const useHomeSummary = (
  options?: Omit<
    UseQueryOptions<HomeSummaryResponse, FetchError>,
    "queryKey" | "queryFn"
  >
) => {
  const { data, ...rest } = useQuery({
    queryKey: homeQueryKeys.all,
    queryFn: async () =>
      sdk.client.fetch<HomeSummaryResponse>("/admin/home/summary", {
        method: "GET",
      }),
    // Refresh every 2 minutes: the cockpit shows time-sensitive operational data.
    refetchInterval: 2 * 60 * 1000,
    staleTime: 60 * 1000,
    ...options,
  })

  return { ...rest, summary: data }
}
