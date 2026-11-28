#!/usr/bin/env bash
# A frontend that declares independence may not be another frontend's markup repainted.
#
# This exists because "port the design" was answered three times with a recolour
# of frontend-default, and every gate in the repo passed each time: the fork
# satisfied all 45 contract slots, imported nothing forbidden, fetched nothing,
# and typechecked. Purity, boundaries, pin and resolution are all blind to it,
# because none of them look at *markup*. Reviewers were not blind to it — it was
# obvious on sight — which is precisely the kind of failure that belongs in CI
# rather than in someone's patience.
#
# Two things are checked, for frontends that opt in via `stheno.independent` in
# their package.json:
#
#   1. No file may be structurally identical to the same file in frontend-default
#      once styling is stripped. Changing class names is not porting.
#   2. No component may be imported from @medusajs/ui. The design system belongs
#      to the frontend; borrowing Medusa's primitives is how the DOM stays Medusa's
#      even after the class names stop being.
#
# A frontend mid-conversion still has debt, and a gate that stays red until it is
# finished is a gate everyone learns to skip. So this is a RATCHET: declare the
# current debt in package.json as `stheno.forkDebt`, and the build fails the
# moment the count goes UP. New forking is blocked today; the existing debt is
# paid down by lowering the numbers. Absent `forkDebt` means the debt must be 0.
#
# frontend-default is exempt from both by definition — it *is* the Medusa-derived
# frontend, and ADR-002 says so deliberately.
set -euo pipefail
cd "$(dirname "$0")/.."

PKGS="apps/storefront/packages"
BASE="$PKGS/frontend-default/src"
FAIL=0
FOUND=0

for pkg in "$PKGS"/frontend-*/; do
  name="$(basename "$pkg")"
  case "$name" in frontend-default|frontend-fixture-*) continue ;; esac

  independent="$(node -p "(require('./$pkg/package.json').stheno||{}).independent === true" 2>/dev/null || echo false)"
  if [ "$independent" != "true" ]; then
    echo "  $name: not declared independent, skipped."
    continue
  fi
  FOUND=$((FOUND + 1))

  debt_files="$(node -p "((require('./$pkg/package.json').stheno||{}).forkDebt||{}).files||0" 2>/dev/null || echo 0)"
  debt_ui="$(node -p "((require('./$pkg/package.json').stheno||{}).forkDebt||{}).medusaUi||0" 2>/dev/null || echo 0)"

  node - "$pkg/src" "$BASE" "$name" "$debt_files" "$debt_ui" <<'NODE'
const fs = require("fs")
const path = require("path")
const [themeDir, baseDir, name, debtFiles, debtUi] = process.argv.slice(2)
const allowedForked = Number(debtFiles)
const allowedUi = Number(debtUi)

// Strip everything that only decides how it looks. What is left is the markup:
// element structure, props, and the shape of the component. If that still
// matches frontend-default, the file was forked, whatever its palette says.
const normalise = (file) =>
  fs
    .readFileSync(file, "utf8")
    .replace(/className=(\{[^}]*\}|"[^"]*")/g, "C")
    .replace(/\s+/g, " ")
    .trim()

const walk = (dir) =>
  fs.existsSync(dir)
    ? fs.readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
        const p = path.join(dir, e.name)
        return e.isDirectory() ? walk(p) : /\.tsx?$/.test(p) ? [p] : []
      })
    : []

const files = walk(themeDir)
const forked = []
const medusaUi = []

for (const file of files) {
  const twin = path.join(baseDir, path.relative(themeDir, file))
  if (fs.existsSync(twin) && normalise(file) === normalise(twin)) {
    forked.push(path.relative(themeDir, file))
  }

  // `clx` is a classname joiner, not a component — it renders nothing.
  const src = fs.readFileSync(file, "utf8")
  const imports = [...src.matchAll(/import\s*\{([^}]*)\}\s*from\s*"@medusajs\/ui"/g)]
  const named = imports
    .flatMap((m) => m[1].split(","))
    .map((s) => s.trim().split(/\s+as\s+/)[0])
    .filter((s) => s && s !== "clx")
  if (named.length) {
    medusaUi.push(`${path.relative(themeDir, file)}  (${[...new Set(named)].join(", ")})`)
  }
}

let bad = 0

const report = (label, hits, allowed, detail) => {
  if (hits.length > allowed) {
    console.error(
      `FAIL: ${name} — ${hits.length} file(s) ${detail}, but the declared debt is ${allowed}.`
    )
    console.error("  New forking is not allowed. Offenders:")
    hits.slice(0, 15).forEach((f) => console.error("    " + f))
    if (hits.length > 15) console.error(`    ... ${hits.length - 15} more`)
    bad = 1
    return
  }
  if (hits.length < allowed) {
    console.log(
      `  ${name}: ${label} down to ${hits.length} (declared ${allowed}) — lower stheno.forkDebt.${
        label === "forked" ? "files" : "medusaUi"
      } to ${hits.length} so it cannot creep back.`
    )
    return
  }
  if (hits.length) {
    console.log(`  ${name}: ${hits.length} ${label} — at the declared debt, not above it.`)
  }
}

report("forked", forked, allowedForked, "are frontend-default's markup with different styling")
report("medusa-ui", medusaUi, allowedUi, "render @medusajs/ui components")

if (!bad && !forked.length && !medusaUi.length) {
  console.log(
    `PASS: ${name} — ${files.length} file(s), none forked from frontend-default, no @medusajs/ui components.`
  )
} else if (!bad) {
  console.log(`PASS (ratchet): ${name} — debt did not increase.`)
}
process.exit(bad)
NODE
  # shellcheck disable=SC2181
  [ $? -eq 0 ] || FAIL=1
done

if [ "$FOUND" -eq 0 ]; then
  echo "No frontend declares stheno.independent — nothing to check."
fi
exit "$FAIL"
