import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    // Scoped so the Playwright specs in e2e/ and tests/e2e/ are not collected.
    include: ["tests/unit/**/*.test.ts?(x)"],
  },
})
