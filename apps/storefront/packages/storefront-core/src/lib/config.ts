import { getCacheScopeHeader } from "@lib/util/cache-scope"
import { getLocaleHeader } from "@lib/util/get-locale-header"
import Medusa, { FetchArgs, FetchInput } from "@medusajs/js-sdk"

// Defaults to the standard Stheno server port
let STHENO_BACKEND_URL = "http://localhost:9000"

if (process.env.STHENO_BACKEND_URL) {
  STHENO_BACKEND_URL = process.env.STHENO_BACKEND_URL
} else if (process.env.STHENO_BUILD_BACKEND_URL) {
  // Docker build only. The build container has no access to the compose network
  // (service names like `engine` do not resolve at build time), so prerender
  // fetches use this loopback/host-gateway URL if one is supplied. Never read
  // at request time: STHENO_BACKEND_URL above always wins when set.
  STHENO_BACKEND_URL = process.env.STHENO_BUILD_BACKEND_URL
}

export const sdk = new Medusa({
  baseUrl: STHENO_BACKEND_URL,
  debug: process.env.NODE_ENV === "development",
  publishableKey: process.env.NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY,
})

const originalFetch = sdk.client.fetch.bind(sdk.client)

sdk.client.fetch = async <T>(
  input: FetchInput,
  init?: FetchArgs
): Promise<T> => {
  const headers = init?.headers ?? {}
  let localeHeader: Record<string, string | null> | undefined
  try {
    localeHeader = await getLocaleHeader()
    headers["x-medusa-locale"] ??= localeHeader["x-medusa-locale"]
  } catch {}

  // Varies the Next Data Cache key by tenant. Every cached fetch below requests a URL
  // identical for every merchant, and the cache matches on URL, method, headers and body —
  // so without this, one merchant's catalogue is served to another the moment one process
  // serves more than one store. A cache discriminator only; the engine still resolves
  // tenancy from Host. See lib/util/cache-scope.ts.
  const cacheScope = await getCacheScopeHeader()

  // Spread last: a caller must not be able to widen its own cache scope.
  const newHeaders = {
    ...localeHeader,
    ...headers,
    ...cacheScope,
  }
  init = {
    ...init,
    headers: newHeaders,
  }
  return originalFetch(input, init)
}
