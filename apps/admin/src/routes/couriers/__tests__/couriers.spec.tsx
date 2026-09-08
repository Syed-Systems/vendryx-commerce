import { describe, expect, it, vi } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { render, screen } from "@testing-library/react"
import { CourierLogo } from "../components/courier-logo"
import { CouriersList } from "../couriers-list"
import { MemoryRouter } from "react-router-dom"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { TooltipProvider } from "@medusajs/ui"

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
      <TooltipProvider>
        <MemoryRouter>{component}</MemoryRouter>
      </TooltipProvider>
    </QueryClientProvider>
  )
}

describe("Courier Logistics Components", () => {
  describe("CourierLogo", () => {
    it("renders Pathao logo badge correctly", () => {
      const { container } = render(<CourierLogo id="pathao" />)
      expect(container.textContent).toContain("P")
      expect((container.firstChild as HTMLElement)?.getAttribute("title")).toBe(
        "Pathao Courier Logistics"
      )
    })

    it("renders Steadfast logo badge correctly", () => {
      const { container } = render(<CourierLogo id="steadfast" />)
      expect(container.textContent).toContain("SF")
      expect((container.firstChild as HTMLElement)?.getAttribute("title")).toBe(
        "Steadfast Courier Services"
      )
    })

    it("renders RedX logo badge correctly", () => {
      const { container } = render(<CourierLogo id="redx" />)
      expect(container.textContent).toContain("RX")
      expect((container.firstChild as HTMLElement)?.getAttribute("title")).toBe(
        "RedX Parcel Delivery"
      )
    })
  })

  describe("CouriersList", () => {
    it("renders Pathao, Steadfast, and RedX service entries", () => {
      renderWithProviders(<CouriersList />)

      expect(screen.queryByText("Courier Logistics")).not.toBeNull()
      expect(screen.queryByText("Pathao Courier")).not.toBeNull()
      expect(screen.queryByText("Steadfast Courier")).not.toBeNull()
      expect(screen.queryByText("RedX Logistics")).not.toBeNull()
    })

    it("displays default zone charges for all courier tiers", () => {
      renderWithProviders(<CouriersList />)

      expect(screen.queryAllByText(/Inside Dhaka:/i).length).toBeGreaterThan(0)
      expect(screen.queryAllByText(/Suburbs:/i).length).toBeGreaterThan(0)
      expect(screen.queryAllByText(/Outside Dhaka:/i).length).toBeGreaterThan(0)
    })
  })
})
