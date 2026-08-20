import { Badge, Button, Drawer, Input, Label, Select, Text, toast } from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"

import {
  getSubscription,
  listTiers,
  patchSubscription,
  type EffectiveEntitlements,
  type Subscription,
} from "./api"
import { humanise } from "./plans"

/** `<input type="date">` wants `yyyy-mm-dd`; the API speaks RFC 3339. */
const toDateInput = (iso: string | null) => (iso ? iso.slice(0, 10) : "")

/**
 * Midday UTC, not midnight.
 *
 * A trial that ends at `2026-09-01T00:00:00Z` has already ended for a merchant in Auckland
 * on the 31st and still has a day to run in Honolulu. Midday puts the whole planet on the
 * same side of the date the operator actually picked.
 */
const fromDateInput = (value: string) => (value ? `${value}T12:00:00Z` : null)

type FeatureChoice = "inherit" | "grant" | "revoke"

const choiceOf = (overrides: Record<string, boolean>, feature: string): FeatureChoice => {
  if (!(feature in overrides)) return "inherit"
  return overrides[feature] ? "grant" : "revoke"
}

type Draft = {
  tier: string
  status: string
  trial_started_at: string
  trial_ends_at: string
  term_ends_at: string
  quota_overrides: Record<string, string>
  window_overrides: Record<string, string>
  feature_overrides: Record<string, boolean>
}

const draftOf = (sub: Subscription): Draft => ({
  tier: sub.tier,
  status: sub.status,
  trial_started_at: toDateInput(sub.trial_started_at),
  trial_ends_at: toDateInput(sub.trial_ends_at),
  term_ends_at: toDateInput(sub.term_ends_at),
  // Held as strings so an empty field means "no override" rather than 0, which the engine
  // reads as *unlimited* — the opposite of what clearing a box looks like it should mean.
  quota_overrides: Object.fromEntries(
    Object.entries(sub.quota_overrides ?? {}).map(([k, v]) => [k, String(v)])
  ),
  window_overrides: Object.fromEntries(
    Object.entries(sub.window_overrides ?? {}).map(([k, v]) => [k, String(v)])
  ),
  feature_overrides: { ...(sub.feature_overrides ?? {}) },
})

const limitsOf = (draft: Record<string, string>): Record<string, number> =>
  Object.fromEntries(
    Object.entries(draft)
      .filter(([, v]) => v.trim() !== "")
      .map(([k, v]) => [k, Number.parseInt(v, 10)])
      .filter(([, v]) => Number.isFinite(v as number) && (v as number) >= 0)
  )

function OverrideField({
  label,
  planValue,
  value,
  onChange,
}: {
  label: string
  planValue: number | undefined
  value: string
  onChange: (next: string) => void
}) {
  return (
    <div className="flex flex-col gap-y-1.5">
      <Label size="xsmall" htmlFor={`ov-${label}`}>
        {humanise(label)}
      </Label>
      <Input
        id={`ov-${label}`}
        type="number"
        min={0}
        value={value}
        placeholder={planValue === 0 ? "unlimited (plan)" : `${planValue ?? 0} (plan)`}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  )
}

function Effective({ effective }: { effective: EffectiveEntitlements | null }) {
  if (!effective) {
    return null
  }
  return (
    <div className="bg-ui-bg-subtle rounded-lg p-4">
      <Text size="small" weight="plus" className="mb-2">
        What this merchant gets
      </Text>
      <div className="mb-3 flex flex-wrap gap-1.5">
        {effective.features.length === 0 ? (
          <Text size="xsmall" className="text-ui-fg-muted">
            No gated features.
          </Text>
        ) : (
          effective.features.map((f) => (
            <Badge key={f} size="xsmall" color="green">
              {humanise(f)}
            </Badge>
          ))
        )}
      </div>
      <div className="text-ui-fg-subtle grid grid-cols-1 gap-x-6 gap-y-1 sm:grid-cols-2">
        {Object.entries({ ...effective.quotas, ...effective.windows }).map(([k, v]) => (
          <div key={k} className="flex justify-between text-xs">
            <span>{humanise(k)}</span>
            <span className="tabular-nums">{v === 0 ? "unlimited" : v}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

export function SubscriptionDrawer({
  tenantId,
  tenantName,
  open,
  onOpenChange,
}: {
  tenantId: string
  tenantName: string
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()
  const [draft, setDraft] = useState<Draft | null>(null)

  const { data, isLoading } = useQuery({
    queryKey: ["platform", "subscription", tenantId],
    queryFn: () => getSubscription(tenantId),
    enabled: open,
    retry: false,
  })

  const { data: plans } = useQuery({
    queryKey: ["platform", "tiers"],
    queryFn: listTiers,
    enabled: open,
    retry: false,
  })

  useEffect(() => {
    if (data?.subscription) {
      setDraft(draftOf(data.subscription))
    }
  }, [data])

  const plan = plans?.tiers.find((t) => t.tier === draft?.tier)

  const save = useMutation({
    mutationFn: () => {
      if (!draft) throw new Error("nothing to save")
      return patchSubscription(tenantId, {
        tier: draft.tier,
        status: draft.status,
        trial_started_at: fromDateInput(draft.trial_started_at),
        trial_ends_at: fromDateInput(draft.trial_ends_at),
        term_ends_at: fromDateInput(draft.term_ends_at),
        quota_overrides: limitsOf(draft.quota_overrides),
        window_overrides: limitsOf(draft.window_overrides),
        feature_overrides: draft.feature_overrides,
      })
    },
    onSuccess: () => {
      toast.success(`${tenantName} updated`)
      queryClient.invalidateQueries({ queryKey: ["platform"] })
      onOpenChange(false)
    },
    onError: (error: Error) => toast.error(error.message),
  })

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <Drawer.Content>
        <Drawer.Header>
          <Drawer.Title>{tenantName}</Drawer.Title>
          <Drawer.Description>
            Plan, term, and the negotiated exceptions that apply to this merchant alone.
          </Drawer.Description>
        </Drawer.Header>

        <Drawer.Body className="flex flex-col gap-y-6 overflow-y-auto">
          {isLoading || !draft ? (
            <Text size="small" className="text-ui-fg-muted">
              Loading subscription…
            </Text>
          ) : (
            <>
              {data?.error && (
                <div className="bg-ui-tag-red-bg text-ui-tag-red-text rounded-lg p-3">
                  <Text size="small">
                    This subscription does not resolve: {data.error}. Every request for
                    this merchant is falling back to the least-privileged plan until it is
                    fixed.
                  </Text>
                </div>
              )}

              <div className="grid grid-cols-2 gap-4">
                <div className="flex flex-col gap-y-1.5">
                  <Label size="xsmall">Plan</Label>
                  <Select
                    value={draft.tier}
                    onValueChange={(tier) => setDraft({ ...draft, tier })}
                  >
                    <Select.Trigger>
                      <Select.Value />
                    </Select.Trigger>
                    <Select.Content>
                      {(plans?.tiers ?? []).map((t) => (
                        <Select.Item key={t.tier} value={t.tier}>
                          {humanise(t.tier)}
                        </Select.Item>
                      ))}
                    </Select.Content>
                  </Select>
                </div>
                <div className="flex flex-col gap-y-1.5">
                  <Label size="xsmall">Status</Label>
                  <Select
                    value={draft.status}
                    onValueChange={(status) => setDraft({ ...draft, status })}
                  >
                    <Select.Trigger>
                      <Select.Value />
                    </Select.Trigger>
                    <Select.Content>
                      {(plans?.catalog.statuses ?? []).map((s) => (
                        <Select.Item key={s} value={s}>
                          {humanise(s)}
                        </Select.Item>
                      ))}
                    </Select.Content>
                  </Select>
                </div>
              </div>

              <div>
                <Text size="small" weight="plus" className="mb-3">
                  Term
                </Text>
                <div className="grid grid-cols-3 gap-3">
                  {(
                    [
                      ["trial_started_at", "Trial started"],
                      ["trial_ends_at", "Trial ends"],
                      ["term_ends_at", "Term ends"],
                    ] as const
                  ).map(([key, label]) => (
                    <div key={key} className="flex flex-col gap-y-1.5">
                      <Label size="xsmall" htmlFor={key}>
                        {label}
                      </Label>
                      <Input
                        id={key}
                        type="date"
                        value={draft[key]}
                        onChange={(e) => setDraft({ ...draft, [key]: e.target.value })}
                      />
                    </div>
                  ))}
                </div>
                <Text size="xsmall" className="text-ui-fg-muted mt-2">
                  Clearing a date removes it. Nothing is deleted when a term ends — the
                  merchant keeps every product and order and is only blocked from creating
                  more.
                </Text>
              </div>

              <div>
                <Text size="small" weight="plus" className="mb-3">
                  Feature exceptions
                </Text>
                <div className="flex flex-col gap-y-2">
                  {(plans?.catalog.features ?? []).map((feature) => {
                    const inPlan = plan?.features.includes(feature) ?? false
                    return (
                      <div
                        key={feature}
                        className="flex items-center justify-between gap-x-3"
                      >
                        <div className="flex flex-col">
                          <Text size="small">{humanise(feature)}</Text>
                          <Text size="xsmall" className="text-ui-fg-muted">
                            {inPlan ? "in this plan" : "not in this plan"}
                          </Text>
                        </div>
                        <Select
                          size="small"
                          value={choiceOf(draft.feature_overrides, feature)}
                          onValueChange={(choice) =>
                            setDraft((d) => {
                              if (!d) return d
                              const next = { ...d.feature_overrides }
                              if (choice === "inherit") {
                                delete next[feature]
                              } else {
                                next[feature] = choice === "grant"
                              }
                              return { ...d, feature_overrides: next }
                            })
                          }
                        >
                          <Select.Trigger className="w-[130px]">
                            <Select.Value />
                          </Select.Trigger>
                          <Select.Content>
                            <Select.Item value="inherit">Plan default</Select.Item>
                            <Select.Item value="grant">Grant</Select.Item>
                            <Select.Item value="revoke">Revoke</Select.Item>
                          </Select.Content>
                        </Select>
                      </div>
                    )
                  })}
                </div>
              </div>

              <div>
                <Text size="small" weight="plus" className="mb-3">
                  Limit exceptions
                </Text>
                <div className="grid grid-cols-2 gap-3">
                  {(plans?.catalog.quotas ?? []).map((key) => (
                    <OverrideField
                      key={key}
                      label={key}
                      planValue={plan?.quotas[key]}
                      value={draft.quota_overrides[key] ?? ""}
                      onChange={(next) =>
                        setDraft({
                          ...draft,
                          quota_overrides: { ...draft.quota_overrides, [key]: next },
                        })
                      }
                    />
                  ))}
                  {(plans?.catalog.windows ?? []).map((key) => (
                    <OverrideField
                      key={key}
                      label={key}
                      planValue={plan?.windows[key]}
                      value={draft.window_overrides[key] ?? ""}
                      onChange={(next) =>
                        setDraft({
                          ...draft,
                          window_overrides: { ...draft.window_overrides, [key]: next },
                        })
                      }
                    />
                  ))}
                </div>
                <Text size="xsmall" className="text-ui-fg-muted mt-2">
                  Leave blank to follow the plan. 0 means unlimited.
                </Text>
              </div>

              <Effective effective={data?.effective ?? null} />
            </>
          )}
        </Drawer.Body>

        <Drawer.Footer>
          <Drawer.Close asChild>
            <Button variant="secondary" size="small">
              Cancel
            </Button>
          </Drawer.Close>
          <Button
            size="small"
            disabled={!draft}
            isLoading={save.isPending}
            onClick={() => save.mutate()}
          >
            Save
          </Button>
        </Drawer.Footer>
      </Drawer.Content>
    </Drawer>
  )
}
