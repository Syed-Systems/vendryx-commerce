import { describe, expect, it, vi } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { fireEvent, render, screen } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { MemoryRouter } from "react-router-dom"
import { BrandingView, CURATED_FONTS, DEFAULT_BRANDING_VALUES } from "./branding-view"

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
})

const renderWithProviders = (component: React.ReactElement) => {
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>{component}</MemoryRouter>
    </QueryClientProvider>
  )
}

describe("P26-07: Admin Branding UI with Live Preview", () => {
  it("renders branding header and description", () => {
    renderWithProviders(<BrandingView />)
    expect(screen.getByText("Storefront Branding")).toBeDefined()
    expect(screen.getByText(/Customise brand identity, curated typography/)).toBeDefined()
  })

  it("contains all 7 curated font options", () => {
    expect(CURATED_FONTS.length).toBe(7)
    const keys = CURATED_FONTS.map((f) => f.key)
    expect(keys).toContain("inter")
    expect(keys).toContain("geist")
    expect(keys).toContain("noto_sans_bengali")
    expect(keys).toContain("playfair")
    expect(keys).toContain("plus_jakarta_sans")
    expect(keys).toContain("roboto")
    expect(keys).toContain("cinzel")
  })

  it("renders live preview frame with initial default store name", () => {
    renderWithProviders(<BrandingView />)
    const previewName = screen.getByTestId("preview-store-name")
    expect(previewName.textContent).toBe(DEFAULT_BRANDING_VALUES.store_name)
  })

  it("updates live preview immediately when store name changes", () => {
    renderWithProviders(<BrandingView />)
    const storeNameInput = screen.getByLabelText("Store Name") as HTMLInputElement
    fireEvent.change(storeNameInput, { target: { value: "Apex Timepieces" } })

    const previewName = screen.getByTestId("preview-store-name")
    expect(previewName.textContent).toBe("Apex Timepieces")
  })

  it("updates live preview frame styling when primary and background colors change", () => {
    renderWithProviders(<BrandingView />)
    const primaryInput = screen.getByLabelText("Primary Color") as HTMLInputElement
    const bgInput = screen.getByLabelText("Background Color") as HTMLInputElement

    fireEvent.change(primaryInput, { target: { value: "#E11D48" } })
    fireEvent.change(bgInput, { target: { value: "#0F172A" } })

    const previewFrame = screen.getByTestId("branding-preview-frame")
    expect(previewFrame.style.backgroundColor).toBe("rgb(15, 23, 42)")
  })

  it("resets form and preview back to defaults on clicking Reset Defaults", () => {
    renderWithProviders(<BrandingView />)
    const storeNameInput = screen.getByLabelText("Store Name") as HTMLInputElement
    fireEvent.change(storeNameInput, { target: { value: "Temporary Store" } })

    const resetButton = screen.getByText("Reset Defaults")
    fireEvent.click(resetButton)

    expect(storeNameInput.value).toBe(DEFAULT_BRANDING_VALUES.store_name)
    const previewName = screen.getByTestId("preview-store-name")
    expect(previewName.textContent).toBe(DEFAULT_BRANDING_VALUES.store_name)
  })
})
