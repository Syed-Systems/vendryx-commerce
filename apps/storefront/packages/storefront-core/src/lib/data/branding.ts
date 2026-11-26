"use server"

import { sdk } from "@lib/config"
import type { TenantBranding } from "../branding"

/** Reads the current tenant's persisted branding for server-rendered token injection. */
export async function getStoreBranding(host: string | null): Promise<TenantBranding | null> {
  if (!host) return null

  try {
    const { branding } = await sdk.client.fetch<{ branding: TenantBranding }>(
      "/store/branding",
      {
        method: "GET",
        // The engine resolves the tenant from Host; forward the request host through
        // the storefront proxy rather than using the shared backend URL's host.
        headers: { host },
        cache: "no-store",
      }
    )
    return branding
  } catch {
    return null
  }
}
