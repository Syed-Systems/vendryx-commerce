import { readFileSync } from "node:fs"

import { expect, test } from "vitest"

import { FRONTEND_ID } from "@stheno/frontend-default"

// Core is not imported at runtime here on purpose: its public API re-exports
// "use server" modules that reach for next/headers, which has no meaning
// outside a Next request. The entry points are asserted structurally instead.
const corePkg = JSON.parse(
  readFileSync("packages/storefront-core/package.json", "utf8")
)

test("the theme package resolves and declares its identity", () => {
  expect(FRONTEND_ID).toBe("default")
})

test("core exposes exactly the entry points a theme is allowed to reach", () => {
  expect(Object.keys(corePkg.exports).sort()).toEqual([
    ".",
    "./lib/*",
    "./middleware",
    "./server",
  ])
})

test("the theme pins a core major", () => {
  const themePkg = JSON.parse(
    readFileSync("packages/frontend-default/package.json", "utf8")
  )
  expect(themePkg.peerDependencies["@stheno/storefront-core"]).toMatch(/^\^?\d/)
})
