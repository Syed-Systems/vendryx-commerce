import { NextResponse, type NextRequest } from "next/server"
import { normalizeAndValidateHost } from "./core/tenant-resolver"

/**
 * Edge/Request-level Tenant Host Guard.
 *
 * Defends against Host Header Injection and multi-tenant cache poisoning:
 * 1. Normalizes & validates the incoming Host header strictly against RFC hostname characters.
 * 2. Rejects malformed/injected Host headers with HTTP 400 before SSR executes.
 * 3. Propagates validated hostname to SSR and appends Vary: Host + baseline security headers.
 */
export function middleware(request: NextRequest) {
  const rawHost = request.headers.get("host")
  const normalized = normalizeAndValidateHost(rawHost)

  if (rawHost && !normalized) {
    return new NextResponse("Invalid Host header", { status: 400 })
  }

  const requestHeaders = new Headers(request.headers)
  if (normalized) {
    requestHeaders.set("x-vendryx-tenant-host", normalized)
  }

  const response = NextResponse.next({
    request: { headers: requestHeaders },
  })

  response.headers.set("Vary", "Host")
  response.headers.set("X-Content-Type-Options", "nosniff")
  response.headers.set("Referrer-Policy", "strict-origin-when-cross-origin")

  return response
}

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
}
