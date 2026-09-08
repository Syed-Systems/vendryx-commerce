import { describe, expect, it, vi } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { render, screen, fireEvent } from "@testing-library/react"
import { HelpCenterView } from "../help-center"
import { MemoryRouter } from "react-router-dom"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"

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

describe("HelpCenterView Component", () => {
  it("renders documentation tab with configuration guides", () => {
    renderWithProviders(<HelpCenterView />)

    expect(screen.getByText(/Help & Documentation/i)).not.toBeNull()
    expect(screen.getByText(/Connecting bKash Merchant Checkout/i)).not.toBeNull()
    expect(screen.getByText(/Steadfast Courier Integration/i)).not.toBeNull()
    expect(screen.getByText(/Custom Domain DNS Verification & SSL/i)).not.toBeNull()
  })

  it("filters articles by category when category filter button is clicked", () => {
    renderWithProviders(<HelpCenterView />)

    const paymentsBtn = screen.getByRole("button", { name: "Payments" })
    fireEvent.click(paymentsBtn)

    expect(screen.getByText(/Connecting bKash Merchant Checkout/i)).not.toBeNull()
    expect(screen.getByText(/Configuring Cash on Delivery/i)).not.toBeNull()
  })

  it("switches to contact support tab and shows verified engineering channels", () => {
    renderWithProviders(<HelpCenterView />)

    const contactTabBtn = screen.getByRole("button", { name: /Contact Support/i })
    fireEvent.click(contactTabBtn)

    expect(screen.getByText(/Syed Systems Engineering Support/i)).not.toBeNull()
    expect(screen.getByText(/support@syedsystems.com/i)).not.toBeNull()
    expect(screen.getByText(/Engine Status & Standards/i)).not.toBeNull()
  })
})
