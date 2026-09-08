import { describe, expect, it, vi, beforeEach } from "vitest"

const storageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString()
    },
    removeItem: (key: string) => {
      delete store[key]
    },
    clear: () => {
      store = {}
    },
  }
})()

vi.hoisted(() => {
  const g = global as any
  g.__BACKEND_URL__ = "http://localhost:9000"
  g.__AUTH_TYPE__ = "session"
  g.__JWT_TOKEN_STORAGE_KEY__ = ""
})

if (typeof window !== "undefined") {
  Object.defineProperty(window, "localStorage", { value: storageMock, writable: true })
}
if (typeof global !== "undefined") {
  Object.defineProperty(global, "localStorage", { value: storageMock, writable: true })
}

import { render, screen, fireEvent } from "@testing-library/react"
import { DomainsView } from "../domains-view"
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

describe("DomainsView Component", () => {
  beforeEach(() => {
    storageMock.clear()
  })

  it("renders default Vendryx subdomain card", () => {
    renderWithProviders(<DomainsView />)

    expect(screen.getByText(/Default Subdomain/i)).not.toBeNull()
    expect(screen.getByText(/Always active and secured by Vendryx Cloud/i)).not.toBeNull()
    expect(screen.getByRole("button", { name: /Visit store/i })).not.toBeNull()
  })

  it("shows custom domain entry form when no custom domain configured", () => {
    renderWithProviders(<DomainsView />)

    expect(screen.getByText(/Custom Primary Domain/i)).not.toBeNull()
    expect(screen.getByPlaceholderText(/e\.g\. shop\.syedsystems\.com/i)).not.toBeNull()
    expect(screen.getByRole("button", { name: /Connect domain/i })).not.toBeNull()
  })

  it("allows entering a domain and renders required DNS records", async () => {
    renderWithProviders(<DomainsView />)

    const input = screen.getByPlaceholderText(/e\.g\. shop\.syedsystems\.com/i)
    fireEvent.change(input, { target: { value: "shop.syedstore.com" } })

    const connectButton = screen.getByRole("button", { name: /Connect domain/i })
    fireEvent.click(connectButton)

    // Should now show configured domain details and DNS table
    expect(screen.getByText("shop.syedstore.com")).not.toBeNull()
    expect(screen.getByText(/Pending DNS Verification/i)).not.toBeNull()
    expect(screen.getByText(/Required DNS Configuration/i)).not.toBeNull()
    expect(screen.getAllByText("shops.vendryx.com").length).toBeGreaterThan(0)
    expect(screen.getByRole("button", { name: /Verify DNS records/i })).not.toBeNull()
  })
})
