import { chromium } from "playwright"
import { tmpdir } from "node:os"
import { join } from "node:path"

const origin = process.env.MAHAL_PRODUCTION_ORIGIN ?? "http://127.0.0.1:8003"
const routes = [
  "/us",
  "/us/store",
  "/us/products/stheno-performance-hoodie",
  "/us/cart",
  "/us/checkout",
  "/us/account",
]
const sessionBound = new Set(["/us/checkout", "/us/account"])
const browser = await chromium.launch({ headless: true })
const results = []
const failures = []

for (const route of routes) {
  const page = await browser.newPage({ viewport: { width: 390, height: 844 } })
  const errors = []
  page.on("pageerror", (error) => errors.push(error.message))
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text())
  })
  const response = await page.goto(`${origin}${route}`, { waitUntil: "networkidle" })
  const metrics = await page.evaluate(() => ({
    title: document.title,
    headings: document.querySelectorAll("h1, h2").length,
    overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    mahalHeader: Boolean(document.querySelector(".site-header")),
  }))

  const expectedNoSession = sessionBound.has(route) && response?.status() === 404
  if (!response?.ok() && !expectedNoSession) failures.push(`${route}: HTTP ${response?.status()}`)
  const actionableErrors = expectedNoSession
    ? errors.filter((error) => !error.includes("404 (Not Found)"))
    : errors
  if (actionableErrors.length) failures.push(`${route}: ${actionableErrors.join(" | ")}`)
  if (metrics.headings === 0) failures.push(`${route}: no page headings`)
  if (metrics.overflow > 1) failures.push(`${route}: ${metrics.overflow}px horizontal overflow`)
  if (!metrics.mahalHeader && !route.includes("checkout")) failures.push(`${route}: Mahal header missing`)

  if (route === "/us") {
    await page.screenshot({ path: join(tmpdir(), "mahal-production-home-mobile.png"), fullPage: true })
  }
  if (route.includes("products/")) {
    await page.screenshot({ path: join(tmpdir(), "mahal-production-product-mobile.png"), fullPage: true })
  }

  results.push({ route, status: response?.status(), ...metrics })
  await page.close()
}

await browser.close()
console.log(JSON.stringify({ results, failures }, null, 2))
if (failures.length) process.exitCode = 1
