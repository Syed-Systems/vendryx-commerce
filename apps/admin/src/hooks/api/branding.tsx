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

    /**
     * Optimistic save.
     *
     * This used to be `onSuccess -> invalidateQueries`, so the preview kept the
     * old colours until the request came back and the cache refetched. Saving
     * your own store's branding is a repeated, low-stakes action where the wait
     * is the whole cost, so the change is applied locally first and the request
     * happens behind it.
     *
     * `cancelQueries` first, or an in-flight refetch can land after the
     * optimistic write and overwrite it with the old value -- the form would
     * flicker back to what the user just changed.
     */
    onMutate: async (payload) => {
      await queryClient.cancelQueries({ queryKey: [BRANDING_QUERY_KEY] })
      const previous = queryClient.getQueryData<TenantBranding>([BRANDING_QUERY_KEY])

      if (previous) {
        queryClient.setQueryData<TenantBranding>([BRANDING_QUERY_KEY], {
          ...previous,
          ...payload,
        })
      }

      return { previous }
    },

    /**
     * A rejected save must not leave the preview showing a state the server
     * refused. Restore exactly what was there before the optimistic write.
     * The caller already surfaces the failure as a toast.
     */
    onError: (_err, _payload, context) => {
      if (context?.previous) {
        queryClient.setQueryData([BRANDING_QUERY_KEY], context.previous)
      }
    },

    /**
     * Reconcile either way. On success this confirms the optimistic value with
     * the server's own copy; after a rollback it re-reads, in case a concurrent
     * write from another tab landed while this one was failing.
     */
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: [BRANDING_QUERY_KEY] })
    },
  })
}
