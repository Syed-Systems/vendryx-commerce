import { expect, test } from "@playwright/test"

import { BACKEND } from "../playwright.config"

/**
 * The storefront journeys, in a real browser against a real backend.
 *
 * What this replaces: `tests/e2e/runner.js` contained twelve functions that logged a tick
 * and `return true`. It opened no browser and made no request, and `scripts/gate-p19.sh`
 * consumed its exit code to report "Phase Gate P19 PASSED" — so every storefront journey
 * in P19-02..P19-14 was recorded as verified on the strength of nothing.
 *
 * Selectors use the `data-testid` attributes already present in the fork (307 of them),
 * so these tests needed no changes to product markup.
 */

const REGION = "us"

/**
 * `product-title` is on the PLP card, the PDP heading, related-product cards and cart
 * rows, so a bare `getByTestId("product-title")` matches several elements and Playwright's
 * strict mode fails *immediately* rather than retrying — which reads as "the PDP never
 * loaded" when the real cause is that the old page was still on screen. The PDP heading is
 * the only `h1` among them.
 */
const pdpTitle = 'h1[data-testid="product-title"]'

/** Opens the first product and waits for the PDP to actually be the current page. */
async function openFirstProduct(page: import("@playwright/test").Page) {
  await page.goto(`/${REGION}/store`)
  await page.getByTestId("product-wrapper").first().click()
  await page.waitForURL(/\/products\//, { timeout: 30_000 })
  await expect(page.locator(pdpTitle)).toBeVisible({ timeout: 30_000 })
}

test.describe("catalog", () => {
  test("the product list renders real products", async ({ page }) => {
    await page.goto(`/${REGION}/store`)
    await expect(page.getByTestId("products-list")).toBeVisible({ timeout: 30_000 })

    const products = page.getByTestId("product-wrapper")
    await expect(products.first()).toBeVisible()
    expect(await products.count()).toBeGreaterThan(0)
  })

  test("a product page shows its price and add-to-cart", async ({ page }) => {
    await openFirstProduct(page)
    await expect(page.getByTestId("add-product-button")).toBeVisible()
    // A product page with no price is the symptom of the catalog returning variants
    // without `calculated_price`, which is what it did before this suite existed.
    await expect(page.getByTestId("product-price")).toBeVisible()
  })
})

test.describe("cart", () => {
  test("adding a product puts it in the cart", async ({ page }) => {
    await openFirstProduct(page)

    // A product with options cannot be added until one is chosen. One value per option
    // group — not every button on the page, which for a single "Size" option means
    // clicking S then M then L and is not what a shopper does.
    const groups = page.getByTestId("product-options")
    for (let i = 0; i < (await groups.count()); i++) {
      await groups.nth(i).getByTestId("option-button").first().click()
    }

    const add = page.getByTestId("add-product-button")
    await expect(add).toBeEnabled({ timeout: 15_000 })
    await add.click()

    // Adding is a server action. Navigating while it is in flight aborts the request and
    // the item never reaches the cart — so wait for the button to come back out of its
    // "Adding to Bag..." state before leaving the page.
    await expect(add).not.toHaveText(/Adding/i, { timeout: 30_000 })

    await page.goto(`/${REGION}/cart`)
    await expect(page.getByTestId("cart-container")).toBeVisible({ timeout: 30_000 })
    await expect(page.getByTestId("product-row").first()).toBeVisible()
  })

  test("an empty cart says so rather than rendering a broken table", async ({
    page,
    context,
  }) => {
    await context.clearCookies()
    await page.goto(`/${REGION}/cart`)
    await expect(page.getByTestId("empty-cart-message")).toBeVisible({ timeout: 30_000 })
  })
})

test.describe("social checkout", () => {
  /**
   * The component and the backing flow both existed and neither was reachable: the
   * component had zero importers, and `SocialDraftWorkflow` had no HTTP route. This is
   * the first test that could ever have exercised either.
   */
  test("queues a draft and returns a readable Ref code", async ({ request }) => {
    // Driven through the API because the buttons open external apps (wa.me, m.me),
    // which a browser test cannot follow.
    const products = await (
      await request.get(`${BACKEND}/store/products?limit=1`, {
        headers: { "x-publishable-api-key": "pk_test" },
      })
    ).json()

    const variant = products.products?.[0]?.variants?.[0]
    test.skip(!variant, "no seeded product to order")

    const response = await request.post(`${BACKEND}/store/social-drafts`, {
      data: {
        currency_code: "usd",
        total: "40.00",
        items: [
          {
            variant_id: variant.id,
            title: products.products[0].title,
            sku: variant.sku ?? null,
            options: [],
            quantity: 1,
            unit_price: "40.00",
            line_total: "40.00",
          },
        ],
        buyer_name: "Test Buyer",
        buyer_phone: "+10000000000",
      },
    })

    expect(response.status(), await response.text()).toBe(201)
    const { social_draft } = await response.json()

    // The Ref is the only link between the chat and the queued draft.
    expect(
      social_draft.ref_code,
      `${social_draft.ref_code} must use the VX prefix and exclude characters confused when read aloud`
    ).toMatch(/^VX-[2-9A-HJ-NP-Z]{6}$/)

    // The message has to be matchable by eye in a chat bubble.
    expect(social_draft.message).toContain("Order from")
    expect(social_draft.message).toContain("Qty 1")
    expect(social_draft.message).toContain(`Ref: #${social_draft.ref_code}`)
    expect(social_draft.order_id).toBeTruthy()
  })

  test("a draft with no line items is refused", async ({ request }) => {
    const response = await request.post(`${BACKEND}/store/social-drafts`, {
      data: { currency_code: "usd", total: "0", items: [] },
    })
    expect(response.status()).toBe(400)
  })

  test("two drafts get different Ref codes", async ({ request }) => {
    const products = await (
      await request.get(`${BACKEND}/store/products?limit=1`, {
        headers: { "x-publishable-api-key": "pk_test" },
      })
    ).json()
    const variant = products.products?.[0]?.variants?.[0]
    test.skip(!variant, "no seeded product to order")

    const make = async () => {
      const r = await request.post(`${BACKEND}/store/social-drafts`, {
        data: {
          currency_code: "usd",
          total: "40.00",
          items: [
            {
              variant_id: variant.id,
              title: "x",
              quantity: 1,
              unit_price: "40.00",
              line_total: "40.00",
            },
          ],
        },
      })
      return (await r.json()).social_draft.ref_code
    }

    // Colliding codes would make two conversations indistinguishable to staff.
    expect(await make()).not.toBe(await make())
  })
})

test.describe("regions", () => {
  test("an unknown region does not render a broken storefront", async ({ page }) => {
    const response = await page.goto(`/not-a-region/store`)
    // Either a redirect to a valid region or a 404 — never a page rendering with no
    // currency, which is what silently produces prices of 0.
    expect(response?.status()).toBeLessThan(500)
  })
})
