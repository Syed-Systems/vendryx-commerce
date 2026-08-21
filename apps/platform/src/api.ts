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
  status_reason?: string | null
  status_changed_at?: string | null
  tier: string
  subscription_status: string
  trial_ends_at: string | null
  term_ends_at: string | null
  created_at: string
}

export interface TenantDomain {
  domain: string
  tenant_id: string
  is_primary: boolean
  created_at: string
}

export interface DomainVerification {
  domain: string
  dns_pointed: boolean
  tls_issued: boolean
  status: string
}

export interface MerchantHealth {
  tenant_id: string
  orders_last_7d: number
  gmv_last_7d: { amount: string; currency: string }
  last_activity_at: string | null
  health_verdict: "healthy" | "critical" | "onboarding"
}

export interface OnboardingState {
  tenant_id: string
  stage: string
  created: boolean
  product_added: boolean
  payment_connected: boolean
  first_order_received: boolean
}

export interface TenantExportBundle {
  tenant: Tenant
  subscription: unknown
  domains: TenantDomain[]
  invoices: Invoice[]
  billing_attempts: BillingAttempt[]
  exported_at: string
}

export const listTenants = (params?: { search?: string; tier?: string; status?: string }) => {
  const query = new URLSearchParams()
  if (params?.search) query.set("search", params.search)
  if (params?.tier) query.set("tier", params.tier)
  if (params?.status) query.set("status", params.status)
  const qs = query.toString() ? `?${query.toString()}` : ""
  return request<{ tenants: Tenant[]; count: number }>(`/platform/tenants${qs}`)
}

export const listTenantDomains = (tenantId: string) =>
  request<{ domains: TenantDomain[]; count: number }>(`/platform/tenants/${tenantId}/domains`)

export const addTenantDomain = (tenantId: string, domain: string, is_primary = false) =>
  request<{ domain: TenantDomain }>(`/platform/tenants/${tenantId}/domains`, {
    method: "POST",
    body: JSON.stringify({ domain, is_primary }),
  })

export const removeTenantDomain = (tenantId: string, domain: string) =>
  request<void>(`/platform/tenants/${tenantId}/domains/${domain}`, {
    method: "DELETE",
  })

export const setPrimaryDomain = (tenantId: string, domain: string) =>
  request<{ domain: TenantDomain }>(`/platform/tenants/${tenantId}/domains/${domain}/primary`, {
    method: "POST",
  })

export const getDomainVerification = (tenantId: string, domain: string) =>
  request<{ verification: DomainVerification }>(
    `/platform/tenants/${tenantId}/domains/${domain}/verification`
  )

export const getTenantHealth = (tenantId: string) =>
  request<{ health: MerchantHealth }>(`/platform/tenants/${tenantId}/health`)

export const getTenantOnboarding = (tenantId: string) =>
  request<{ onboarding: OnboardingState }>(`/platform/tenants/${tenantId}/onboarding`)

export const exportTenantData = (tenantId: string) =>
  request<{ export: TenantExportBundle }>(`/platform/tenants/${tenantId}/export`)

export const purgeClosedTenants = (retention_days: number) =>
  request<{ purge: { purged_count: number; purged_tenant_ids: string[] } }>(
    "/platform/tenants/purge",
    {
      method: "POST",
      body: JSON.stringify({ retention_days }),
    }
  )

export const suspendTenant = (tenantId: string, reason: string) =>
  request<{ tenant: Tenant }>(`/platform/tenants/${tenantId}/suspend`, {
    method: "POST",
    body: JSON.stringify({ reason }),
  })

export const reactivateTenant = (tenantId: string, reason: string) =>
  request<{ tenant: Tenant }>(`/platform/tenants/${tenantId}/reactivate`, {
    method: "POST",
    body: JSON.stringify({ reason }),
  })

export const closeTenant = (tenantId: string, confirmation_tenant_id: string, reason: string) =>
  request<{ tenant: Tenant }>(`/platform/tenants/${tenantId}/close`, {
    method: "POST",
    body: JSON.stringify({ confirmation_tenant_id, reason }),
  })

export const setTier = (tenantId: string, tier: string) =>
  request<{ subscription: { tier: string } }>(
    `/platform/tenants/${tenantId}/tier`,
    { method: "POST", body: JSON.stringify({ tier }) }
  )

/** A plan, as the control plane sees it. */
export interface TierView {
  tier: string
  /** "default" while config/tiers.toml still governs; "stored" once it has been edited. */
  source: "default" | "stored"
  features: string[]
  /** Every known key. 0 means unlimited — the one sentinel, matching the engine. */
  quotas: Record<string, number>
  windows: Record<string, number>
}

/**
 * Every gate the engine knows how to apply.
 *
 * Rendered from the server rather than a constant here, so a feature added in Rust shows
 * up as a new checkbox without a second edit in this file that would drift out of step.
 */
export interface Catalog {
  features: string[]
  quotas: string[]
  windows: string[]
  statuses: string[]
}

export interface Subscription {
  tenant_id: string
  tier: string
  status: string
  trial_started_at: string | null
  trial_ends_at: string | null
  term_ends_at: string | null
  quota_overrides: Record<string, number>
  window_overrides: Record<string, number>
  feature_overrides: Record<string, boolean>
}

/** Tier plus overrides, resolved to what the merchant actually gets. */
export interface EffectiveEntitlements {
  tier: string
  features: string[]
  quotas: Record<string, number>
  windows: Record<string, number>
}

export const listTiers = () =>
  request<{ tiers: TierView[]; catalog: Catalog }>("/platform/tiers")

export const saveTier = (
  tier: string,
  body: { features: string[]; quotas: Record<string, number>; windows: Record<string, number> }
) =>
  request<{ tier: TierView }>(`/platform/tiers/${tier}`, {
    method: "PUT",
    body: JSON.stringify(body),
  })

/** Drops the stored override so the plan means what the shipped config says again. */
export const resetTier = (tier: string) =>
  request<{ tier: TierView }>(`/platform/tiers/${tier}`, { method: "DELETE" })

export const getSubscription = (tenantId: string) =>
  request<{
    subscription: Subscription
    effective: EffectiveEntitlements | null
    error: string | null
  }>(`/platform/tenants/${tenantId}/subscription`)

/**
 * Only the keys present are changed. Dates are tri-state: absent leaves the date alone,
 * an explicit null clears it.
 */
export const patchSubscription = (
  tenantId: string,
  patch: Partial<{
    tier: string
    status: string
    trial_started_at: string | null
    trial_ends_at: string | null
    term_ends_at: string | null
    quota_overrides: Record<string, number>
    window_overrides: Record<string, number>
    feature_overrides: Record<string, boolean>
  }>
) =>
  request<{ subscription: Subscription; effective: EffectiveEntitlements | null }>(
    `/platform/tenants/${tenantId}/subscription`,
    { method: "PATCH", body: JSON.stringify(patch) }
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

/* -- Operators and the audit trail (P29) ------------------------------------------- */

export interface Operator {
  id: string
  name: string
  redacted_token: string
  created_at: string
  last_used_at: string | null
  revoked_at: string | null
  revoked: boolean
}

export interface OperatorListing {
  operators: Operator[]
  count: number
  caller: { id: string; name: string; bootstrap: boolean }
  bootstrap_enabled: boolean
}

export const listOperators = () => request<OperatorListing>("/platform/operators")

export const createOperator = (name: string) =>
  request<{ operator: Operator; token: string; notice: string }>("/platform/operators", {
    method: "POST",
    body: JSON.stringify({ name }),
  })

export const revokeOperator = (id: string) =>
  request<{ operator: Operator }>(`/platform/operators/${id}`, { method: "DELETE" })

export interface AuditEntry {
  id: string
  actor_id: string
  actor_name: string
  action: string
  entity_type: string
  entity_id: string
  diff: unknown
  created_at: string
}

export const listAudit = (params: { entityId?: string; before?: string; limit?: number } = {}) => {
  const query = new URLSearchParams()
  if (params.entityId) query.set("entity_id", params.entityId)
  if (params.before) query.set("before", params.before)
  if (params.limit) query.set("limit", String(params.limit))
  const suffix = query.toString()
  return request<{ entries: AuditEntry[]; count: number; next_cursor: string | null }>(
    `/platform/audit${suffix ? `?${suffix}` : ""}`
  )
}

/* -- Billing (P30) ---------------------------------------------------------------- */

export interface MoneyAmount {
  amount: string | number
  currency: string
}

export interface Invoice {
  id: string
  tenant_id: string
  tier: string
  period_start: string
  period_end: string
  subtotal: MoneyAmount
  tax_total: MoneyAmount
  credit_total: MoneyAmount
  total: MoneyAmount
  status: string
  issued_at: string
  paid_at: string | null
}

export interface BillingAttempt {
  id: string
  invoice_id: string
  tenant_id: string
  provider_id: string
  idempotency_key: string
  request_hash: string
  state: string
  amount: MoneyAmount
  provider_reference: string | null
  error: string | null
  created_at: string
  updated_at: string
}

export const listInvoices = () =>
  request<{ invoices: Invoice[]; count: number }>("/platform/invoices")

export const listBillingAttempts = () =>
  request<{ attempts: BillingAttempt[]; count: number }>("/platform/billing-attempts")

export const retryInvoice = (invoice: Invoice, providerId: string) =>
  request<{ attempt: BillingAttempt }>(`/platform/invoices/${invoice.id}/retry`, {
    method: "POST",
    body: JSON.stringify({
      provider_id: providerId,
      idempotency_key: `manual:${invoice.id}:${Date.now()}`,
      request_hash: `manual:${invoice.id}:${invoice.total.amount}`,
    }),
  })

export const creditInvoice = (
  invoiceId: string,
  body: {
    credit_minor: number
    discount_minor: number
    coupon_minor: number
    currency: string
    reason: string
  }
) =>
  request<{ invoice: Invoice }>(`/platform/invoices/${invoiceId}/credit`, {
    method: "POST",
    body: JSON.stringify(body),
  })

/* -- Payment Providers (Gateways) ------------------------------------------------ */

export interface CredentialField {
  key: string
  label: string
  kind: "text" | "secret" | "pem" | "url" | "bool"
  required: boolean
}

export type CredentialHealth = "unconfigured" | "valid" | "expiring" | "failing_auth" | "unknown"

export interface PlatformPaymentProvider {
  id: string
  name: string
  configured: boolean
  credential_health: CredentialHealth
  credential_schema: CredentialField[]
  credential: {
    provider_id: string
    credential_version: number
    health: CredentialHealth
    fields: Record<string, string | boolean | number>
    updated_at: string
  } | null
}

export const listPlatformPaymentProviders = () =>
  request<{ payment_providers: PlatformPaymentProvider[]; count: number }>("/platform/payment-providers")

export const upsertPlatformPaymentProviderCredentials = (
  providerId: string,
  credentials: Record<string, unknown>
) =>
  request<{ payment_provider: PlatformPaymentProvider }>(
    `/platform/payment-providers/${providerId}/credentials`,
    {
      method: "POST",
      body: JSON.stringify({ credentials }),
    }
  )

/* -- P32: Oversight & Operations ------------------------------------------------ */

export interface ProviderOversight {
  id: string
  name: string
  kind: string
  supported_regions: string[]
  is_enabled_globally: bool
  active_merchants_count: number
}

export interface ProviderHealthReport {
  provider_id: string
  health_verdict: string
  total_calls_24h: number
  failed_auth_count_24h: number
  last_successful_call: string | null
  last_auth_failure: string | null
}

export interface OutboundLedgerItem {
  id: string
  tenant_id: string
  provider_id: string
  action: string
  state: string
  age_seconds: number
  amount: { amount: string; currency: string } | null
  created_at: string
}

export interface SweepStatus {
  last_sweep_at: string
  rows_converged_last_run: number
  rows_stranded: number
  is_scheduled: boolean
}

export interface DeadLetterWebhook {
  id: string
  tenant_id: string
  provider_id: string
  event_type: string
  failure_reason: string
  attempts: number
  last_attempt_at: string
}

export interface SettlementReconciliation {
  tenant_id: string
  provider_id: string
  statement_period: string
  gateway_statement_total: { amount: string; currency: string }
  platform_ledger_total: { amount: string; currency: string }
  variance_amount: { amount: string; currency: string }
  status: string
}

export interface CodSettlementOversight {
  tenant_id: string
  courier_id: string
  collected_by_courier: { amount: string; currency: string }
  remitted_to_merchant: { amount: string; currency: string }
  outstanding_courier_hold: { amount: string; currency: string }
  value_at_risk_status: string
}

export interface CourierOversight {
  id: string
  name: string
  tracking_supported: boolean
  active_consignments: number
  credential_health: string
}

export interface StrandedConsignment {
  id: string
  tenant_id: string
  courier_id: string
  merchant_reference: string
  status: string
  age_minutes: number
}

export interface UnknownStatusAlarm {
  id: string
  domain: string
  provider_id: string
  raw_status: string
  first_seen_at: string
  count: number
}

export const listProvidersOversight = () =>
  request<{ providers: ProviderOversight[]; count: number }>("/platform/providers")

export const getProviderHealth = (id: string) =>
  request<{ health: ProviderHealthReport }>(`/platform/providers/${id}/health`)

export const listOutboundLedger = () =>
  request<{ ledger_requests: OutboundLedgerItem[]; count: number }>("/platform/ledger/outbound")

export const getSweepStatus = () =>
  request<{ sweep_status: SweepStatus }>("/platform/ledger/sweep-status")

export const triggerReconciliationSweep = () =>
  request<{ sweep_result: Record<string, unknown> }>("/platform/ledger/sweep", {
    method: "POST",
  })

export const listWebhookOversight = () =>
  request<{
    delivery_health: string
    dead_letters: DeadLetterWebhook[]
    dead_letters_count: number
  }>("/platform/webhooks/health")

export const replayDeadLetterWebhook = (webhook_id: string) =>
  request<{ status: string; webhook_id: string }>("/platform/webhooks/replay", {
    method: "POST",
    body: JSON.stringify({ webhook_id }),
  })

export const toggleGatewayKillSwitch = (id: string, killed: boolean, reason: string) =>
  request<{ provider_id: string; is_killed: boolean }>(`/platform/gateways/${id}/kill-switch`, {
    method: "POST",
    body: JSON.stringify({ killed, reason }),
  })

export const listSettlementReconciliations = () =>
  request<{ settlements: SettlementReconciliation[]; count: number }>("/platform/settlements")

export const listCodSettlements = () =>
  request<{ cod_settlements: CodSettlementOversight[]; count: number }>("/platform/settlements/cod")

export const listCouriersOversight = () =>
  request<{ couriers: CourierOversight[]; count: number }>("/platform/couriers")

export const listStrandedConsignments = () =>
  request<{ stranded_consignments: StrandedConsignment[]; count: number }>(
    "/platform/consignments/stranded"
  )

export const listUnknownStatusAlarms = () =>
  request<{ alarms: UnknownStatusAlarm[]; count: number }>("/platform/alarms/unknown-status")

/* -- P33: Growth Analytics ------------------------------------------------------- */

export interface GmvMetrics {
  daily_gmv: { amount: string; currency: string }
  weekly_gmv: { amount: string; currency: string }
  monthly_gmv: { amount: string; currency: string }
  total_gmv: { amount: string; currency: string }
}

export interface FunnelMetrics {
  total_signups: number
  activated_merchants: number
  paid_subscribers: number
  activation_rate: number
  paid_conversion_rate: number
}

export interface PlanDistributionItem {
  tier: string
  count: number
  percentage: number
  mrr: { amount: string; currency: string }
}

export interface TopMerchantGmv {
  tenant_id: string
  name: string
  tier: string
  gmv_30d: { amount: string; currency: string }
}

export interface ChurnCohort {
  signup_month: string
  initial_count: number
  active_count: number
  churned_count: number
  retention_rate: number
}

export interface QuotaHeadroom {
  tenant_id: string
  name: string
  tier: string
  metric: string
  usage: number
  limit: number
  usage_percentage: number
  is_near_limit: boolean
}

export const getGmvMetrics = () =>
  request<{ gmv_metrics: GmvMetrics }>("/platform/analytics/gmv")

export const getFunnelMetrics = () =>
  request<{ funnel: FunnelMetrics }>("/platform/analytics/funnel")

export const getPlanDistribution = () =>
  request<{ distribution: PlanDistributionItem[] }>("/platform/analytics/plan-distribution")

export const getTopMerchants = (limit = 10) =>
  request<{ top_merchants: TopMerchantGmv[] }>(`/platform/analytics/top-merchants?limit=${limit}`)

export const getChurnCohorts = () =>
  request<{ cohorts: ChurnCohort[] }>("/platform/analytics/churn-cohorts")

export const getQuotaHeadroom = () =>
  request<{ headroom: QuotaHeadroom[] }>("/platform/analytics/quota-headroom")

/* -- P34: Support Operations ---------------------------------------------------- */

export interface ImpersonationResponse {
  token: string
  tenant_id: string
  expires_at: string
  scopes: string[]
}

export interface TenantError {
  id: string
  tenant_id: string
  message: string
  endpoint: string
  status_code: number
  timestamp: string
}

export interface WorkflowExecution {
  id: string
  tenant_id: string
  workflow_name: string
  status: string
  current_step: string
  created_at: string
}

export const startImpersonation = (
  tenantId: string,
  body: { duration_minutes?: number; reason: string }
) =>
  request<{ impersonation: ImpersonationResponse }>(`/platform/tenants/${tenantId}/impersonate`, {
    method: "POST",
    body: JSON.stringify(body),
  })

export const endImpersonation = (tenantId: string, token: string) =>
  request<{ status: string; tenant_id: string }>(`/platform/tenants/${tenantId}/impersonate/end`, {
    method: "POST",
    body: JSON.stringify({ token }),
  })

export const getTenantErrors = (tenantId: string) =>
  request<{ tenant_id: string; errors: TenantError[]; count: number }>(
    `/platform/tenants/${tenantId}/errors`
  )

export const triggerManualJob = (job_name: string, tenant_id?: string) =>
  request<{ job_name: string; status: string; triggered_at: string }>(`/platform/jobs/trigger`, {
    method: "POST",
    body: JSON.stringify({ job_name, tenant_id }),
  })

export const listWorkflows = () =>
  request<{ workflows: WorkflowExecution[]; count: number }>("/platform/workflows")

export const toggleTenantReadOnly = (tenantId: string, read_only: boolean, reason: string) =>
  request<{ tenant_id: string; is_read_only: boolean }>(`/platform/tenants/${tenantId}/read-only`, {
    method: "POST",
    body: JSON.stringify({ read_only, reason }),
  })

/* -- P35: Platform Configuration & Themes --------------------------------------- */

export interface ThemeCatalogItem {
  id: string
  name: string
  description: string
  latest_version: string
  available_versions: string[]
  requires_tier_feature?: string
}

export interface TenantThemeAssignment {
  tenant_id: string
  theme_id: string
  version: string
  is_custom: boolean
  assigned_at: string
}

export interface GlobalFeatureFlag {
  key: string
  description: string
  is_enabled: boolean
  rollout_percentage: number
}

export interface PlatformDefaults {
  default_region: string
  default_currency: string
  default_tax_rate: number
  support_email: string
}

export interface MessagingConfig {
  email_provider: string
  sms_provider: string
  from_email: string
  order_confirmation_template: string
}

export interface Announcement {
  id: string
  title: string
  message: string
  severity: "info" | "warning" | "critical"
  expires_at: string
  created_at: string
}

export const getThemeCatalog = () =>
  request<{ themes: ThemeCatalogItem[]; count: number }>("/platform/themes")

export const assignTenantTheme = (tenantId: string, theme_id: string, version?: string) =>
  request<{ assignment: TenantThemeAssignment }>(`/platform/tenants/${tenantId}/theme`, {
    method: "POST",
    body: JSON.stringify({ theme_id, version }),
  })

export const rolloutThemeVersion = (themeId: string, target_version: string, percentage: number) =>
  request<{ theme_id: string; target_version: string; percentage: number }>(
    `/platform/themes/${themeId}/rollout`,
    {
      method: "POST",
      body: JSON.stringify({ target_version, percentage }),
    }
  )

export const getFeatureFlags = () =>
  request<{ feature_flags: GlobalFeatureFlag[]; count: number }>("/platform/feature-flags")

export const updateFeatureFlag = (
  key: string,
  is_enabled: boolean,
  rollout_percentage?: number
) =>
  request<{ feature_flag: GlobalFeatureFlag }>(`/platform/feature-flags/${key}`, {
    method: "PUT",
    body: JSON.stringify({ is_enabled, rollout_percentage }),
  })

export const getPlatformDefaults = () =>
  request<{ defaults: PlatformDefaults }>("/platform/config/defaults")

export const updatePlatformDefaults = (defaults: PlatformDefaults) =>
  request<{ defaults: PlatformDefaults }>("/platform/config/defaults", {
    method: "PUT",
    body: JSON.stringify(defaults),
  })

export const getMessagingConfig = () =>
  request<{ messaging: MessagingConfig }>("/platform/config/messaging")

export const updateMessagingConfig = (messaging: MessagingConfig) =>
  request<{ messaging: MessagingConfig }>("/platform/config/messaging", {
    method: "PUT",
    body: JSON.stringify(messaging),
  })

export const getAnnouncements = () =>
  request<{ announcements: Announcement[]; count: number }>("/platform/announcements")

export const createAnnouncement = (
  title: string,
  message: string,
  severity?: string,
  expires_in_days?: number
) =>
  request<{ announcement: Announcement }>("/platform/announcements", {
    method: "POST",
    body: JSON.stringify({ title, message, severity, expires_in_days }),
  })



