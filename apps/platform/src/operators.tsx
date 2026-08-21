/**
 * Who holds the control plane, and how to stop one of them holding it.
 *
 * The surface that can read and rewrite every merchant's billing was gated on one shared
 * string until P29. This page is the difference between "someone changed Acme's tier" and
 * "Ayesha changed Acme's tier at 14:02".
 *
 * Two things on this page are load-bearing and easy to get wrong in a redesign:
 *
 *  1. **The issued token is rendered once.** There is no "show again" because the server
 *     cannot answer it — only the SHA-256 is stored. The panel therefore refuses to
 *     disappear on an outside click.
 *  2. **Revoking yourself is allowed, and warned about.** Forbidding it would be worse:
 *     the one credential you most need to revoke in a hurry is the one that just leaked,
 *     and it is usually the one in your own browser.
 */
import { ExclamationCircle, Plus, Trash } from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Heading,
  Input,
  Label,
  Prompt,
  StatusBadge,
  Table,
  Text,
  toast,
} from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"

import { createOperator, listOperators, revokeOperator, type Operator } from "./api"

/** `2026-08-21 14:02`, or an em dash. Local time: the operator reading it is here. */
function when(value: string | null): string {
  if (!value) return "—"
  const date = new Date(value)
  return `${date.toLocaleDateString()} ${date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  })}`
}

/**
 * The one and only rendering of a live credential.
 *
 * Deliberately a blocking panel rather than a toast: a toast that auto-dismisses would
 * throw away a secret that cannot be reissued, and the operator would not find out until
 * the next time they tried to use it.
 */
function IssuedToken({ name, token, onDone }: { name: string; token: string; onDone: () => void }) {
  const [copied, setCopied] = useState(false)

  return (
    <Container className="border-ui-border-interactive mb-4 border p-4">
      <div className="mb-2 flex items-center gap-x-2">
        <ExclamationCircle className="text-ui-fg-interactive" />
        <Heading level="h3">Token for {name}</Heading>
      </div>
      <Text size="small" className="text-ui-fg-subtle mb-3">
        This is the only time this token is shown. It is stored as a hash, so nobody — including
        this deployment — can recover it. If it is lost, issue a new one and revoke this.
      </Text>
      <div className="flex items-center gap-x-2">
        <Input readOnly value={token} className="font-mono" onFocus={(e) => e.target.select()} />
        <Button
          variant="secondary"
          onClick={() => {
            navigator.clipboard.writeText(token).then(
              () => setCopied(true),
              () => toast.error("Could not copy. Select the field and copy manually.")
            )
          }}
        >
          {copied ? "Copied" : "Copy"}
        </Button>
      </div>
      <div className="mt-3 flex justify-end">
        <Button size="small" onClick={onDone} disabled={!copied}>
          {copied ? "Done" : "Copy the token first"}
        </Button>
      </div>
    </Container>
  )
}

function IssueForm({ onIssued }: { onIssued: (name: string, token: string) => void }) {
  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const queryClient = useQueryClient()

  const mutation = useMutation({
    mutationFn: () => createOperator(name.trim()),
    onSuccess: (result) => {
      queryClient.invalidateQueries({ queryKey: ["platform", "operators"] })
      queryClient.invalidateQueries({ queryKey: ["platform", "audit"] })
      onIssued(result.operator.name, result.token)
      setName("")
      setOpen(false)
    },
    onError: (error: Error) => toast.error(error.message),
  })

  if (!open) {
    return (
      <Button variant="secondary" size="small" onClick={() => setOpen(true)}>
        <Plus /> Issue credential
      </Button>
    )
  }

  return (
    <form
      className="flex items-end gap-x-2"
      onSubmit={(event) => {
        event.preventDefault()
        mutation.mutate()
      }}
    >
      <div className="flex flex-col gap-y-1.5">
        <Label size="small" htmlFor="operator-name">
          Name
        </Label>
        <Input
          id="operator-name"
          autoFocus
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="Ayesha"
        />
      </div>
      <Button type="submit" size="small" isLoading={mutation.isPending} disabled={!name.trim()}>
        Issue
      </Button>
      <Button variant="secondary" size="small" type="button" onClick={() => setOpen(false)}>
        Cancel
      </Button>
    </form>
  )
}

function RevokeButton({ operator, isSelf }: { operator: Operator; isSelf: boolean }) {
  const queryClient = useQueryClient()

  const mutation = useMutation({
    mutationFn: () => revokeOperator(operator.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["platform", "operators"] })
      queryClient.invalidateQueries({ queryKey: ["platform", "audit"] })
      toast.success(`${operator.name} can no longer reach the control plane.`)
    },
    onError: (error: Error) => toast.error(error.message),
  })

  return (
    <Prompt>
      <Prompt.Trigger asChild>
        <Button variant="transparent" size="small" disabled={operator.revoked}>
          <Trash />
        </Button>
      </Prompt.Trigger>
      <Prompt.Content>
        <Prompt.Header>
          <Prompt.Title>Revoke {operator.name}?</Prompt.Title>
          <Prompt.Description>
            {isSelf
              ? "This is the credential this browser is using. Revoking it signs you out immediately, and you will need another credential or the bootstrap token to get back in."
              : "This credential stops working at once. The audit rows it wrote stay, and keep naming it."}
          </Prompt.Description>
        </Prompt.Header>
        <Prompt.Footer>
          <Prompt.Cancel>Cancel</Prompt.Cancel>
          <Prompt.Action onClick={() => mutation.mutate()}>Revoke</Prompt.Action>
        </Prompt.Footer>
      </Prompt.Content>
    </Prompt>
  )
}

export function Operators() {
  const [issued, setIssued] = useState<{ name: string; token: string } | null>(null)

  const { data, isLoading } = useQuery({
    queryKey: ["platform", "operators"],
    queryFn: listOperators,
    retry: false,
  })

  const operators = data?.operators ?? []
  const live = operators.filter((o) => !o.revoked)

  return (
    <div className="flex flex-col gap-y-4">
      {issued && (
        <IssuedToken name={issued.name} token={issued.token} onDone={() => setIssued(null)} />
      )}

      {/* The bootstrap token identifies a secret, not a person. An operator holding it
          should know that every row they write says `op_bootstrap`. */}
      {data?.caller.bootstrap && (
        <Container className="border-ui-border-error border p-4">
          <Heading level="h3" className="mb-1">
            You are using the bootstrap token
          </Heading>
          <Text size="small" className="text-ui-fg-subtle">
            Everything you change is recorded against <code>op_bootstrap</code>, which names a
            shared secret rather than a person. Issue yourself a named credential and use that
            instead. The bootstrap token stays valid as a way back in if every named credential
            is revoked.
          </Text>
        </Container>
      )}

      <Container className="p-0">
        <div className="flex items-center justify-between px-6 py-4">
          <div>
            <Heading level="h2">Operators</Heading>
            <Text size="small" className="text-ui-fg-subtle">
              {live.length} active
              {operators.length > live.length && `, ${operators.length - live.length} revoked`}
              {data && ` · signed in as ${data.caller.name}`}
            </Text>
          </div>
          <IssueForm onIssued={(name, token) => setIssued({ name, token })} />
        </div>

        <Table>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell>Name</Table.HeaderCell>
              <Table.HeaderCell>Token</Table.HeaderCell>
              <Table.HeaderCell>Issued</Table.HeaderCell>
              <Table.HeaderCell>Last used</Table.HeaderCell>
              <Table.HeaderCell>Status</Table.HeaderCell>
              <Table.HeaderCell />
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {isLoading ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">Loading…</Table.Cell>
              </Table.Row>
            ) : operators.length === 0 ? (
              <Table.Row>
                <Table.Cell className="text-ui-fg-muted py-8 text-center">
                  No named credentials yet. Everything is running on the bootstrap token.
                </Table.Cell>
              </Table.Row>
            ) : (
              operators.map((operator) => {
                const isSelf = operator.id === data?.caller.id
                return (
                  <Table.Row key={operator.id}>
                    <Table.Cell>
                      {operator.name}
                      {isSelf && (
                        <Badge size="2xsmall" className="ml-2">
                          you
                        </Badge>
                      )}
                    </Table.Cell>
                    <Table.Cell className="font-mono text-ui-fg-subtle">
                      {operator.redacted_token}
                    </Table.Cell>
                    <Table.Cell>{when(operator.created_at)}</Table.Cell>
                    <Table.Cell>
                      {/* A credential that has never been used is a credential that can
                          be revoked without asking anyone. */}
                      {operator.last_used_at ? when(operator.last_used_at) : "never used"}
                    </Table.Cell>
                    <Table.Cell>
                      <StatusBadge color={operator.revoked ? "red" : "green"}>
                        {operator.revoked ? `revoked ${when(operator.revoked_at)}` : "active"}
                      </StatusBadge>
                    </Table.Cell>
                    <Table.Cell>
                      <RevokeButton operator={operator} isSelf={isSelf} />
                    </Table.Cell>
                  </Table.Row>
                )
              })
            )}
          </Table.Body>
        </Table>
      </Container>
    </div>
  )
}
