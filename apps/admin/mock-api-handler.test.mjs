import test from "node:test"
import assert from "node:assert/strict"
import { createMockApiState, handleMockApiRequest } from "./mock-api-handler.mjs"

test("returns authenticated merchant user and active store", () => {
  const state = createMockApiState()
  const meRes = handleMockApiRequest(state, { method: "GET", pathname: "/admin/users/me" })
  assert.equal(meRes.status, 200)
  assert.equal(meRes.body.user.email, "owner@vendryx.store")

  const storesRes = handleMockApiRequest(state, { method: "GET", pathname: "/admin/stores" })
  assert.equal(storesRes.status, 200)
  assert.equal(storesRes.body.stores.length, 1)
  assert.equal(storesRes.body.stores[0].name, "Vendryx Flagship Store")
})

test("persists onboarding dismiss and reset state transitions", () => {
  const state = createMockApiState()
  assert.equal(handleMockApiRequest(state, { method: "GET", pathname: "/admin/onboarding" }).body.onboarding.dismissed, false)

  handleMockApiRequest(state, { method: "POST", pathname: "/admin/onboarding/dismiss" })
  assert.equal(handleMockApiRequest(state, { method: "GET", pathname: "/admin/onboarding" }).body.onboarding.dismissed, true)

  handleMockApiRequest(state, { method: "POST", pathname: "/admin/onboarding/reset" })
  assert.equal(handleMockApiRequest(state, { method: "GET", pathname: "/admin/onboarding" }).body.onboarding.dismissed, false)
})

test("updates branding and courier rates in memory", () => {
  const state = createMockApiState()
  const updated = handleMockApiRequest(state, {
    method: "POST",
    pathname: "/admin/store/branding",
    body: { store_name: "Atelier Studio", color_primary: "#0F766E" },
  })
  assert.equal(updated.status, 200)
  assert.equal(updated.body.branding.store_name, "Atelier Studio")
  assert.equal(updated.body.branding.color_primary, "#0F766E")

  const rateUpdate = handleMockApiRequest(state, {
    method: "POST",
    pathname: "/admin/couriers/pathao/rates",
    body: { inside_dhaka: 75 },
  })
  assert.equal(rateUpdate.status, 200)
  assert.equal(rateUpdate.body.rates.inside_dhaka, 75)
})

test("generates analytics time series and supports customer creation", () => {
  const state = createMockApiState()
  const rev = handleMockApiRequest(state, {
    method: "GET",
    pathname: "/admin/reports/revenue-over-time",
    searchParams: new URLSearchParams("window_days=14&currency=USD"),
  })
  assert.equal(rev.status, 200)
  assert.equal(rev.body.report.data.time_series.length, 14)

  const initialCount = state.customers.length
  const created = handleMockApiRequest(state, {
    method: "POST",
    pathname: "/admin/customers",
    body: { email: "new@example.com", first_name: "Sadia", last_name: "Noor" },
  })
  assert.equal(created.status, 200)
  assert.equal(state.customers.length, initialCount + 1)
})
