import {
  CheckCircleSolid,
  CurrencyDollar,
  ExclamationCircle,
  ShoppingCart,
  Tag,
} from "@medusajs/icons"
import { Container, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"
import type { TodayPulse } from "../../../hooks/api/today-pulse"

// ─────────────────────────────────────────────────────────────────────────────
// Store Vitals — 4-metric command grid
// ─────────────────────────────────────────────────────────────────────────────
//
// Always visible: a busy store owner needs to see their core metrics at a glance,
// even during setup or on days with zero sales.
// Every figure is strictly sourced from live backend data (AGENTS.md standing rule).

function formatCurrency(value: string | null | number, currencyCode: string = "USD"): string {
  if (value === null || value === undefined) return "$0.00"
  try {
    const num = typeof value === "string" ? parseFloat(value) : value
    if (isNaN(num)) return "$0.00"
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: currencyCode.toUpperCase(),
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(num)
  } catch {
    return `$0.00`
  }
}

interface StatCardProps {
  icon: React.ReactNode
  iconBg: string
  label: string
  value: string | number
  subtext: string
  href?: string
  badge?: {
    text: string
    variant: "neutral" | "green" | "amber" | "blue"
  }
}

const StatCard = ({
  icon,
  iconBg,
  label,
  value,
  subtext,
  href,
  badge,
}: StatCardProps) => {
  const content = (
    <Container className="flex flex-col justify-between p-5 border border-ui-border-base rounded-xl bg-ui-bg-base transition-all hover:border-ui-border-strong hover:shadow-elevation-card-hover group">
      <div>
        <div className="flex items-center justify-between gap-2 mb-3">
          <div className="flex items-center gap-2.5">
            <div
              className={`flex items-center justify-center w-8 h-8 rounded-lg ${iconBg} shadow-sm shrink-0`}
            >
              {icon}
            </div>
            <Text size="small" weight="plus" className="text-ui-fg-subtle group-hover:text-ui-fg-base transition-colors">
              {label}
            </Text>
          </div>
          {badge && (
            <span
              className={`text-[11px] font-medium px-2 py-0.5 rounded-full border ${
                badge.variant === "green"
                  ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                  : badge.variant === "amber"
                  ? "bg-amber-500/10 text-amber-600 border-amber-500/20"
                  : badge.variant === "blue"
                  ? "bg-blue-500/10 text-blue-600 border-blue-500/20"
                  : "bg-ui-bg-subtle text-ui-fg-muted border-ui-border-base"
              }`}
            >
              {badge.text}
            </span>
          )}
        </div>

        <Heading
          level="h2"
          className="text-2xl font-bold tracking-tight text-ui-fg-base tabular-nums mb-1"
        >
          {value}
        </Heading>
      </div>

      <Text size="xsmall" className="text-ui-fg-muted mt-2 truncate">
        {subtext}
      </Text>
    </Container>
  )

  if (href) {
    return (
      <Link to={href} className="no-underline block">
        {content}
      </Link>
    )
  }

  return content
}

export const TodayPulseCard = ({
  today,
  totalOrders = 0,
  productsCount = 0,
  pendingOrdersCount = 0,
  lowStockCount = 0,
  currencyCode = "USD",
  isLoading = false,
}: {
  today: TodayPulse | null
  totalOrders?: number
  productsCount?: number
  pendingOrdersCount?: number
  lowStockCount?: number
  currencyCode?: string
  isLoading?: boolean
}) => {
  if (isLoading) {
    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {[0, 1, 2, 3].map((i) => (
          <Container key={i} className="p-5 animate-pulse border border-ui-border-base rounded-xl">
            <div className="h-8 w-8 bg-ui-bg-component rounded-lg mb-3" />
            <div className="h-7 bg-ui-bg-component rounded w-24 mb-2" />
            <div className="h-3 bg-ui-bg-component rounded w-32" />
          </Container>
        ))}
      </div>
    )
  }

  const revenueVal = today?.revenue ? formatCurrency(today.revenue, today.currency_code || currencyCode) : formatCurrency(0, currencyCode)
  const ordersToday = today?.order_count ?? 0
  const urgentCount = pendingOrdersCount + lowStockCount

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {/* 1. Revenue */}
      <StatCard
        icon={<CurrencyDollar className="w-4 h-4 text-emerald-600" />}
        iconBg="bg-emerald-500/10 border border-emerald-500/20"
        label="Today's Revenue"
        value={revenueVal}
        subtext={ordersToday > 0 ? `${ordersToday} orders today` : "Live store revenue stream"}
        badge={{ text: "Today", variant: "green" }}
      />

      {/* 2. Orders */}
      <StatCard
        icon={<ShoppingCart className="w-4 h-4 text-blue-600" />}
        iconBg="bg-blue-500/10 border border-blue-500/20"
        label="Orders"
        value={totalOrders.toLocaleString()}
        subtext={
          pendingOrdersCount > 0
            ? `${pendingOrdersCount} awaiting payment/capture`
            : totalOrders === 0
            ? "Awaiting first customer checkout"
            : "All orders up to date"
        }
        href="/orders"
        badge={pendingOrdersCount > 0 ? { text: `${pendingOrdersCount} pending`, variant: "amber" } : undefined}
      />

      {/* 3. Catalog Products */}
      <StatCard
        icon={<Tag className="w-4 h-4 text-purple-600" />}
        iconBg="bg-purple-500/10 border border-purple-500/20"
        label="Active Products"
        value={productsCount.toLocaleString()}
        subtext={productsCount === 0 ? "Add your first product to start selling" : "Published in catalog"}
        href="/products"
      />

      {/* 4. Operational Health */}
      <StatCard
        icon={
          urgentCount > 0 ? (
            <ExclamationCircle className="w-4 h-4 text-amber-600" />
          ) : (
            <CheckCircleSolid className="w-4 h-4 text-emerald-600" />
          )
        }
        iconBg={
          urgentCount > 0
            ? "bg-amber-500/10 border border-amber-500/20"
            : "bg-emerald-500/10 border border-emerald-500/20"
        }
        label="Store Health"
        value={urgentCount > 0 ? `${urgentCount} to Review` : "Optimal"}
        subtext={
          urgentCount > 0
            ? `${pendingOrdersCount} pending · ${lowStockCount} low stock`
            : "0 operational bottlenecks"
        }
        href={urgentCount > 0 ? "/orders" : undefined}
        badge={{
          text: urgentCount > 0 ? "Action needed" : "Healthy",
          variant: urgentCount > 0 ? "amber" : "green",
        }}
      />
    </div>
  )
}
