import { ArrowPath } from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Heading,
  Input,
  Label,
  Select,
  StatusBadge,
  Table,
  Text,
  toast,
} from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useMemo, useState } from "react"

import {
  creditInvoice,
  listBillingAttempts,
  listInvoices,
  listTenants,
  retryInvoice,
  type BillingAttempt,
  type Invoice,
  type MoneyAmount,
} from "./api"

const formatMoney = (money: MoneyAmount) =>
  new Intl.NumberFormat("en-BD", {
    style: "currency",
    currency: money.currency.toUpperCase(),
    maximumFractionDigits: 2,
  }).format(Number(money.amount))

const formatDate = (iso: string) => new Date(iso).toLocaleDateString()

const badgeColor = (status: string) => {
  if (status === "paid" || status === "succeeded" || status === "active") return "green"
  if (status === "unknown" || status === "past_due" || status === "grace") return "orange"
  if (status === "failed" || status === "suspended" || status === "uncollectible") return "red"
  return "grey"
}

function BillingSummary({
  invoices,
  attempts,
  dunningCount,
}: {
  invoices: Invoice[]
  attempts: BillingAttempt[]
  dunningCount: number
}) {
  const open = invoices.filter((invoice) => invoice.status === "open").length
  const paidTotal = invoices
    .filter((invoice) => invoice.status === "paid")
    .reduce((sum, invoice) => sum + Number(invoice.total.amount), 0)
  const unknown = attempts.filter((attempt) => attempt.state === "unknown").length

  return (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
      <Container className="p-4">
        <Text size="small" weight="plus" className="text-ui-fg-subtle">
          Open invoices
        </Text>
        <Text size="xlarge" weight="plus" className="mt-2 tabular-nums">
          {open}
        </Text>
      </Container>
      <Container className="p-4">
        <Text size="small" weight="plus" className="text-ui-fg-subtle">
          Paid total
        </Text>
        <Text size="xlarge" weight="plus" className="mt-2 tabular-nums">
          {new Intl.NumberFormat("en-BD", {
            style: "currency",
            currency: "BDT",
            maximumFractionDigits: 0,
          }).format(paidTotal)}
        </Text>
      </Container>
      <Container className="p-4">
        <Text size="small" weight="plus" className="text-ui-fg-subtle">
          Dunning
        </Text>
        <Text size="xlarge" weight="plus" className="mt-2 tabular-nums">
          {dunningCount}
        </Text>
      </Container>
      <Container className="p-4">
        <Text size="small" weight="plus" className="text-ui-fg-subtle">
          Unknown attempts
        </Text>
        <Text size="xlarge" weight="plus" className="mt-2 tabular-nums">
          {unknown}
        </Text>
      </Container>
    </div>
  )
}

function CreditPanel({ invoice }: { invoice: Invoice | null }) {
  const queryClient = useQueryClient()
  const [amount, setAmount] = useState("")
  const [reason, setReason] = useState("")

  const mutation = useMutation({
    mutationFn: () => {
      if (!invoice) throw new Error("Select an invoice")
      const minor = Math.round(Number(amount) * 100)
      return creditInvoice(invoice.id, {
        credit_minor: minor,
        discount_minor: 0,
        coupon_minor: 0,
        currency: invoice.total.currency,
        reason: reason.trim(),
      })
    },
    onSuccess: () => {
      toast.success("Credit applied")
      setAmount("")
      setReason("")
      queryClient.invalidateQueries({ queryKey: ["platform"] })
    },
    onError: (error: Error) => toast.error(error.message),
  })

  return (
    <Container className="p-4">
      <div className="mb-3 flex items-start justify-between gap-x-3">
        <div>
          <Heading level="h3">Manual credit</Heading>
          {invoice ? (
            <Text size="xsmall" className="text-ui-fg-muted font-mono">
              {invoice.id}
            </Text>
          ) : null}
        </div>
        {invoice ? <Badge size="xsmall">{invoice.status}</Badge> : null}
      </div>
      <div className="grid grid-cols-1 gap-3 md:grid-cols-[120px_1fr_auto]">
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Amount</Label>
          <Input
            type="number"
            min={0}
            step="0.01"
            value={amount}
            disabled={!invoice}
            onChange={(event) => setAmount(event.target.value)}
          />
        </div>
        <div className="flex flex-col gap-y-1.5">
          <Label size="xsmall">Reason</Label>
          <Input
            value={reason}
            disabled={!invoice}
            onChange={(event) => setReason(event.target.value)}
          />
        </div>
        <div className="flex items-end">
          <Button
            size="small"
            disabled={!invoice || !amount || !reason.trim() || Number(amount) <= 0}
            isLoading={mutation.isPending}
            onClick={() => mutation.mutate()}
          >
            Apply
          </Button>
        </div>
      </div>
    </Container>
  )
}

export function Billing() {
  const queryClient = useQueryClient()
  const [provider, setProvider] = useState("bkash")
  const [creditTarget, setCreditTarget] = useState<Invoice | null>(null)

  const invoices = useQuery({
    queryKey: ["platform", "billing", "invoices"],
    queryFn: listInvoices,
    retry: false,
  })
  const attempts = useQuery({
    queryKey: ["platform", "billing", "attempts"],
    queryFn: listBillingAttempts,
    retry: false,
  })
  const tenants = useQuery({
    queryKey: ["platform", "tenants"],
    // Wrapped, not passed bare: react-query calls queryFn with its own context
    // object, which would arrive as listTenants' `params` argument.
    queryFn: () => listTenants(),
    retry: false,
  })

  const tenantById = useMemo(
    () => new Map((tenants.data?.tenants ?? []).map((tenant) => [tenant.id, tenant])),
    [tenants.data?.tenants]
  )
  const dunningCount = (tenants.data?.tenants ?? []).filter((tenant) =>
    ["past_due", "grace", "suspended"].includes(tenant.subscription_status)
  ).length

  const retry = useMutation({
    mutationFn: (invoice: Invoice) => retryInvoice(invoice, provider),
    onSuccess: () => {
      toast.success("Retry recorded")
      queryClient.invalidateQueries({ queryKey: ["platform"] })
    },
    onError: (error: Error) => toast.error(error.message),
  })

  const invoiceRows = invoices.data?.invoices ?? []
  const attemptRows = attempts.data?.attempts ?? []

  if (invoices.isError || attempts.isError || tenants.isError) {
    return (
      <Container className="p-6">
        <Heading level="h2" className="mb-2">
          Could not read billing
        </Heading>
        <Text size="small" className="text-ui-fg-subtle">
          {((invoices.error || attempts.error || tenants.error) as Error).message}
        </Text>
      </Container>
    )
  }

  return (
    <div className="flex flex-col gap-y-5">
      <div className="flex items-start justify-between gap-x-4">
        <div>
          <Heading level="h1">Billing</Heading>
          <Text size="small" className="text-ui-fg-subtle">
            Invoices, collection attempts, and dunning state.
          </Text>
        </div>
        <div className="flex items-center gap-x-2">
          <Select value={provider} onValueChange={setProvider} size="small">
            <Select.Trigger className="w-[150px]">
              <Select.Value />
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="bkash">bKash</Select.Item>
              <Select.Item value="sslcommerz">SSLCommerz</Select.Item>
            </Select.Content>
          </Select>
          <Button
            variant="secondary"
            size="small"
            aria-label="Refresh"
            isLoading={invoices.isFetching || attempts.isFetching || tenants.isFetching}
            onClick={() => queryClient.invalidateQueries({ queryKey: ["platform"] })}
          >
            <ArrowPath />
          </Button>
        </div>
      </div>

      <BillingSummary
        invoices={invoiceRows}
        attempts={attemptRows}
        dunningCount={dunningCount}
      />
      <CreditPanel invoice={creditTarget} />

      <Container className="overflow-hidden p-0">
        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Invoice</Table.HeaderCell>
              <Table.HeaderCell>Merchant</Table.HeaderCell>
              <Table.HeaderCell>Period</Table.HeaderCell>
              <Table.HeaderCell>Status</Table.HeaderCell>
              <Table.HeaderCell>Dunning</Table.HeaderCell>
              <Table.HeaderCell className="text-right">Total</Table.HeaderCell>
              <Table.HeaderCell />
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {invoices.isLoading ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">
                  Loading invoices...
                </Table.Cell>
              </Table.Row>
            ) : invoiceRows.length === 0 ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">
                  No invoices yet.
                </Table.Cell>
              </Table.Row>
            ) : (
              invoiceRows.map((invoice) => {
                const tenant = tenantById.get(invoice.tenant_id)
                return (
                  <Table.Row key={invoice.id}>
                    <Table.Cell className="font-mono text-xs">{invoice.id}</Table.Cell>
                    <Table.Cell>
                      <div className="flex flex-col">
                        <Text size="small" weight="plus">
                          {tenant?.name ?? invoice.tenant_id}
                        </Text>
                        <Text size="xsmall" className="text-ui-fg-muted">
                          {invoice.tier}
                        </Text>
                      </div>
                    </Table.Cell>
                    <Table.Cell className="text-ui-fg-subtle text-xs tabular-nums">
                      {formatDate(invoice.period_start)} - {formatDate(invoice.period_end)}
                    </Table.Cell>
                    <Table.Cell>
                      <StatusBadge color={badgeColor(invoice.status)}>
                        {invoice.status}
                      </StatusBadge>
                    </Table.Cell>
                    <Table.Cell>
                      <StatusBadge color={badgeColor(tenant?.subscription_status ?? "missing")}>
                        {tenant?.subscription_status ?? "missing"}
                      </StatusBadge>
                    </Table.Cell>
                    <Table.Cell className="text-right tabular-nums">
                      {formatMoney(invoice.total)}
                    </Table.Cell>
                    <Table.Cell>
                      <div className="flex justify-end gap-x-2">
                        <Button
                          variant="secondary"
                          size="small"
                          isLoading={retry.isPending}
                          onClick={() => retry.mutate(invoice)}
                        >
                          Retry
                        </Button>
                        <Button
                          variant="secondary"
                          size="small"
                          onClick={() => setCreditTarget(invoice)}
                        >
                          Credit
                        </Button>
                      </div>
                    </Table.Cell>
                  </Table.Row>
                )
              })
            )}
          </Table.Body>
        </Table>
      </Container>

      <Container className="overflow-hidden p-0">
        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Attempt</Table.HeaderCell>
              <Table.HeaderCell>Invoice</Table.HeaderCell>
              <Table.HeaderCell>Provider</Table.HeaderCell>
              <Table.HeaderCell>State</Table.HeaderCell>
              <Table.HeaderCell className="text-right">Amount</Table.HeaderCell>
              <Table.HeaderCell>Created</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {attempts.isLoading ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">
                  Loading attempts...
                </Table.Cell>
              </Table.Row>
            ) : attemptRows.length === 0 ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">
                  No attempts yet.
                </Table.Cell>
              </Table.Row>
            ) : (
              attemptRows.map((attempt) => (
                <Table.Row key={attempt.id}>
                  <Table.Cell className="font-mono text-xs">{attempt.id}</Table.Cell>
                  <Table.Cell className="font-mono text-xs">{attempt.invoice_id}</Table.Cell>
                  <Table.Cell>{attempt.provider_id}</Table.Cell>
                  <Table.Cell>
                    <StatusBadge color={badgeColor(attempt.state)}>
                      {attempt.state}
                    </StatusBadge>
                  </Table.Cell>
                  <Table.Cell className="text-right tabular-nums">
                    {formatMoney(attempt.amount)}
                  </Table.Cell>
                  <Table.Cell className="text-ui-fg-subtle text-xs tabular-nums">
                    {formatDate(attempt.created_at)}
                  </Table.Cell>
                </Table.Row>
              ))
            )}
          </Table.Body>
        </Table>
      </Container>
    </div>
  )
}
