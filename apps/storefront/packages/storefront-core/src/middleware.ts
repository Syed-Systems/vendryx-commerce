import { HttpTypes } from "@medusajs/types"
import { NextRequest, NextResponse } from "next/server"

import {
  createHostKeyedCache,
  normalizeHost,
} from "./lib/util/host-keyed-cache"

const PUBLISHABLE_API_KEY = process.env.NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY
const DEFAULT_REGION = process.env.NEXT_PUBLIC_DEFAULT_REGION || "us"
const STHENO_BACKEND_URL = process.env.STHENO_BACKEND_URL || "http://localhost:9000"

/**
 * Region maps, **keyed by host**.
 *
 * This was a single module-level Map shared by everything the process served. That is
 * safe only while one process serves one merchant. Under ADR-022 one process serves many,
 * and regions are tenant-owned — migration 20260819000006 made `region.country`
 * tenant-scoped after the same class of mistake returned empty regions and 500'd every
 * page for every tenant but one. Unkeyed, this cache hands one merchant's regions, and
 * their tenant-owned region ids, to the next merchant whose request lands on this process.
 *
 * The host is the tenant (ADR-022), so it is the correct key both before and after that
 * change lands.
 *
 * The partitioning itself lives in `lib/util/host-keyed-cache`, which imports nothing from
 * Next so the isolation guarantee can be tested directly — see
 * `tests/unit/host-keyed-cache.test.ts`.
 */
const regionCaches = createHostKeyedCache(
  () => new Map<string, HttpTypes.StoreRegion>()
)
const regionTenantIds = createHostKeyedCache(() => "")

async function getRegionMap(request: NextRequest) {
  const merchantHost = request.headers.get("host")
  if (!merchantHost) {
    throw new Error("Storefront middleware requires the incoming Host header")
  }
  const host = normalizeHost(merchantHost, "__no_host")
  const entry = regionCaches.entryFor(host)
  const tenantEntry = regionTenantIds.entryFor(host)
  const regionMap = entry.value
  const regionMapUpdated = entry.updatedAt

  if (
    !regionMap.keys().next().value ||
    regionMapUpdated < Date.now() - 3600 * 1000
  ) {
    const configResponse = await fetch(engineRequestUrl("/store/config"), {
      headers: {
        Host: merchantHost,
        "x-publishable-api-key": PUBLISHABLE_API_KEY!,
      },
      cache: "no-store",
    })
    const config = await configResponse.json()
    if (!configResponse.ok) {
      throw new Error(config.message)
    }
    tenantEntry.value = tenantIdFromStoreConfig(config)

    // Fetch regions from the backend. We can't use the JS client here because middleware is running on Edge and the client needs a Node environment.
    const response = await fetch(engineRequestUrl("/store/regions"), {
      headers: {
        Host: merchantHost,
        "x-publishable-api-key": PUBLISHABLE_API_KEY!,
      },
      next: {
        revalidate: 3600,
        tags: [`regions-${host}`],
      },
      cache: "force-cache",
    })
    const json = await response.json()

    if (!response.ok) {
      throw new Error(json.message)
    }

    const regions = json.regions as HttpTypes.StoreRegion[]
    if (!regions?.length) {
      throw new Error(
        "No regions found. Please set up regions in the Stheno dashboard."
      )
    }

    // Create a map of country codes to regions.
    regions.forEach((region: HttpTypes.StoreRegion) => {
      region.countries?.forEach((c) => {
        entry.value.set(c.iso_2 ?? "", region)
      })
    })

    entry.updatedAt = Date.now()
  }

  return { regions: entry.value, tenantId: tenantEntry.value }
}

export function engineRequestUrl(
  pathname: string,
  backendUrl: string = STHENO_BACKEND_URL
): URL {
  return new URL(pathname, backendUrl)
}

export function tenantIdFromStoreConfig(value: unknown): string {
  if (!value || typeof value !== "object" || !("tenant_id" in value)) return ""
  const tenantId = (value as { tenant_id?: unknown }).tenant_id
  return typeof tenantId === "string" && /^[A-Za-z0-9_-]{1,900}$/.test(tenantId)
    ? tenantId
    : ""
}

function tagAtom(value: string): string | null {
  let decoded: string
  try {
    decoded = decodeURIComponent(value)
  } catch {
    return null
  }
  const bytes = new TextEncoder().encode(decoded)
  if (!bytes.length || bytes.length > 900) return null

  let encoded = ""
  bytes.forEach((byte) => {
    if (
      (byte >= 0x30 && byte <= 0x39) ||
      (byte >= 0x41 && byte <= 0x5a) ||
      (byte >= 0x61 && byte <= 0x7a) ||
      byte === 0x5f ||
      byte === 0x2d
    ) {
      encoded += String.fromCharCode(byte)
    } else {
      encoded += `%${byte.toString(16).toUpperCase().padStart(2, "0")}`
    }
  })
  return encoded
}

export function cacheHeadersForPage(
  pathname: string,
  tenantId: string,
  hasCredentials: boolean
): Headers | null {
  if (!tenantId || hasCredentials) return null

  const [, , ...path] = pathname.split("/")
  let tags: string[]
  if (path[0] === "products" && path.length === 2 && path[1]) {
    const product = tagAtom(path[1])
    if (!product) return null
    tags = [`tenant:${tenantId}:product:${product}`]
  } else if (path[0] === "collections" && path.length === 2 && path[1]) {
    const collection = tagAtom(path[1])
    if (!collection) return null
    tags = [
      `tenant:${tenantId}:collection:${collection}`,
      `tenant:${tenantId}:listing`,
    ]
  } else if (
    path.length === 1 &&
    (path[0] === "products" ||
      path[0] === "collections" ||
      path[0] === "categories")
  ) {
    tags = [`tenant:${tenantId}:listing`]
  } else if (path[0] === "categories" && path.length > 1) {
    const category = tagAtom(path[path.length - 1])
    if (!category) return null
    tags = [
      `tenant:${tenantId}:category:${category}`,
      `tenant:${tenantId}:listing`,
    ]
  } else if (
    path.length === 0 ||
    path[0] === "store"
  ) {
    tags = [`tenant:${tenantId}:listing`]
  } else {
    return null
  }

  if (
    tags.some(
      (tag) =>
        tag.length > 1024 ||
        !/^tenant:[A-Za-z0-9_-]+:(?:(?:product|collection|category):(?:[A-Za-z0-9_-]|%[0-9A-F]{2})+|listing)$/.test(
          tag
        )
    )
  ) {
    return null
  }

  const headers = new Headers()
  headers.set("Cache-Control", "public, s-maxage=3600, stale-while-revalidate=86400")
  headers.set("Cache-Tag", tags.join(","))
  headers.set("Surrogate-Key", tags.join(" "))
  headers.set("Vary", "Accept-Encoding")
  return headers
}

export function countryCodeFromHeaders(headers: Headers): string | undefined {
  return headers.get("x-country-code")?.toLowerCase()
}

function continueRequest(request: NextRequest, tenantId: string) {
  const response = NextResponse.next()
  const cacheHeaders = cacheHeadersForPage(
    request.nextUrl.pathname,
    tenantId,
    request.headers.has("cookie") || request.headers.has("authorization")
  )
  cacheHeaders?.forEach((value, key) => response.headers.set(key, value))
  return response
}

/**
 * Fetches regions from the backend and sets the region cookie.
 * @param request
 * @param response
 */
async function getCountryCode(
  request: NextRequest,
  regionMap: Map<string, HttpTypes.StoreRegion | number>
) {
  try {
    let countryCode

    const edgeCountryCode = countryCodeFromHeaders(request.headers)

    const urlCountryCode = request.nextUrl.pathname.split("/")[1]?.toLowerCase()

    if (urlCountryCode && regionMap.has(urlCountryCode)) {
      countryCode = urlCountryCode
    } else if (edgeCountryCode && regionMap.has(edgeCountryCode)) {
      countryCode = edgeCountryCode
    } else if (regionMap.has(DEFAULT_REGION)) {
      countryCode = DEFAULT_REGION
    } else if (regionMap.keys().next().value) {
      countryCode = regionMap.keys().next().value
    }

    return countryCode
  } catch (error) {
    if (process.env.NODE_ENV === "development") {
      console.error(
        "Middleware.ts: Error getting the country code. Did you set up regions in the Stheno dashboard and define a STHENO_BACKEND_URL environment variable?"
      )
    }
  }
}

/**
 * Middleware to handle region selection and onboarding status.
 */
export async function middleware(request: NextRequest) {
  // check if the url is a static asset
  if (request.nextUrl.pathname.includes(".")) {
    return NextResponse.next()
  }

  // The tenant, as far as the storefront can see it (ADR-022). Port is stripped for the
  // same reason tenant_layer.rs strips it: shop.example.com and shop.example.com:8443 are
  // one merchant. Falls back to a constant only so a request with no Host still resolves —
  // that path is single-tenant by definition, since there is nothing to tell tenants apart.
  const { regions: regionMap, tenantId } = await getRegionMap(request)

  const countryCode = regionMap && (await getCountryCode(request, regionMap))

  const urlHasCountryCode =
    countryCode && request.nextUrl.pathname.split("/")[1].includes(countryCode)

  // If one of the country codes is already in the url, pass through immediately with zero cookie writes (ADR-024)
  if (urlHasCountryCode) {
    return continueRequest(request, tenantId)
  }

  const redirectPath =
    request.nextUrl.pathname === "/" ? "" : request.nextUrl.pathname

  const queryString = request.nextUrl.search ? request.nextUrl.search : ""

  // If no country code is set, we redirect to the relevant region.
  if (!urlHasCountryCode && countryCode) {
    const redirectUrl = `${request.nextUrl.origin}/${countryCode}${redirectPath}${queryString}`
    return NextResponse.redirect(redirectUrl, 307)
  } else if (!urlHasCountryCode && !countryCode) {
    // Handle case where no valid country code exists (empty regions)
    return new NextResponse(
      "No valid regions configured. Please set up regions with countries in the Stheno dashboard.",
      { status: 500 }
    )
  }

  return continueRequest(request, tenantId)
}

export const config = {
  matcher: [
    "/((?!api|_next/static|_next/image|favicon.ico|images|assets|png|svg|jpg|jpeg|gif|webp).*)",
  ],
}
