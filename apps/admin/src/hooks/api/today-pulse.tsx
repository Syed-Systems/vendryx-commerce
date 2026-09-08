import { useQuery } from "@tanstack/react-query"
import { FetchError } from "@medusajs/js-sdk"
import { sdk } from "../../lib/client"
import { queryKeysFactory } from "../../lib/query-key-factory"

// ─────────────────────────────────────────────────────────────────────────────
// Today's Pulse — separate endpoint (Option B)
// ─────────────────────────────────────────────────────────────────────────────
//
// Fetches today's revenue, order count, and items sold from a dedicated
// endpoint. Separate from the home summary so that a failure here does not
// block the attention strip or recent orders.
//
// The backend may not implement this endpoint yet. When it 404s, the hook
// returns null data and isError: true — the component simply hides itself.
// No invented numbers, no fallbacks.

export interface TodayPulse {
  revenue: string | null
  currency_code: string
  order_count: number
  items_sold: number
}

interface TodayPulseResponse {
  today: TodayPulse
}

const TODAY_QUERY_KEY = "today-pulse" as const
export const todayPulseQueryKeys = queryKeysFactory(TODAY_QUERY_KEY)

export const useTodayPulse = () => {
  const { data, ...rest } = useQuery<TodayPulseResponse, FetchError>({
    queryKey: todayPulseQueryKeys.all,
    queryFn: async () =>
      sdk.client.fetch<TodayPulseResponse>("/admin/reports/today", {
        method: "GET",
      }),
    // Refresh every 5 minutes — today's numbers change throughout the day
    // but not so fast that sub-minute polling is justified.
    refetchInterval: 5 * 60 * 1000,
    staleTime: 2 * 60 * 1000,
    // If the endpoint doesn't exist yet, don't retry endlessly.
    retry: 1,
  })

  return { ...rest, today: data?.today ?? null }
}
