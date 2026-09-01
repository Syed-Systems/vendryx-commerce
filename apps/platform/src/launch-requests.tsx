import { ArrowPath, CheckCircle, ExclamationCircle, Funnel, User } from "@medusajs/icons"
import { Badge, Button, Container, Heading, Select, StatusBadge, Table, Text, toast } from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"

import {
  listLaunchRequests,
  transitionLaunchRequest,
  type LaunchRequest,
  type LaunchStatus,
} from "./api"

const statusColor: Record<LaunchStatus, "green" | "orange" | "grey" | "red" | "blue"> = {
  submitted: "grey",
  in_review: "orange",
  provisioning: "blue",
  online: "green",
  needs_changes: "red",
  rejected: "red",
}

const statusLabel: Record<LaunchStatus, string> = {
  submitted: "Submitted",
  in_review: "In review",
  provisioning: "Provisioning",
  online: "Online",
  needs_changes: "Needs changes",
  rejected: "Rejected",
}

function RequestActions({ request }: { request: LaunchRequest }) {
  const queryClient = useQueryClient()
  const [developer, setDeveloper] = useState(request.assigned_developer ?? "")
  const mutation = useMutation({
    mutationFn: (input: { action: Parameters<typeof transitionLaunchRequest>[1]; note?: string }) =>
      transitionLaunchRequest(request.id, input.action, {
        assigned_developer: developer || undefined,
        note: input.note,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["platform", "launch-requests"] })
      toast.success("Launch request updated")
    },
    onError: (error: Error) => toast.error(error.message),
  })

  const act = (action: Parameters<typeof transitionLaunchRequest>[1]) => mutation.mutate({ action })
  const isProBlocked = request.tier === "pro" && request.payment_status !== "confirmed"

  return (
    <div className="flex min-w-[220px] flex-col gap-y-2">
      <div className="flex gap-x-2">
        <Select value={developer} onValueChange={setDeveloper}>
          <Select.Trigger><Select.Value placeholder="Assign developer" /></Select.Trigger>
          <Select.Content>
            <Select.Item value="unassigned">Unassigned</Select.Item>
            <Select.Item value="developer">Assigned developer</Select.Item>
          </Select.Content>
        </Select>
        <Button size="small" variant="secondary" onClick={() => act("assign")} disabled={mutation.isPending || !developer} aria-label="Assign developer">
          <User />
        </Button>
      </div>
      <div className="flex flex-wrap gap-1.5">
        {request.status === "submitted" && <Button size="small" variant="secondary" onClick={() => act("request_changes")} disabled={mutation.isPending}>Request changes</Button>}
        {(["submitted", "in_review", "needs_changes"] as LaunchStatus[]).includes(request.status) && <Button size="small" onClick={() => act("begin_provisioning")} disabled={mutation.isPending || isProBlocked}>Begin provisioning</Button>}
        {request.status === "provisioning" && <Button size="small" onClick={() => act("activate")} disabled={mutation.isPending}>Activate</Button>}
        {request.status === "provisioning" && <Button size="small" variant="secondary" onClick={() => act("retry")} disabled={mutation.isPending}><ArrowPath /> Retry</Button>}
        {(["needs_changes", "rejected"] as LaunchStatus[]).includes(request.status) && <Button size="small" variant="secondary" onClick={() => act("reopen")} disabled={mutation.isPending}>Reopen</Button>}
        {request.status !== "online" && request.status !== "rejected" && <Button size="small" variant="danger" onClick={() => act("reject")} disabled={mutation.isPending}>Reject</Button>}
      </div>
      {isProBlocked && <Text size="xsmall" className="text-ui-fg-subtle">Pro provisioning waits for confirmed payment.</Text>}
    </div>
  )
}

export function LaunchRequests() {
  const { data, isLoading, isError, error } = useQuery({
    queryKey: ["platform", "launch-requests"],
    queryFn: listLaunchRequests,
    retry: false,
  })

  return (
    <div className="flex flex-col gap-y-5">
      <div className="flex items-start justify-between gap-x-4">
        <div>
          <div className="mb-1 flex items-center gap-x-2"><Funnel className="text-ui-fg-subtle" /><Heading level="h1">Launch requests</Heading></div>
          <Text size="small" className="text-ui-fg-subtle">The normal merchant-onboarding queue. Manual tenant creation stays exceptional and audited in Merchants.</Text>
        </div>
        <Badge color="grey">{data?.count ?? 0} queued</Badge>
      </div>

      {isLoading && <Container className="p-6"><Text size="small">Loading launch requests…</Text></Container>}
      {isError && <Container className="p-6"><div className="flex items-center gap-x-2"><ExclamationCircle className="text-ui-fg-error" /><Heading level="h2">Launch queue unavailable</Heading></div><Text size="small" className="text-ui-fg-subtle mt-2">{(error as Error).message}</Text></Container>}
      {data && data.launch_requests.length === 0 && <Container className="p-8"><CheckCircle className="text-ui-fg-subtle mb-3" /><Heading level="h2">No launch requests waiting</Heading><Text size="small" className="text-ui-fg-subtle mt-1">New merchant requests will appear here after a verified account submits its store details.</Text></Container>}
      {data && data.launch_requests.length > 0 && <Container className="overflow-hidden p-0"><Table><Table.Header><Table.Row><Table.HeaderCell>Merchant</Table.HeaderCell><Table.HeaderCell>Store request</Table.HeaderCell><Table.HeaderCell>Commercial</Table.HeaderCell><Table.HeaderCell>Customer status</Table.HeaderCell><Table.HeaderCell>Validation</Table.HeaderCell><Table.HeaderCell>Actions</Table.HeaderCell></Table.Row></Table.Header><Table.Body>{data.launch_requests.map((request) => <Table.Row key={request.id}><Table.Cell><div className="flex flex-col"><Text size="small" weight="plus">{request.business_name || request.merchant_name || request.merchant_email}</Text><Text size="xsmall" className="text-ui-fg-subtle">{request.merchant_email}</Text></div></Table.Cell><Table.Cell><div className="flex flex-col"><Text size="small" weight="plus">{request.store_name}</Text><Text size="xsmall" className="text-ui-fg-subtle">{request.subdomain}.stheno.shop</Text></div></Table.Cell><Table.Cell><div className="flex gap-x-2"><Badge color={request.tier === "pro" ? "blue" : "grey"}>{request.tier.toUpperCase()}</Badge><Badge color={request.payment_status === "confirmed" || request.payment_status === "not_required" ? "green" : "orange"}>{request.payment_status.replace("_", " ")}</Badge><Badge color={request.buyer_accounts ? "purple" : "grey"}>{request.buyer_accounts ? "accounts" : "guest"}</Badge></div></Table.Cell><Table.Cell><StatusBadge color={statusColor[request.status]}>{statusLabel[request.status]}</StatusBadge></Table.Cell><Table.Cell><Text size="xsmall" className={request.validation_findings.length ? "text-ui-fg-error" : "text-ui-fg-subtle"}>{request.validation_findings.length ? `${request.validation_findings.length} findings` : "Ready"}</Text></Table.Cell><Table.Cell><RequestActions request={request} /></Table.Cell></Table.Row>)}</Table.Body></Table></Container>}
    </div>
  )
}
