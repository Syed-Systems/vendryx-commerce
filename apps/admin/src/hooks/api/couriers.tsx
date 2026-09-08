import { useMutation, useQuery } from "@tanstack/react-query"
import { sdk } from "../../lib/client"
import { queryClient } from "../../lib/query-client"

export interface CourierField {
  key: string
  label: string
  kind: "secret" | "text" | "number" | "bool"
  required: boolean
  placeholder?: string
}

export interface CourierRates {
  insideDhaka: number
  subDhaka: number
  outsideDhaka: number
}

export interface CourierProbeRecord {
  verified: boolean
  status: string
  latency_ms: number
  message: string
  probed_at: string
}

export interface CourierProvider {
  id: string
  name: string
  status: "connected" | "needs_verification" | "failed" | "not_configured"
  is_enabled: boolean
  webhookUrl: string
  fields: CourierField[]
  credentials: Record<string, any>
  rates: CourierRates
  last_probe?: CourierProbeRecord | null
}

export interface CourierProbeResponse {
  verified: boolean
  status: string
  latency_ms: number
  message: string
  courier_id: string
}

export const COURIERS_QUERY_KEY = "couriers" as const

export const useCouriers = () => {
  return useQuery({
    queryKey: [COURIERS_QUERY_KEY],
    queryFn: async () => {
      const res = await sdk.client.fetch<{ couriers: any[] }>(
        "/admin/couriers",
        { method: "GET" }
      )
      return (res.couriers || []).map((c) => ({
        id: c.id,
        name: c.name,
        status: c.status,
        is_enabled: c.is_enabled,
        webhookUrl: c.webhook_url,
        fields: c.fields || [],
        credentials: c.credentials || {},
        rates: {
          insideDhaka: Number(c.rates?.inside_dhaka ?? 60),
          subDhaka: Number(c.rates?.sub_dhaka ?? 100),
          outsideDhaka: Number(c.rates?.outside_dhaka ?? 130),
        },
        last_probe: c.last_probe,
      })) as CourierProvider[]
    },
  })
}

export const useUpdateCourierCredentials = () => {
  return useMutation({
    mutationFn: async ({
      id,
      credentials,
      is_enabled,
    }: {
      id: string
      credentials: Record<string, any>
      is_enabled?: boolean
    }) => {
      const res = await sdk.client.fetch<{ courier: any }>(
        `/admin/couriers/${id}`,
        {
          method: "POST",
          body: { credentials, is_enabled },
        }
      )
      return res.courier
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [COURIERS_QUERY_KEY] })
    },
  })
}

export const useProbeCourier = () => {
  return useMutation({
    mutationFn: async (id: string) => {
      const res = await sdk.client.fetch<CourierProbeResponse>(
        `/admin/couriers/${id}/probe`,
        {
          method: "POST",
        }
      )
      return res
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [COURIERS_QUERY_KEY] })
    },
  })
}

export const useUpdateCourierRates = () => {
  return useMutation({
    mutationFn: async ({
      id,
      rates,
    }: {
      id: string
      rates: CourierRates
    }) => {
      const res = await sdk.client.fetch<{ id: string; rates: any }>(
        `/admin/couriers/${id}/rates`,
        {
          method: "POST",
          body: {
            inside_dhaka: rates.insideDhaka,
            sub_dhaka: rates.subDhaka,
            outside_dhaka: rates.outsideDhaka,
          },
        }
      )
      return res
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [COURIERS_QUERY_KEY] })
    },
  })
}
