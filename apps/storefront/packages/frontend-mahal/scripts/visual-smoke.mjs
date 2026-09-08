import { chromium } from "playwright"

const baseUrl = process.env.MAHAL_QA_ORIGIN ?? "http://127.0.0.1:8004"
const routes = ["/", "/catalog", "/product", "/cart", "/checkout", "/account"]
const viewports = [
  { name: "mobile", width: 390, height: 844 },
  { name: "desktop", width: 1178, height: 812 },
]
const browser = await chromium.launch({ headless: true })
const failures = []
const results = []

for (const viewport of viewports) {
  for (const route of routes) {
    const page = await browser.newPage({ viewport })
    const errors = []
    page.on("pageerror", (error) => errors.push(error.message))
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text())
    })

    const response = await page.goto(`${baseUrl}/#${route}`, { waitUntil: "networkidle" })
    const metrics = await page.evaluate(() => ({
      title: document.title,
      bodyHeight: document.body.scrollHeight,
      horizontalOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
      headings: document.querySelectorAll("h1, h2").length,
      images: [...document.images].map((image) => ({
        source: image.currentSrc || image.src,
        complete: image.complete,
        width: image.naturalWidth,
      })),
    }))
    const brokenImages = metrics.images.filter((image) => !image.complete || image.width === 0)

    if (response?.status() !== 200) failures.push(`${viewport.name} ${route}: HTTP ${response?.status()}`)
    if (errors.length) failures.push(`${viewport.name} ${route}: ${errors.join(" | ")}`)
    if (brokenImages.length) failures.push(`${viewport.name} ${route}: ${brokenImages.length} broken images`)
    if (metrics.horizontalOverflow > 1) {
      failures.push(`${viewport.name} ${route}: ${metrics.horizontalOverflow}px horizontal overflow`)
    }
    if (metrics.headings === 0) failures.push(`${viewport.name} ${route}: no headings rendered`)

    results.push({
      viewport: viewport.name,
      route,
      status: response?.status(),
      bodyHeight: metrics.bodyHeight,
      headings: metrics.headings,
      images: metrics.images.length,
      horizontalOverflow: metrics.horizontalOverflow,
    })
    await page.close()
  }
}

const interactionPage = await browser.newPage({ viewport: viewports[0] })
await interactionPage.goto(`${baseUrl}/#/`, { waitUntil: "networkidle" })
await interactionPage.getByRole("button", { name: "Open navigation" }).click()
if (!(await interactionPage.getByRole("region", { name: "Jewellery menu" }).isVisible())) {
  failures.push("mobile menu: did not open")
}
await interactionPage.getByRole("button", { name: "Shopping bag" }).click()
if (!(await interactionPage.getByRole("dialog", { name: "YOUR BAG (1)" }).isVisible())) {
  failures.push("cart drawer: did not open")
}
await interactionPage.getByRole("button", { name: "Close bag" }).click()
if (await interactionPage.getByRole("dialog", { name: "YOUR BAG (1)" }).isVisible()) {
  failures.push("cart drawer: did not close")
}
await interactionPage.close()
await browser.close()

console.log(JSON.stringify({ results, failures }, null, 2))
if (failures.length) process.exitCode = 1
