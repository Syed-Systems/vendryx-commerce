import { expect, test } from "@playwright/test"
import { OWNER_PASSWORD } from "../playwright.config"

/**
 * The inventory flow end to end: item → location → stock → reserve → fulfil/release.
 *
 * This is the flow the admin's inventory screens sit on, and almost none of it was real
 * before: `GET /admin/inventory-items` returned a hardcoded empty array even though the
 * service could list them, `GET /admin/inventory-items/{id}` returned a mock object with
 * a fabricated SKU for *every* id, `GET /admin/reservations` always returned `[]`, and
 * there was no endpoint to change stock at all — a level could be created once and never
 * adjusted.
 *
 * The assertions below are about arithmetic, not just status codes: available must equal
 * stocked minus reserved at every step, because that identity is what stops overselling.
 */

const BASE = process.env.STHENO_BACKEND_URL ?? "http://127.0.0.1:9000"
const HOST = "pro.localhost"
const OWNER = `owner@${HOST}`

type Ctx = { headers: Record<string, string> }

async function signIn(request: any): Promise<Ctx> {
  const response = await request.post(`${BASE}/auth/user/emailpass`, {
    headers: { Host: HOST },
    data: { email: OWNER, password: OWNER_PASSWORD },
  })
  expect(response.ok(), "owner sign-in").toBeTruthy()
  const { token } = await response.json()
  return { headers: { Host: HOST, Authorization: `Bearer ${token}` } }
}

const unique = (prefix: string) => `${prefix}-${Date.now()}-${Math.floor(Math.random() * 1e4)}`

test.describe("inventory flow", () => {
  test("item, location, stock, reservation and release keep the arithmetic honest", async ({
    request,
  }) => {
    const { headers } = await signIn(request)

    // ── 1. A stock location ────────────────────────────────────────────────
    const location = await (
      await request.post(`${BASE}/admin/stock-locations`, {
        headers,
        data: { name: unique("Warehouse") },
      })
    ).json()
    const locationId = location.stock_location?.id ?? location.data?.id
    expect(locationId, "stock location id").toBeTruthy()

    // ── 2. An inventory item ───────────────────────────────────────────────
    const sku = unique("SKU")
    const itemResponse = await request.post(`${BASE}/admin/inventory-items`, {
      headers,
      data: { sku, requires_shipping: true },
    })
    expect(itemResponse.status()).toBe(201)
    const item = await itemResponse.json()
    const itemId = item.inventory_item?.id ?? item.data?.id
    expect(itemId).toBeTruthy()

    // It must appear in the list. This is the assertion the old hardcoded `[]` could
    // never have satisfied.
    const list = await (
      await request.get(`${BASE}/admin/inventory-items`, { headers })
    ).json()
    expect(
      list.inventory_items.some((i: any) => i.sku === sku),
      "the created item appears in the list"
    ).toBeTruthy()

    // And fetching it by id must return *that* item, not a mock.
    const fetched = await (
      await request.get(`${BASE}/admin/inventory-items/${itemId}`, { headers })
    ).json()
    expect(fetched.inventory_item.sku, "fetched by id returns the real item").toBe(sku)

    // ── 3. Stock at that location ──────────────────────────────────────────
    await request.post(`${BASE}/admin/inventory-items/${itemId}/location-levels`, {
      headers,
      data: { inventory_item_id: itemId, location_id: locationId, stocked_quantity: 10 },
    })

    const readLevel = async () => {
      const body = await (
        await request.get(`${BASE}/admin/inventory-items/${itemId}`, { headers })
      ).json()
      return body.location_levels.find((l: any) => l.location_id === locationId)
    }

    let level = await readLevel()
    expect(level.stocked_quantity).toBe(10)
    expect(level.reserved_quantity).toBe(0)
    expect(level.available_quantity).toBe(10)

    // ── 4. Reserve against it ──────────────────────────────────────────────
    const reservation = await (
      await request.post(`${BASE}/admin/reservations`, {
        headers,
        data: { inventory_item_id: itemId, location_id: locationId, quantity: 4 },
      })
    ).json()
    const reservationId = reservation.reservation?.id ?? reservation.data?.id
    expect(reservationId).toBeTruthy()

    level = await readLevel()
    expect(level.stocked_quantity, "reserving does not consume stock").toBe(10)
    expect(level.reserved_quantity).toBe(4)
    expect(level.available_quantity, "available = stocked - reserved").toBe(6)

    // The reservation is listable and fetchable — both previously returned nothing.
    const reservations = await (
      await request.get(`${BASE}/admin/reservations?inventory_item_id=${itemId}`, {
        headers,
      })
    ).json()
    expect(reservations.count).toBe(1)
    expect(reservations.reservations[0].quantity).toBe(4)

    // ── 5. Stock adjustment ────────────────────────────────────────────────
    const adjusted = await (
      await request.post(
        `${BASE}/admin/inventory-items/${itemId}/location-levels/${locationId}`,
        { headers, data: { stocked_quantity: 25 } }
      )
    ).json()
    expect(adjusted.inventory_level.stocked_quantity).toBe(25)
    expect(adjusted.inventory_level.available_quantity, "reservation still held").toBe(21)

    // ── 6. Stock may not drop below what is already reserved ───────────────
    //
    // The guard that prevents overselling: 4 units are promised to an order, so stock
    // cannot be set to 2 no matter what a stock count says.
    const tooLow = await request.post(
      `${BASE}/admin/inventory-items/${itemId}/location-levels/${locationId}`,
      { headers, data: { stocked_quantity: 2 } }
    )
    expect(tooLow.status(), "cannot set stock below reserved").toBe(400)

    const negative = await request.post(
      `${BASE}/admin/inventory-items/${itemId}/location-levels/${locationId}`,
      { headers, data: { stocked_quantity: -1 } }
    )
    expect(negative.status(), "cannot set negative stock").toBe(400)

    // Neither rejection may have moved anything.
    level = await readLevel()
    expect(level.stocked_quantity).toBe(25)
    expect(level.reserved_quantity).toBe(4)

    // ── 7. Releasing returns the quantity to available ─────────────────────
    const released = await request.delete(
      `${BASE}/admin/reservations/${reservationId}`,
      { headers }
    )
    expect(released.status()).toBe(200)

    level = await readLevel()
    expect(level.reserved_quantity, "release frees the reservation").toBe(0)
    expect(level.available_quantity, "stock returns to fully available").toBe(25)
    expect(level.stocked_quantity, "release does not invent stock").toBe(25)
  })

  test("reserving more than is available is refused", async ({ request }) => {
    const { headers } = await signIn(request)

    const location = await (
      await request.post(`${BASE}/admin/stock-locations`, {
        headers,
        data: { name: unique("Warehouse") },
      })
    ).json()
    const locationId = location.stock_location?.id ?? location.data?.id

    const item = await (
      await request.post(`${BASE}/admin/inventory-items`, {
        headers,
        data: { sku: unique("SKU"), requires_shipping: true },
      })
    ).json()
    const itemId = item.inventory_item?.id ?? item.data?.id

    await request.post(`${BASE}/admin/inventory-items/${itemId}/location-levels`, {
      headers,
      data: { inventory_item_id: itemId, location_id: locationId, stocked_quantity: 3 },
    })

    // Overselling is the failure inventory tracking exists to prevent, so this must be
    // refused rather than driving available negative.
    const oversell = await request.post(`${BASE}/admin/reservations`, {
      headers,
      data: { inventory_item_id: itemId, location_id: locationId, quantity: 5 },
    })
    expect(oversell.status(), "cannot reserve more than available").toBeGreaterThanOrEqual(400)

    const body = await (
      await request.get(`${BASE}/admin/inventory-items/${itemId}`, { headers })
    ).json()
    const level = body.location_levels.find((l: any) => l.location_id === locationId)
    expect(level.reserved_quantity, "the refused reservation left nothing behind").toBe(0)
    expect(level.available_quantity).toBe(3)
  })

  test("inventory is tenant-scoped", async ({ request }) => {
    // An inventory item created by Pro must be invisible to Free (ADR-012).
    const { headers } = await signIn(request)
    const sku = unique("SKU-ISOLATED")
    await request.post(`${BASE}/admin/inventory-items`, {
      headers,
      data: { sku, requires_shipping: true },
    })

    const freeAuth = await request.post(`${BASE}/auth/user/emailpass`, {
      headers: { Host: "free.localhost" },
      data: { email: "owner@free.localhost", password: OWNER_PASSWORD },
    })
    const { token } = await freeAuth.json()

    const seenByFree = await (
      await request.get(`${BASE}/admin/inventory-items`, {
        headers: { Host: "free.localhost", Authorization: `Bearer ${token}` },
      })
    ).json()

    expect(
      seenByFree.inventory_items.some((i: any) => i.sku === sku),
      "another tenant's inventory item must not be visible"
    ).toBeFalsy()
  })
})
