import { defineConfig, devices } from "@playwright/test"

/**
 * Storefront end-to-end tests.
 *
 * These replace `tests/e2e/runner.js`, which printed a tick and returned `true` for
 * twelve suites without opening a browser — so gate P19 reported PASS while nothing had
 * ever been verified. Every assertion here drives a real browser against a real backend.
 */
export const BACKEND = process.env.STHENO_BACKEND_URL ?? "http://pro.localhost:9000"

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: process.env.CI ? "list" : [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: process.env.STHENO_STOREFRONT_URL ?? "http://localhost:8000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
})
