/**
 * What the control plane has done, and who did it.
 *
 * Every write on `/platform/*` is billing-affecting and crosses tenants, so this is the
 * page that answers the only question that matters after an incident: who changed what,
 * when, and what was the value before.
 *
 * The `before` half of a diff is why this is a table with an expandable row rather than a
 * list of sentences. "Moved to Free" is not an answer to "who took them off Pro".
 */
import { ChevronDown, ChevronRight } from "@medusajs/icons"
import { Badge, Button, Container, Heading, Input, Table, Text } from "@medusajs/ui"
import { useInfiniteQuery } from "@tanstack/react-query"
import { useState } from "react"

import { listAudit, type AuditEntry } from "./api"

/** The actions the control plane can record, in the words an operator would use. */
const ACTION_LABELS: Record<string, string> = {
  "tenant.create": "provisioned",
  "tenant.tier_set": "changed plan",
  "subscription.patch": "edited subscription",
  "tier.define": "redefined plan",
  "tier.reset": "restored plan default",
  "operator.create": "issued credential",
  "operator.revoke": "revoked credential",
}

/** Actions that change what a merchant is charged for or entitled to. */
const BILLING_ACTIONS = new Set([
  "tenant.tier_set",
  "subscription.patch",
  "tier.define",
  "tier.reset",
])

function when(value: string): string {
  const date = new Date(value)
  return `${date.toLocaleDateString()} ${date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  })}`
}

function Row({ entry }: { entry: AuditEntry }) {
  const [open, setOpen] = useState(false)
  const hasDiff = entry.diff !== null && entry.diff !== undefined

  return (
    <>
      <Table.Row
        className={hasDiff ? "cursor-pointer" : undefined}
        onClick={() => hasDiff && setOpen((o) => !o)}
      >
        <Table.Cell className="w-8">
          {hasDiff && (open ? <ChevronDown /> : <ChevronRight />)}
        </Table.Cell>
        <Table.Cell className="whitespace-nowrap">{when(entry.created_at)}</Table.Cell>
        <Table.Cell>
          {entry.actor_name}
          {/* The bootstrap token names a secret, not a person. Saying so here stops a
              reader from concluding an individual is accountable when nobody is. */}
          {entry.actor_id === "op_bootstrap" && (
            <Badge size="2xsmall" color="orange" className="ml-2">
              shared token
            </Badge>
          )}
        </Table.Cell>
        <Table.Cell>
          <Badge size="2xsmall" color={BILLING_ACTIONS.has(entry.action) ? "blue" : "grey"}>
            {ACTION_LABELS[entry.action] ?? entry.action}
          </Badge>
        </Table.Cell>
        <Table.Cell className="font-mono text-ui-fg-subtle">{entry.entity_id}</Table.Cell>
      </Table.Row>
      {open && hasDiff && (
        <Table.Row>
          {/* A raw td because @medusajs/ui's Table.Cell does not type colSpan, and the
              diff has to span the row it belongs to rather than sit in one column. */}
          <td colSpan={5} className="bg-ui-bg-subtle px-6">
            <pre className="text-ui-fg-subtle overflow-x-auto py-2 text-xs">
              {JSON.stringify(entry.diff, null, 2)}
            </pre>
          </td>
        </Table.Row>
      )}
    </>
  )
}

export function Audit() {
  const [filter, setFilter] = useState("")
  // Applied on submit rather than on every keystroke: each change is a round trip, and a
  // half-typed tenant id matches nothing.
  const [applied, setApplied] = useState("")

  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } = useInfiniteQuery({
    queryKey: ["platform", "audit", applied],
    queryFn: ({ pageParam }) =>
      listAudit({ entityId: applied || undefined, before: pageParam, limit: 50 }),
    initialPageParam: undefined as string | undefined,
    // Null means the trail is exhausted. Returning undefined is what stops react-query
    // asking for another page.
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    retry: false,
  })

  const entries = data?.pages.flatMap((page) => page.entries) ?? []

  return (
    <Container className="p-0">
      <div className="flex items-center justify-between px-6 py-4">
        <div>
          <Heading level="h2">Audit trail</Heading>
          <Text size="small" className="text-ui-fg-subtle">
            Every write the control plane has made, newest first.
          </Text>
        </div>
        <form
          className="flex items-center gap-x-2"
          onSubmit={(event) => {
            event.preventDefault()
            setApplied(filter.trim())
          }}
        >
          <Input
            size="small"
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
            placeholder="Filter by tenant or tier"
          />
          <Button type="submit" variant="secondary" size="small">
            Filter
          </Button>
          {applied && (
            <Button
              variant="transparent"
              size="small"
              type="button"
              onClick={() => {
                setFilter("")
                setApplied("")
              }}
            >
              Clear
            </Button>
          )}
        </form>
      </div>

      <Table>
        <Table.Header>
          <Table.Row>
            <Table.HeaderCell />
            <Table.HeaderCell>When</Table.HeaderCell>
            <Table.HeaderCell>Who</Table.HeaderCell>
            <Table.HeaderCell>Did what</Table.HeaderCell>
            <Table.HeaderCell>To</Table.HeaderCell>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {isLoading ? (
            <Table.Row>
              <Table.Cell className="text-ui-fg-muted py-8 text-center">Loading…</Table.Cell>
            </Table.Row>
          ) : entries.length === 0 ? (
            <Table.Row>
              <Table.Cell className="text-ui-fg-muted py-8 text-center">
                {applied
                  ? `Nothing recorded against ${applied}.`
                  : "Nothing has been changed through the control plane yet."}
              </Table.Cell>
            </Table.Row>
          ) : (
            entries.map((entry) => <Row key={entry.id} entry={entry} />)
          )}
        </Table.Body>
      </Table>

      {hasNextPage && (
        <div className="flex justify-center px-6 py-4">
          <Button
            variant="secondary"
            size="small"
            isLoading={isFetchingNextPage}
            onClick={() => fetchNextPage()}
          >
            Load older
          </Button>
        </div>
      )}
    </Container>
  )
}
