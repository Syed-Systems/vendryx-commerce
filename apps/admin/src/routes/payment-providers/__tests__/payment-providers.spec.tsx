import { describe, expect, it, vi } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { render, screen } from "@testing-library/react"
import { ProviderLogo } from "../components/provider-logo"
import { CodRulesCard } from "../components/cod-rules-card"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { MemoryRouter } from "react-router-dom"

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

describe("Payment Providers Components", () => {
  describe("ProviderLogo", () => {
    it("renders bKash logo badge correctly", () => {
      const { container } = render(<ProviderLogo id="pp_bkash_bkash" />)
      expect(container.querySelector("img")?.getAttribute("alt")).toBe("bKash")
      expect((container.firstChild as HTMLElement)?.getAttribute("title")).toBe(
        "bKash Payment Gateway"
      )
    })

    it("renders Stripe logo badge correctly", () => {
      const { container } = render(<ProviderLogo id="pp_stripe_stripe" />)
      expect(container.querySelector("img")?.getAttribute("alt")).toBe("Stripe")
      expect((container.firstChild as HTMLElement)?.getAttribute("title")).toBe(
        "Stripe Global Payment Gateway"
      )
    })

    it("renders SSLCommerz logo badge correctly", () => {
      const { container } = render(<ProviderLogo id="pp_sslcommerz_sslcommerz" />)
      expect(container.querySelector("img")?.getAttribute("alt")).toBe("SSLCommerz")
      expect((container.firstChild as HTMLElement)?.getAttribute("title")).toBe(
        "SSLCommerz Payment Gateway"
      )
    })

    it("renders Cash on Delivery / Offline logo badge correctly", () => {
      const { container } = render(<ProviderLogo id="pp_system_default" />)
      expect((container.firstChild as HTMLElement)?.getAttribute("title")).toBe(
        "Cash on Delivery & Parcel Collection"
      )
    })
  })

  describe("CodRulesCard", () => {
    it("renders COD rules configuration controls", () => {
      renderWithProviders(<CodRulesCard />)
      expect(screen.queryByText("Cash on Delivery (COD) Rules")).not.toBeNull()
      expect(screen.queryByText("Maximum Order Amount (৳)")).not.toBeNull()
      expect(screen.queryByText("Dhaka Metro Exclusive")).not.toBeNull()
      expect(screen.queryByText("Require Order Verification Call / SMS")).not.toBeNull()
      expect(screen.queryByText("Customer Checkout Note")).not.toBeNull()
    })

    it("starts with Save button disabled when no changes were made", () => {
      renderWithProviders(<CodRulesCard />)
      const saveButton = screen.getByRole("button", { name: /Save COD Rules/i }) as HTMLButtonElement
      expect(saveButton.disabled).toBe(true)
    })
  })
})
