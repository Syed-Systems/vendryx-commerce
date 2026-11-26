import { describe, expect, it, vi, beforeEach } from "vitest"

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

import { render, screen } from "@testing-library/react"
import { MemoryRouter } from "react-router-dom"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { TooltipProvider } from "@medusajs/ui"

import { Home } from "../home/home"
import { OnboardingWizard } from "../onboarding/onboarding-wizard"
import { CouriersList } from "../couriers/couriers-list"
import { SettingsHub } from "../settings/settings-hub"
import { AnalyticsDashboard } from "../analytics/analytics-dashboard"
import { BrandingView } from "../branding/branding-view"

import * as homeHook from "../../hooks/api/home"
import * as onboardingHook from "../../hooks/api/onboarding"
import * as couriersHook from "../../hooks/api/couriers"
import * as storeHook from "../../hooks/api/store"
import * as currenciesHook from "../../hooks/api/currencies"
import * as brandingHook from "../../hooks/api/branding"
import { sdk } from "../../lib/client"

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

describe("Merchant Routes Smoke Suite (ADM-050 / ADM-051)", () => {
  beforeEach(() => {
    vi.clearAllMocks()

    vi.spyOn(homeHook, "useHomeSummary").mockReturnValue({
      summary: {
        pending_orders: { count: 0, items: [] },
        low_stock: { count: 0, threshold: 10, items: [] },
        period: "today",
      },
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as any)

    vi.spyOn(onboardingHook, "useOnboarding").mockReturnValue({
      data: {
        completion_percentage: 40,
        dismissed: false,
        is_completed: false,
        completed_steps_count: 2,
        total_steps_count: 5,
        steps: [
          {
            id: "store_profile",
            title: "Store Profile",
            description: "Store name and currency",
            completed: true,
            href: "/settings/store",
          },
          {
            id: "product_catalog",
            title: "First Product",
            description: "Add a product",
            completed: false,
            href: "/products/new",
          },
        ],
      },
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    } as any)

    vi.spyOn(couriersHook, "useCouriers").mockReturnValue({
      couriers: [
        {
          id: "pathao",
          name: "Pathao Courier",
          status: "connected",
          is_enabled: true,
          webhook_url: "https://api.vendryx.com/webhooks/pathao",
          masked_credentials: { client_id: "••••1234" },
          rates: { inside_dhaka: 60, sub_dhaka: 100, outside_dhaka: 130 },
          last_probe_at: "2026-09-08T00:00:00Z",
          last_probe_latency_ms: 120,
        },
      ],
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    } as any)

    vi.spyOn(storeHook, "useStore").mockReturnValue({
      store: {
        id: "store_123",
        name: "Vendryx Flagship Store",
        supported_currencies: [{ currency_code: "bdt", is_default: true }],
      },
      isLoading: false,
    } as any)

    vi.spyOn(currenciesHook, "useCurrencies").mockReturnValue({
      currencies: [{ code: "bdt", symbol: "৳" }],
      isLoading: false,
    } as any)

    vi.spyOn(brandingHook, "useBranding").mockReturnValue({
      data: {
        store_name: "Vendryx Demo",
        logo_url: "https://vendryx.com/logo.svg",
        color_primary: "#000000",
        color_accent: "#3B82F6",
      },
      isLoading: false,
      isError: false,
    } as any)

    vi.spyOn(sdk.client, "fetch").mockImplementation(async () => {
      return { report: { data: {} } } as any
    })
  })

  it("renders Home Cockpit route without uncaught exceptions", () => {
    renderWithProviders(<Home />)
    expect(screen.getByRole("heading", { name: /Good (morning|afternoon|evening)/i })).toBeDefined()
    expect(screen.getByRole("heading", { name: /recent orders/i })).toBeDefined()
  })

  it("renders Onboarding Wizard route without uncaught exceptions", () => {
    renderWithProviders(<OnboardingWizard />)
    expect(screen.getByRole("heading", { name: /store setup guide/i })).toBeDefined()
    expect(screen.getByText(/store profile/i)).toBeDefined()
  })

  it("renders Courier Logistics route without uncaught exceptions", () => {
    renderWithProviders(<CouriersList />)
    expect(screen.getByRole("heading", { name: /courier logistics/i })).toBeDefined()
    expect(screen.getByText("Pathao Courier")).toBeDefined()
  })

  it("renders Settings Hub route without uncaught exceptions", () => {
    renderWithProviders(<SettingsHub />)
    expect(screen.getByRole("heading", { name: /^settings$/i })).toBeDefined()
    expect(screen.getByText("Delivery & Fulfillment")).toBeDefined()
  })

  it("renders Analytics Dashboard route without uncaught exceptions", () => {
    renderWithProviders(<AnalyticsDashboard />)
    expect(screen.getByRole("heading", { name: /analytics & executive reports/i })).toBeDefined()
  })

  it("renders Store Branding route without uncaught exceptions", () => {
    renderWithProviders(<BrandingView />)
    expect(screen.getByRole("heading", { name: /storefront branding/i })).toBeDefined()
  })
})
