import { expect, test } from "@playwright/test"
import { BACKEND, OWNER_PASSWORD, TENANTS, ownerEmail } from "../playwright.config"

/**
 * A product created in the admin, priced and stocked, and then bought from the storefront.
 *
 * This journey could not complete at all. Three separate breaks sat on it, each invisible
 * from either end alone:
 *
 * 1. `POST /admin/products/{id}/variants` deserialized `prices` and `options` and stored
 *    neither. A merchant saw a 200 and had nothing saved.
 * 2. `GET /store/products` returned a flat product — no variants, no options, no prices —
 *    so the storefront could not render a price or resolve a selection to a variant.
 * 3. `POST /store/carts/{id}/line-items` took `unit_price` from the request body and
 *    defaulted it to zero, so the price a shopper paid was whatever they sent.
 *
 * Each of those is individually testable, and each was individually invisible: the admin
 * returned success, the storefront returned a product, and the cart returned a cart. Only
 * running the whole path shows it.
 */

type Tenant = keyof typeof TENANTS

const hostFor = (tenant: Tenant) => ({ Host: TENANTS[tenant] })

async function login(request: any, tenant: Tenant): Promise<string> {
  const response = await request.post(`${BACKEND}/auth/user/emailpass`, {
    headers: hostFor(tenant),
    data: { email: ownerEmail(tenant), password: OWNER_PASSWORD },
  })
  expect(response.ok(), `login failed for ${tenant}`).toBeTruthy()
  return (await response.json()).token as string
}

test.describe("admin to storefront round trip", () => {
  test("a product priced in the admin is priced on the storefront", async ({
    request,
  }) => {
    const tenant: Tenant = "pro"
    const token = await login(request, tenant)
    const admin = { ...hostFor(tenant), Authorization: `Bearer ${token}` }

    // Unique per run so repeated runs do not collide on the handle.
    const suffix = Date.now().toString(36)
    const handle = `roundtrip-${suffix}`

    const created = await request.post(`${BACKEND}/admin/products`, {
      headers: admin,
      data: { title: `Round Trip ${suffix}`, handle },
    })
    expect(created.status(), await created.text()).toBe(201)
    const productId = (await created.json()).product.id

    // An option, then a value, then a variant that names it. A variant with no options
    // cannot be selected on a product page that has any.
    const option = await request.post(
      `${BACKEND}/admin/products/${productId}/options`,
      { headers: admin, data: { title: "Size", values: ["M"] } }
    )
    expect(option.ok(), await option.text()).toBeTruthy()

    const variant = await request.post(
      `${BACKEND}/admin/products/${productId}/variants`,
      {
        headers: admin,
        data: {
          title: "Round Trip / M",
          sku: `RT-${suffix}`,
          options: { Size: "M" },
          prices: [{ currency_code: "usd", amount: 33.5 }],
        },
      }
    )
    expect(variant.status(), await variant.text()).toBe(201)
    const variantId = (await variant.json()).variant.id

    // Publish it. Products are created as drafts and the store serves published ones only,
    // so this is the step a merchant takes to put it in front of shoppers — without it the
    // storefront is right to not list it.
    const published = await request.post(`${BACKEND}/admin/products/${productId}`, {
      headers: admin,
      data: { status: "published" },
    })
    expect(published.ok(), await published.text()).toBeTruthy()

    // The storefront's view of the same product.
    const listed = await request.get(
      `${BACKEND}/store/products?handle=${handle}`,
      { headers: { ...hostFor(tenant), "x-publishable-api-key": "pk_test" } }
    )
    expect(listed.ok(), await listed.text()).toBeTruthy()
    const product = (await listed.json()).products[0]
    expect(product, "the product the admin just created is not in the catalog").toBeTruthy()

    const storeVariant = product.variants.find((v: any) => v.id === variantId)
    expect(storeVariant, "the variant is missing from the store payload").toBeTruthy()

    // The price the merchant typed, back out the other side.
    expect(Number(storeVariant.calculated_price.calculated_amount)).toBe(33.5)
    expect(storeVariant.calculated_price.currency_code).toBe("usd")

    // The option pairing the storefront matches a shopper's selection against.
    expect(storeVariant.options).toContainEqual(
      expect.objectContaining({ value: "M" })
    )
  })

  test("a shopper cannot set their own price", async ({ request }) => {
    const tenant: Tenant = "pro"
    const store = { ...hostFor(tenant), "x-publishable-api-key": "pk_test" }

    const products = await request.get(`${BACKEND}/store/products?limit=1`, {
      headers: store,
    })
    const variant = (await products.json()).products[0]?.variants?.[0]
    expect(variant, "no seeded variant to price").toBeTruthy()
    const listed = Number(variant.calculated_price.calculated_amount)

    const regions = await request.get(`${BACKEND}/store/regions`, { headers: store })
    const region = (await regions.json()).regions.find(
      (r: any) => r.currency_code === "usd"
    )

    const cart = await request.post(`${BACKEND}/store/carts`, {
      headers: store,
      data: { region_id: region.id, currency_code: "usd" },
    })
    const cartId = (await cart.json()).cart.id

    // The attack: name a price of one cent for something that costs more.
    const added = await request.post(`${BACKEND}/store/carts/${cartId}/line-items`, {
      headers: store,
      data: { variant_id: variant.id, quantity: 1, unit_price: 0.01 },
    })
    expect(added.ok(), await added.text()).toBeTruthy()

    const item = (await added.json()).cart.items[0]
    expect(Number(item.unit_price)).toBe(listed)
    expect(Number(item.unit_price)).not.toBe(0.01)
  })
})
