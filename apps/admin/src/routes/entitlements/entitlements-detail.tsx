import { ArrowDownTray, ArrowPath, CheckCircle, CreditCard, Sparkles, XCircle } from "@medusajs/icons"
import { Badge, Button, Container, Heading, Table, Text, toast } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useEffect, useRef, useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"
import { sdk } from "../../lib/client"

const spanAllColumns = { colSpan: 4 } as React.TdHTMLAttributes<HTMLTableCellElement>
const spanSixColumns = { colSpan: 6 } as React.TdHTMLAttributes<HTMLTableCellElement>

interface QuotaItem {
  key: string
  limit?: number
  used: number
  state: string
}

interface WindowItem {
  key: string
  limit?: number
}

interface EntitlementsData {
  tier: string
  trial_ends_at: string | null
  term_ends_at: string | null
  quotas: QuotaItem[]
  windows: WindowItem[]
}

interface NotificationChannelUsage {
  sent: number
  reserved: number
  limit: number | null
  remaining: number | null
}

interface NotificationUsageData {
  usage_date_utc: string
  email: NotificationChannelUsage
  whatsapp: NotificationChannelUsage
}

interface FeatureFlagsData {
  feature_flags: Record<string, boolean>
}

interface InvoiceItem {
  id: string
  date: string
  period: string
  amount: string
  currency: string
  status: "paid" | "pending"
  pdf_url?: string | null
  hosted_invoice_url?: string | null
}

interface SubscriptionData {
  tier: string
  status: string
  trial_ends_at?: string | null
  term_ends_at: string | null
  grace_ends_at?: string | null
  plan_amount: string | null
  currency: string | null
  billing_interval?: string | null
  pro_plan?: {
    amount: string
    currency: string
    interval: string
  } | null
}

const humanise = (key: string) => key.replace(/_/g, " ")
const formatInstant = (iso: string | null) => {
  if (!iso) return null
  const timestamp = Date.parse(iso)
  return Number.isFinite(timestamp)
    ? new Date(timestamp).toLocaleString(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      })
    : null
}

export const EntitlementsDetail = () => {
  const [isPlanModalOpen, setIsPlanModalOpen] = useState(false)
  const [isPaymentMethodModalOpen, setIsPaymentMethodModalOpen] = useState(false)
  const [now, setNow] = useState(() => Date.now())
  const planModalRef = useRef<HTMLDivElement>(null)
  const paymentModalRef = useRef<HTMLDivElement>(null)

  const {
    data: entitlements,
    isLoading: isEntitlementsLoading,
    isError: isEntitlementsError,
    refetch: refetchEntitlements,
  } = useQuery<EntitlementsData>({
    queryKey: ["admin", "entitlements"],
    queryFn: () => sdk.client.fetch<EntitlementsData>("/admin/entitlements"),
  })

  const {
    data: flags,
    isError: isFlagsError,
    refetch: refetchFlags,
  } = useQuery<FeatureFlagsData>({
    queryKey: ["admin", "feature-flags"],
    queryFn: () => sdk.client.fetch<FeatureFlagsData>("/admin/feature-flags"),
  })

  const {
    data: notificationUsage,
    isLoading: isNotificationUsageLoading,
    isError: isNotificationUsageError,
  } = useQuery<{ usage: NotificationUsageData }>({
    queryKey: ["admin", "notifications", "usage"],
    queryFn: () => sdk.client.fetch<{ usage: NotificationUsageData }>("/admin/notifications/usage"),
    refetchInterval: 60_000,
  })

  const { data: billingSubscription } = useQuery<{ subscription: SubscriptionData }>({
    queryKey: ["admin", "billing", "subscription"],
    queryFn: async () => {
      try {
        return await sdk.client.fetch<{ subscription: SubscriptionData }>("/admin/billing/subscription")
      } catch {
        return {
          subscription: {
            tier: "free",
            status: "unknown",
            trial_ends_at: null,
            term_ends_at: null,
            grace_ends_at: null,
            plan_amount: null,
            currency: "USD",
            billing_interval: null,
            pro_plan: null,
          },
        }
      }
    },
  })

  const { data: billingInvoices } = useQuery<{ invoices: InvoiceItem[] }>({
    queryKey: ["admin", "billing", "invoices"],
    queryFn: async () => {
      try {
        return await sdk.client.fetch<{ invoices: InvoiceItem[] }>("/admin/billing/invoices")
      } catch {
        return { invoices: [] }
      }
    },
  })

  const tier = billingSubscription?.subscription?.tier || entitlements?.tier || "free"
  const trialEnds = formatInstant(
    billingSubscription?.subscription?.trial_ends_at ?? entitlements?.trial_ends_at ?? null
  )
  const trialEndsAt = billingSubscription?.subscription?.trial_ends_at ?? entitlements?.trial_ends_at ?? null
  const termEnds = formatInstant(
    billingSubscription?.subscription?.term_ends_at ?? entitlements?.term_ends_at ?? null
  )
  const inGrace = billingSubscription?.subscription?.status === "grace"
  const overQuota = entitlements?.quotas?.some((q) => q.state === "grace") ?? false
  const graceEndsAt = billingSubscription?.subscription?.grace_ends_at ?? null
  useEffect(() => {
    if (!inGrace && tier !== "trial") return
    const timer = window.setInterval(() => setNow(Date.now()), 60_000)
    return () => window.clearInterval(timer)
  }, [inGrace, graceEndsAt, tier, trialEndsAt])
  useEffect(() => {
    const dialog = isPlanModalOpen
      ? planModalRef.current
      : isPaymentMethodModalOpen
        ? paymentModalRef.current
        : null
    if (!dialog) return

    const previouslyFocused = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null
    dialog.focus()

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (isPlanModalOpen) setIsPlanModalOpen(false)
        else setIsPaymentMethodModalOpen(false)
        return
      }
      if (event.key !== "Tab") return

      const focusable = Array.from(
        dialog.querySelectorAll<HTMLElement>(
          'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
        )
      ).filter((element) => element.getAttribute("aria-hidden") !== "true")
      if (focusable.length === 0) {
        event.preventDefault()
        dialog.focus()
        return
      }

      const first = focusable[0]
      const last = focusable[focusable.length - 1]
      const activeElement = document.activeElement
      if (
        event.shiftKey &&
        (!dialog.contains(activeElement) || activeElement === first || activeElement === dialog)
      ) {
        event.preventDefault()
        last.focus()
      } else if (!event.shiftKey && (!dialog.contains(activeElement) || activeElement === last)) {
        event.preventDefault()
        first.focus()
      }
    }

    document.addEventListener("keydown", onKeyDown)
    return () => {
      document.removeEventListener("keydown", onKeyDown)
      if (previouslyFocused?.isConnected) previouslyFocused.focus()
    }
  }, [isPaymentMethodModalOpen, isPlanModalOpen])
  const graceEndEpoch = graceEndsAt ? Date.parse(graceEndsAt) : Number.NaN
  const graceEnds = formatInstant(graceEndsAt)
  const graceDaysRemaining = Number.isFinite(graceEndEpoch)
    ? Math.max(0, Math.ceil((graceEndEpoch - now) / 86_400_000))
    : null
  const trialEndEpoch = trialEndsAt ? Date.parse(trialEndsAt) : Number.NaN
  const trialDaysRemaining = Number.isFinite(trialEndEpoch)
    ? Math.max(0, Math.ceil((trialEndEpoch - now) / 86_400_000))
    : null
  const subscriptionStatus = billingSubscription?.subscription?.status
  const statusLabel =
    subscriptionStatus && subscriptionStatus !== "unknown"
      ? subscriptionStatus.replace(/_/g, " ")
      : "Status unavailable"
  const statusColor = inGrace ? "orange" : tier === "pro" ? "green" : "blue"
  const gateable = Object.entries(flags?.feature_flags ?? {})
    .filter(([, enabled]) => typeof enabled === "boolean")
    .sort(([a], [b]) => a.localeCompare(b))

  const invoices = billingInvoices?.invoices || []
  const proPlan = billingSubscription?.subscription?.pro_plan
  const proPlanPrice = proPlan
    ? `${proPlan.currency.toUpperCase()} ${proPlan.amount} / ${proPlan.interval}`
    : "Pricing unavailable"
  const tierPrice =
    tier === "trial"
      ? "Trial access"
      : tier === "free"
      ? "Free plan"
      : billingSubscription?.subscription?.plan_amount &&
          billingSubscription.subscription.currency &&
          billingSubscription.subscription.billing_interval
        ? `${billingSubscription.subscription.currency.toUpperCase()} ${billingSubscription.subscription.plan_amount} / ${billingSubscription.subscription.billing_interval}`
        : "Pricing unavailable"

  const handleDownloadInvoice = (invoiceId: string) => {
    const inv = invoices.find((i: any) => i.id === invoiceId)
    if (inv?.pdf_url) {
      window.open(inv.pdf_url, "_blank")
    } else if (inv?.hosted_invoice_url) {
      window.open(inv.hosted_invoice_url, "_blank")
    } else {
      toast.info(`Invoice receipt for ${invoiceId} is available in your billing portal.`)
    }
  }

  const handleOpenBillingPortal = async (showPlanFallback = true) => {
    try {
      const res = await sdk.client.fetch<{ portal_url?: string; url?: string }>("/admin/billing/portal-url", {
        method: "GET",
      })
      const targetUrl = res?.portal_url || res?.url
      if (targetUrl && typeof window !== "undefined") {
        window.open(targetUrl, "_blank")
        return
      }
    } catch {
      // A missing portal falls back to in-page plan details for plan changes.
    }
    if (showPlanFallback) {
      setIsPlanModalOpen(true)
    } else {
      toast.info("Billing portal is unavailable. Contact platform support to manage payment methods.")
    }
  }

  return (
    <SingleColumnPage widgets={{ after: [], before: [] }}>
      <div className="flex flex-col gap-y-6">
        <div className="flex flex-col gap-y-4 md:flex-row md:items-center md:justify-between">
          <div>
            <Text size="xsmall" weight="plus" className="text-ui-fg-muted">
              Platform Settings
            </Text>
            <div className="mt-1 flex items-center gap-x-2.5">
              <Heading level="h1" className="text-ui-fg-base">
                Subscription &amp; Billing
              </Heading>
              <Badge size="small" color={tier === "pro" ? "green" : "blue"} className="uppercase">
                {tier}
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle mt-1">
              Review this store&apos;s effective subscription tier, enforced quotas, billing history, and available services.
            </Text>
          </div>
          <div className="flex items-center gap-x-2">
            <Button
              variant="secondary"
              size="small"
              onClick={() => {
                refetchEntitlements()
                refetchFlags()
              }}
              aria-label="Refresh plan and access details"
            >
              <ArrowPath className="text-ui-fg-subtle me-1.5" />
              Refresh
            </Button>
            <Button
              variant="primary"
              size="small"
              onClick={() => void handleOpenBillingPortal()}
            >
              <Sparkles className="me-1.5" />
              Change plan
            </Button>
          </div>
        </div>

        {(isEntitlementsError || isFlagsError) && (
          <Container role="alert" className="border-ui-tag-red-border bg-ui-tag-red-bg p-4">
            <Text size="small" weight="plus" className="text-ui-tag-red-text">
              Some plan details could not be loaded.
            </Text>
            <Text size="small" className="text-ui-tag-red-text mt-1">
              Refresh to try again. No store settings have been changed.
            </Text>
          </Container>
        )}

        {inGrace && (
          <Container
            role="status"
            aria-live="polite"
            className="border-ui-tag-orange-border bg-ui-tag-orange-bg p-4"
          >
            <Text size="small" weight="plus" className="text-ui-tag-orange-text">
              Your Pro subscription is in its seven-day grace period.
            </Text>
            <Text size="small" className="text-ui-tag-orange-text mt-1">
              {graceEnds ? (
                <>
                  {graceDaysRemaining !== null && (
                    <>
                      {graceDaysRemaining} day{graceDaysRemaining === 1 ? "" : "s"} remain. {" "}
                    </>
                  )}
                  Pro features remain available through {graceEnds}. Renew before then to keep
                  them enabled.
                </>
              ) : (
                "Renew now to keep Pro features enabled. The grace end date is unavailable."
              )}
            </Text>
            <Button
              variant="secondary"
              size="small"
              className="mt-3"
              onClick={() => void handleOpenBillingPortal()}
            >
              Renew Pro
            </Button>
          </Container>
        )}

        {tier === "trial" && (
          <Container
            role="status"
            aria-live="polite"
            className="border-ui-tag-orange-border bg-ui-tag-orange-bg p-4"
          >
            <Text size="small" weight="plus" className="text-ui-tag-orange-text">
              Your Pro trial is active.
            </Text>
            <Text size="small" className="text-ui-tag-orange-text mt-1">
              {trialEnds && trialDaysRemaining !== null
                ? trialDaysRemaining === 0
                  ? `Your trial ended at ${trialEnds}. Refresh plan details to see your current access.`
                  : `Your trial ends in ${trialDaysRemaining} day${trialDaysRemaining === 1 ? "" : "s"} (${trialEnds}). Review upgrade options to keep Pro features.`
                : "Your trial end date is unavailable. Contact platform support to confirm your plan."}
            </Text>
            <Button
              variant="secondary"
              size="small"
              className="mt-3"
              onClick={() => void handleOpenBillingPortal()}
            >
              View upgrade options
            </Button>
          </Container>
        )}

        {overQuota && (
          <Container className="border-ui-tag-orange-border bg-ui-tag-orange-bg p-4">
            <Text size="small" weight="plus" className="text-ui-tag-orange-text">
              This store is over one of its plan limits.
            </Text>
            <Text size="small" className="text-ui-tag-orange-text mt-1">
              Existing work stays live and editable. New items in the affected resource are
              blocked until the limit is resolved.
            </Text>
          </Container>
        )}

        {/* Current Plan Overview Card */}
        <Container className="p-6">
          <div className="flex flex-col gap-y-2 sm:flex-row sm:items-center sm:justify-between border-b border-ui-border-base pb-4">
            <div>
              <Heading level="h2" className="text-ui-fg-base text-base">
                Current Plan Overview
              </Heading>
              <Text size="small" className="text-ui-fg-subtle mt-0.5">
                Managed by Syed Systems Cloud platform billing.
              </Text>
            </div>
            <Badge color={statusColor} size="small" className="w-fit capitalize">
              {statusLabel}
            </Badge>
          </div>

          <div className="grid grid-cols-1 gap-6 pt-5 md:grid-cols-4">
            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Plan Name
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1 capitalize">
                Vendryx {tier}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                Full commerce engine features
              </Text>
            </div>

            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Pricing Cadence
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {tierPrice}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                Plan terms follow the platform billing configuration.
              </Text>
            </div>

            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Billing Cycle
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {termEnds ? `Term ends ${termEnds}` : "No paid term end is recorded"}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                Billing dates reflect the current subscription record.
              </Text>
            </div>

            <div>
              <Text size="xsmall" weight="plus" className="text-ui-fg-muted uppercase">
                Trial Status
              </Text>
              <Text size="small" weight="plus" className="text-ui-fg-base mt-1">
                {tier === "trial" && trialEnds ? `Ends ${trialEnds}` : tier === "trial" ? "End date unavailable" : "Not on trial"}
              </Text>
              <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                Access follows this store&apos;s current entitlement configuration.
              </Text>
            </div>
          </div>
        </Container>

        {/* Quotas & Resource Usage */}
        <Container className="overflow-hidden p-0">
          <div className="border-ui-border-base border-b p-6">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Limits &amp; Usage Quotas
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Limits apply to new catalog items and seats. Existing products and staff stay available after a plan change.
            </Text>
          </div>
          <Table>
            <Table.Header>
              <Table.Row>
                <Table.HeaderCell>Resource</Table.HeaderCell>
                <Table.HeaderCell>Used</Table.HeaderCell>
                <Table.HeaderCell>Limit</Table.HeaderCell>
                <Table.HeaderCell>State</Table.HeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {isEntitlementsLoading ? (
                <Table.Row>
                  <Table.Cell {...spanAllColumns} className="text-ui-fg-muted py-6 text-center">
                    Loading limits...
                  </Table.Cell>
                </Table.Row>
              ) : (
                <>
                  {(entitlements?.quotas ?? []).map((quota) => (
                    <Table.Row key={quota.key}>
                      <Table.Cell className="text-ui-fg-base font-medium capitalize">
                        {humanise(quota.key)}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-base font-mono font-semibold">
                        {quota.used}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-subtle font-mono">
                        {quota.limit !== undefined && quota.limit > 0 ? quota.limit : "Unlimited"}
                      </Table.Cell>
                      <Table.Cell>
                        <Badge size="xsmall" color={quota.state === "grace" ? "orange" : "green"}>
                          {quota.state === "grace" ? "Over limit" : "Within limit"}
                        </Badge>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                  {(entitlements?.windows ?? []).map((window) => (
                    <Table.Row key={window.key}>
                      <Table.Cell className="text-ui-fg-base font-medium capitalize">
                        {humanise(window.key)}
                      </Table.Cell>
                      <Table.Cell className="text-ui-fg-muted">Real-time metrics</Table.Cell>
                      <Table.Cell className="text-ui-fg-subtle font-mono">
                        {window.limit !== undefined && window.limit > 0 ? `${window.limit} days` : "Unlimited history"}
                      </Table.Cell>
                      <Table.Cell>
                        <Badge size="xsmall" color="green">
                          Included
                        </Badge>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </>
              )}
            </Table.Body>
          </Table>
        </Container>

        <Container className="p-6">
          <div className="mb-4">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Daily notification usage
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Email and WhatsApp usage is counted by UTC day. In-flight messages are included in the remaining allowance.
            </Text>
          </div>
          {isNotificationUsageLoading ? (
            <Text size="small" className="text-ui-fg-muted">Loading notification usage…</Text>
          ) : isNotificationUsageError || !notificationUsage?.usage ? (
            <Text size="small" className="text-ui-fg-subtle">
              Notification usage is temporarily unavailable.
            </Text>
          ) : (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              {(["email", "whatsapp"] as const).map((channel) => {
                const usage = notificationUsage.usage[channel]
                const label = channel === "email" ? "Email" : "WhatsApp"
                return (
                  <div key={channel} className="rounded-lg border border-ui-border-base p-4">
                    <Text size="small" weight="plus" className="text-ui-fg-base">{label}</Text>
                    <Text size="small" className="text-ui-fg-subtle mt-1">
                      {usage.sent} sent · {usage.reserved} in progress
                    </Text>
                    <Text size="small" className="text-ui-fg-base mt-2 font-medium">
                      {usage.limit === null
                        ? "No daily cap"
                        : `${usage.remaining ?? 0} remaining of ${usage.limit}`}
                    </Text>
                  </div>
                )
              })}
            </div>
          )}
          {notificationUsage?.usage && (
            <Text size="xsmall" className="text-ui-fg-muted mt-3">
              Usage date: {notificationUsage.usage.usage_date_utc} (UTC)
            </Text>
          )}
        </Container>

        {/* Payment Method Card */}
        <Container className="p-6">
          <div className="flex flex-col gap-y-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-start gap-x-3.5">
              <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-ui-border-base bg-ui-bg-subtle text-ui-fg-muted">
                <CreditCard />
              </div>
              <div>
                <div className="flex items-center gap-x-2">
                  <Text size="small" weight="plus" className="text-ui-fg-base">
                    Default Payment Method
                  </Text>
                  <Badge color="blue" size="2xsmall">
                    Not configured
                  </Badge>
                </div>
                <Text size="xsmall" className="text-ui-fg-subtle mt-0.5">
                  No saved subscription payment method is available.
                </Text>
              </div>
            </div>

            <Button
              size="small"
              variant="secondary"
              onClick={() => setIsPaymentMethodModalOpen(true)}
            >
              Update payment method
            </Button>
          </div>
        </Container>

        {/* Invoice & Billing History Table (Reference 8.png) */}
        <Container className="overflow-hidden p-0">
          <div className="border-ui-border-base border-b p-6">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Invoices &amp; Payment History
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Review billing records and receipts available for this store.
            </Text>
          </div>
          <Table>
            <Table.Header>
              <Table.Row>
                <Table.HeaderCell>Invoice Number</Table.HeaderCell>
                <Table.HeaderCell>Date</Table.HeaderCell>
                <Table.HeaderCell>Billing Period</Table.HeaderCell>
                <Table.HeaderCell>Amount</Table.HeaderCell>
                <Table.HeaderCell>Status</Table.HeaderCell>
                <Table.HeaderCell className="text-right">Receipt</Table.HeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {invoices.length === 0 ? (
                <Table.Row>
                  <Table.Cell {...spanSixColumns} className="text-ui-fg-muted py-8 text-center text-xs">
                    No billing invoices issued yet. Official receipts appear here after each billing settlement.
                  </Table.Cell>
                </Table.Row>
              ) : (
                invoices.map((inv) => (
                  <Table.Row key={inv.id}>
                    <Table.Cell className="font-mono text-xs text-ui-fg-base font-medium">
                      {inv.id}
                    </Table.Cell>
                    <Table.Cell className="text-xs text-ui-fg-subtle">
                      {inv.date}
                    </Table.Cell>
                    <Table.Cell className="text-xs text-ui-fg-subtle">
                      {inv.period}
                    </Table.Cell>
                    <Table.Cell className="font-mono text-xs text-ui-fg-base font-semibold">
                      {inv.currency} {inv.amount}
                    </Table.Cell>
                    <Table.Cell>
                      <Badge color={inv.status === "paid" ? "green" : "orange"} size="2xsmall">
                        {inv.status === "paid" ? "Paid" : "Pending"}
                      </Badge>
                    </Table.Cell>
                    <Table.Cell className="text-right">
                      <Button
                        size="small"
                        variant="transparent"
                        onClick={() => handleDownloadInvoice(inv.id)}
                        aria-label={`Download invoice ${inv.id}`}
                      >
                        <ArrowDownTray className="h-3.5 w-3.5 text-ui-fg-subtle" />
                      </Button>
                    </Table.Cell>
                  </Table.Row>
                ))
              )}
            </Table.Body>
          </Table>
        </Container>

        {/* Included Services */}
        <Container className="p-6">
          <div className="border-ui-border-base border-b pb-4">
            <Heading level="h2" className="text-ui-fg-base text-base">
              Included Platform Services
            </Heading>
            <Text size="small" className="text-ui-fg-subtle">
              Features enabled on your merchant account. Services outside the plan return not found at the server.
            </Text>
          </div>
          <div className="grid grid-cols-1 gap-3 pt-5 sm:grid-cols-2 md:grid-cols-3">
            {gateable.map(([feature, enabled]) => (
              <div
                key={feature}
                className="border-ui-border-base bg-ui-bg-subtle flex items-center gap-x-2.5 rounded-lg border p-3"
              >
                {enabled ? (
                  <CheckCircle className="text-ui-tag-green-icon shrink-0" aria-hidden="true" />
                ) : (
                  <XCircle className="text-ui-fg-muted shrink-0" aria-hidden="true" />
                )}
                <Text
                  size="small"
                  weight="plus"
                  className={enabled ? "text-ui-fg-base capitalize" : "text-ui-fg-muted capitalize"}
                >
                  {humanise(feature)}
                </Text>
                <Badge size="xsmall" color={enabled ? "green" : "grey"} className="ms-auto">
                  {enabled ? "Included" : "Not in plan"}
                </Badge>
              </div>
            ))}
          </div>
        </Container>
      </div>

      {/* Plan Upgrade / Switch Modal */}
      {isPlanModalOpen && (
        <div
          ref={planModalRef}
          role="dialog"
          aria-modal="true"
          aria-labelledby="plan-modal-title"
          tabIndex={-1}
          className="fixed inset-0 z-50 flex items-center justify-center bg-ui-bg-overlay/50 p-4 backdrop-blur-sm"
        >
          <div className="w-full max-w-xl rounded-xl border border-ui-border-base bg-ui-bg-base p-6 shadow-xl">
            <div className="flex items-center justify-between pb-4 border-b border-ui-border-base">
              <div>
                <Heading id="plan-modal-title" level="h2">Select Subscription Plan</Heading>
                <Text size="xsmall" className="text-ui-fg-subtle">
                  Choose the plan that matches your business volume
                </Text>
              </div>
              <Button
                variant="transparent"
                size="small"
                onClick={() => setIsPlanModalOpen(false)}
                aria-label="Close"
              >
                ✕
              </Button>
            </div>

            <div className="grid grid-cols-1 gap-4 py-5 sm:grid-cols-2">
              {/* Free Plan */}
              <div className={`rounded-xl border p-4 flex flex-col justify-between ${tier === "free" ? "border-ui-tag-blue-border bg-ui-tag-blue-bg/30" : "border-ui-border-base"}`}>
                <div>
                  <div className="flex items-center justify-between">
                    <Heading level="h3" className="text-base font-semibold">Free Tier</Heading>
                    {tier === "free" && <Badge color="blue" size="2xsmall">Current</Badge>}
                  </div>
                  <Text size="large" weight="plus" className="mt-2 text-ui-fg-base">
                    $0.00 USD <span className="text-xs text-ui-fg-subtle font-normal">/ month</span>
                  </Text>
                  <ul className="mt-4 space-y-2 text-xs text-ui-fg-subtle">
                    <li>• Up to 100 products</li>
                    <li>• Up to 3 staff seats</li>
                    <li>• 30-day analytics history</li>
                    <li>• Standard manual fulfillment</li>
                  </ul>
                </div>
                <Button
                  variant="secondary"
                  size="small"
                  className="mt-5 w-full"
                  disabled={tier === "free"}
                  onClick={() => {
                    setIsPlanModalOpen(false)
                    toast.info("Contact platform administrator to downgrade.")
                  }}
                >
                  {tier === "free" ? "Active" : "Downgrade"}
                </Button>
              </div>

              {/* Pro Plan */}
              <div className={`rounded-xl border p-4 flex flex-col justify-between ${tier === "pro" ? "border-ui-tag-green-border bg-ui-tag-green-bg/30" : "border-ui-border-base"}`}>
                <div>
                  <div className="flex items-center justify-between">
                    <Heading level="h3" className="text-base font-semibold">Pro Plan</Heading>
                    {tier === "pro" && <Badge color="green" size="2xsmall">Current</Badge>}
                  </div>
                  <Text size="large" weight="plus" className="mt-2 text-ui-fg-base">
                    {proPlanPrice}
                  </Text>
                  <ul className="mt-4 space-y-2 text-xs text-ui-fg-subtle">
                    <li>• Unlimited products</li>
                    <li>• Up to 10 staff seats</li>
                    <li>• Payment gateways (bKash, SSLCommerz, Stripe)</li>
                    <li>• Courier automated dispatch (Pathao, Steadfast)</li>
                    <li>• Custom domain &amp; theme</li>
                  </ul>
                </div>
                <Button
                  variant="primary"
                  size="small"
                  className="mt-5 w-full"
                  disabled={tier === "pro" || !proPlan}
                  onClick={() => {
                    setIsPlanModalOpen(false)
                    handleOpenBillingPortal()
                  }}
                >
                  {tier === "pro" ? "Active" : proPlan ? "Upgrade to Pro" : "Pricing unavailable"}
                </Button>
              </div>
            </div>

            <div className="flex justify-end pt-3 border-t border-ui-border-base">
              <Button
                variant="secondary"
                size="small"
                onClick={() => setIsPlanModalOpen(false)}
              >
                Close
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Payment Method Modal */}
      {isPaymentMethodModalOpen && (
        <div
          ref={paymentModalRef}
          role="dialog"
          aria-modal="true"
          aria-labelledby="payment-modal-title"
          tabIndex={-1}
          className="fixed inset-0 z-50 flex items-center justify-center bg-ui-bg-overlay/50 p-4 backdrop-blur-sm"
        >
          <div className="w-full max-w-md rounded-xl border border-ui-border-base bg-ui-bg-base p-6 shadow-xl">
            <div className="flex items-center justify-between pb-4 border-b border-ui-border-base">
              <div>
                <Heading id="payment-modal-title" level="h2">Update Billing Method</Heading>
                <Text size="xsmall" className="text-ui-fg-subtle">
                  Select payment instrument for platform subscription
                </Text>
              </div>
              <Button
                variant="transparent"
                size="small"
                onClick={() => setIsPaymentMethodModalOpen(false)}
                aria-label="Close"
              >
                ✕
              </Button>
            </div>

            <div className="py-4 space-y-3">
              <Text size="small" className="text-ui-fg-subtle">
                No saved subscription payment method is recorded. Payment methods are managed
                through the platform billing portal.
              </Text>
              <Button
                size="small"
                variant="secondary"
                onClick={() => {
                  setIsPaymentMethodModalOpen(false)
                  void handleOpenBillingPortal(false)
                }}
              >
                Open billing portal
              </Button>
            </div>

            <div className="flex justify-end pt-3 border-t border-ui-border-base">
              <Button
                variant="secondary"
                size="small"
                onClick={() => setIsPaymentMethodModalOpen(false)}
              >
                Done
              </Button>
            </div>
          </div>
        </div>
      )}
    </SingleColumnPage>
  )
}

