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

export const AVAILABLE_TENANTS: TenantDefinition[] = [
  {
    id: "ten_trial_appliance",
    name: "Vendryx Appliances",
    genre: "appliance",
    tier: "trial",
    tierLabel: "7-Day Free Trial",
    host: "trial-appliance.localhost",
    defaultEmail: "owner@trial-appliance.localhost",
    description: "Industrial luxury appliances, active trial lifecycle",
  },
  {
    id: "ten_expired_clothing",
    name: "Vendryx Fashion & Clothing",
    genre: "clothing",
    tier: "free",
    tierLabel: "Trial Expired (Free)",
    host: "expired-clothing.localhost",
    defaultEmail: "owner@expired-clothing.localhost",
    description: "Couture & bridal collection, free tier limits",
  },
  {
    id: "ten_pro_tech",
    name: "Vendryx Pro Tech",
    genre: "tech",
    tier: "pro",
    tierLabel: "Pro Active",
    host: "pro-tech.localhost",
    defaultEmail: "owner@pro-tech.localhost",
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

  if (
    normalized.includes("appliance") ||
    normalized.includes("trial") ||
    normalized.includes("ten_trial")
  ) {
    return "trial-appliance.localhost"
  }

  if (
    normalized.includes("clothing") ||
    normalized.includes("fashion") ||
    normalized.includes("mahal") ||
    normalized.includes("free") ||
    normalized.includes("ten_expired")
  ) {
    return "expired-clothing.localhost"
  }

  if (
    normalized.includes("tech") ||
    normalized.includes("pro") ||
    normalized.includes("ten_pro")
  ) {
    return "pro-tech.localhost"
  }

  // Default to the first available tenant
  return AVAILABLE_TENANTS[0].host
}
