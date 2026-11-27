import { defineConfig, devices } from "@playwright/test"

/**
 * Runs against `scripts/dev-tenants.sh up`, not against a mock. The suites this replaces
 * printed a tick and returned `true`, so a green run meant nothing; every assertion here
 * goes over the wire to a real Stheno server and a real Postgres.
 *
 * One server now serves every tenant and tells them apart by Host (ADR-012), where each
 * tier used to be its own process on its own port. Tests therefore vary the `Host`
 * header rather than the base URL.
 */
export const BACKEND = process.env.STHENO_BACKEND_URL ?? "http://127.0.0.1:9000"

/** Tenant → the host that routes to it. */
export const TENANTS = {
  trial: "trial.localhost",
  free: "free.localhost",
  pro: "pro.localhost",
} as const

export const ownerEmail = (tenant: keyof typeof TENANTS) => `owner@${TENANTS[tenant]}`

export const OWNER_PASSWORD = process.env.STHENO_DEV_PASSWORD ?? "Sup3rSecret!"

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: process.env.CI ? "list" : [["list"], ["html", { open: "never" }]],
  use: {
    // The admin dev server listens on 3000 (vite.config.cts `server.port`).
    baseURL: process.env.STHENO_ADMIN_URL ?? "http://127.0.0.1:3000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: process.env.STHENO_ADMIN_URL
    ? undefined
    : {
        command: "npm run dev",
        url: "http://127.0.0.1:3000",
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
        // Without this, vite.config.cts serves its built-in mock API and the suite would
        // pass against fake data. Point the dev server's proxy at the real engine.
        env: { VITE_STHENO_BACKEND_URL: BACKEND },
      },
})
