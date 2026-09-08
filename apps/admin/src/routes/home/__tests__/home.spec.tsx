import { describe, expect, it, vi } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__STOREFRONT_URL__ = "http://localhost:8000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { render, screen } from "@testing-library/react"
import { MemoryRouter } from "react-router-dom"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { TooltipProvider } from "@medusajs/ui"
import { Home } from "../home"
import * as homeHook from "../../../hooks/api/home"
import * as brandingHook from "../../../hooks/api/branding"
import * as usersHook from "../../../hooks/api/users"
import * as onboardingHook from "../../../hooks/api/onboarding"
import * as todayPulseHook from "../../../hooks/api/today-pulse"
import * as recentOrdersHook from "../../../hooks/api/recent-orders"
import * as productsHook from "../../../hooks/api/products"
import * as storeHook from "../../../hooks/api/store"

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

// ── Shared mock defaults ────────────────────────────────────────────────
const mockDefaults = () => {
  vi.spyOn(usersHook, "useMe").mockReturnValue({
    user: { first_name: "Syed", last_name: "Rahman" },
    isLoading: false,
  } as any)

  vi.spyOn(brandingHook, "useBranding").mockReturnValue({
    data: {
      store_name: "Stheno Flagship Store",
      font_key: "inter",
      color_primary: "#111827",
      color_accent: "#2563EB",
      color_background: "#FFFFFF",
    },
    isLoading: false,
  } as any)

  vi.spyOn(onboardingHook, "useOnboarding").mockReturnValue({
    data: { is_completed: true, dismissed: false, completion_percentage: 100, completed_steps_count: 5, total_steps_count: 5, steps: [] },
    isLoading: false,
  } as any)

  vi.spyOn(todayPulseHook, "useTodayPulse").mockReturnValue({
    today: null,
    isLoading: false,
    isError: true,
  } as any)

  vi.spyOn(recentOrdersHook, "useRecentOrders").mockReturnValue({
    orders: null,
    totalCount: 0,
    isLoading: false,
  } as any)

  vi.spyOn(productsHook, "useProducts").mockReturnValue({
    products: [],
    count: 12,
    isLoading: false,
  } as any)

  vi.spyOn(storeHook, "useStore").mockReturnValue({
    store: {
      supported_currencies: [{ currency_code: "usd", is_default: true }],
    },
    isLoading: false,
  } as any)
}

describe("Home Cockpit Component", () => {
  it("renders time-aware greeting with user name and quick actions", () => {
    mockDefaults()
    vi.spyOn(homeHook, "useHomeSummary").mockReturnValue({
      summary: { pending_orders: { count: 0, items: [] }, low_stock: { count: 0, threshold: 5, items: [] }, errors: [] },
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as any)

    renderWithProviders(<Home />)

    // Should contain the user's first name in the greeting
    expect(screen.getByText(/Syed/)).toBeDefined()
    // Should display the live store indicator
    expect(screen.getByText("Live Store")).toBeDefined()
    // Should have Add Product button
    expect(screen.getByText("Add Product")).toBeDefined()
  })

  it("renders the Store Vitals cards (Revenue, Orders, Products, Health)", () => {
    mockDefaults()
    vi.spyOn(homeHook, "useHomeSummary").mockReturnValue({
      summary: { pending_orders: { count: 0, items: [] }, low_stock: { count: 0, threshold: 5, items: [] }, errors: [] },
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as any)

    renderWithProviders(<Home />)

    expect(screen.getByText("Today's Revenue")).toBeDefined()
    expect(screen.getByText("Active Products")).toBeDefined()
    expect(screen.getByText("Store Health")).toBeDefined()
  })

  it("renders attention strip with pending orders and low stock counts when items exist", () => {
    mockDefaults()
    vi.spyOn(homeHook, "useHomeSummary").mockReturnValue({
      summary: {
        pending_orders: {
          count: 2,
          items: [
            {
              id: "order_123",
              display_id: 1001,
              created_at: "2026-09-08T00:00:00Z",
              currency_code: "usd",
              total: "49.99",
              payment_status: "awaiting",
            },
          ],
        },
        low_stock: {
          count: 3,
          threshold: 5,
          items: [],
        },
        errors: [],
      },
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as any)

    renderWithProviders(<Home />)

    expect(screen.getByText("Action Required")).toBeDefined()
    expect(screen.getByText("Orders Awaiting Processing")).toBeDefined()
    expect(screen.getByText("Low Stock Inventory Alerts")).toBeDefined()
  })

  it("renders first sale launchpad when orders stream is empty", () => {
    mockDefaults()
    vi.spyOn(homeHook, "useHomeSummary").mockReturnValue({
      summary: {
        pending_orders: { count: 0, items: [] },
        low_stock: { count: 0, threshold: 5, items: [] },
        errors: [],
      },
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as any)

    renderWithProviders(<Home />)

    expect(screen.getByText(/Your store is live and ready for checkout/i)).toBeDefined()
    expect(screen.getByText("1. Add Products")).toBeDefined()
    expect(screen.getByText("2. Payment Gateways")).toBeDefined()
  })

  it("renders recent orders when orders exist", () => {
    mockDefaults()
    vi.spyOn(homeHook, "useHomeSummary").mockReturnValue({
      summary: {
        pending_orders: { count: 0, items: [] },
        low_stock: { count: 0, threshold: 5, items: [] },
        errors: [],
      },
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as any)

    vi.spyOn(recentOrdersHook, "useRecentOrders").mockReturnValue({
      orders: [
        {
          id: "order_abc",
          display_id: 1042,
          currency_code: "bdt",
          total: "1250.00",
          status: "pending",
          fulfillment_status: "not_fulfilled",
          payment_status: "awaiting",
          created_at: "2026-09-08T23:50:00Z",
          shipping_address: { first_name: "Rahim", last_name: null },
        },
      ],
      totalCount: 1,
      isLoading: false,
    } as any)

    renderWithProviders(<Home />)

    expect(screen.getByText("Recent Orders")).toBeDefined()
    expect(screen.getByText("#1042")).toBeDefined()
    expect(screen.getByText("Rahim")).toBeDefined()
  })

  it("renders error state when summary fails", () => {
    mockDefaults()
    vi.spyOn(homeHook, "useHomeSummary").mockReturnValue({
      summary: undefined,
      isLoading: false,
      isError: true,
      error: { message: "Network error" },
      refetch: vi.fn(),
    } as any)

    renderWithProviders(<Home />)

    expect(screen.getByText("Could not load dashboard")).toBeDefined()
    expect(screen.getByText("Network error")).toBeDefined()
  })
})
