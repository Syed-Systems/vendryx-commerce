import type { Metadata } from "next"
import { headers } from "next/headers"
import { StorefrontShell } from "../core/StorefrontShell"
import { resolveTenantRequest } from "../core/tenant-resolver"

export const dynamic = "force-dynamic"

interface HomePageProps {
  searchParams?: {
    store?: string | string[]
    plan?: string | string[]
    mode?: string | string[]
  }
}

function firstParam(value: string | string[] | undefined): string | null {
  if (Array.isArray(value)) return value[0] ?? null
  return value ?? null
}

function resolveCurrentTenant(searchParams?: HomePageProps["searchParams"]) {
  const hdrs = headers()
  const rawHost = hdrs.get("x-vendryx-tenant-host") ?? hdrs.get("host")

  return resolveTenantRequest({
    rawHost,
    storeQuery: firstParam(searchParams?.store),
    planQuery: firstParam(searchParams?.plan),
    modeQuery: firstParam(searchParams?.mode),
    route: "/",
  })
}

export async function generateMetadata({ searchParams }: HomePageProps): Promise<Metadata> {
  const tenant = resolveCurrentTenant(searchParams)

  return {
    title: tenant.seoTitle,
    description: tenant.seoDescription,
    alternates: {
      canonical: tenant.canonicalOrigin,
    },
    openGraph: {
      title: tenant.seoTitle,
      description: tenant.seoDescription,
      url: tenant.canonicalOrigin,
      siteName: tenant.brandName,
      type: "website",
    },
  }
}

export default function StorefrontHomePage({ searchParams }: HomePageProps) {
  const tenant = resolveCurrentTenant(searchParams)
  return <StorefrontShell tenant={tenant} />
}
