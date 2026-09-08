import { chromium } from "playwright"
import { tmpdir } from "node:os"
import { join } from "node:path"

const browser = await chromium.launch({ headless: true })
const page = await browser.newPage({ viewport: { width: 390, height: 844 } })
const logs = []

page.on("console", (message) => logs.push(`${message.type()}: ${message.text()}`))
page.on("pageerror", (error) => logs.push(`pageerror: ${error.message}`))
page.on("requestfailed", (request) =>
  logs.push(`requestfailed: ${request.url()} ${request.failure()?.errorText ?? "unknown"}`)
)
page.on("response", (response) => {
  if (response.status() >= 400) logs.push(`response: ${response.status()} ${response.url()}`)
})

const baseUrl = process.env.MAHAL_QA_URL ?? "http://127.0.0.1:8003/#/"
const response = await page.goto(baseUrl, { waitUntil: "networkidle" })
await page.screenshot({ path: join(tmpdir(), "mahal-vite-qa.png"), fullPage: true })

console.log(
  JSON.stringify(
    {
      status: response?.status(),
      title: await page.title(),
      text: (await page.locator("body").innerText()).slice(0, 600),
      root: (await page.locator("#root").innerHTML()).slice(0, 600),
      overflow: await page.evaluate(() =>
        [...document.querySelectorAll("*")]
          .map((element) => ({
            tag: element.tagName,
            className: element.className,
            right: element.getBoundingClientRect().right,
            width: element.getBoundingClientRect().width,
          }))
          .filter((element) => element.right > document.documentElement.clientWidth + 1)
          .slice(0, 20)
      ),
      logs,
    },
    null,
    2
  )
)

await browser.close()
