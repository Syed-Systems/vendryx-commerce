/**
 * Control-plane API client.
 *
 * Separate from the admin dashboard's SDK on purpose: this surface reads across every
 * tenant and carries an operator credential, where `@medusajs/js-sdk` is built around a
 * single store's session (ADR-012).
 */

const BACKEND = __BACKEND_URL__

/** Where the operator token lives. Deliberately session-scoped, not persistent. */
const TOKEN_KEY = "stheno_platform_token"

export function getToken(): string | null {
  return sessionStorage.getItem(TOKEN_KEY)
}

export function setToken(token: string) {
  sessionStorage.setItem(TOKEN_KEY, token)
}

export function clearToken() {
  sessionStorage.removeItem(TOKEN_KEY)
}

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number
  ) {
    super(message)
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const token = getToken()
  const response = await fetch(`${BACKEND}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(init?.headers ?? {}),
    },
  })

  if (!response.ok) {
    // The control plane answers 404 rather than 401 to an unauthorised caller, so it
    // does not confirm it exists. That means a wrong token and a wrong URL look the
    // same here — the message says so rather than guessing.
    const body = await response.json().catch(() => ({}))
    throw new ApiError(
      body.message || "Request failed. Check the operator token.",
      response.status
    )
  }

  return response.json() as Promise<T>
}

export interface Tenant {
  id: string
  name: string
  status: string
  tier: string
  subscription_status: string
  trial_ends_at: string | null
  term_ends_at: string | null
  created_at: string
}

export const listTenants = () =>
  request<{ tenants: Tenant[]; count: number }>("/platform/tenants")

export const setTier = (tenantId: string, tier: string) =>
  request<{ subscription: { tier: string } }>(
    `/platform/tenants/${tenantId}/tier`,
    { method: "POST", body: JSON.stringify({ tier }) }
  )

export const createTenant = (input: {
  id: string
  name: string
  tier: string
  domain?: string
}) =>
  request<{ tenant: { id: string } }>("/platform/tenants", {
    method: "POST",
    body: JSON.stringify(input),
  })

/** Per-tenant health, read through the tenant's own host so the server resolves it. */
export async function tenantHealth(): Promise<{
  status: string
  database: { pool_saturation: number; pool_size: number; pool_max_size: number }
} | null> {
  try {
    const response = await fetch(`${BACKEND}/health`)
    if (!response.ok) return null
    return await response.json()
  } catch {
    return null
  }
}
