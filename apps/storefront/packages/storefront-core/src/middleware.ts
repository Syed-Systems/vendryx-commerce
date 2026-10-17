import { HttpTypes } from "@medusajs/types"
import { NextRequest, NextResponse } from "next/server"

import {
  createHostKeyedCache,
  normalizeHost,
} from "./lib/util/host-keyed-cache"

const BACKEND_URL = process.env.STHENO_BACKEND_URL
const PUBLISHABLE_API_KEY = process.env.NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY
const DEFAULT_REGION = process.env.NEXT_PUBLIC_DEFAULT_REGION || "us"

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

async function getRegionMap(host: string) {
  const entry = regionCaches.entryFor(host)
  const regionMap = entry.value
  const regionMapUpdated = entry.updatedAt

  if (!BACKEND_URL) {
    throw new Error(
      "Middleware.ts: Error fetching regions. Did you set up regions in the Stheno dashboard and define a STHENO_BACKEND_URL environment variable?"
    )
  }

  if (
    !regionMap.keys().next().value ||
    regionMapUpdated < Date.now() - 3600 * 1000
  ) {
    // Fetch regions from the backend. We can't use the JS client here because middleware is running on Edge and the client needs a Node environment.
    const { regions } = await fetch(`${BACKEND_URL}/store/regions`, {
      headers: {
        "x-publishable-api-key": PUBLISHABLE_API_KEY!,
      },
      next: {
        revalidate: 3600,
        tags: [`regions-${host}`],
      },
      cache: "force-cache",
    }).then(async (response) => {
      const json = await response.json()

      if (!response.ok) {
        throw new Error(json.message)
      }

      return json
    })

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

  return entry.value
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

    const vercelCountryCode = request.headers
      .get("x-vercel-ip-country")
      ?.toLowerCase()

    const urlCountryCode = request.nextUrl.pathname.split("/")[1]?.toLowerCase()

    if (urlCountryCode && regionMap.has(urlCountryCode)) {
      countryCode = urlCountryCode
    } else if (vercelCountryCode && regionMap.has(vercelCountryCode)) {
      countryCode = vercelCountryCode
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
  const host = normalizeHost(request.headers.get("host"), "__no_host")

  const regionMap = await getRegionMap(host)

  const countryCode = regionMap && (await getCountryCode(request, regionMap))

  const urlHasCountryCode =
    countryCode && request.nextUrl.pathname.split("/")[1].includes(countryCode)

  // If one of the country codes is already in the url, pass through immediately with zero cookie writes (ADR-024)
  if (urlHasCountryCode) {
    return NextResponse.next()
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

  return NextResponse.next()
}

/**
 * No `config` export here on purpose.
 *
 * It used to live beside this handler, and the app shell re-exported it. Next
 * reads the middleware matcher by statically analysing the app-root module, so a
 * re-exported matcher is invisible to it and the middleware runs on every
 * request — including `/_next/image`, which it region-resolved and redirected to
 * `/{countryCode}/_next/image`, a path that is not a route. Every `<Image>` on
 * the site therefore 404'd. `Thumbnail` had been using `next/image` long before
 * this surfaced, so product images were broken too.
 *
 * The matcher is now a literal in `apps/storefront/src/middleware.ts`. A second
 * copy here would be the same bug waiting to be reintroduced.
 */
