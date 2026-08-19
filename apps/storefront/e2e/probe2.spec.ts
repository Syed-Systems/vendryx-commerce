import { expect, test } from "@playwright/test"

test("probe checkout", async ({ page }) => {
  page.on("pageerror", (e) => console.log("PAGEERROR:", e.message))
  page.on("response", async (r) => {
    if (r.status() >= 400) console.log("HTTP", r.status(), r.url())
  })

  await page.goto("/us/store")
  await page.getByTestId("product-wrapper").first().click()
  await page.waitForURL(/\/products\//, { timeout: 30000 })
  const groups = page.getByTestId("product-options")
  for (let i = 0; i < (await groups.count()); i++) {
    await groups.nth(i).getByTestId("option-button").first().click()
  }
  const add = page.getByTestId("add-product-button")
  await expect(add).toBeEnabled({ timeout: 15000 })
  await add.click()
  await expect(add).not.toHaveText(/Adding/i, { timeout: 30000 })

  await page.goto("/us/cart")
  await page.getByTestId("checkout-button").click()
  await expect(page.getByTestId("shipping-email-input")).toBeVisible({ timeout: 30000 })
  await page.getByTestId("shipping-email-input").fill("shopper@example.com")
  await page.getByTestId("shipping-first-name-input").fill("Ada")
  await page.getByTestId("shipping-last-name-input").fill("Lovelace")
  await page.getByTestId("shipping-address-input").fill("1 Analytical Way")
  await page.getByTestId("shipping-postal-code-input").fill("94105")
  await page.getByTestId("shipping-city-input").fill("San Francisco")
  await page.getByTestId("shipping-province-input").fill("CA")
  await page.getByTestId("shipping-phone-input").fill("+15550000000")
  await page.getByTestId("submit-address-button").click()
  await page.waitForTimeout(6000)

  console.log("URL:", page.url())
  console.log("address error:", await page.getByTestId("address-error-message").allTextContents())
  console.log("delivery container:", await page.getByTestId("delivery-options-container").count())
  console.log("radios:", await page.getByTestId("delivery-option-radio").count())
  const body = await page.locator("body").innerText()
  console.log("BODY SNIP:", body.replace(/\s+/g, " ").slice(0, 1200))
})
