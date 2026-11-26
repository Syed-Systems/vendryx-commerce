import { useMutation, useQuery } from "@tanstack/react-query"
import { sdk } from "../../lib/client"
import { queryClient } from "../../lib/query-client"

export interface TenantBranding {
  id?: string | null
  store_name?: string | null
  logo_file_id?: string | null
  favicon_file_id?: string | null
  font_key?: string | null
  color_primary?: string | null
  color_secondary?: string | null
  color_accent?: string | null
  color_background?: string | null
  color_surface?: string | null
  color_text?: string | null
  logo_url?: string | null
  favicon_url?: string | null
  updated_at?: string | null
}

export const BRANDING_QUERY_KEY = "branding" as const

export const useBranding = () => {
  return useQuery({
    queryKey: [BRANDING_QUERY_KEY],
    queryFn: async () => {
      const res = await sdk.client.fetch<{ branding: TenantBranding }>(
        "/admin/store/branding",
        { method: "GET" }
      )
      return res.branding
    },
  })
}

export const useUpsertBranding = () => {
  return useMutation({
    mutationFn: async (payload: Partial<TenantBranding>) => {
      const res = await sdk.client.fetch<{ branding: TenantBranding }>(
        "/admin/store/branding",
        {
          method: "POST",
          body: payload,
        }
      )
      return res.branding
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [BRANDING_QUERY_KEY] })
    },
  })
}

export interface StorefrontAppearance {
  storefront_archetype: "baseline" | "editorial" | "dense_catalog"
  product_card_style: "standard" | "editorial" | "dense"
  product_image_aspect: "square" | "portrait" | "landscape"
}

export const APPEARANCE_QUERY_KEY = "storefront-appearance" as const

export const useStorefrontAppearance = () => useQuery({
  queryKey: [APPEARANCE_QUERY_KEY],
  queryFn: async () => {
    const res = await sdk.client.fetch<{ appearance: StorefrontAppearance }>(
      "/admin/store/appearance", { method: "GET" }
    )
    return res.appearance
  },
})

export const useUpsertStorefrontAppearance = () => useMutation({
  mutationFn: async (payload: StorefrontAppearance) => {
    const res = await sdk.client.fetch<{ appearance: StorefrontAppearance }>(
      "/admin/store/appearance", { method: "POST", body: payload }
    )
    return res.appearance
  },
  onSuccess: () => queryClient.invalidateQueries({ queryKey: [APPEARANCE_QUERY_KEY] }),
})
