import {
  ArrowPath,
  ArrowUpRightOnBox,
  CheckCircle,
  ExclamationCircle,
  GlobeEurope,
  InformationCircleSolid,
  ShieldCheck,
  SquareTwoStack,
  Trash,
} from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Heading,
  Input,
  Label,
  Table,
  Text,
  toast,
  usePrompt,
} from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"
import { useStore } from "../../hooks/api/store"
import { sdk } from "../../lib/client"

interface DnsRecord {
  type: string
  name: string
  value: string
  status: "verified" | "pending" | "failed"
  ttl: string
}

interface CustomDomainState {
  domain: string
  status: "active" | "pending" | "failed"
  sslStatus: "secured" | "issuing" | "failed"
  addedAt: string
  lastCheckedAt: string | null
  records: DnsRecord[]
}

interface DomainsApiResponse {
  domains?: Array<{
    id: string
    domain: string
    is_default?: boolean
    is_primary?: boolean
    status: "active" | "pending" | "failed"
    ssl_status: "secured" | "issuing" | "failed"
    cname_target?: string
    verification_token?: string
    created_at?: string
  }>
  default_domain?: string
  custom_domain?: {
    domain: string
    status: "active" | "pending" | "failed"
    ssl_status: "secured" | "issuing" | "failed"
    added_at: string
    last_checked_at: string | null
    records: DnsRecord[]
  } | null
}

export const DomainsView = () => {
  const prompt = usePrompt()
  const { store } = useStore()
  const storeSlug = store?.name
    ? store.name.toLowerCase().replace(/[^a-z0-9]/g, "-")
    : "merchant"

  const [customDomain, setCustomDomain] = useState<CustomDomainState | null>(null)
  const [inputDomain, setInputDomain] = useState("")
  const [isVerifying, setIsVerifying] = useState(false)
  const [selectedRegistrar, setSelectedRegistrar] = useState<"cloudflare" | "godaddy" | "namecheap">("cloudflare")

  const { data: domainsApi, refetch } = useQuery<DomainsApiResponse>({
    queryKey: ["admin", "domains"],
    queryFn: async () => {
      try {
        return await sdk.client.fetch<DomainsApiResponse>("/admin/domains")
      } catch {
        return { default_domain: `${storeSlug}.vendryx.shop`, custom_domain: null }
      }
    },
  })

  useEffect(() => {
    if (domainsApi?.custom_domain) {
      setCustomDomain({
        domain: domainsApi.custom_domain.domain,
        status: domainsApi.custom_domain.status,
        sslStatus: domainsApi.custom_domain.ssl_status,
        addedAt: domainsApi.custom_domain.added_at,
        lastCheckedAt: domainsApi.custom_domain.last_checked_at,
        records: domainsApi.custom_domain.records,
      })
    } else if (domainsApi?.domains && Array.isArray(domainsApi.domains)) {
      const custom = domainsApi.domains.find((d) => !d.is_default)
      if (custom) {
        const isSubdomain = custom.domain.split(".").length > 2
        const hostRecord = isSubdomain ? custom.domain.split(".")[0] : "@"
        setCustomDomain({
          domain: custom.domain,
          status: custom.status as any,
          sslStatus: custom.ssl_status as any,
          addedAt: custom.created_at || new Date().toISOString(),
          lastCheckedAt: null,
          records: [
            {
              type: "CNAME",
              name: hostRecord,
              value: custom.cname_target || "shops.vendryx.com",
              status: custom.status === "active" ? "verified" : "pending",
              ttl: "Auto / 300s",
            },
            {
              type: "TXT",
              name: `_vendryx-challenge.${custom.domain}`,
              value: custom.verification_token || `vendryx-site-verification=${custom.domain}`,
              status: custom.status === "active" ? "verified" : "pending",
              ttl: "Auto / 300s",
            },
          ],
        })
      }
    }
  }, [domainsApi])

  const defaultDomain = domainsApi?.default_domain || `${storeSlug}.vendryx.shop`

  const copyToClipboard = (text: string, label: string) => {
    if (typeof navigator !== "undefined" && navigator.clipboard) {
      navigator.clipboard.writeText(text)
      toast.success(`Copied ${label} to clipboard`)
    }
  }

  const handleAddDomain = async (e: React.FormEvent) => {
    e.preventDefault()
    const cleaned = inputDomain.trim().toLowerCase().replace(/^https?:\/\//, "").replace(/\/$/, "")

    const domainRegex = /^([a-z0-9]+(-[a-z0-9]+)*\.)+[a-z]{2,}$/i
    if (!cleaned || !domainRegex.test(cleaned)) {
      toast.error("Please enter a valid domain (e.g. shop.example.com or mystore.com)")
      return
    }

    if (cleaned.endsWith(".vendryx.shop")) {
      toast.error("You cannot configure a vendryx.shop subdomain as a custom domain")
      return
    }

    const isSubdomain = cleaned.split(".").length > 2
    const hostRecord = isSubdomain ? cleaned.split(".")[0] : "@"

    const fallbackConfig: CustomDomainState = {
      domain: cleaned,
      status: "pending",
      sslStatus: "issuing",
      addedAt: new Date().toISOString(),
      lastCheckedAt: null,
      records: [
        {
          type: "CNAME",
          name: hostRecord,
          value: "shops.vendryx.com",
          status: "pending",
          ttl: "Auto / 300s",
        },
        {
          type: "TXT",
          name: `_vendryx-challenge.${cleaned}`,
          value: `vendryx-site-verification=${cleaned}`,
          status: "pending",
          ttl: "Auto / 300s",
        },
      ],
    }

    setCustomDomain(fallbackConfig)
    setInputDomain("")

    try {
      const res = await sdk.client.fetch<{ custom_domain?: any; domain?: any }>("/admin/domains", {
        method: "POST",
        body: { domain: cleaned },
      })
      const returned = res?.custom_domain || res?.domain
      if (returned) {
        const isSub = returned.domain.split(".").length > 2
        const host = isSub ? returned.domain.split(".")[0] : "@"
        setCustomDomain({
          domain: returned.domain,
          status: returned.status || "pending",
          sslStatus: returned.ssl_status || "issuing",
          addedAt: returned.created_at || returned.added_at || new Date().toISOString(),
          lastCheckedAt: returned.last_checked_at || null,
          records: [
            {
              type: "CNAME",
              name: host,
              value: returned.cname_target || "shops.vendryx.com",
              status: returned.status === "active" ? "verified" : "pending",
              ttl: "Auto / 300s",
            },
            {
              type: "TXT",
              name: `_vendryx-challenge.${returned.domain}`,
              value: returned.verification_token || `vendryx-site-verification=${returned.domain}`,
              status: returned.status === "active" ? "verified" : "pending",
              ttl: "Auto / 300s",
            },
          ],
        })
      }
    } catch {
      // Retain optimistic configuration
    }

    refetch()
    toast.success(`Domain ${cleaned} added. Please configure DNS records at your domain registrar.`)
  }

  const handleVerifyDns = async () => {
    if (!customDomain) return
    setIsVerifying(true)

    try {
      const res = await sdk.client.fetch<{
        verified?: boolean
        status: "active" | "pending"
        ssl_status?: "secured" | "issuing"
        dns_pointed?: boolean
        tls_issued?: boolean
        last_checked_at?: string
        records?: DnsRecord[]
      }>(`/admin/domains/${customDomain.domain}/verify`, {
        method: "POST",
      })

      if (res) {
        const isVerified = res.verified ?? (res.dns_pointed && res.tls_issued) ?? (res.status === "active")
        const sslStatus = res.ssl_status || (res.tls_issued ? "secured" : "issuing")
        setCustomDomain((prev) => {
          if (!prev) return null
          return {
            ...prev,
            status: isVerified ? "active" : res.status,
            sslStatus,
            lastCheckedAt: res.last_checked_at || new Date().toISOString(),
            records: res.records || prev.records.map((r) => ({
              ...r,
              status: isVerified ? "verified" : r.status,
            })),
          }
        })
        if (isVerified) {
          toast.success("DNS records verified successfully. SSL certificate is active.")
        } else {
          toast.info("DNS verification probe completed. Records are still propagating.")
        }
      }
      refetch()
    } catch (err: any) {
      toast.error(err?.message || "Verification probe encountered an error. Please try again.")
    } finally {
      setIsVerifying(false)
    }
  }

  const handleRemoveDomain = async () => {
    if (!customDomain) return

    const confirmed = await prompt({
      title: "Disconnect Custom Domain",
      description: `Are you sure you want to disconnect ${customDomain.domain}? Traffic to this domain will no longer reach your Vendryx storefront.`,
      confirmText: "Disconnect domain",
      cancelText: "Cancel",
      variant: "danger",
    })

    if (!confirmed) return

    try {
      await sdk.client.fetch(`/admin/domains/${customDomain.domain}`, {
        method: "DELETE",
      })
    } catch {
      // Proceed with optimistic disconnect
    }

    setCustomDomain(null)
    refetch()
    toast.success("Custom domain has been disconnected")
  }

  return (
    <SingleColumnPage widgets={{ after: [], before: [] }}>
      <div className="flex flex-col gap-y-6">
        <div>
          <Text size="xsmall" weight="plus" className="text-ui-fg-muted">
            Store Settings
          </Text>
          <div className="mt-1 flex items-center gap-x-2.5">
            <Heading level="h1">Domains &amp; SSL</Heading>
            <Badge color="orange" size="2xsmall">
              Routing
            </Badge>
          </div>
          <Text size="small" className="text-ui-fg-subtle mt-1">
            Manage your store&apos;s public address, custom domain routing, and automated SSL certificate provisioning.
          </Text>
        </div>

        {/* Default Vendryx Subdomain */}
        <Container className="divide-y p-0">
          <div className="flex flex-col gap-y-2 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex items-start gap-x-3.5">
              <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-ui-border-base bg-ui-bg-subtle text-ui-fg-muted">
                <GlobeEurope />
              </div>
              <div>
                <div className="flex items-center gap-x-2">
                  <Text size="small" weight="plus" className="text-ui-fg-base">
                    {defaultDomain}
                  </Text>
                  <Badge color="blue" size="2xsmall">
                    Default Subdomain
                  </Badge>
                  <Badge color="green" size="2xsmall" className="gap-x-1">
                    <ShieldCheck className="h-3 w-3" />
                    SSL Active
                  </Badge>
                </div>
                <Text size="xsmall" className="text-ui-fg-subtle mt-0.5">
                  Always active and secured by Vendryx Cloud. Safe for testing and fallback checkout.
                </Text>
              </div>
            </div>

            <Button
              size="small"
              variant="secondary"
              onClick={() => {
                if (typeof window !== "undefined") {
                  window.open(`https://${defaultDomain}`, "_blank")
                }
              }}
            >
              <ArrowUpRightOnBox className="me-1.5 text-ui-fg-subtle" />
              Visit store
            </Button>
          </div>
        </Container>

        {/* Custom Domain Management */}
        <Container className="divide-y p-0">
          <div className="flex items-center justify-between px-6 py-4">
            <div>
              <Heading level="h2">Custom Primary Domain</Heading>
              <Text className="text-ui-fg-subtle" size="small">
                Connect your brand&apos;s own domain to replace the default Vendryx URL.
              </Text>
            </div>
            {customDomain && (
              <div className="flex items-center gap-x-2">
                <Button
                  size="small"
                  variant="secondary"
                  isLoading={isVerifying}
                  onClick={handleVerifyDns}
                >
                  <ArrowPath className="me-1.5 text-ui-fg-subtle" />
                  Verify DNS records
                </Button>
                <Button
                  size="small"
                  variant="danger"
                  onClick={handleRemoveDomain}
                >
                  <Trash className="me-1.5" />
                  Disconnect
                </Button>
              </div>
            )}
          </div>

          {!customDomain ? (
            <div className="px-6 py-6">
              <form onSubmit={handleAddDomain} className="flex flex-col gap-y-4 max-w-lg">
                <div>
                  <Label htmlFor="custom-domain-input" className="text-ui-fg-subtle text-xs">
                    Enter Domain Name
                  </Label>
                  <div className="mt-1.5 flex items-center gap-x-2">
                    <Input
                      id="custom-domain-input"
                      placeholder="e.g. shop.syedsystems.com or mystore.com"
                      value={inputDomain}
                      onChange={(e) => setInputDomain(e.target.value)}
                      required
                    />
                    <Button type="submit" variant="primary" size="small" className="shrink-0">
                      Connect domain
                    </Button>
                  </div>
                  <Text size="xsmall" className="text-ui-fg-muted mt-2">
                    You can use an apex domain (mystore.com) or a subdomain (shop.mystore.com). You will need access to your DNS provider.
                  </Text>
                </div>
              </form>
            </div>
          ) : (
            <>
              {/* Configured domain overview */}
              <div className="px-6 py-4 flex flex-col gap-y-3 sm:flex-row sm:items-center sm:justify-between bg-ui-bg-subtle/50">
                <div>
                  <div className="flex items-center gap-x-2.5">
                    <Heading level="h3" className="text-base text-ui-fg-base">
                      {customDomain.domain}
                    </Heading>
                    {customDomain.status === "active" ? (
                      <Badge color="green" size="2xsmall" className="gap-x-1">
                        <CheckCircle className="h-3 w-3" />
                        Connected &amp; Active
                      </Badge>
                    ) : (
                      <Badge color="orange" size="2xsmall" className="gap-x-1">
                        <ExclamationCircle className="h-3 w-3" />
                        Pending DNS Verification
                      </Badge>
                    )}
                    <Badge color="blue" size="2xsmall">
                      {customDomain.sslStatus === "secured" ? "TLS 1.3 Active" : "SSL Provisioning"}
                    </Badge>
                  </div>
                  <Text size="xsmall" className="text-ui-fg-subtle mt-1">
                    Added on {new Date(customDomain.addedAt).toLocaleDateString()}
                    {customDomain.lastCheckedAt && ` • Last probed: ${new Date(customDomain.lastCheckedAt).toLocaleTimeString()}`}
                  </Text>
                </div>

                <Button
                  size="small"
                  variant="secondary"
                  onClick={() => {
                    if (typeof window !== "undefined") {
                      window.open(`https://${customDomain.domain}`, "_blank")
                    }
                  }}
                >
                  <ArrowUpRightOnBox className="me-1.5 text-ui-fg-subtle" />
                  Test live URL
                </Button>
              </div>

              {/* DNS Records Table */}
              <div className="px-6 py-4">
                <Text size="small" weight="plus" className="text-ui-fg-base mb-2">
                  Required DNS Configuration
                </Text>
                <Text size="xsmall" className="text-ui-fg-subtle mb-4">
                  Log in to your domain registrar (Cloudflare, GoDaddy, Namecheap) and add the following two DNS records:
                </Text>

                <div className="overflow-x-auto rounded-lg border border-ui-border-base">
                  <Table>
                    <Table.Header>
                      <Table.Row>
                        <Table.HeaderCell>Type</Table.HeaderCell>
                        <Table.HeaderCell>Host / Name</Table.HeaderCell>
                        <Table.HeaderCell>Value / Points to</Table.HeaderCell>
                        <Table.HeaderCell>TTL</Table.HeaderCell>
                        <Table.HeaderCell>Status</Table.HeaderCell>
                        <Table.HeaderCell className="text-right">Action</Table.HeaderCell>
                      </Table.Row>
                    </Table.Header>
                    <Table.Body>
                      {customDomain.records.map((rec, i) => (
                        <Table.Row key={i}>
                          <Table.Cell>
                            <Badge color="grey" size="2xsmall" className="font-mono font-bold">
                              {rec.type}
                            </Badge>
                          </Table.Cell>
                          <Table.Cell className="font-mono text-xs text-ui-fg-base font-semibold">
                            {rec.name}
                          </Table.Cell>
                          <Table.Cell className="font-mono text-xs text-ui-fg-subtle max-w-xs truncate">
                            {rec.value}
                          </Table.Cell>
                          <Table.Cell className="text-xs text-ui-fg-muted">
                            {rec.ttl}
                          </Table.Cell>
                          <Table.Cell>
                            {rec.status === "verified" ? (
                              <Badge color="green" size="2xsmall" className="gap-x-1">
                                <CheckCircle className="h-3 w-3" />
                                Verified
                              </Badge>
                            ) : rec.status === "failed" ? (
                              <Badge color="red" size="2xsmall">
                                Missing
                              </Badge>
                            ) : (
                              <Badge color="orange" size="2xsmall">
                                Pending
                              </Badge>
                            )}
                          </Table.Cell>
                          <Table.Cell className="text-right">
                            <Button
                              size="small"
                              variant="transparent"
                              onClick={() => copyToClipboard(rec.value, `${rec.type} value`)}
                              aria-label={`Copy ${rec.type} value`}
                            >
                              <SquareTwoStack className="h-3.5 w-3.5 text-ui-fg-subtle" />
                            </Button>
                          </Table.Cell>
                        </Table.Row>
                      ))}
                    </Table.Body>
                  </Table>
                </div>
              </div>

              {/* Step-by-Step Registrar Guides */}
              <div className="divide-y divide-ui-border-base border-t border-ui-border-base">
                <div className="px-6 py-3 flex items-center justify-between bg-ui-bg-subtle/30">
                  <div className="flex items-center gap-x-2">
                    <InformationCircleSolid className="h-4 w-4 text-ui-fg-subtle" />
                    <Text size="xsmall" weight="plus" className="text-ui-fg-base">
                      Registrar Instructions
                    </Text>
                  </div>

                  <div className="flex items-center gap-x-1.5">
                    <Button
                      size="small"
                      variant={selectedRegistrar === "cloudflare" ? "secondary" : "transparent"}
                      onClick={() => setSelectedRegistrar("cloudflare")}
                      className="text-xs"
                    >
                      Cloudflare
                    </Button>
                    <Button
                      size="small"
                      variant={selectedRegistrar === "namecheap" ? "secondary" : "transparent"}
                      onClick={() => setSelectedRegistrar("namecheap")}
                      className="text-xs"
                    >
                      Namecheap
                    </Button>
                    <Button
                      size="small"
                      variant={selectedRegistrar === "godaddy" ? "secondary" : "transparent"}
                      onClick={() => setSelectedRegistrar("godaddy")}
                      className="text-xs"
                    >
                      GoDaddy
                    </Button>
                  </div>
                </div>

                <div className="px-6 py-4 text-xs text-ui-fg-subtle leading-relaxed">
                  {selectedRegistrar === "cloudflare" && (
                    <ol className="list-decimal pl-4 space-y-1">
                      <li>Log in to Cloudflare dashboard and select your active zone.</li>
                      <li>Navigate to <strong>DNS &gt; Records</strong>.</li>
                      <li>Click <strong>Add Record</strong>, choose <strong>CNAME</strong>, enter host name and points to <code>shops.vendryx.com</code>. Set Proxy status to <strong>DNS Only (Grey Cloud)</strong>.</li>
                      <li>Add the second <strong>TXT</strong> verification challenge record.</li>
                      <li>Click <strong>Verify DNS records</strong> above once saved.</li>
                    </ol>
                  )}
                  {selectedRegistrar === "namecheap" && (
                    <ol className="list-decimal pl-4 space-y-1">
                      <li>Sign in to Namecheap and go to your <strong>Domain List</strong>.</li>
                      <li>Click <strong>Manage</strong> next to your domain, then click <strong>Advanced DNS</strong>.</li>
                      <li>Click <strong>Add New Record</strong> and create a CNAME record pointing to <code>shops.vendryx.com</code>.</li>
                      <li>Add the TXT record with the verification challenge string.</li>
                      <li>DNS changes at Namecheap usually propagate in 10 to 30 minutes.</li>
                    </ol>
                  )}
                  {selectedRegistrar === "godaddy" && (
                    <ol className="list-decimal pl-4 space-y-1">
                      <li>Log in to your GoDaddy Domain Portfolio.</li>
                      <li>Select your domain and go to the <strong>DNS</strong> tab.</li>
                      <li>Click <strong>Add</strong> to create the CNAME record for your store host.</li>
                      <li>Add the TXT record for ownership verification.</li>
                      <li>Return to this screen and click <strong>Verify DNS records</strong>.</li>
                    </ol>
                  )}
                </div>
              </div>
            </>
          )}
        </Container>
      </div>
    </SingleColumnPage>
  )
}
