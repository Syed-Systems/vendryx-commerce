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

/** True only for a host this build knows how to talk to. */
function isKnownTenantHost(host: string | null | undefined): host is string {
  return !!host && AVAILABLE_TENANTS.some((t) => t.host === host)
}

export function getActiveTenantHost(): string {
  if (typeof window === "undefined") {
    return AVAILABLE_TENANTS[0].host
  }

  // Every stored value is validated against AVAILABLE_TENANTS before use.
  //
  // The cookie carries a one-year expiry, so a browser that picked a dashboard
  // before the hosts were corrected still holds "trial-appliance.localhost" --
  // a tenant the engine has never had. Returning it verbatim sent
  // x-stheno-tenant-host: trial-appliance.localhost on every request and the
  // engine answered 404, which is the same "Register an auth provider" screen
  // arriving from a stale cookie rather than a missing provider. A fresh browser
  // never reproduced it, so the value has to be rejected here rather than
  // assumed trustworthy.

  // 1. Check document.cookie
  const match = document.cookie.match(/stheno_tenant_host=([^;]+)/)
  const fromCookie = match?.[1] ? decodeURIComponent(match[1].trim()) : null
  if (isKnownTenantHost(fromCookie)) {
    return fromCookie
  }

  // 2. Check localStorage
  const fromStorage = localStorage.getItem(STORAGE_KEY)
  if (isKnownTenantHost(fromStorage)) {
    return fromStorage
  }

  // 3. Anything stale is cleared, not merely ignored, so the next load does not
  //    re-read it and the stored state cannot drift from the chosen dashboard.
  if (fromCookie || fromStorage) {
    clearActiveTenantHost()
  }

  return AVAILABLE_TENANTS[0].host
}

/** Drops a stored tenant host. Used to evict values this build cannot resolve. */
function clearActiveTenantHost(): void {
  if (typeof window === "undefined") return
  document.cookie = `${STORAGE_KEY}=; path=/; max-age=0; SameSite=Lax`
  localStorage.removeItem(STORAGE_KEY)
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
