import {
  ArrowRight,
  CreditCard,
  CubeSolid,
  Sparkles,
  Tag,
  Users,
} from "@medusajs/icons"
import { Container, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"

const SHORTCUTS = [
  {
    title: "Payment Providers",
    description: "Manage COD, bKash & gateways",
    href: "/payment-providers",
    icon: <CreditCard className="w-4 h-4 text-blue-600" />,
    badge: "Payments",
  },
  {
    title: "Inventory & Stock",
    description: "Stock levels and replenishment",
    href: "/inventory",
    icon: <CubeSolid className="w-4 h-4 text-purple-600" />,
    badge: "Catalog",
  },
  {
    title: "Promotions & Discounts",
    description: "Coupon codes and sales rules",
    href: "/promotions",
    icon: <Tag className="w-4 h-4 text-emerald-600" />,
    badge: "Marketing",
  },
  {
    title: "Customer Directory",
    description: "Registered buyers and groups",
    href: "/customers",
    icon: <Users className="w-4 h-4 text-amber-600" />,
    badge: "Customers",
  },
]

export const QuickOperationsCard = () => {
  return (
    <Container className="flex flex-col p-0 overflow-hidden border border-ui-border-base rounded-xl bg-ui-bg-base shadow-sm">
      <div className="flex items-center justify-between px-6 py-4 border-b border-ui-border-base bg-ui-bg-subtle/30">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-ui-fg-subtle" />
          <Heading level="h3" className="text-sm font-semibold text-ui-fg-base">
            Store Management
          </Heading>
        </div>
        <Text size="xsmall" className="text-ui-fg-muted">
          Quick shortcuts
        </Text>
      </div>

      <div className="p-3 divide-y divide-ui-border-base">
        {SHORTCUTS.map((item) => (
          <Link
            key={item.title}
            to={item.href}
            className="flex items-center justify-between p-3 rounded-lg hover:bg-ui-bg-subtle/40 transition-colors group no-underline"
          >
            <div className="flex items-center gap-3 min-w-0">
              <div className="p-2 rounded-lg bg-ui-bg-component border border-ui-border-base shrink-0 group-hover:scale-105 transition-transform">
                {item.icon}
              </div>
              <div className="flex flex-col min-w-0">
                <Text size="small" weight="plus" className="text-ui-fg-base group-hover:text-ui-fg-interactive transition-colors truncate">
                  {item.title}
                </Text>
                <Text size="xsmall" className="text-ui-fg-subtle truncate">
                  {item.description}
                </Text>
              </div>
            </div>
            <ArrowRight className="w-4 h-4 text-ui-fg-muted group-hover:text-ui-fg-base transition-colors shrink-0 ml-2" />
          </Link>
        ))}
      </div>
    </Container>
  )
}
