/**
 * The value that keeps one tenant's cached responses out of another tenant's requests.
 *
 * Every `force-cache` fetch in `lib/data/` requests a URL that is identical for every
 * merchant — `/store/regions`, `/store/collections`, `/store/products` — against a single
 * `STHENO_BACKEND_URL`. Next's Data Cache matches on URL, method, **headers** and body, so
 * with nothing tenant-varying in the request, one merchant's catalogue is served to the
 * next merchant whose request lands on the process.
 *
 * `store-config.ts` is the only caller that noticed, and it opted out with `no-store` and
 * a comment naming the hazard exactly: "caching it would let one tenant's answer be served
 * to another, which is the shape of a real leak rather than a stale render". That
 * reasoning applies to all eighteen cached fetches, not just that one.
 *
 * # This is a cache discriminator, not a tenant claim
 *
 * It exists to vary the cache key and nothing else. The engine resolves tenancy from the
 * `Host` header and must continue to (ADR-022) — `tenant_layer.rs` calls that "the single
 * assumption the whole scheme rests on". Do not teach the engine to trust this header:
 * it is client-influenced in exactly the way Host is, without the proxy that overwrites it.
 * The name says `cache-scope` rather than `tenant` so that stays obvious.
 *
 * # After ADR-022
 *
 * Once the API is served same-origin, the base URL itself varies per merchant and the
 * cache key varies with it, making this redundant but harmless. It stays as the belt to
 * that change's braces — and as the reason the base URL must never be collapsed back to
 * one shared backend for all tenants.
 */

import { normalizeHost } from "./host-keyed-cache"

export const CACHE_SCOPE_HEADER = "x-stheno-cache-scope"

/**
 * No request context — build, prerender, or a script. There is no tenant to confuse with
 * another there, so a shared bucket is correct rather than merely convenient.
 */
const NO_REQUEST = "__no_request"

/**
 * The current request's host, lowercased and stripped of its port, matching how
 * `tenant_layer.rs` and `middleware.ts` normalise it: `shop.example.com` and
 * `shop.example.com:8443` are one merchant.
 */
export async function getCacheScopeHeader(): Promise<Record<string, string>> {
  try {
    const { headers } = await import("next/headers")
    const host = (await headers()).get("host")

    return { [CACHE_SCOPE_HEADER]: normalizeHost(host, NO_REQUEST) }
  } catch {
    return { [CACHE_SCOPE_HEADER]: NO_REQUEST }
  }
}
