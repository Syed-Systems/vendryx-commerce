#!/usr/bin/env bash
# P27-09 — the bundler binds the frontend it was asked for.
#
# This exists because the first implementation silently did not. resolve.alias
# and Next's tsconfig-paths resolver hook the same resolution step, Next's wins,
# and a STHENO_FRONTEND=matrix build came out full of frontend-default markup while
# reporting "Compiled successfully". Every gate in this repo passed. Only
# grepping the emitted chunks caught it.
#
# So this asserts the mapping directly, from next.config.js, in under a second —
# rather than trusting a build that has already proven it can lie about this.
set -euo pipefail
cd "$(dirname "$0")/.."

APP="apps/storefront"

node - "$APP" <<'NODE'
const fs = require("fs")
const path = require("path")
const app = path.resolve(process.argv[2])

// next.config.js runs check-env-variables at require time. This gate resolves
// module paths and never talks to the backend, so a placeholder is enough.
process.env.NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY ||= "pk_theme_switch_gate"

// Stand in for webpack: capture the request rewriter next.config installs.
function loadRewriter(frontend) {
  process.env.STHENO_FRONTEND = frontend
  delete require.cache[require.resolve(path.join(app, "next.config.js"))]
  const config = require(path.join(app, "next.config.js"))

  let rewrite = null
  class NormalModuleReplacementPlugin {
    constructor(_pattern, fn) {
      rewrite = fn
    }
  }
  const plugins = []
  config.webpack({ plugins, resolve: { alias: {} } }, {
    webpack: { NormalModuleReplacementPlugin },
  })

  if (!rewrite) {
    throw new Error(`${frontend}: next.config.js installed no frontend rewriter`)
  }
  return (request) => {
    const r = { request }
    rewrite(r)
    return r.request.replace(/\\/g, "/")
  }
}

let failed = 0
const check = (label, actual, expected) => {
  const ok = actual.includes(expected)
  if (!ok) {
    console.error(`FAIL: ${label}\n  expected to contain: ${expected}\n  got: ${actual}`)
    failed = 1
  }
  return ok
}

const frontends = fs
  .readdirSync(path.join(app, "packages"), { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && /^frontend-(?!fixture-)/.test(entry.name))
  .map((entry) => entry.name.replace(/^frontend-/, ""))
  .sort()

for (const frontend of frontends) {
  const map = loadRewriter(frontend)

  // A frontend slot resolves into that frontend, and no other.
  check(
    `${frontend}: @frontend/* binds frontend-${frontend}`,
    map("@frontend/home/components/hero"),
    `packages/frontend-${frontend}/src/modules/home/components/hero`
  )

  // A frontend's own internal imports follow it too, or a second frontend silently
  // renders the first frontend's internals.
  check(
    `${frontend}: @modules/* binds frontend-${frontend}`,
    map("@modules/products/components/thumbnail"),
    `packages/frontend-${frontend}/src/modules/products/components/thumbnail`
  )

  // ADR-013 decision 4 and 2: checkout and routing are core's, in every frontend.
  check(
    `${frontend}: checkout stays in core`,
    map("@modules/checkout/templates/checkout-form"),
    "packages/storefront-core/src/modules/checkout/templates/checkout-form"
  )
  check(
    `${frontend}: routing stays in core`,
    map("@modules/common/components/localized-client-link"),
    "packages/storefront-core/src/modules/common/components/localized-client-link"
  )
}

// The distDirs must differ, or two parallel frontends overwrite each other's build.
const dirs = frontends.map((frontend) => {
  process.env.STHENO_FRONTEND = frontend
  delete require.cache[require.resolve(path.join(app, "next.config.js"))]
  return require(path.join(app, "next.config.js")).distDir
})
const uniqueDirs = new Set(dirs)
if (uniqueDirs.size !== dirs.length) {
  console.error(`FAIL: frontends share build dirs (${dirs.join(", ")}); they cannot run in parallel.`)
  failed = 1
}

if (!failed) {
  console.log(`PASS: ${frontends.join(", ")} each bind their own frontend (${dirs.join(", ")}).`)
}
process.exit(failed)
NODE
