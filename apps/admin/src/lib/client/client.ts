import Medusa from "@medusajs/js-sdk"
import { getActiveTenantHost } from "../tenant"

export const backendUrl = __BACKEND_URL__ ?? "/"
const authType = __AUTH_TYPE__ ?? "session"
const jwtTokenStorageKey = __JWT_TOKEN_STORAGE_KEY__ || undefined

export const sdk = new Medusa({
  baseUrl: backendUrl,
  auth: {
    type: authType,
    jwtTokenStorageKey,
  },
  /**
   * Names the tenant on every request.
   *
   * The dev proxy resolves the merchant from `x-stheno-tenant-host`, falling back
   * to the `stheno_tenant_host` cookie. Neither reached the engine on its own:
   * the SDK only attaches cookies when `auth.type === "session"`, and this app
   * uses `jwt`, so `credentials` is omitted and the cookie never left the
   * browser. The proxy then forwarded Host: 127.0.0.1, which the engine resolves
   * to ten_pro, so Trial and Free both returned "Invalid credentials" while Pro
   * appeared to work.
   *
   * A getter, not a plain value: the SDK spreads this object into the request
   * headers on every call, and spread invokes getters. That keeps the tenant
   * live, so switching dashboards mid-session takes effect on the next request
   * instead of being frozen at module load.
   */
  globalHeaders: {
    get "x-stheno-tenant-host"() {
      return getActiveTenantHost()
    },
  },
})

// useful when you want to call the BE from the console and try things out quickly
if (typeof window !== "undefined") {
  ;(window as any).__sdk = sdk
}
