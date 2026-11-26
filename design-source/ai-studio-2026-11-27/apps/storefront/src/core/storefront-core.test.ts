import test from "node:test"
import assert from "node:assert/strict"
import {
  buildHostOnlyCookie,
  buildTenantCacheKey,
  isRegisteredTenantHost,
  normalizeAndValidateHost,
  resolveTenantRequest,
  TENANT_DIRECTORY,
} from "./tenant-resolver"
import { validateThemeConfig } from "./theme-schema"
import {
  calculateLockedCheckout,
  createOrderReceipt,
  formatMinorPrice,
  validateCheckoutForm,
} from "./checkout-engine"

test("Host validation blocks Host Header Injection and resolves registered tenants", () => {
  assert.equal(normalizeAndValidateHost("MaisonVela.com:443"), "maisonvela.com")
  assert.equal(normalizeAndValidateHost("evil.com\r\nX-Injected: 1"), null)
  assert.equal(normalizeAndValidateHost("bad..host.com"), null)
  assert.equal(isRegisteredTenantHost("maisonvela.com"), true)
  assert.equal(isRegisteredTenantHost("kovahydration.com"), true)
  assert.equal(isRegisteredTenantHost("homedine.shop"), true)
  assert.equal(isRegisteredTenantHost("attacker.example.com"), false)

  const maisonPro = resolveTenantRequest({ rawHost: "maisonvela.com" })
  assert.equal(maisonPro.tenantId, "ten_maison_vela_01")
  assert.equal(maisonPro.themeId, "maison-cuir")
  assert.equal(maisonPro.showPlatformBadge, false)

  const kovaFree = resolveTenantRequest({ rawHost: "kova.vendryx.shop" })
  assert.equal(kovaFree.tenantId, "ten_kova_hydro_02")
  assert.equal(kovaFree.themeId, "hydro-vessel")
  assert.equal(kovaFree.plan, "free")
  assert.equal(kovaFree.showPlatformBadge, true)

  const homedinePro = resolveTenantRequest({ rawHost: "homedine.shop" })
  assert.equal(homedinePro.tenantId, "ten_homedine_eco_03")
  assert.equal(homedinePro.themeId, "homedine-eco")
  assert.equal(homedinePro.showPlatformBadge, false)

  const homedineFree = resolveTenantRequest({ rawHost: "homedine.vendryx.shop" })
  assert.equal(homedineFree.tenantId, "ten_homedine_eco_03")
  assert.equal(homedineFree.plan, "free")
  assert.equal(homedineFree.showPlatformBadge, true)

  const homedinePreview = resolveTenantRequest({
    rawHost: "localhost:3000",
    storeQuery: "homedine",
  })
  assert.equal(homedinePreview.tenantId, "ten_homedine_eco_03")
  assert.equal(homedinePreview.themeId, "homedine-eco")
})

test("Tenant cache keys partition strictly by tenant and cookies stay host-only", () => {
  const keyA = buildTenantCacheKey({
    tenantId: "ten_maison_vela_01",
    route: "/products/tote",
    currency: "USD",
    themeVersion: "v1.4.2-live",
  })
  const keyB = buildTenantCacheKey({
    tenantId: "ten_kova_hydro_02",
    route: "/products/tote",
    currency: "USD",
    themeVersion: "v2.1.0-live",
  })
  assert.notEqual(keyA, keyB)
  assert.ok(keyA.includes("ten_maison_vela_01"))
  assert.ok(keyB.includes("ten_kova_hydro_02"))

  const cookie = buildHostOnlyCookie("sf_cart", "cart_123")
  assert.equal(cookie.includes("Domain="), false)
  assert.ok(cookie.includes("HttpOnly"))
  assert.ok(cookie.includes("Secure"))
  assert.ok(cookie.includes("SameSite=Lax"))
})

test("Zod theme schema blocks XSS, javascript: URIs, and schema drift with safe fallback", () => {
  const fallback = TENANT_DIRECTORY["maison-vela"].publishedTheme

  const validCheck = validateThemeConfig(fallback, fallback)
  assert.equal(validCheck.valid, true)
  assert.equal(validCheck.fellBackToDefault, false)

  const xssPayload = {
    ...fallback,
    sections: [
      {
        ...fallback.sections[0],
        headline: "<script>alert(1)</script>Steal",
        ctaHref: "javascript:alert(document.cookie)",
      },
      fallback.sections[1],
    ],
  }
  const rejected = validateThemeConfig(xssPayload, fallback)
  assert.equal(rejected.valid, false)
  assert.equal(rejected.fellBackToDefault, true)
  assert.equal(rejected.config.versionTag, fallback.versionTag)

  const badHexPayload = {
    ...fallback,
    tokens: { ...fallback.tokens, accent: "red; background:url(javascript:1)" },
  }
  const rejectedHex = validateThemeConfig(badHexPayload, fallback)
  assert.equal(rejectedHex.valid, false)
  assert.equal(rejectedHex.fellBackToDefault, true)
})

test("Locked checkout engine uses exact integer math and validates guest fields", () => {
  const lowOrder = calculateLockedCheckout(
    [
      {
        variantId: "kv-loop-cobalt-500",
        productId: "kv-apex-01",
        title: "KOVA Loop Vessel",
        variantLabel: "Cobalt · 500ml",
        imagePath: "/media/bottle-cobalt.jpg",
        unitPriceMinor: 3800,
        quantity: 2,
      },
    ],
    "suburb",
    "cod"
  )
  assert.equal(lowOrder.subtotalMinor, 7600)
  assert.equal(lowOrder.shippingMinor, 900)
  assert.equal(lowOrder.codFeeMinor, 200)
  assert.equal(lowOrder.totalMinor, 8700)
  assert.equal(formatMinorPrice(lowOrder.totalMinor, "USD"), "$87")

  const luxuryOrder = calculateLockedCheckout(
    [
      {
        variantId: "mv-tote-cognac",
        productId: "mv-tote-01",
        title: "Le Cabas Structuré",
        variantLabel: "Saddle Cognac",
        imagePath: "/media/leather-tote.jpg",
        unitPriceMinor: 48500,
        quantity: 1,
      },
    ],
    "nationwide",
    "stripe"
  )
  assert.equal(luxuryOrder.qualifiesForFreeShipping, true)
  assert.equal(luxuryOrder.shippingMinor, 0)
  assert.equal(luxuryOrder.codFeeMinor, 0)
  assert.equal(luxuryOrder.totalMinor, 48500)
  assert.equal(formatMinorPrice(luxuryOrder.totalMinor, "USD"), "$485")

  const badForm = validateCheckoutForm({
    fullName: "A",
    phone: "123",
    zone: "inside_city",
    address: "short",
    paymentMethod: "cod",
  })
  assert.equal(badForm.valid, false)
  assert.ok(badForm.errors.fullName)
  assert.ok(badForm.errors.phone)
  assert.ok(badForm.errors.address)

  const receipt = createOrderReceipt({
    tenantId: "ten_maison_vela_01",
    input: {
      fullName: "Nadia Rahman",
      phone: "+880 1711-842910",
      zone: "inside_city",
      address: "House 14, Road 27, Dhanmondi, Dhaka",
      paymentMethod: "bkash",
    },
    totals: luxuryOrder,
    currency: "USD",
    sequenceSeed: 1088,
  })
  assert.equal(receipt.orderNumber, "#VND-1088")
  assert.equal(receipt.totalFormatted, "$485")
})

test("Homedine catalog filtering, swatch fallback, and carousel wrap-around stay pure", async () => {
  const {
    PRODUCTS,
    filterHomedineProducts,
    getActiveSwatch,
    getNextCarouselIndex,
    getPrevCarouselIndex,
    getVisibleWindow,
  } = await import("../themes/homedine-eco/catalog")

  assert.equal(PRODUCTS.length, 6)
  assert.equal(filterHomedineProducts(PRODUCTS, "kettle").length, 1)
  assert.equal(filterHomedineProducts(PRODUCTS, "", "cupeco").length, 2)
  assert.equal(getActiveSwatch(PRODUCTS[0], "unknown").id, PRODUCTS[0].swatches[0].id)
  assert.equal(getNextCarouselIndex(2, 6, 4), 0)
  assert.equal(getPrevCarouselIndex(0, 6, 4), 2)
  assert.equal(getVisibleWindow(PRODUCTS, 2, 4).length, 4)
})
