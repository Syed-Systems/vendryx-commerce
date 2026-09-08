import {
  ArrowRight,
  ArrowUpRightOnBox,
  BuildingStorefront,
  CreditCard,
  ShoppingCart,
  Tag,
} from "@medusajs/icons"
import { Badge, Button, Container, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"
import { useDate } from "../../../hooks/use-date"
import { STHENO_STOREFRONT_URL } from "../../../lib/storefront"
import type { RecentOrderItem } from "../../../hooks/api/recent-orders"

// ─────────────────────────────────────────────────────────────────────────────
// Recent Orders Card & First Sale Launchpad
// ─────────────────────────────────────────────────────────────────────────────
//
// Shows live order workstream when orders exist.
// When store has 0 orders, provides an actionable First Sale Launchpad rather
// than an empty dead-end box.

function formatTotal(total: string | null, currencyCode: string): string {
  if (total === null || total === undefined) return "—"
  try {
    const amount = parseFloat(total)
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: currencyCode.toUpperCase(),
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(amount)
  } catch {
    return total
  }
}

function getCustomerName(order: RecentOrderItem): string {
  const first = order.shipping_address?.first_name ?? ""
  const last = order.shipping_address?.last_name ?? ""
  const full = [first, last].filter(Boolean).join(" ").trim()
  return full || "Guest Customer"
}

function getStatusBadgeColor(
  status: string
): "orange" | "green" | "red" | "grey" {
  switch (status) {
    case "pending":
    case "requires_action":
      return "orange"
    case "completed":
      return "green"
    case "canceled":
    case "archived":
      return "red"
    default:
      return "grey"
  }
}

const SkeletonRow = () => (
  <div className="flex items-center justify-between py-3.5 animate-pulse">
    <div className="flex flex-col gap-1.5 flex-1">
      <div className="h-4 bg-ui-bg-component rounded w-44" />
      <div className="h-3 bg-ui-bg-component rounded w-28" />
    </div>
    <div className="h-5 bg-ui-bg-component rounded w-20" />
  </div>
)

export const RecentOrdersCard = ({
  orders,
  isLoading,
}: {
  orders: RecentOrderItem[] | null
  isLoading: boolean
}) => {
  const { getRelativeDate } = useDate()

  const hasOrders = orders && orders.length > 0

  return (
    <Container className="flex flex-col p-0 overflow-hidden border border-ui-border-base rounded-xl bg-ui-bg-base shadow-sm">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-ui-border-base bg-ui-bg-subtle/30">
        <div className="flex items-center gap-2.5">
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-ui-bg-component border border-ui-border-base text-ui-fg-base shadow-xs">
            <ShoppingCart className="w-4 h-4 text-ui-fg-subtle" />
          </div>
          <div>
            <Heading level="h3" className="text-sm font-semibold text-ui-fg-base">
              Recent Orders
            </Heading>
            <Text size="xsmall" className="text-ui-fg-subtle">
              {hasOrders
                ? "Live incoming customer purchase stream"
                : "Real-time order activity will stream here"}
            </Text>
          </div>
        </div>

        <Button variant="transparent" size="small" asChild className="text-xs">
          <Link to="/orders">
            View all orders
            <ArrowRight className="w-3.5 h-3.5 ml-1" />
          </Link>
        </Button>
      </div>

      {/* Loading Skeleton */}
      {isLoading && (
        <div className="px-6 divide-y divide-ui-border-base">
          <SkeletonRow />
          <SkeletonRow />
          <SkeletonRow />
        </div>
      )}

      {/* Empty State: First Sale Launchpad */}
      {!isLoading && !hasOrders && (
        <div className="p-6 flex flex-col gap-5">
          <div className="flex items-start gap-3.5 p-4 rounded-xl bg-ui-bg-subtle/50 border border-ui-border-base">
            <div className="p-2.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-600 shrink-0">
              <BuildingStorefront className="w-5 h-5" />
            </div>
            <div>
              <Heading level="h3" className="text-sm font-semibold text-ui-fg-base mb-1">
                Your store is live and ready for checkout
              </Heading>
              <Text size="small" className="text-ui-fg-subtle leading-relaxed">
                Customers can browse your catalog, add items to cart, and complete orders on your storefront. When an order is placed, it will immediately appear here for payment capture and fulfillment.
              </Text>
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <Link
              to="/products"
              className="flex flex-col justify-between p-3.5 rounded-lg border border-ui-border-base hover:border-ui-border-strong hover:bg-ui-bg-subtle/30 transition-all group no-underline"
            >
              <div className="flex items-center gap-2 mb-2">
                <Tag className="w-4 h-4 text-purple-600" />
                <Text size="small" weight="plus" className="text-ui-fg-base group-hover:text-ui-fg-interactive transition-colors">
                  1. Add Products
                </Text>
              </div>
              <Text size="xsmall" className="text-ui-fg-muted">
                Ensure active items and inventory are published.
              </Text>
            </Link>

            <Link
              to="/payment-providers"
              className="flex flex-col justify-between p-3.5 rounded-lg border border-ui-border-base hover:border-ui-border-strong hover:bg-ui-bg-subtle/30 transition-all group no-underline"
            >
              <div className="flex items-center gap-2 mb-2">
                <CreditCard className="w-4 h-4 text-blue-600" />
                <Text size="small" weight="plus" className="text-ui-fg-base group-hover:text-ui-fg-interactive transition-colors">
                  2. Payment Gateways
                </Text>
              </div>
              <Text size="xsmall" className="text-ui-fg-muted">
                Enable Cash on Delivery (COD) or card providers.
              </Text>
            </Link>

            <a
              href={STHENO_STOREFRONT_URL}
              target="_blank"
              rel="noreferrer"
              className="flex flex-col justify-between p-3.5 rounded-lg border border-ui-border-base hover:border-ui-border-strong hover:bg-ui-bg-subtle/30 transition-all group no-underline"
            >
              <div className="flex items-center justify-between gap-2 mb-2">
                <div className="flex items-center gap-2">
                  <ShoppingCart className="w-4 h-4 text-emerald-600" />
                  <Text size="small" weight="plus" className="text-ui-fg-base group-hover:text-ui-fg-interactive transition-colors">
                    3. Test Checkout
                  </Text>
                </div>
                <ArrowUpRightOnBox className="w-3.5 h-3.5 text-ui-fg-muted" />
              </div>
              <Text size="xsmall" className="text-ui-fg-muted">
                Place a test order to preview customer receipts.
              </Text>
            </a>
          </div>
        </div>
      )}

      {/* Populated Order Rows */}
      {!isLoading && hasOrders && (
        <div className="px-6 divide-y divide-ui-border-base">
          {orders.map((order) => (
            <div
              key={order.id}
              className="flex items-center justify-between py-3.5 group hover:bg-ui-bg-subtle/20 transition-colors -mx-6 px-6"
            >
              <div className="flex flex-col gap-0.5 min-w-0 flex-1">
                <div className="flex items-center gap-2.5">
                  <Link
                    to={`/orders/${order.id}`}
                    className="txt-compact-small-plus text-ui-fg-base hover:text-ui-fg-interactive font-medium"
                  >
                    #{order.display_id}
                  </Link>
                  <Text size="xsmall" className="text-ui-fg-subtle truncate">
                    {getCustomerName(order)}
                  </Text>
                </div>
                <Text size="xsmall" className="text-ui-fg-muted">
                  {getRelativeDate(order.created_at)}
                </Text>
              </div>

              <div className="flex items-center gap-3 shrink-0 ml-4">
                <Text
                  size="small"
                  weight="plus"
                  className="text-ui-fg-base tabular-nums font-semibold"
                >
                  {formatTotal(order.total, order.currency_code)}
                </Text>
                <Badge
                  color={getStatusBadgeColor(order.status)}
                  size="2xsmall"
                  className="capitalize"
                >
                  {order.status.replace(/_/g, " ")}
                </Badge>
              </div>
            </div>
          ))}
        </div>
      )}
    </Container>
  )
}
