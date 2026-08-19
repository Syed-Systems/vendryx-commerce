import { expect, test } from "@playwright/test"
import { OWNER_PASSWORD } from "../playwright.config"

/**
 * The analytics screen, in the browser.
 *
 * The reporting endpoints have returned a daily `time_series` since the module landed
 * and the screen plotted none of it. These assertions are what stop that regressing:
 * the chart has to be in the DOM with real geometry, not merely the totals.
 */

const OWNER = "owner@pro.localhost"

test.beforeEach(async ({ page }) => {
  // A real sign-in through the form, not a synthesised token. This is what caught the
  // CORS gap: the dashboard is served from 127.0.0.1:5173 while the deployment only
  // allowed localhost:5173, so `GET /auth/user/providers` was blocked by the browser
  // and the login screen rendered its "no auth provider configured" empty state with
  // no form at all. Injecting a token would have sailed straight past that.
  await page.goto("/login")
  await page.getByPlaceholder(/email/i).fill(OWNER)
  await page.getByPlaceholder(/password/i).fill(OWNER_PASSWORD)
  await page.getByRole("button", { name: /continue|sign in|log in/i }).click()
  await page.waitForURL((url) => !url.pathname.includes("/login"), {
    timeout: 20_000,
  })
})

test("analytics renders its charts, not just totals", async ({ page }) => {
  await page.goto("/analytics")

  await expect(
    page.getByRole("heading", { name: "Analytics & Executive Reports" })
  ).toBeVisible()

  // Both plots present and named for screen readers.
  const revenue = page.getByRole("img", { name: /Revenue per day/ })
  const orders = page.getByRole("img", { name: /Orders per day/ })
  await expect(revenue).toBeVisible()
  await expect(orders).toBeVisible()

  // Two measures of different scale get two plots. A single chart here would mean a
  // dual y-axis, which is the one thing dataviz never allows.
  await expect(page.getByRole("heading", { name: "Revenue over time" })).toBeVisible()
  await expect(page.getByRole("heading", { name: "Orders over time" })).toBeVisible()
})

test("the window control re-scopes every panel below it", async ({ page }) => {
  await page.goto("/analytics")

  const requests: string[] = []
  page.on("request", (request) => {
    if (request.url().includes("/admin/reports/")) {
      requests.push(request.url())
    }
  })

  await page.getByText("Last 30 Days").click()
  await page.getByRole("option", { name: "Last 7 Days" }).click()

  // Filters scope everything below them, so every report refetches on the same slice
  // and the numbers on screen always agree with each other.
  await expect
    .poll(() => requests.filter((url) => url.includes("window_days=7")).length, {
      timeout: 10_000,
    })
    .toBeGreaterThanOrEqual(4)
})

test("the chart survives a theme switch", async ({ page }) => {
  await page.goto("/analytics")
  await expect(page.getByRole("img", { name: /Revenue per day/ })).toBeVisible()

  // The marks take their colour through currentColor off a `dark:` class, so the
  // plot has to keep rendering when the ThemeProvider flips the root class.
  await page.evaluate(() => document.documentElement.classList.add("dark"))
  await expect(page.getByRole("img", { name: /Revenue per day/ })).toBeVisible()
  await expect(page.getByRole("heading", { name: "Revenue over time" })).toBeVisible()
})
