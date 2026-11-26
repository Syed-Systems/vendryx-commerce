import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import "@testing-library/jest-dom/vitest"
import userEvent from "@testing-library/user-event"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

const { fetchMock } = vi.hoisted(() => ({ fetchMock: vi.fn() }))

vi.mock("../../lib/client", () => ({
  sdk: { client: { fetch: fetchMock } },
}))

import { EntitlementsDetail } from "./entitlements-detail"

const renderPage = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <EntitlementsDetail />
    </QueryClientProvider>
  )
}

const billingSubscription = {
  subscription: {
    tier: "free",
    status: "active",
    trial_ends_at: null,
    term_ends_at: null,
    grace_ends_at: null,
    plan_amount: null,
    currency: "USD",
    billing_interval: "month",
    pro_plan: { amount: "1500.00", currency: "BDT", interval: "month" },
  },
}

beforeEach(() => {
  fetchMock.mockImplementation(defaultResponse)
})

afterEach(() => {
  fetchMock.mockReset()
  vi.restoreAllMocks()
})

describe("entitlements billing portal journeys", () => {
  it("falls back to the plan dialog when the billing portal is unavailable and supports keyboard dismissal", async () => {
    const user = userEvent.setup()
    fetchMock.mockImplementation(async (path: string) => {
      if (path === "/admin/billing/portal-url") {
        throw new Error("billing portal unavailable")
      }
      return defaultResponse(path)
    })
    renderPage()

    const changePlan = await screen.findByRole("button", { name: "Change plan" })
    await user.click(changePlan)

    const dialog = await screen.findByRole("dialog", { name: "Select Subscription Plan" })
    expect(dialog.getAttribute("aria-modal")).toBe("true")
    expect(dialog).toHaveFocus()

    const closeButtons = screen.getAllByRole("button", { name: "Close" })
    fireEvent.keyDown(document, { key: "Tab", shiftKey: true })
    expect(closeButtons[1]).toHaveFocus()
    fireEvent.keyDown(document, { key: "Tab" })
    expect(closeButtons[0]).toHaveFocus()

    fireEvent.keyDown(document, { key: "Escape" })
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument())
    expect(changePlan).toHaveFocus()
  })

  it("opens a returned billing portal URL in a new tab", async () => {
    const user = userEvent.setup()
    const open = vi.spyOn(window, "open").mockImplementation(() => null)
    fetchMock.mockImplementation(async (path: string) => {
      if (path === "/admin/billing/portal-url") {
        return { portal_url: "https://billing.example.test/session" }
      }
      return defaultResponse(path)
    })
    renderPage()

    await user.click(await screen.findByRole("button", { name: "Change plan" }))

    await waitFor(() => {
      expect(open).toHaveBeenCalledWith("https://billing.example.test/session", "_blank")
    })
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument()
  })
})

function defaultResponse(path: string) {
  switch (path) {
    case "/admin/entitlements":
      return { tier: "free", trial_ends_at: null, term_ends_at: null, quotas: [], windows: [] }
    case "/admin/feature-flags":
      return { feature_flags: {} }
    case "/admin/notifications/usage":
      return {
        usage: {
          usage_date_utc: "2026-10-25",
          email: { sent: 0, reserved: 0, limit: 200, remaining: 200 },
          whatsapp: { sent: 0, reserved: 0, limit: 300, remaining: 300 },
        },
      }
    case "/admin/billing/subscription":
      return billingSubscription
    case "/admin/billing/invoices":
      return { invoices: [] }
    default:
      throw new Error(`Unexpected request: ${path}`)
  }
}
