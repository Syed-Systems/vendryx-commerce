import {
  ArrowPath,
  ArrowUturnLeft,
  BuildingStorefront,
  Channels,
  CreditCard,
  CurrencyDollar,
  DocumentText,
  GlobeEurope,
  Key,
  Language,
  LockClosedSolid,
  MagnifyingGlass,
  MapPin,
  ReceiptPercent,
  Sparkles,
  Tag,
  TruckFast,
  User,
  Users,
} from "@medusajs/icons"
import { Badge, Container, Heading, Input, Text } from "@medusajs/ui"
import { useMemo, useState } from "react"
import { SettingsRow } from "../../components/settings/settings-row"
import { SettingsSection } from "../../components/settings/settings-section"
import { SingleColumnPage } from "../../components/layout/pages"

interface SettingItem {
  id: string
  title: string
  description: string
  to: string
  icon: React.ReactNode
  synonyms: string[]
}

interface SettingCategory {
  id: string
  title: string
  description: string
  items: SettingItem[]
}

const SETTINGS_CATEGORIES: SettingCategory[] = [
  {
    id: "store-operations",
    title: "Store & Operations",
    description: "Store details, brand identity, fulfillment facilities, and operational markets.",
    items: [
      {
        id: "onboarding",
        title: "Store Setup Guide",
        description: "Milestone progression checklist for products, payments, couriers, and launch.",
        to: "/onboarding",
        icon: <Sparkles />,
        synonyms: ["onboarding", "setup", "wizard", "checklist", "launch", "milestones", "guide"],
      },
      {
        id: "branding",
        title: "Storefront Branding",
        description: "Store name, brand palette, curated typography, and live preview.",
        to: "/settings/branding",
        icon: <BuildingStorefront />,
        synonyms: ["branding", "theme", "colors", "font", "logo", "favicon", "appearance"],
      },
      {
        id: "store",
        title: "Store Details & Policies",
        description: "Store name, currency, language, and customer legal policies.",
        to: "/settings/store",
        icon: <BuildingStorefront />,
        synonyms: ["store", "brand", "policies", "legal", "terms", "privacy", "refund"],
      },
      {
        id: "locations",
        title: "Stock Locations",
        description: "Warehouses, fulfillment depots, and retail pickup addresses.",
        to: "/settings/locations",
        icon: <MapPin />,
        synonyms: ["locations", "stock", "warehouse", "pickup", "inventory", "depot"],
      },
      {
        id: "regions",
        title: "Operating Regions",
        description: "Currencies, payment providers, and tax regimes by geography.",
        to: "/settings/regions",
        icon: <GlobeEurope />,
        synonyms: ["regions", "country", "currency", "markets", "international"],
      },
    ],
  },
  {
    id: "product-catalog",
    title: "Product Catalog",
    description: "Taxonomies, classifications, tags, and product attribute definitions.",
    items: [
      {
        id: "product-types",
        title: "Product Types",
        description: "High-level classification groups for catalog items.",
        to: "/settings/product-types",
        icon: <Tag />,
        synonyms: ["types", "catalog", "category", "classification"],
      },
      {
        id: "product-tags",
        title: "Product Tags",
        description: "Search tags and promotional labels applied to items.",
        to: "/settings/product-tags",
        icon: <Tag />,
        synonyms: ["tags", "labels", "search", "filters"],
      },
      {
        id: "property-labels",
        title: "Property Labels",
        description: "Custom table display labels and metadata attributes.",
        to: "/settings/property-labels",
        icon: <DocumentText />,
        synonyms: ["property", "labels", "metadata", "attributes", "columns"],
      },
    ],
  },
  {
    id: "sales-customers",
    title: "Sales & Customers",
    description: "Sales channels, regional taxation, and multilingual storefront locales.",
    items: [
      {
        id: "sales-channels",
        title: "Sales Channels",
        description: "Online web storefronts, social commerce, and mobile points of sale.",
        to: "/settings/sales-channels",
        icon: <Channels />,
        synonyms: ["channels", "storefront", "pos", "social", "facebook", "instagram"],
      },
      {
        id: "tax-regions",
        title: "Tax Regions & Rates",
        description: "National VAT, customs duty, and localized tax overrides.",
        to: "/settings/tax-regions",
        icon: <ReceiptPercent />,
        synonyms: ["tax", "vat", "gst", "rates", "duty"],
      },
      {
        id: "translations",
        title: "Store Translations & Locales",
        description: "Localized product titles, descriptions, and checkout strings.",
        to: "/settings/translations",
        icon: <Language />,
        synonyms: ["translations", "language", "locales", "bengali", "english", "i18n"],
      },
    ],
  },
  {
    id: "delivery-fulfillment",
    title: "Delivery & Fulfillment",
    description: "Domestic couriers, zone rates, parcel tracking, and return logistics.",
    items: [
      {
        id: "couriers",
        title: "Courier Logistics",
        description: "Pathao, Steadfast, and RedX automated booking, rate zones, and probes.",
        to: "/settings/couriers",
        icon: <TruckFast />,
        synonyms: ["courier", "pathao", "steadfast", "redx", "delivery", "shipping", "rates", "dhaka"],
      },
      {
        id: "return-reasons",
        title: "Return Reasons",
        description: "Merchant-authorized reasons for parcel returns and exchanges.",
        to: "/settings/return-reasons",
        icon: <ArrowUturnLeft />,
        synonyms: ["returns", "claims", "damaged", "exchange", "reverse logistics"],
      },
      {
        id: "refund-reasons",
        title: "Refund Reasons",
        description: "Audit codes and accounting categorizations for payment returns.",
        to: "/settings/refund-reasons",
        icon: <CurrencyDollar />,
        synonyms: ["refunds", "accounting", "credit", "reversal"],
      },
    ],
  },
  {
    id: "payments-finance",
    title: "Payments & Finance",
    description: "Gateway credentials, Cash-on-Delivery rules, and platform subscription.",
    items: [
      {
        id: "payment-providers",
        title: "Payment Providers & COD Rules",
        description: "bKash, SSLCommerz, Stripe, and Cash on Delivery order thresholds.",
        to: "/settings/payment-providers",
        icon: <CreditCard />,
        synonyms: ["payment", "bkash", "sslcommerz", "stripe", "cod", "cash", "gateways"],
      },
      {
        id: "billing",
        title: "Subscription & Invoices",
        description: "Vendryx platform tier, usage quota status, and monthly VAT receipts.",
        to: "/settings/billing",
        icon: <ReceiptPercent />,
        synonyms: ["billing", "subscription", "plan", "invoices", "pricing", "tier", "pro"],
      },
    ],
  },
  {
    id: "security-access",
    title: "Security & Access",
    description: "Merchant team members, role-based access control, and operator credentials.",
    items: [
      {
        id: "users",
        title: "Users & Staff Permissions",
        description: "Invite teammates, manage admin seats, and assign RBAC roles.",
        to: "/settings/users",
        icon: <Users />,
        synonyms: ["users", "staff", "team", "roles", "permissions", "rbac", "access"],
      },
      {
        id: "profile",
        title: "Personal Profile & Password",
        description: "Authenticated login email, password rotation, and active device session.",
        to: "/settings/profile",
        icon: <User />,
        synonyms: ["profile", "password", "session", "security", "credentials"],
      },
    ],
  },
  {
    id: "developer-integrations",
    title: "Developer & Integrations",
    description: "Custom storefront domains, TLS certificates, API tokens, and workflows.",
    items: [
      {
        id: "domains",
        title: "Domains & SSL Certificates",
        description: "Custom primary domain routing and automated Let's Encrypt TLS.",
        to: "/settings/domains",
        icon: <GlobeEurope />,
        synonyms: ["domains", "dns", "ssl", "tls", "cname", "custom domain"],
      },
      {
        id: "publishable-api-keys",
        title: "Publishable API Keys",
        description: "Client-safe tokens scoped to storefront sales channels.",
        to: "/settings/publishable-api-keys",
        icon: <Key />,
        synonyms: ["publishable", "api", "keys", "tokens", "storefront api"],
      },
      {
        id: "secret-api-keys",
        title: "Secret API Keys",
        description: "Elevated administrative API keys for backend integrations.",
        to: "/settings/secret-api-keys",
        icon: <LockClosedSolid />,
        synonyms: ["secret", "api", "backend", "token", "webhooks"],
      },
      {
        id: "workflows",
        title: "Workflow Engine Executions",
        description: "Real-time state and compensation logs for durable commerce sagas.",
        to: "/settings/workflows",
        icon: <ArrowPath />,
        synonyms: ["workflows", "sagas", "executions", "durability", "engine"],
      },
    ],
  },
]

export const SettingsHub = () => {
  const [searchQuery, setSearchQuery] = useState("")

  const normalizedQuery = searchQuery.trim().toLowerCase()

  const filteredCategories = useMemo(() => {
    if (!normalizedQuery) {
      return SETTINGS_CATEGORIES
    }

    return SETTINGS_CATEGORIES.map((category) => {
      const matchingItems = category.items.filter((item) => {
        const titleMatch = item.title.toLowerCase().includes(normalizedQuery)
        const descMatch = item.description.toLowerCase().includes(normalizedQuery)
        const synonymMatch = item.synonyms.some((s) => s.toLowerCase().includes(normalizedQuery))
        return titleMatch || descMatch || synonymMatch
      })

      return {
        ...category,
        items: matchingItems,
      }
    }).filter((category) => category.items.length > 0)
  }, [normalizedQuery])

  const totalMatchingItems = filteredCategories.reduce(
    (acc, cat) => acc + cat.items.length,
    0
  )

  return (
    <SingleColumnPage widgets={{ after: [], before: [] }}>
      <div className="flex flex-col gap-y-6">
        {/* Header */}
        <div className="flex flex-col gap-y-1">
          <div className="flex items-center gap-x-2">
            <Heading level="h1">Settings</Heading>
            <Badge color="blue" size="2xsmall">
              Hub
            </Badge>
          </div>
          <Text size="small" className="text-ui-fg-subtle">
            Configure store operations, commerce policies, security credentials, and platform routing.
          </Text>
        </div>

        {/* Search Bar */}
        <Container className="p-4 bg-ui-bg-base">
          <div className="relative flex items-center">
            <MagnifyingGlass className="absolute left-3.5 h-4 w-4 text-ui-fg-muted pointer-events-none" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search settings (e.g. tax, bkash, domains, staff, rbac)..."
              className="pl-10 text-sm h-10"
              aria-label="Search settings"
            />
          </div>
          {normalizedQuery && (
            <Text size="xsmall" className="text-ui-fg-muted mt-2">
              Found {totalMatchingItems} {totalMatchingItems === 1 ? "setting" : "settings"} matching "{searchQuery}"
            </Text>
          )}
        </Container>

        {/* Categories / Results */}
        {filteredCategories.length === 0 ? (
          <Container className="p-12 text-center flex flex-col items-center justify-center">
            <Text size="small" weight="plus" className="text-ui-fg-base">
              No matching settings found
            </Text>
            <Text size="xsmall" className="text-ui-fg-subtle mt-1 max-w-sm">
              No setting matched "{searchQuery}". Try searching for terms like "vat", "cash", "domain", or "users".
            </Text>
          </Container>
        ) : (
          <div className="flex flex-col gap-y-8">
            {filteredCategories.map((category) => (
              <SettingsSection
                key={category.id}
                title={category.title}
                description={category.description}
              >
                {category.items.map((item) => (
                  <SettingsRow
                    key={item.id}
                    title={item.title}
                    description={item.description}
                    icon={item.icon}
                    to={item.to}
                  />
                ))}
              </SettingsSection>
            ))}
          </div>
        )}
      </div>
    </SingleColumnPage>
  )
}
