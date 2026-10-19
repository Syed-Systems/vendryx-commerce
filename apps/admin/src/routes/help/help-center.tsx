import {
  ChatBubble,
  DocumentText,
  InformationCircleSolid,
  ShieldCheck,
  Envelope,
} from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Heading,
  Input,
  Text,
} from "@medusajs/ui"
import { useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"

interface HelpArticle {
  id: string
  category: "payments" | "couriers" | "domains" | "store" | "inventory"
  title: string
  summary: string
  content: string
}

const ARTICLES: HelpArticle[] = [
  {
    id: "bkash-setup",
    category: "payments",
    title: "Connecting bKash Merchant Checkout",
    summary: "Step-by-step guide to obtaining bKash App Key, Secret, and Username from bKash PGW portal.",
    content:
      "To accept bKash payments, you require a registered bKash Merchant Account. 1) Log in to bKash PGW Merchant Portal. 2) Retrieve your App Key, App Secret, Username, and Password under API Credentials. 3) In Vendryx Admin, navigate to Settings > Payment Providers > bKash and click Configure. 4) Paste credentials into the AES-256-GCM encrypted vault. 5) Click 'Verify connection' to perform an authentic handshake token probe.",
  },
  {
    id: "cod-rules",
    category: "payments",
    title: "Configuring Cash on Delivery (COD) Rules",
    summary: "Set maximum order thresholds, restrict to Dhaka Metro, and enforce phone verification.",
    content:
      "Vendryx provides dedicated Cash on Delivery rules to minimize fake orders and returns: 1) Go to Settings > Payment Providers. 2) Under the COD Rules card, specify the Maximum Order Amount (e.g. ৳25,000). 3) Toggle 'Restrict to Dhaka Metro' if you only offer COD within the capital. 4) Enable 'Phone verification required' to mandate OTP or operator confirmation before courier dispatch.",
  },
  {
    id: "pathao-logistics",
    category: "couriers",
    title: "Pathao Merchant API Integration & Webhook",
    summary: "Automated parcel booking, delivery tracking, and store pickup address mapping.",
    content:
      "Connect your Pathao merchant account: 1) Log in to merchant.pathao.com and obtain your Client ID, Client Secret, Username, and Password. 2) In Vendryx Admin, navigate to Settings > Courier Logistics. 3) Click 'Configure' on Pathao and input your credentials. 4) Copy the provided Vendryx Webhook URL and register it in your Pathao merchant dashboard for real-time delivery status callbacks.",
  },
  {
    id: "steadfast-logistics",
    category: "couriers",
    title: "Steadfast Courier Integration",
    summary: "Nationwide delivery with Steadfast API Key and Secret Key.",
    content:
      "Steadfast Courier provides reliable door-to-door delivery across all 64 districts. 1) Retrieve your API Key and Secret Key from the Steadfast Merchant panel. 2) Enter credentials into the Vendryx Courier modal. 3) Configure zone rates under 'Zone Delivery Rates' to adjust inside/outside Dhaka charges.",
  },
  {
    id: "custom-domain",
    category: "domains",
    title: "Custom Domain DNS Verification & SSL",
    summary: "Point your apex domain or subdomain to Vendryx with CNAME and TXT verification records.",
    content:
      "Vendryx provides automated Let's Encrypt TLS/SSL provisioning. 1) In Settings > Domains & SSL, enter your custom domain (e.g. shop.yourbrand.com). 2) At your domain registrar (Cloudflare, GoDaddy, Namecheap), add a CNAME record pointing to shops.vendryx.com. 3) Add the TXT verification challenge record. 4) Click 'Verify DNS records' to initiate automated certificate issuance.",
  },
  {
    id: "inventory-reservations",
    category: "inventory",
    title: "Stock Reservations and TTL Expiry",
    summary: "How checkout stock reservations prevent double-selling during flash sales.",
    content:
      "When a customer begins checkout, items are reserved for 15 minutes. If payment is completed, reservations convert to permanent order line items. If abandoned or payment fails, reservations safely expire and return items to available inventory with zero lock contention.",
  },
]

export const HelpCenter = () => {
  const [activeTab, setActiveTab] = useState<"docs" | "contact">("docs")
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedCategory, setSelectedCategory] = useState<string>("all")
  const [expandedArticleId, setExpandedArticleId] = useState<string | null>("bkash-setup")

  const filteredArticles = ARTICLES.filter((art) => {
    const matchesCat = selectedCategory === "all" || art.category === selectedCategory
    const matchesSearch =
      searchQuery === "" ||
      art.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      art.summary.toLowerCase().includes(searchQuery.toLowerCase()) ||
      art.content.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesCat && matchesSearch
  })

  return (
    <SingleColumnPage widgets={{ after: [], before: [] }}>
      <div className="flex flex-col gap-y-6">
        {/* Header */}
        <div className="flex flex-col gap-y-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <div className="flex items-center gap-x-2">
              <Heading level="h1">Help &amp; Documentation</Heading>
              <Badge color="blue" size="2xsmall">
                Support
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle mt-1">
              Official setup guides, configuration manuals, and Syed Systems engineering contact.
            </Text>
          </div>

          <div className="flex items-center gap-x-2">
            <Button
              size="small"
              variant={activeTab === "docs" ? "primary" : "secondary"}
              onClick={() => setActiveTab("docs")}
            >
              <DocumentText className="me-1.5" />
              Documentation
            </Button>
            <Button
              size="small"
              variant={activeTab === "contact" ? "primary" : "secondary"}
              onClick={() => setActiveTab("contact")}
            >
              <ChatBubble className="me-1.5" />
              Contact Support
            </Button>
          </div>
        </div>

        {activeTab === "docs" && (
          <>
            {/* Search and Category Filter Card */}
            <Container className="p-4">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div className="relative flex-1 max-w-md">
                  <Input
                    placeholder="Search documentation (e.g. bKash, Steadfast, DNS, SSL)..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                  />
                </div>

                <div className="flex flex-wrap gap-1.5">
                  <Button
                    size="small"
                    variant={selectedCategory === "all" ? "primary" : "secondary"}
                    onClick={() => setSelectedCategory("all")}
                  >
                    All Topics
                  </Button>
                  <Button
                    size="small"
                    variant={selectedCategory === "payments" ? "primary" : "secondary"}
                    onClick={() => setSelectedCategory("payments")}
                  >
                    Payments
                  </Button>
                  <Button
                    size="small"
                    variant={selectedCategory === "couriers" ? "primary" : "secondary"}
                    onClick={() => setSelectedCategory("couriers")}
                  >
                    Couriers
                  </Button>
                  <Button
                    size="small"
                    variant={selectedCategory === "domains" ? "primary" : "secondary"}
                    onClick={() => setSelectedCategory("domains")}
                  >
                    Domains &amp; SSL
                  </Button>
                  <Button
                    size="small"
                    variant={selectedCategory === "inventory" ? "primary" : "secondary"}
                    onClick={() => setSelectedCategory("inventory")}
                  >
                    Inventory
                  </Button>
                </div>
              </div>
            </Container>

            {/* Articles List / Accordion */}
            <Container className="divide-y p-0">
              <div className="flex items-center justify-between px-6 py-4">
                <Heading level="h2">Configuration &amp; Integration Guides</Heading>
                <Text size="xsmall" className="text-ui-fg-subtle">
                  Showing {filteredArticles.length} guides
                </Text>
              </div>

              <div className="divide-y divide-ui-border-base">
                {filteredArticles.map((art) => {
                  const isExpanded = expandedArticleId === art.id
                  return (
                    <div key={art.id} className="p-6 transition-colors hover:bg-ui-bg-subtle/40">
                      <div
                        className="flex items-center justify-between cursor-pointer"
                        onClick={() => setExpandedArticleId(isExpanded ? null : art.id)}
                      >
                        <div className="flex items-start gap-x-3.5">
                          <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-ui-border-base bg-ui-bg-subtle text-ui-fg-muted">
                            <DocumentText />
                          </div>
                          <div>
                            <div className="flex items-center gap-x-2">
                              <Text size="small" weight="plus" className="text-ui-fg-base">
                                {art.title}
                              </Text>
                              <Badge color="grey" size="2xsmall" className="capitalize">
                                {art.category}
                              </Badge>
                            </div>
                            <Text size="xsmall" className="text-ui-fg-subtle mt-0.5">
                              {art.summary}
                            </Text>
                          </div>
                        </div>

                        <Button variant="transparent" size="small">
                          {isExpanded ? "Collapse" : "Read guide"}
                        </Button>
                      </div>

                      {isExpanded && (
                        <div className="mt-4 pt-4 border-t border-ui-border-base text-xs text-ui-fg-subtle leading-relaxed pl-11">
                          <p>{art.content}</p>
                        </div>
                      )}
                    </div>
                  )
                })}
              </div>
            </Container>

            {/* Support Desk Banner */}
            <Container className="p-6 bg-ui-bg-subtle/50">
              <div className="flex flex-col gap-y-3 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-start gap-x-3">
                  <InformationCircleSolid className="h-5 w-5 text-ui-fg-interactive mt-0.5 shrink-0" />
                  <div>
                    <Text size="small" weight="plus" className="text-ui-fg-base">
                      Need custom engineering or operational support?
                    </Text>
                    <Text size="xsmall" className="text-ui-fg-subtle mt-0.5">
                      Our Syed Systems operations desk is available Sunday to Thursday, 09:00 to 20:00 BST.
                    </Text>
                  </div>
                </div>
                <Button
                  size="small"
                  variant="primary"
                  onClick={() => setActiveTab("contact")}
                >
                  <Envelope className="me-1.5" />
                  Contact engineering
                </Button>
              </div>
            </Container>
          </>
        )}

        {activeTab === "contact" && (
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
            {/* Primary Contact Card */}
            <Container className="p-6 flex flex-col justify-between">
              <div>
                <div className="flex items-center gap-x-2 mb-2">
                  <Envelope className="h-5 w-5 text-ui-fg-interactive" />
                  <Heading level="h2">Merchant Operations Desk</Heading>
                </div>
                <Text size="small" className="text-ui-fg-subtle mb-4">
                  For account assistance, gateway integrations, or critical operational inquiries, reach out directly to the Syed Systems engineering support team.
                </Text>

                <div className="space-y-3 border-t border-ui-border-base pt-4 text-xs">
                  <div>
                    <Text weight="plus" className="text-ui-fg-base">Support Email</Text>
                    <a
                      href="mailto:support@syedsystems.com?subject=Vendryx%20Merchant%20Support%20Inquiry"
                      className="text-ui-fg-interactive hover:underline font-mono"
                    >
                      support@syedsystems.com
                    </a>
                  </div>
                  <div>
                    <Text weight="plus" className="text-ui-fg-base">Hours of Operation</Text>
                    <Text className="text-ui-fg-subtle">Sunday – Thursday: 9:00 AM – 8:00 PM BST</Text>
                  </div>
                  <div>
                    <Text weight="plus" className="text-ui-fg-base">Priority Escalations</Text>
                    <Text className="text-ui-fg-subtle">Payment gateway &amp; checkout availability monitored 24/7</Text>
                  </div>
                </div>
              </div>

              <div className="mt-6 pt-4 border-t border-ui-border-base">
                <Button
                  size="small"
                  variant="primary"
                  className="w-full"
                  onClick={() => {
                    if (typeof window !== "undefined") {
                      window.location.href = "mailto:support@syedsystems.com?subject=Vendryx%20Merchant%20Support%20Inquiry"
                    }
                  }}
                >
                  <Envelope className="me-1.5" />
                  Send email inquiry
                </Button>
              </div>
            </Container>

            {/* Platform Health & SLA Card */}
            <Container className="p-6 flex flex-col justify-between">
              <div>
                <div className="flex items-center gap-x-2 mb-2">
                  <ShieldCheck className="h-5 w-5 text-ui-tag-green-icon" />
                  <Heading level="h2">Engine Status &amp; Standards</Heading>
                </div>
                <Text size="small" className="text-ui-fg-subtle mb-4">
                  Vendryx operates on the high-performance Stheno commerce kernel, engineered for zero lock contention and strict transactional safety.
                </Text>

                <div className="space-y-3 border-t border-ui-border-base pt-4 text-xs">
                  <div className="flex items-center justify-between">
                    <Text className="text-ui-fg-base font-medium">Core API Engine</Text>
                    <Badge color="blue" size="2xsmall">Active</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <Text className="text-ui-fg-base font-medium">Payment Idempotency</Text>
                    <Badge color="green" size="2xsmall">Enforced</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <Text className="text-ui-fg-base font-medium">Tenant Data Isolation</Text>
                    <Badge color="green" size="2xsmall">Active</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <Text className="text-ui-fg-base font-medium">TLS / SSL Automation</Text>
                    <Badge color="green" size="2xsmall">Managed</Badge>
                  </div>
                </div>
              </div>

              <div className="mt-6 pt-4 border-t border-ui-border-base">
                <Text size="xsmall" className="text-ui-fg-muted italic text-center">
                  Engine Version: 2.19.0 · Syed Systems Commerce Platform
                </Text>
              </div>
            </Container>
          </div>
        )}
      </div>
    </SingleColumnPage>
  )
}

export const HelpCenterView = HelpCenter

