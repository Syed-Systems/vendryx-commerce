import { useQuery } from "@tanstack/react-query"
import { FetchError } from "@medusajs/js-sdk"
import { sdk } from "../../lib/client"
import { queryKeysFactory } from "../../lib/query-key-factory"

// ─────────────────────────────────────────────────────────────────────────────
// Recent Orders — separate endpoint (Option B)
// ─────────────────────────────────────────────────────────────────────────────
//
// Fetches the 5 most recent orders for the dashboard overview. Uses the
// existing admin orders list endpoint with tight field selection and limit.
//
// This is a read-only convenience query — the full orders page has its own
// paginated, filterable list. This one just shows "the work stream."

export interface RecentOrderItem {
  id: string
  display_id: number
  currency_code: string
  total: string | null
  status: string
  fulfillment_status: string
  payment_status: string
  created_at: string
  /** From shipping_address.first_name — null for orders without an address. */
  shipping_address?: {
    first_name?: string | null
    last_name?: string | null
  } | null
}

interface RecentOrdersResponse {
  orders: RecentOrderItem[]
  count: number
}

const RECENT_ORDERS_QUERY_KEY = "recent-orders" as const
export const recentOrdersQueryKeys = queryKeysFactory(RECENT_ORDERS_QUERY_KEY)

export const useRecentOrders = () => {
  const { data, ...rest } = useQuery<RecentOrdersResponse, FetchError>({
    queryKey: recentOrdersQueryKeys.all,
    queryFn: async () =>
      sdk.client.fetch<RecentOrdersResponse>(
        "/admin/orders?limit=5&offset=0&order=-created_at&fields=id,display_id,currency_code,total,status,fulfillment_status,payment_status,created_at,shipping_address.first_name,shipping_address.last_name",
        { method: "GET" }
      ),
    // Refresh every 2 minutes — orders are the primary real-time concern.
    refetchInterval: 2 * 60 * 1000,
    staleTime: 60 * 1000,
  })

  return { ...rest, orders: data?.orders ?? null, totalCount: data?.count ?? 0 }
}
