import { defineConfig } from "vitest/config"
import react from "@vitejs/plugin-react"

// The spec files under src/ have been in the tree without a runner or a declared
// `vitest` dependency, so none of them have ever executed. This is the smallest config
// that runs them: jsdom for the component specs, globals so the existing files work
// without importing `describe`/`it`, and no coverage thresholds until there is a
// baseline worth holding.
export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: "jsdom",
    include: ["src/**/*.spec.{ts,tsx}"],
  },
})
