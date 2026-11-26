import { expect, test } from "vitest"

import {
  cacheHeadersForPage,
  countryCodeFromHeaders,
  engineRequestUrl,
  tenantIdFromStoreConfig,
} from "../../packages/storefront-core/src/middleware"

test("tenant id is read from the explicit store config field", () => {
  expect(tenantIdFromStoreConfig({ tenant_id: "ten_abc" })).toBe("ten_abc")
  expect(tenantIdFromStoreConfig({ tenant_id: "" })).toBe("")
  expect(tenantIdFromStoreConfig({ tenant_id: "ten other" })).toBe("")
  expect(tenantIdFromStoreConfig({})).toBe("")
})

test("internal engine fetches use the configured backend URL, not the request origin", () => {
  const requestOrigin = "https://shop.example.test"
  const configuredBackendUrl = "http://proxy.internal:8080"
  const url = engineRequestUrl("/store/config", configuredBackendUrl)

  expect(url.toString()).toBe("http://proxy.internal:8080/store/config")
  expect(url.origin).not.toBe(requestOrigin)
})

test("public product and catalog pages receive tenant-scoped cache tags", () => {
  const product = cacheHeadersForPage("/bd/products/blue-shirt", "ten_abc", false)
  const products = cacheHeadersForPage("/bd/products", "ten_abc", false)
  const listing = cacheHeadersForPage("/bd/collections/summer", "ten_abc", false)

  expect(product?.get("Cache-Tag")).toBe("tenant:ten_abc:product:blue-shirt")
  expect(product?.get("Surrogate-Key")).toBe("tenant:ten_abc:product:blue-shirt")
  expect(products?.get("Cache-Tag")).toBe("tenant:ten_abc:listing")
  expect(listing?.get("Cache-Tag")).toBe(
    "tenant:ten_abc:collection:summer,tenant:ten_abc:listing"
  )
  expect(listing?.get("Cache-Control")).toContain("public, s-maxage=")
  expect(product?.has("Set-Cookie")).toBe(false)
  expect(product?.get("Vary")).toBe("Accept-Encoding")
})

test("private requests and non-catalog pages are not marked cacheable", () => {
  expect(cacheHeadersForPage("/bd/cart", "ten_abc", false)).toBeNull()
  expect(cacheHeadersForPage("/bd/products/blue-shirt", "ten_abc", true)).toBeNull()
  expect(cacheHeadersForPage("/bd/products/blue-shirt", "", false)).toBeNull()
})

test("Bangla product handles use the same deterministic ASCII tag atom", () => {
  const handle = "শার্ট"
  const product = cacheHeadersForPage(
    `/bd/products/${encodeURIComponent(handle)}`,
    "ten_abc",
    false
  )

  expect(product?.get("Cache-Tag")).toBe(
    "tenant:ten_abc:product:%E0%A6%B6%E0%A6%BE%E0%A6%B0%E0%A7%8D%E0%A6%9F"
  )
  expect(product?.get("Surrogate-Key")).toBe(product?.get("Cache-Tag"))
})

test("country selection reads the vendor-neutral request header", () => {
  expect(countryCodeFromHeaders(new Headers({ "x-country-code": "BD" }))).toBe(
    "bd"
  )
})
