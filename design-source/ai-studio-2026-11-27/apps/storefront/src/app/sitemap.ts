import type { MetadataRoute } from "next"
import { headers } from "next/headers"
import { resolveTenantRequest } from "../core/tenant-resolver"

export const dynamic = "force-dynamic"

export default function sitemap(): MetadataRoute.Sitemap {
  const hdrs = headers()
  const rawHost = hdrs.get("x-vendryx-tenant-host") ?? hdrs.get("host")
  const tenant = resolveTenantRequest({ rawHost, route: "/sitemap.xml" })
  const now = new Date()

  return [
    {
      url: `${tenant.canonicalOrigin}/`,
      lastModified: now,
      changeFrequency: "daily",
      priority: 1,
    },
    ...tenant.catalog.map((product) => ({
      url: `${tenant.canonicalOrigin}/products/${product.handle}`,
      lastModified: now,
      changeFrequency: "weekly" as const,
      priority: 0.85,
    })),
  ]
}
