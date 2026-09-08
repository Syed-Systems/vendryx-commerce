import { Badge, Container, Heading, Text } from "@medusajs/ui"
import { SingleColumnPage } from "../../components/layout/pages"

interface ReleaseEntry {
  version: string
  date: string
  isCurrent: boolean
  tagline: string
  highlights: {
    category: string
    title: string
    description: string
  }[]
}

const RELEASES: ReleaseEntry[] = [
  {
    version: "v2.19.0",
    date: "September 8, 2026",
    isCurrent: true,
    tagline: "Vendryx Core Product Rebrand, Provider Verification & Logistics Control",
    highlights: [
      {
        category: "Branding & Interface",
        title: "Vendryx Brand Architecture & Warm Slate Theme",
        description:
          "Full merchant interface adaptation under Syed Systems with `--vx-*` warm canvas palette, 64px topbar, and responsive viewport support across desktop, tablet, and mobile.",
      },
      {
        category: "Payments",
        title: "Verified Merchant Payment Gateways & AES-256-GCM Vault",
        description:
          "Live handshake connection probes for bKash (token grant) and Stripe (balance lookup). Honest status badges (Connected, Needs Verification, Not Configured) and secret opacity.",
      },
      {
        category: "Logistics",
        title: "Courier Logistics Engine (Pathao, Steadfast, RedX)",
        description:
          "Integrated courier configuration with zone rate overrides for Inside Dhaka, Suburbs, and Outside Dhaka, plus one-click webhook callback registration.",
      },
      {
        category: "Custom Domains",
        title: "Custom Domain Routing & Let's Encrypt SSL Tracker",
        description:
          "Point custom apex or subdomains to Vendryx with automated CNAME and TXT challenge verification, accompanied by Cloudflare and GoDaddy setup instructions.",
      },
      {
        category: "Storefront",
        title: "Cash on Delivery & Policy Management",
        description:
          "Dedicated COD controls with Dhaka Metro restriction and maximum order value caps. Legal policy editors for Returns, Terms, Privacy, and Shipping.",
      },
    ],
  },
  {
    version: "v2.18.0",
    date: "August 20, 2026",
    isCurrent: false,
    tagline: "High-Throughput Inventory Engine & Multi-Currency Settlement",
    highlights: [
      {
        category: "Commerce Engine",
        title: "Stock Reservations with Automated Expiry",
        description:
          "Atomic inventory reservations during checkout prevent overselling during high-concurrency campaigns. TTL release returns unpurchased stock without locking Postgres tables.",
      },
      {
        category: "Localization",
        title: "Bangladeshi Taka (BDT ৳) Native Engine",
        description:
          "Full localization for Bangladeshi commerce: decimal-accurate pricing, divisional addresses, and English/Bengali UI support.",
      },
    ],
  },
  {
    version: "v2.17.0",
    date: "July 15, 2026",
    isCurrent: false,
    tagline: "Postgres Schema Isolation & Cross-Module Contract Boundaries",
    highlights: [
      {
        category: "Architecture",
        title: "Zero Cross-Schema SQL Enforcement",
        description:
          "Strict module isolation verified by static analysis gates: modules remain autonomous leaves interacting solely via workflows, query links, and transactional event streams.",
      },
      {
        category: "Security",
        title: "Anti-IDOR Ownership Verification",
        description:
          "Scoped access resolution prevents unauthorized access to customer records, returning 404 on unowned resources.",
      },
    ],
  },
]

export const ReleaseNotesView = () => {
  return (
    <SingleColumnPage widgets={{ after: [], before: [] }}>
      <div className="flex flex-col gap-y-6">
        <div className="flex flex-col gap-y-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <Text size="xsmall" weight="plus" className="text-ui-fg-muted">
              Platform Changelog
            </Text>
            <div className="mt-1 flex items-center gap-x-2.5">
              <Heading level="h1">Release Notes</Heading>
              <Badge color="green" size="2xsmall">
                v2.19.0 Current
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle mt-1">
              Track new features, merchant tools, security improvements, and system updates shipped to Vendryx.
            </Text>
          </div>
        </div>

        {/* Timeline of releases */}
        <div className="space-y-6">
          {RELEASES.map((rel) => (
            <Container key={rel.version} className="p-6">
              <div className="flex flex-col gap-y-2 sm:flex-row sm:items-center sm:justify-between border-b border-ui-border-base pb-4">
                <div className="flex items-center gap-x-3">
                  <Badge
                    size="small"
                    color={rel.isCurrent ? "green" : "grey"}
                    className="font-mono text-xs font-semibold"
                  >
                    {rel.version}
                  </Badge>
                  {rel.isCurrent && (
                    <Badge color="orange" size="2xsmall">
                      Current Release
                    </Badge>
                  )}
                  <Text size="xsmall" className="text-ui-fg-subtle">
                    Released {rel.date}
                  </Text>
                </div>
              </div>

              <div className="pt-4">
                <Heading level="h2" className="text-base font-semibold text-ui-fg-base">
                  {rel.tagline}
                </Heading>

                <div className="mt-4 grid grid-cols-1 gap-4 md:grid-cols-2">
                  {rel.highlights.map((item, idx) => (
                    <div
                      key={idx}
                      className="rounded-lg border border-ui-border-base bg-ui-bg-subtle/40 p-4"
                    >
                      <div className="flex items-center gap-x-2 mb-1.5">
                        <Badge size="2xsmall" color="blue" className="uppercase text-[10px]">
                          {item.category}
                        </Badge>
                        <Text size="small" weight="plus" className="text-ui-fg-base">
                          {item.title}
                        </Text>
                      </div>
                      <Text size="xsmall" className="text-ui-fg-subtle leading-relaxed">
                        {item.description}
                      </Text>
                    </div>
                  ))}
                </div>
              </div>
            </Container>
          ))}
        </div>
      </div>
    </SingleColumnPage>
  )
}
