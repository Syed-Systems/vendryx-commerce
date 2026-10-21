import {
  ArrowRight,
  CubeSolid,
  ExclamationCircle,
  ShoppingCart,
} from "@medusajs/icons"
import { Container, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"

// ─────────────────────────────────────────────────────────────────────────────
// Operational Attention Strip
// ─────────────────────────────────────────────────────────────────────────────
//
// Exception-driven alerts for urgent merchant actions.
// Appears ONLY when real operational bottlenecks exist (e.g., pending payment,
// out of stock). When everything is clear, it returns null and gets out of the way.

interface AttentionItem {
  key: string
  count: number
  label: string
  description: string
  href: string
  color: "amber" | "red"
  icon: React.ReactNode
}

export const AttentionStrip = ({
  pendingOrderCount,
  lowStockCount,
  lowStockThreshold,
}: {
  pendingOrderCount: number
  lowStockCount: number
  lowStockThreshold: number
}) => {
  const items: AttentionItem[] = []

  if (pendingOrderCount > 0) {
    items.push({
      key: "pending",
      count: pendingOrderCount,
      label: "Orders Awaiting Processing",
      description: "Payment confirmation or capture needed before fulfillment",
      href: "/orders?payment_status=awaiting",
      color: "amber",
      icon: <ShoppingCart className="w-4 h-4 text-amber-600" />,
    })
  }

  if (lowStockCount > 0) {
    items.push({
      key: "stock",
      count: lowStockCount,
      label: "Low Stock Inventory Alerts",
      description: `Variants at or below ${lowStockThreshold}-unit threshold`,
      href: "/inventory",
      color: "red",
      icon: <CubeSolid className="w-4 h-4 text-red-600" />,
    })
  }

  // If no items require urgent attention, stay out of the way!
  if (items.length === 0) {
    return null
  }

  // Urgency reads from the icon and the tinted surface. The 4px left border
  // that used to sit here was a third, redundant cue for the same meaning --
  // and a thick coloured side-border is the single most recognisable tell of a
  // generated UI, so it was costing more than it said.
  const borderStyles = {
    amber: "bg-amber-500/[0.07]",
    red: "bg-rose-500/[0.07]",
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2 px-1">
        <ExclamationCircle className="w-4 h-4 text-amber-600 shrink-0" />
        <Text size="xsmall" weight="plus" className="text-ui-fg-base uppercase tracking-wider">
          Action Required
        </Text>
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {items.map((item) => (
          <Link
            key={item.key}
            to={item.href}
            className="group no-underline block"
          >
            <Container
              className={`flex items-center justify-between p-4 border border-ui-border-base rounded-xl transition-[color,background-color,border-color,box-shadow,transform,opacity] hover:shadow-elevation-card-hover ${borderStyles[item.color]}`}
            >
              <div className="flex items-center gap-3 min-w-0">
                <div className="p-2 rounded-lg bg-ui-bg-base border border-ui-border-base shadow-sm shrink-0">
                  {item.icon}
                </div>
                <div className="flex flex-col gap-0.5 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="inline-flex items-center justify-center px-2 py-0.5 rounded-full text-xs font-bold bg-ui-bg-base border border-ui-border-base text-ui-fg-base tabular-nums">
                      {item.count}
                    </span>
                    <Heading level="h3" className="text-sm font-semibold text-ui-fg-base truncate">
                      {item.label}
                    </Heading>
                  </div>
                  <Text size="xsmall" className="text-ui-fg-subtle truncate">
                    {item.description}
                  </Text>
                </div>
              </div>

              <div className="flex items-center text-ui-fg-muted group-hover:text-ui-fg-base transition-colors shrink-0 ml-3">
                <ArrowRight className="w-4 h-4" />
              </div>
            </Container>
          </Link>
        ))}
      </div>
    </div>
  )
}
