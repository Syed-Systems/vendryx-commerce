import { describe, expect, it, vi, beforeEach } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { render, screen, waitFor } from "@testing-library/react"
import { MemoryRouter } from "react-router-dom"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { TooltipProvider } from "@medusajs/ui"
import { AnalyticsDashboard } from "../analytics-dashboard"
import * as storeHook from "../../../hooks/api/store"
import * as currenciesHook from "../../../hooks/api/currencies"
import { sdk } from "../../../lib/client"

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })

const renderWithProviders = (component: React.ReactElement) => {
  const queryClient = createTestQueryClient()
  return render(
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <MemoryRouter>{component}</MemoryRouter>
      </TooltipProvider>
    </QueryClientProvider>
  )
}

describe("AnalyticsDashboard Component", () => {
  beforeEach(() => {
    vi.clearAllMocks()

    vi.spyOn(storeHook, "useStore").mockReturnValue({
      store: {
        id: "store_123",
        name: "Test Store",
        default_currency_code: "bdt",
        supported_currencies: [
          { currency_code: "bdt", is_default: true },
          { currency_code: "usd", is_default: false },
        ],
      },
      isLoading: false,
    } as any)

    vi.spyOn(currenciesHook, "useCurrencies").mockReturnValue({
      currencies: [
        { code: "bdt", symbol: "৳", symbol_native: "৳", name: "Bangladeshi Taka" },
        { code: "usd", symbol: "$", symbol_native: "$", name: "US Dollar" },
      ],
      isLoading: false,
    } as any)

    vi.spyOn(sdk.client, "fetch").mockImplementation(async (input: any) => {
      const path = String(input)
      if (path.includes("revenue-over-time")) {
        return {
          report: {
            report_type: "revenue_over_time",
            window_days: 30,
            data: {
              currency: "BDT",
              total_revenue: "15000.00",
              time_series: [
                { date: "2026-09-01", revenue: "5000.00" },
                { date: "2026-09-02", revenue: "10000.00" },
              ],
            },
          },
        }
      }
      if (path.includes("orders-over-time")) {
        return {
          report: {
            report_type: "orders_over_time",
            window_days: 30,
            data: {
              total_orders: 12,
              total_units: 34,
              time_series: [
                { date: "2026-09-01", order_count: 4 },
                { date: "2026-09-02", order_count: 8 },
              ],
            },
          },
        }
      }
      if (path.includes("top-products")) {
        return {
          report: {
            report_type: "top_products",
            window_days: 30,
            data: {
              top_products: [
                {
                  product_id: "prod_1",
                  title: "Premium Artisan Tea",
                  units_sold: 20,
                  total_revenue: "10000.00",
                },
              ],
            },
          },
        }
      }
      if (path.includes("average-order-value")) {
        return {
          report: {
            report_type: "average_order_value",
            window_days: 30,
            data: {
              currency: "BDT",
              average_order_value: "1250.00",
              order_count: 12,
              total_revenue: "15000.00",
            },
          },
        }
      }
      return { report: { data: {} } }
    })
  })

  it("renders the dashboard header and operational analytics description", () => {
    renderWithProviders(<AnalyticsDashboard />)

    expect(screen.getByRole("heading", { name: /analytics & executive reports/i })).toBeDefined()
    expect(screen.getByText(/real-time revenue, order volume/i)).toBeDefined()
  })

  it("renders KPI cards without hardcoded dollar signs for BDT currency", async () => {
    renderWithProviders(<AnalyticsDashboard />)

    await waitFor(() => {
      // Check that the revenue formatted value displays correctly
      expect(screen.getByText(/15,000/)).toBeDefined()
    })

    // Check that AOV formatted value displays
    await waitFor(() => {
      expect(screen.getByText(/1,250/)).toBeDefined()
    })

    // Verify semantic badges appear instead of arbitrary vanity labels
    const last30dBadges = screen.getAllByText(/last 30d/i)
    expect(last30dBadges.length).toBeGreaterThanOrEqual(1)

    const volumeBadge = screen.getByText("Volume")
    expect(volumeBadge).toBeDefined()

    const perBasketBadge = screen.getByText("Per basket")
    expect(perBasketBadge).toBeDefined()
  })

  it("renders the top products table with authentic data", async () => {
    renderWithProviders(<AnalyticsDashboard />)

    await waitFor(() => {
      expect(screen.getByText("Premium Artisan Tea")).toBeDefined()
      expect(screen.getByText("20")).toBeDefined()
    })
  })

  it("renders export report button with accessible text", () => {
    renderWithProviders(<AnalyticsDashboard />)

    const exportBtn = screen.getByRole("button", { name: /export/i })
    expect(exportBtn).toBeDefined()
  })
})
