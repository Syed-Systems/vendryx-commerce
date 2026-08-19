import { expect, test } from "@playwright/test"

/**
 * The checkout, end to end: cart → address → delivery → payment → placed order.
 *
 * No test had ever run this, and it could not have passed if one had. The delivery step
 * has nothing to offer unless a fulfillment set, a service zone covering the region's
 * countries, and a shipping option all exist — and the seed created none of them, so the
 * checkout stopped dead one step in. The `fulfillment_provider` registry was also scoped
 * to a tenant that no request ever runs as, which made a shipping option impossible to
 * create even by hand.
 *
 * Everything before this test asserted pieces: a cart has items, an option is selectable,
 * a payment provider is listed. All of that was true while the journey was impossible.
 */

const REGION = "us"

async function addFirstProductToCart(page: import("@playwright/test").Page) {
  await page.goto(`/${REGION}/store`)
  await page.getByTestId("product-wrapper").first().click()
  await page.waitForURL(/\/products\//, { timeout: 30_000 })
  await expect(page.locator('h1[data-testid="product-title"]')).toBeVisible({
    timeout: 30_000,
  })

  const groups = page.getByTestId("product-options")
  for (let i = 0; i < (await groups.count()); i++) {
    await groups.nth(i).getByTestId("option-button").first().click()
  }

  const add = page.getByTestId("add-product-button")
  await expect(add).toBeEnabled({ timeout: 15_000 })
  await add.click()
  // Adding is a server action; navigating mid-flight aborts it.
  await expect(add).not.toHaveText(/Adding/i, { timeout: 30_000 })
}

test.describe("checkout", () => {
  test("a shopper can place an order", async ({ page }) => {
    // Five steps, each a server round trip and a Next navigation. The default 30s budget
    // is shorter than the individual waits inside it, so the test ran out of time before
    // its own assertions did.
    test.setTimeout(180_000)

    await addFirstProductToCart(page)

    await page.goto(`/${REGION}/cart`)
    await expect(page.getByTestId("cart-container")).toBeVisible({ timeout: 30_000 })
    await page.getByTestId("checkout-button").click()

    // --- Address -----------------------------------------------------------
    await expect(page.getByTestId("shipping-email-input")).toBeVisible({
      timeout: 30_000,
    })
    await page.getByTestId("shipping-email-input").fill("shopper@example.com")
    await page.getByTestId("shipping-first-name-input").fill("Ada")
    await page.getByTestId("shipping-last-name-input").fill("Lovelace")
    await page.getByTestId("shipping-address-input").fill("1 Analytical Way")
    await page.getByTestId("shipping-postal-code-input").fill("94105")
    await page.getByTestId("shipping-city-input").fill("San Francisco")
    // Required, and empty by default whenever the region covers more than one country —
    // North America has both US and CA. Without it the browser blocks the submit and the
    // page simply stays on the address step with no error shown.
    await page.getByTestId("shipping-country-select").selectOption("us")
    await page.getByTestId("shipping-province-input").fill("CA")
    await page.getByTestId("shipping-phone-input").fill("+15550000000")
    await page.getByTestId("submit-address-button").click()

    // --- Delivery ----------------------------------------------------------
    // The step that was unreachable. An empty container here means the seed produced no
    // shipping option, not that the shopper picked wrong.
    const deliveryOptions = page.getByTestId("delivery-option-radio")
    await expect(
      deliveryOptions.first(),
      "no delivery option offered — check fulfillment set, service zone and geo zones"
    ).toBeVisible({ timeout: 30_000 })
    await deliveryOptions.first().click()
    await page.getByTestId("submit-delivery-option-button").click()

    // --- Payment -----------------------------------------------------------
    // Manual payment (`pp_system_default`); Stripe is unconfigured in dev. The payment
    // options carry no test id, so this selects by the label the shopper reads.
    const submitPayment = page.getByTestId("submit-payment-button")
    await expect(submitPayment).toBeVisible({ timeout: 30_000 })
    await page.getByText("Manual Payment", { exact: true }).click()
    await expect(submitPayment).toBeEnabled({ timeout: 15_000 })
    await submitPayment.click()

    // --- Review ------------------------------------------------------------
    const placeOrder = page.getByTestId("submit-order-button")
    await expect(placeOrder).toBeVisible({ timeout: 30_000 })
    await expect(placeOrder).toBeEnabled({ timeout: 15_000 })
    await placeOrder.click()

    // --- Confirmation ------------------------------------------------------
    await expect(page.getByTestId("order-complete-container")).toBeVisible({
      timeout: 60_000,
    })
    await expect(page.getByTestId("order-id")).toBeVisible()
    // The order carries the email the shopper typed, which proves the cart that was
    // completed is the one they filled in.
    await expect(page.getByTestId("order-email")).toContainText("shopper@example.com")
  })
})
