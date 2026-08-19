import { expect, test } from "@playwright/test"
import { BACKEND, OWNER_PASSWORD, TENANTS, ownerEmail } from "../playwright.config"

/**
 * Contract tests against the running engine.
 *
 * These exist because the admin API was unauthenticated and the tier gating was never
 * wired, and neither fact was visible from any test in the repository. The suite that
 * nominally covered this printed `✓ [auth] …` and returned `true` without opening a
 * socket.
 *
 * One server now serves every tenant, told apart by Host (ADR-012), so these vary the
 * header rather than the port.
 */

type Tenant = keyof typeof TENANTS

const hostFor = (tenant: Tenant) => ({ Host: TENANTS[tenant] })

async function login(request: any, tenant: Tenant): Promise<string> {
  const response = await request.post(`${BACKEND}/auth/user/emailpass`, {
    headers: hostFor(tenant),
    data: { email: ownerEmail(tenant), password: OWNER_PASSWORD },
  })
  expect(response.ok(), `login failed for ${tenant}`).toBeTruthy()
  const body = await response.json()
  expect(body.token, "login returned no token").toBeTruthy()
  return body.token as string
}

test.describe("admin authentication", () => {
  for (const tenant of Object.keys(TENANTS) as Tenant[]) {
    test(`${tenant}: /admin is closed without a token`, async ({ request }) => {
      const response = await request.get(`${BACKEND}/admin/products`, {
        headers: hostFor(tenant),
      })
      expect(response.status()).toBe(401)
      expect((await response.json()).type).toBe("unauthorized")
    })

    test(`${tenant}: /admin opens with a staff token`, async ({ request }) => {
      const token = await login(request, tenant)
      const response = await request.get(`${BACKEND}/admin/products`, {
        headers: { ...hostFor(tenant), Authorization: `Bearer ${token}` },
      })
      expect(response.status()).toBe(200)
    })

    test(`${tenant}: a customer token is refused by /admin`, async ({ request }) => {
      const email = `buyer-${tenant}-${Date.now()}@stheno.test`
      await request.post(`${BACKEND}/auth/customer/emailpass/register`, {
        headers: hostFor(tenant),
        data: { email, password: OWNER_PASSWORD },
      })
      const auth = await request.post(`${BACKEND}/auth/customer/emailpass`, {
        headers: hostFor(tenant),
        data: { email, password: OWNER_PASSWORD },
      })
      const token = (await auth.json()).token

      const response = await request.get(`${BACKEND}/admin/products`, {
        headers: { ...hostFor(tenant), Authorization: `Bearer ${token}` },
      })
      // 403, not 404: the caller proved who they are and simply is not staff.
      expect(response.status()).toBe(403)
    })
  }

  test("a garbage token is refused, not accepted", async ({ request }) => {
    const response = await request.get(`${BACKEND}/admin/products`, {
      headers: { ...hostFor("free"), Authorization: "Bearer not-a-real-jwt" },
    })
    expect(response.status()).toBe(401)
  })

  test("an unknown host resolves to no tenant", async ({ request }) => {
    // Deliberately 404 and not a fallback to a default tenant: every leak starts as a
    // convenience branch (ADR-012).
    const response = await request.get(`${BACKEND}/admin/products`, {
      headers: { Host: "nobody.localhost" },
    })
    expect(response.status()).toBe(404)
  })

  test("a token from one tenant does not work against another", async ({ request }) => {
    const proToken = await login(request, "pro")

    const response = await request.get(`${BACKEND}/admin/products`, {
      headers: { ...hostFor("free"), Authorization: `Bearer ${proToken}` },
    })

    // 401 rather than 403, and for a reason worth recording: `verify_jwt` resolves the
    // identity by `sub`, and that lookup is itself RLS-scoped — so under Free's tenant
    // Pro's identity does not exist. The database defeats cross-tenant replay without
    // any explicit check in the auth guard.
    expect(response.status()).toBe(401)
  })
})

/**
 * The tier matrix (ENTITLEMENTS.md §8) — the test that catches drift. Tiers are now
 * per-tenant subscription rows rather than per-process config, so this asserts that one
 * process genuinely serves three different plans.
 */
test.describe("tier matrix", () => {
  const expected = {
    trial: { window: 365, maxProducts: undefined, seats: 10, paid: false },
    free: { window: 30, maxProducts: 100, seats: 3, paid: false },
    pro: { window: 365, maxProducts: undefined, seats: 10, paid: true },
  } as const

  for (const tenant of Object.keys(TENANTS) as Tenant[]) {
    const want = expected[tenant]

    test(`${tenant}: analytics horizon is clamped to the entitled window`, async ({
      request,
    }) => {
      const token = await login(request, tenant)
      const response = await request.get(
        `${BACKEND}/admin/reports/revenue-over-time?window_days=365`,
        { headers: { ...hostFor(tenant), Authorization: `Bearer ${token}` } }
      )
      expect(response.status()).toBe(200)
      expect((await response.json()).report.window_days).toBe(want.window)
    })

    test(`${tenant}: quotas and paid features match the tier`, async ({ request }) => {
      const token = await login(request, tenant)
      const headers = { ...hostFor(tenant), Authorization: `Bearer ${token}` }

      const entitlements = await (
        await request.get(`${BACKEND}/admin/entitlements`, { headers })
      ).json()
      expect(entitlements.tier).toBe(tenant)

      const products = entitlements.quotas.find((q: any) => q.key === "max_products")
      const seats = entitlements.quotas.find((q: any) => q.key === "max_staff_seats")
      // Unlimited omits the field rather than reporting 0, which any client that did not
      // know the sentinel would read as "nothing allowed".
      expect(products.limit).toBe(want.maxProducts)
      expect(seats.limit).toBe(want.seats)

      const flags = (
        await (await request.get(`${BACKEND}/admin/feature-flags`, { headers })).json()
      ).feature_flags
      for (const paid of [
        "payment_gateway",
        "courier_dispatch",
        "fraud_score",
        "custom_domain",
      ]) {
        expect(flags[paid], `${tenant}.${paid}`).toBe(want.paid)
      }
    })

    test(`${tenant}: serves the capability flags the dashboard reads`, async ({
      request,
    }) => {
      // The hook treats an absent flag as false, so a name the dashboard asks for and
      // does not get silently hides a working screen.
      const token = await login(request, tenant)
      const flags = (
        await (
          await request.get(`${BACKEND}/admin/feature-flags`, {
            headers: { ...hostFor(tenant), Authorization: `Bearer ${token}` },
          })
        ).json()
      ).feature_flags

      for (const capability of ["rbac", "translation", "view_configurations"]) {
        expect(flags[capability], `${tenant}.${capability}`).toBe(true)
      }
    })
  }
})

/**
 * Endpoints that used to answer from hardcoded literals. Each assertion here is one the
 * old stub would have passed while storing nothing — so they check that what is written
 * comes back, not merely that the route responds.
 */
test.describe("previously stubbed endpoints return real data", () => {
  test("product tags round-trip", async ({ request }) => {
    const token = await login(request, "pro")
    const headers = { ...hostFor("pro"), Authorization: `Bearer ${token}` }
    const value = `tag-${Date.now()}`

    const created = await (
      await request.post(`${BACKEND}/admin/product-tags`, { headers, data: { value } })
    ).json()
    const id = created.product_tag.id

    const list = await (
      await request.get(`${BACKEND}/admin/product-tags`, { headers })
    ).json()
    expect(list.product_tags.some((t: any) => t.value === value)).toBeTruthy()

    await request.delete(`${BACKEND}/admin/product-tags/${id}`, { headers })
    const after = await (
      await request.get(`${BACKEND}/admin/product-tags`, { headers })
    ).json()
    expect(after.product_tags.some((t: any) => t.value === value)).toBeFalsy()
  })

  test("rbac roles come from the database, not three fakes", async ({ request }) => {
    const token = await login(request, "pro")
    const headers = { ...hostFor("pro"), Authorization: `Bearer ${token}` }

    const body = await (await request.get(`${BACKEND}/admin/rbac/roles`, { headers })).json()
    expect(body.count).toBeGreaterThan(0)
    // The stub returned exactly these ids; real roles are keyed by name and carry
    // `is_system_role`, which the fakes never had.
    expect(body.roles.some((r: any) => r.id === "role_admin")).toBeFalsy()
    expect(body.roles.every((r: any) => typeof r.is_system_role === "boolean")).toBeTruthy()
  })

  test("fulfillment providers reflect what was actually registered", async ({
    request,
  }) => {
    const token = await login(request, "pro")
    const headers = { ...hostFor("pro"), Authorization: `Bearer ${token}` }

    const body = await (
      await request.get(`${BACKEND}/admin/fulfillment-providers`, { headers })
    ).json()
    const ids = body.fulfillment_providers.map((p: any) => p.id)
    expect(ids).toContain("manual")
    // The stub advertised a provider this build has never contained.
    expect(ids).not.toContain("shipstation")
  })

  test("locales reflect what is stored", async ({ request }) => {
    const token = await login(request, "pro")
    const headers = { ...hostFor("pro"), Authorization: `Bearer ${token}` }

    const body = await (await request.get(`${BACKEND}/admin/locales`, { headers })).json()
    const codes = body.locales.map((l: any) => l.code)
    // The stub always returned exactly these five, stored or not.
    expect(codes).not.toEqual(["en", "es", "fr", "de", "ar"])
    expect(codes).toContain("en")
  })
})
