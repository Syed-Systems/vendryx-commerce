import { afterEach, describe, expect, it, vi } from "vitest"
import { extractLogoPalette } from "./logo-palette"

describe("logo palette source guardrails", () => {
  afterEach(() => vi.unstubAllGlobals())

  it("rejects SVG data URLs and oversized input before image decoding", async () => {
    const decode = vi.spyOn(window, "Image")

    await expect(extractLogoPalette("data:image/svg+xml;base64,PHN2Zy8+")).resolves.toBeNull()
    await expect(extractLogoPalette("x".repeat(7 * 1024 * 1024 + 1))).resolves.toBeNull()
    expect(decode).not.toHaveBeenCalled()
  })

  it("never fetches a cross-origin logo", async () => {
    const fetchLogo = vi.fn()
    vi.stubGlobal("fetch", fetchLogo)

    await expect(extractLogoPalette("https://outside.example/logo.png")).resolves.toBeNull()
    expect(fetchLogo).not.toHaveBeenCalled()
  })

  it("rejects a same-origin SVG response before reading its body", async () => {
    const fetchLogo = vi.fn().mockResolvedValue(new Response("<svg/>", {
      status: 200,
      headers: { "content-type": "image/svg+xml" },
    }))
    vi.stubGlobal("fetch", fetchLogo)

    await expect(extractLogoPalette("/logo.svg")).resolves.toBeNull()
    expect(fetchLogo).toHaveBeenCalledOnce()
  })
})
