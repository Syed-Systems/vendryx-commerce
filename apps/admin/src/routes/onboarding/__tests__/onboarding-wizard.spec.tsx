import { describe, expect, it, vi } from "vitest"

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
import { OnboardingWizard } from "../onboarding-wizard"
import * as onboardingHook from "../../../hooks/api/onboarding"

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

describe("Onboarding Wizard Component", () => {
  it("renders the Store Setup Guide heading and progress indicator", () => {
    vi.spyOn(onboardingHook, "useOnboarding").mockReturnValue({
      data: {
        is_completed: false,
        dismissed: false,
        completion_percentage: 60,
        completed_steps_count: 3,
        total_steps_count: 5,
        steps: [
          {
            id: "store",
            title: "Store Profile",
            description: "Set your legal store name and default currency",
            completed: true,
            href: "/settings/store",
          },
          {
            id: "products",
            title: "Add Products",
            description: "Create catalog items with pricing, inventory variants, and images",
            completed: true,
            href: "/products/create",
          },
          {
            id: "shipping",
            title: "Couriers & Delivery Zones",
            description: "Connect Pathao, Steadfast, RedX or setup manual shipping rates",
            completed: true,
            href: "/settings/couriers",
          },
          {
            id: "payment",
            title: "Payment Methods",
            description: "Enable Cash on Delivery (COD), bKash, or card gateway",
            completed: false,
            href: "/settings/payment-providers",
          },
          {
            id: "branding",
            title: "Storefront Branding",
            description: "Upload your store logo and customize brand colors",
            completed: false,
            href: "/settings/branding",
          },
        ],
      },
      isLoading: false,
    } as any)

    vi.spyOn(onboardingHook, "useDismissOnboarding").mockReturnValue({
      mutateAsync: vi.fn(),
    } as any)

    vi.spyOn(onboardingHook, "useResetOnboarding").mockReturnValue({
      mutateAsync: vi.fn(),
    } as any)

    renderWithProviders(<OnboardingWizard />)

    expect(screen.getByRole("heading", { name: /store setup guide/i })).toBeDefined()
    expect(screen.getByText(/3 of 5 completed \(60%\)/i)).toBeDefined()
    expect(screen.getByText(/1\. Store Profile/i)).toBeDefined()
    expect(screen.getByText(/3\. Couriers & Delivery Zones/i)).toBeDefined()
    expect(screen.getByText(/4\. Payment Methods/i)).toBeDefined()
  })

  it("renders the celebration banner when all milestones are finished", () => {
    vi.spyOn(onboardingHook, "useOnboarding").mockReturnValue({
      data: {
        is_completed: true,
        dismissed: false,
        completion_percentage: 100,
        completed_steps_count: 5,
        total_steps_count: 5,
        steps: [],
      },
      isLoading: false,
    } as any)

    vi.spyOn(onboardingHook, "useDismissOnboarding").mockReturnValue({
      mutateAsync: vi.fn(),
    } as any)

    vi.spyOn(onboardingHook, "useResetOnboarding").mockReturnValue({
      mutateAsync: vi.fn(),
    } as any)

    renderWithProviders(<OnboardingWizard />)

    expect(screen.getByText(/store setup complete!/i)).toBeDefined()
    expect(screen.getByRole("button", { name: /open merchant home/i })).toBeDefined()
  })
})
