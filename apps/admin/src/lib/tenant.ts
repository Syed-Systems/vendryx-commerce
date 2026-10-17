export interface TenantDefinition {
  id: string
  name: string
  genre: "appliance" | "clothing" | "tech"
  tier: "trial" | "free" | "pro"
  tierLabel: string
  host: string
  defaultEmail: string
  description: string
}

/**
 * The three tenants `scripts/dev-tenants.sh` creates.
 *
 * `id` and `host` must match that script exactly — it registers
 * ten_trial/trial.localhost, ten_free/free.localhost and ten_pro/pro.localhost,
 * and registers each owner as `owner@<host>`. These entries previously carried
 * invented hosts (trial-appliance.localhost, expired-clothing.localhost,
 * pro-tech.localhost) that exist in no tenant table, so the picker pre-filled an
 * email that no owner had and the sign-in returned "Invalid credentials" against
 * a tenant that was never there. The genre, name and tier labels below are
 * display-only and stay as they are.
 */
export const AVAILABLE_TENANTS: TenantDefinition[] = [
  {
    id: "ten_trial",
    name: "Vendryx Appliances",
    genre: "appliance",
    tier: "trial",
    tierLabel: "14-Day Free Trial",
    host: "trial.localhost",
    defaultEmail: "owner@trial.localhost",
    description: "Industrial luxury appliances, active trial lifecycle",
  },
  {
    id: "ten_free",
    name: "Vendryx Fashion & Clothing",
    genre: "clothing",
    tier: "free",
    tierLabel: "Trial Expired (Free)",
    host: "free.localhost",
    defaultEmail: "owner@free.localhost",
    description: "Couture & bridal collection, free tier limits",
  },
  {
    id: "ten_pro",
    name: "Vendryx Pro Tech",
    genre: "tech",
    tier: "pro",
    tierLabel: "Pro Active",
    host: "pro.localhost",
    defaultEmail: "owner@pro.localhost",
    description: "High performance tech store, unmetered pro tier",
  },
]

const STORAGE_KEY = "stheno_tenant_host"

export function getActiveTenantHost(): string {
  if (typeof window === "undefined") {
    return AVAILABLE_TENANTS[0].host
  }

  // 1. Check document.cookie
  const match = document.cookie.match(/stheno_tenant_host=([^;]+)/)
  if (match && match[1]) {
    return decodeURIComponent(match[1].trim())
  }

  // 2. Check localStorage
  const saved = localStorage.getItem(STORAGE_KEY)
  if (saved) {
    return saved
  }

  return AVAILABLE_TENANTS[0].host
}

export function setActiveTenantHost(host: string): void {
  if (typeof window === "undefined") return

  // Set cookie with 1-year expiration
  document.cookie = `stheno_tenant_host=${encodeURIComponent(
    host
  )}; path=/; max-age=31536000; SameSite=Lax`
  localStorage.setItem(STORAGE_KEY, host)
}

export function resolveTenantHostFromEmail(email: string): string {
  const normalized = email.toLowerCase().trim()

  // An exact domain match wins outright. That is the path the picker takes and
  // the only one that cannot be wrong, so it is checked before any keyword
  // guessing below.
  const exact = AVAILABLE_TENANTS.find((t) =>
    normalized.endsWith(`@${t.host}`)
  )
  if (exact) {
    return exact.host
  }

  // Keyword fallback for a hand-typed address. These map to the same three
  // hosts dev-tenants.sh creates.
  if (
    normalized.includes("appliance") ||
    normalized.includes("trial") ||
    normalized.includes("ten_trial")
  ) {
    return "trial.localhost"
  }

  if (
    normalized.includes("clothing") ||
    normalized.includes("fashion") ||
    normalized.includes("mahal") ||
    normalized.includes("free") ||
    normalized.includes("ten_expired")
  ) {
    return "free.localhost"
  }

  if (
    normalized.includes("tech") ||
    normalized.includes("pro") ||
    normalized.includes("ten_pro")
  ) {
    return "pro.localhost"
  }

  // Default to the first available tenant
  return AVAILABLE_TENANTS[0].host
}
