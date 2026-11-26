import type { MetadataRoute } from "next"
import { headers } from "next/headers"
import { resolveTenantRequest } from "../core/tenant-resolver"

export const dynamic = "force-dynamic"

export default function robots(): MetadataRoute.Robots {
  const hdrs = headers()
  const rawHost = hdrs.get("x-vendryx-tenant-host") ?? hdrs.get("host")
  const tenant = resolveTenantRequest({ rawHost, route: "/robots.txt" })

  return {
    rules: [
      {
        userAgent: "*",
        allow: "/",
        disallow: ["/checkout", "/api/"],
      },
    ],
    sitemap: `${tenant.canonicalOrigin}/sitemap.xml`,
    host: tenant.canonicalOrigin,
  }
}
