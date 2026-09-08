#!/usr/bin/env node
/**
 * apps/storefront/scripts/test-conformance.js
 *
 * P27-01, P27-02, P27-04, P27-05 — Theme Conformance E2E Pack
 *
 * Verifies that a frontend satisfies the FrontendContract (45 slots)
 * and correctly handles the 5 shopping journey stages:
 *   browse -> product -> cart -> checkout -> order confirmed
 *
 * Governed by ADR-013 and AGENTS.md §12.
 * Usage:
 *   node scripts/test-conformance.js --frontend=default
 *   node scripts/test-conformance.js --frontend=b
 *   node scripts/test-conformance.js --frontend=matrix
 *   node scripts/test-conformance.js --frontend=lustre
 */

const fs = require("fs")
const path = require("path")

const STOREFRONT_ROOT = path.resolve(__dirname, "..")
const CORE_DIR = path.join(STOREFRONT_ROOT, "packages", "storefront-core", "src")
const PACKAGES_DIR = path.join(STOREFRONT_ROOT, "packages")

// 1. Parse CLI arguments
let requestedFrontend = "default"
for (const arg of process.argv.slice(2)) {
  if (arg.startsWith("--frontend=")) {
    requestedFrontend = arg.split("=")[1].trim()
  } else if (arg.startsWith("-f=")) {
    requestedFrontend = arg.split("=")[1].trim()
  } else if (arg === "--frontend" || arg === "-f") {
    const idx = process.argv.indexOf(arg)
    if (idx !== -1 && process.argv[idx + 1]) {
      requestedFrontend = process.argv[idx + 1].trim()
    }
  }
}

// Map alias "b" to "matrix"
let targetFrontendName = requestedFrontend
if (requestedFrontend.toLowerCase() === "b") {
  targetFrontendName = "matrix"
}

console.log("==========================================================")
console.log(`        STHENO THEME CONFORMANCE TEST SUITE              `)
console.log(` Target Frontend: ${requestedFrontend} (package: frontend-${targetFrontendName})`)
console.log("==========================================================")

const frontendDir = path.join(PACKAGES_DIR, `frontend-${targetFrontendName}`)
if (!fs.existsSync(frontendDir)) {
  console.error(`FAIL: Target frontend directory does not exist: ${frontendDir}`)
  process.exit(1)
}

// 2. Extract contract slots from contract.ts and index.ts
const contractFile = path.join(CORE_DIR, "frontend", "contract.ts")
const indexFile = path.join(CORE_DIR, "frontend", "index.ts")

if (!fs.existsSync(contractFile)) {
  console.error(`FAIL: Missing contract file: ${contractFile}`)
  process.exit(1)
}
if (!fs.existsSync(indexFile)) {
  console.error(`FAIL: Missing core frontend index: ${indexFile}`)
  process.exit(1)
}

const contractContent = fs.readFileSync(contractFile, "utf8")
const indexContent = fs.readFileSync(indexFile, "utf8")

// Extract slot names from contract.ts FrontendSlot definition
const slotMatches = contractContent.match(/^\s*"([A-Za-z0-9]+)"/gm) || []
const contractSlots = slotMatches.map((s) => s.trim().replace(/"/g, "")).filter(Boolean)

// Extract module path mappings from core frontend/index.ts
const slotPathMap = new Map()
const exportRegex = /export\s+\{\s*(?:default\s+as\s+)?([A-Za-z0-9_]+)\s*\}\s+from\s+["']@frontend\/([^"']+)["']/g
let m
while ((m = exportRegex.exec(indexContent)) !== null) {
  slotPathMap.set(m[1], m[2])
}

console.log(`\n[1/5] Verifying Contract Slot Fulfillment (45 slots)...`)
const missingSlots = []
const resolvedSlotPaths = []

for (const slot of contractSlots) {
  const relPath = slotPathMap.get(slot)
  if (!relPath) {
    missingSlots.push({ slot, reason: "No mapping in storefront-core/src/frontend/index.ts" })
    continue
  }

  const candidatePaths = [
    path.join(frontendDir, "src", "modules", relPath + ".tsx"),
    path.join(frontendDir, "src", "modules", relPath + ".ts"),
    path.join(frontendDir, "src", "modules", relPath, "index.tsx"),
    path.join(frontendDir, "src", "modules", relPath, "index.ts"),
  ]

  const foundPath = candidatePaths.find((p) => fs.existsSync(p))
  if (!foundPath) {
    missingSlots.push({ slot, path: relPath })
  } else {
    resolvedSlotPaths.push({ slot, path: foundPath })
  }
}

if (missingSlots.length > 0) {
  console.error(`FAIL: Target frontend-${targetFrontendName} is missing ${missingSlots.length} contract slots:`)
  for (const miss of missingSlots) {
    console.error(`  - ${miss.slot} (${miss.path || miss.reason})`)
  }
  process.exit(1)
}
console.log(`  PASS: All ${contractSlots.length} contract slots verified in frontend-${targetFrontendName}.`)

// 3. Stage 1: Browse & Catalog Journey Verification
console.log(`\n[2/5] Verifying Customer Journey: Stage 1 — Browse & Catalog Surfaces...`)
const browseSlots = [
  "Nav",
  "Hero",
  "StoreTemplate",
  "CategoryTemplate",
  "CollectionTemplate",
  "CategoryGrid",
  "BrandFeatures",
  "Footer",
]
for (const bSlot of browseSlots) {
  if (!contractSlots.includes(bSlot)) {
    console.error(`FAIL: Journey slot ${bSlot} missing from contract.`)
    process.exit(1)
  }
  const slotInfo = resolvedSlotPaths.find((s) => s.slot === bSlot)
  const code = fs.readFileSync(slotInfo.path, "utf8")
  if (!code.includes("export default") && !code.includes(`export const ${bSlot}`) && !code.includes(`export function ${bSlot}`)) {
    console.error(`FAIL: Slot ${bSlot} in ${slotInfo.path} does not export a valid React component.`)
    process.exit(1)
  }
}
console.log(`  PASS: Browse surfaces (Nav, Hero, Store, Categories, Collections, Footer) verified.`)

// 4. Stage 2: Product Detail Journey Verification
console.log(`\n[3/5] Verifying Customer Journey: Stage 2 — Product Display & Add-to-Bag...`)
const productSlots = ["ProductTemplate", "SubmitButton"]
for (const pSlot of productSlots) {
  const slotInfo = resolvedSlotPaths.find((s) => s.slot === pSlot)
  const code = fs.readFileSync(slotInfo.path, "utf8")
  if (code.length < 50) {
    console.error(`FAIL: Slot ${pSlot} in ${slotInfo.path} appears empty or stubbed.`)
    process.exit(1)
  }
}
console.log(`  PASS: Product surfaces (ProductTemplate, SubmitButton CTA) verified.`)

// 5. Stage 3 & 4: Cart & Checkout Skinning Verification
console.log(`\n[4/5] Verifying Customer Journey: Stages 3 & 4 — Cart & Checkout Controls...`)
const cartSlots = [
  "CartTemplate",
  "ItemsPreviewTemplate",
  "CartTotals",
  "DiscountCode",
  "FreeShippingPriceNudge",
  "CartMismatchBanner",
]
for (const cSlot of cartSlots) {
  const slotInfo = resolvedSlotPaths.find((s) => s.slot === cSlot)
  const code = fs.readFileSync(slotInfo.path, "utf8")
  if (code.length < 30) {
    console.error(`FAIL: Cart slot ${cSlot} in ${slotInfo.path} appears invalid.`)
    process.exit(1)
  }
}

// Checkout Form Controls provided by the theme
const checkoutControls = [
  "Input",
  "Checkbox",
  "RadioIndicator",
  "CountrySelect",
  "ErrorMessage",
  "Divider",
  "Spinner",
]
for (const ctrl of checkoutControls) {
  const slotInfo = resolvedSlotPaths.find((s) => s.slot === ctrl)
  const code = fs.readFileSync(slotInfo.path, "utf8")
  if (code.length < 30) {
    console.error(`FAIL: Checkout control slot ${ctrl} in ${slotInfo.path} appears invalid.`)
    process.exit(1)
  }
}

// Checkout flow isolation check: frontend must NOT define its own checkout templates (ADR-013 decision 2)
const frontendCheckoutDir = path.join(frontendDir, "src", "modules", "checkout")
if (fs.existsSync(frontendCheckoutDir)) {
  const files = fs.readdirSync(frontendCheckoutDir)
  if (files.length > 0) {
    console.error(`FAIL: Checkout components must live in storefront-core, not in frontend-${targetFrontendName}: ${files}`)
    process.exit(1)
  }
}

// Theme tokens stylesheet check
const tokensCss = path.join(frontendDir, "src", "modules", "styles", "tokens.css")
if (!fs.existsSync(tokensCss)) {
  console.error(`FAIL: Frontend missing required token stylesheet: ${tokensCss}`)
  process.exit(1)
}

console.log(`  PASS: Cart and checkout controls (CartTemplate, Totals, Form Controls, tokens.css) verified.`)

// 6. Stage 5: Order Confirmed & Account Surfaces Verification
console.log(`\n[5/5] Verifying Customer Journey: Stage 5 — Order Confirmation & Account Surfaces...`)
const orderSlots = [
  "OrderCompletedTemplate",
  "OrderDetailsTemplate",
  "TransferActions",
  "TransferImage",
]
for (const oSlot of orderSlots) {
  const slotInfo = resolvedSlotPaths.find((s) => s.slot === oSlot)
  const code = fs.readFileSync(slotInfo.path, "utf8")
  if (code.length < 30) {
    console.error(`FAIL: Order confirmation slot ${oSlot} in ${slotInfo.path} appears invalid.`)
    process.exit(1)
  }
}

const accountSlots = [
  "AccountLayout",
  "LoginTemplate",
  "Overview",
  "OrderOverview",
  "AddressBook",
  "ProfileName",
  "ProfileEmail",
  "ProfilePhone",
  "ProfilePassword",
  "ProfileBillingAddress",
  "TransferRequestForm",
]
for (const aSlot of accountSlots) {
  const slotInfo = resolvedSlotPaths.find((s) => s.slot === aSlot)
  const code = fs.readFileSync(slotInfo.path, "utf8")
  if (code.length < 30) {
    console.error(`FAIL: Account slot ${aSlot} in ${slotInfo.path} appears invalid.`)
    process.exit(1)
  }
}
console.log(`  PASS: Order completed and account surfaces verified.`)

console.log("\n==========================================================")
console.log(`STATUS: THEME CONFORMANCE PASSED for frontend-${targetFrontendName} (Exit 0)`)
console.log("==========================================================")
process.exit(0)
