import { ArrowPath, CheckCircle, XCircle } from "@medusajs/icons"
import { Badge, Button, Container, Heading, StatusBadge, Table, Text } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useState } from "react"
import { SingleColumnPage } from "../../components/layout/pages"
import { sdk } from "../../lib/client"
import { PaymentProvider, ProviderCredentialModal } from "./components/provider-credential-modal"
import { ProviderRegionModal } from "./components/provider-region-modal"

interface PaymentProvidersResponse {
  payment_providers: PaymentProvider[]
  count: number
}

const spanAllColumns = { colSpan: 6 } as React.TdHTMLAttributes<HTMLTableCellElement>

export const PaymentProvidersList = () => {
  const [selectedForCredentials, setSelectedForCredentials] = useState<PaymentProvider | null>(null)
  const [selectedForRegions, setSelectedForRegions] = useState<PaymentProvider | null>(null)

  const { data, isLoading, refetch, isFetching } = useQuery<PaymentProvidersResponse>({
    queryKey: ["admin", "payment-providers"],
    queryFn: () => sdk.client.fetch<PaymentProvidersResponse>("/admin/payment-providers"),
  })

  const providers = data?.payment_providers || []

  const renderHealthBadge = (provider: PaymentProvider) => {
    if (!provider.configured) {
      return (
        <StatusBadge color="grey">
          Unconfigured
        </StatusBadge>
      )
    }

    switch (provider.credential_health) {
      case "healthy":
        return (
          <StatusBadge color="green">
            Healthy
          </StatusBadge>
        )
      case "degraded":
        return (
          <StatusBadge color="orange">
            Degraded
          </StatusBadge>
        )
      case "failed":
        return (
          <StatusBadge color="red">
            Failed
          </StatusBadge>
        )
      default:
        return (
          <StatusBadge color="grey">
            {provider.credential_health}
          </StatusBadge>
        )
    }
  }

  return (
    <SingleColumnPage
      widgets={{
        after: [],
        before: [],
      }}
    >
      <Container className="p-0">
        <div className="flex items-center justify-between border-b p-6">
          <div className="flex flex-col gap-y-1">
            <div className="flex items-center gap-x-2">
              <Heading level="h1">Payment Providers</Heading>
              <Badge size="small" color="blue">
                {providers.length} registered
              </Badge>
            </div>
            <Text size="small" className="text-ui-fg-subtle">
              Configure merchant gateway credentials and manage region enablement for store checkouts.
            </Text>
          </div>
          <Button
            size="small"
            variant="secondary"
            onClick={() => refetch()}
            isLoading={isFetching}
          >
            <ArrowPath className="mr-1" />
            Refresh
          </Button>
        </div>

        <div className="overflow-x-auto">
          <Table>
            <Table.Header>
              <Table.Row>
                <Table.HeaderCell>Provider</Table.HeaderCell>
                <Table.HeaderCell>Identifier</Table.HeaderCell>
                <Table.HeaderCell>Configuration</Table.HeaderCell>
                <Table.HeaderCell>Credential Health</Table.HeaderCell>
                <Table.HeaderCell>Active Regions</Table.HeaderCell>
                <Table.HeaderCell className="text-right">Actions</Table.HeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {isLoading ? (
                <Table.Row>
                  <Table.Cell {...spanAllColumns} className="text-ui-fg-subtle py-8 text-center">
                    Loading payment providers...
                  </Table.Cell>
                </Table.Row>
              ) : providers.length === 0 ? (
                <Table.Row>
                  <Table.Cell {...spanAllColumns} className="text-ui-fg-subtle py-8 text-center">
                    No payment providers registered in server factory registry.
                  </Table.Cell>
                </Table.Row>
              ) : (
                providers.map((p) => {
                  const isConfigured = p.configured
                  const regionCount = p.enabled_regions?.length || 0

                  return (
                    <Table.Row key={p.id}>
                      <Table.Cell>
                        <div className="flex flex-col">
                          <Text size="small" weight="plus">
                            {p.name}
                          </Text>
                          <Text size="xsmall" className="text-ui-fg-subtle">
                            {p.credential_schema.length} required field(s)
                          </Text>
                        </div>
                      </Table.Cell>

                      <Table.Cell>
                        <code className="text-ui-fg-subtle bg-ui-bg-subtle rounded px-1.5 py-0.5 text-xs">
                          {p.id}
                        </code>
                      </Table.Cell>

                      <Table.Cell>
                        {isConfigured ? (
                          <div className="flex items-center gap-x-1.5 text-xs text-ui-fg-interactive">
                            <CheckCircle className="text-ui-fg-interactive" />
                            <span>Configured</span>
                          </div>
                        ) : (
                          <div className="flex items-center gap-x-1.5 text-xs text-ui-fg-subtle">
                            <XCircle className="text-ui-fg-subtle" />
                            <span>Not configured</span>
                          </div>
                        )}
                      </Table.Cell>

                      <Table.Cell>{renderHealthBadge(p)}</Table.Cell>

                      <Table.Cell>
                        {regionCount > 0 ? (
                          <Badge size="small" color="green">
                            {regionCount} region{regionCount > 1 ? "s" : ""}
                          </Badge>
                        ) : (
                          <Badge size="small" color="grey">
                            Disabled in all
                          </Badge>
                        )}
                      </Table.Cell>

                      <Table.Cell className="text-right">
                        <div className="flex items-center justify-end gap-x-2">
                          <Button
                            size="small"
                            variant="secondary"
                            onClick={() => setSelectedForRegions(p)}
                          >
                            Regions
                          </Button>
                          <Button
                            size="small"
                            variant={isConfigured ? "secondary" : "primary"}
                            onClick={() => setSelectedForCredentials(p)}
                          >
                            {isConfigured ? "Update Keys" : "Setup Keys"}
                          </Button>
                        </div>
                      </Table.Cell>
                    </Table.Row>
                  )
                })
              )}
            </Table.Body>
          </Table>
        </div>
      </Container>

      {/* Credential Modal */}
      <ProviderCredentialModal
        provider={selectedForCredentials}
        open={!!selectedForCredentials}
        onClose={() => setSelectedForCredentials(null)}
        onSuccess={() => refetch()}
      />

      {/* Region Assignment Modal */}
      <ProviderRegionModal
        provider={selectedForRegions}
        open={!!selectedForRegions}
        onClose={() => setSelectedForRegions(null)}
        onSuccess={() => refetch()}
      />
    </SingleColumnPage>
  )
}
