import { ArrowUpRightOnBox, BuildingStorefront, Palette, Plus } from "@medusajs/icons"
import { Button, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"
import { STHENO_STOREFRONT_URL } from "../../../lib/storefront"

function getTimeGreeting(): string {
  const hour = new Date().getHours()
  if (hour < 12) return "Good morning"
  if (hour < 17) return "Good afternoon"
  return "Good evening"
}

function formatDate(): string {
  return new Intl.DateTimeFormat("en-US", {
    weekday: "long",
    month: "long",
    day: "numeric",
  }).format(new Date())
}

export const GreetingHeader = ({
  firstName,
  hasAttentionItems,
  onOpenThemeEdit,
}: {
  firstName: string | null | undefined
  hasAttentionItems: boolean
  onOpenThemeEdit?: () => void
}) => {
  const greeting = getTimeGreeting()
  const name = firstName?.trim() || ""
  const dateStr = formatDate()

  const contextLine = hasAttentionItems
    ? `${dateStr} · Attention items require review`
    : `${dateStr} · All systems operational`

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-2 border-b border-ui-border-base/60">
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-3">
          <Heading className="text-2xl font-bold tracking-tight text-ui-fg-base">
            {greeting}
            {name ? `, ${name}` : ""}
          </Heading>
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-600 border border-emerald-500/20">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            Live Store
          </span>
        </div>
        <Text size="small" className="text-ui-fg-muted">
          {contextLine}
        </Text>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        {onOpenThemeEdit && (
          <Button
            variant="secondary"
            size="small"
            onClick={onOpenThemeEdit}
            className="gap-x-1.5 text-xs font-medium"
          >
            <Palette className="w-3.5 h-3.5 text-ui-fg-subtle" />
            Theme
          </Button>
        )}
        <Button variant="secondary" size="small" asChild className="gap-x-1.5 text-xs font-medium">
          <a href={STHENO_STOREFRONT_URL} target="_blank" rel="noreferrer">
            <BuildingStorefront className="w-3.5 h-3.5 text-ui-fg-subtle" />
            Storefront
            <ArrowUpRightOnBox className="w-3 h-3 ml-0.5 text-ui-fg-muted" />
          </a>
        </Button>
        <Button variant="primary" size="small" asChild className="gap-x-1.5 text-xs font-medium">
          <Link to="/products">
            <Plus className="w-3.5 h-3.5" />
            Add Product
          </Link>
        </Button>
      </div>
    </div>
  )
}
