import { ArrowPath } from "@medusajs/icons"
import {
  Badge,
  Button,
  Checkbox,
  Container,
  Heading,
  Input,
  Label,
  Text,
  toast,
} from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useState } from "react"

import { listTiers, resetTier, saveTier, type Catalog, type TierView } from "./api"

/** `_` separated keys read badly as labels and nowhere else. */
export const humanise = (key: string) =>
  key.replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase())

/**
 * 0 is the engine's single sentinel for "unlimited", for both quotas and windows, so a
 * missing value can never accidentally read as a restriction. The input surfaces that
 * rather than hiding it behind a separate "unlimited" toggle that would give the operator
 * two ways to say the same thing.
 */
const limitHint = (value: number) => (value === 0 ? "unlimited" : String(value))

type Draft = {
  features: Set<string>
  quotas: Record<string, number>
  windows: Record<string, number>
}

const draftOf = (tier: TierView): Draft => ({
  features: new Set(tier.features),
  quotas: { ...tier.quotas },
  windows: { ...tier.windows },
})

const sameAs = (draft: Draft, tier: TierView) =>
  draft.features.size === tier.features.length &&
  tier.features.every((f) => draft.features.has(f)) &&
  Object.entries(draft.quotas).every(([k, v]) => tier.quotas[k] === v) &&
  Object.entries(draft.windows).every(([k, v]) => tier.windows[k] === v)

function LimitField({
  label,
  value,
  onChange,
}: {
  label: string
  value: number
  onChange: (next: number) => void
}) {
  return (
    <div className="flex flex-col gap-y-1.5">
      <Label size="xsmall" htmlFor={label}>
        {humanise(label)}
      </Label>
      <Input
        id={label}
        type="number"
        min={0}
        value={value}
        onChange={(e) => {
          // An empty field or a negative number is not a limit. Clamping to 0 keeps the
          // control in the sentinel's own vocabulary instead of letting NaN reach the API.
          const parsed = Number.parseInt(e.target.value, 10)
          onChange(Number.isFinite(parsed) && parsed > 0 ? parsed : 0)
        }}
      />
      <Text size="xsmall" className="text-ui-fg-muted">
        {limitHint(value)}
      </Text>
    </div>
  )
}

function PlanCard({
  tier,
  catalog,
  onSaved,
}: {
  tier: TierView
  catalog: Catalog
  onSaved: () => void
}) {
  const [draft, setDraft] = useState<Draft>(() => draftOf(tier))

  // The server is the source of truth for what a plan is: after a save or a reset the
  // fresh row replaces the draft, so a rejected edit never lingers on screen looking saved.
  useEffect(() => setDraft(draftOf(tier)), [tier])

  const save = useMutation({
    mutationFn: () =>
      saveTier(tier.tier, {
        features: [...draft.features],
        quotas: draft.quotas,
        windows: draft.windows,
      }),
    onSuccess: () => {
      toast.success(`${humanise(tier.tier)} plan updated`)
      onSaved()
    },
    onError: (error: Error) => toast.error(error.message),
  })

  const reset = useMutation({
    mutationFn: () => resetTier(tier.tier),
    onSuccess: () => {
      toast.success(`${humanise(tier.tier)} back to the shipped default`)
      onSaved()
    },
    onError: (error: Error) => toast.error(error.message),
  })

  const clean = sameAs(draft, tier)

  return (
    <Container className="p-0">
      <div className="border-ui-border-base flex items-center justify-between border-b p-5">
        <div className="flex items-center gap-x-2.5">
          <Heading level="h2" className="capitalize">
            {tier.tier}
          </Heading>
          <Badge size="xsmall" color={tier.source === "stored" ? "orange" : "grey"}>
            {tier.source === "stored" ? "edited" : "shipped default"}
          </Badge>
        </div>
        <div className="flex items-center gap-x-2">
          {tier.source === "stored" && (
            <Button
              variant="transparent"
              size="small"
              isLoading={reset.isPending}
              onClick={() => reset.mutate()}
            >
              Restore default
            </Button>
          )}
          <Button
            size="small"
            disabled={clean}
            isLoading={save.isPending}
            onClick={() => save.mutate()}
          >
            Save plan
          </Button>
        </div>
      </div>

      <div className="flex flex-col gap-y-5 p-5">
        <div>
          <Text size="small" weight="plus" className="text-ui-fg-subtle mb-3">
            Features
          </Text>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {catalog.features.map((feature) => {
              const id = `${tier.tier}-${feature}`
              return (
                <div key={feature} className="flex items-center gap-x-2">
                  <Checkbox
                    id={id}
                    checked={draft.features.has(feature)}
                    onCheckedChange={(checked) =>
                      setDraft((d) => {
                        const features = new Set(d.features)
                        if (checked) {
                          features.add(feature)
                        } else {
                          features.delete(feature)
                        }
                        return { ...d, features }
                      })
                    }
                  />
                  <Label size="small" htmlFor={id} className="cursor-pointer">
                    {humanise(feature)}
                  </Label>
                </div>
              )
            })}
          </div>
        </div>

        <div>
          <Text size="small" weight="plus" className="text-ui-fg-subtle mb-3">
            Quotas &amp; windows
          </Text>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {catalog.quotas.map((key) => (
              <LimitField
                key={key}
                label={key}
                value={draft.quotas[key] ?? 0}
                onChange={(next) =>
                  setDraft((d) => ({ ...d, quotas: { ...d.quotas, [key]: next } }))
                }
              />
            ))}
            {catalog.windows.map((key) => (
              <LimitField
                key={key}
                label={key}
                value={draft.windows[key] ?? 0}
                onChange={(next) =>
                  setDraft((d) => ({ ...d, windows: { ...d.windows, [key]: next } }))
                }
              />
            ))}
          </div>
        </div>
      </div>
    </Container>
  )
}

export function Plans() {
  const queryClient = useQueryClient()
  const { data, isLoading, isError, error, refetch, isFetching } = useQuery({
    queryKey: ["platform", "tiers"],
    queryFn: listTiers,
    retry: false,
  })

  // Every merchant on a plan is affected by an edit to it, so their resolved
  // entitlements are stale the moment a plan is saved.
  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["platform"] })

  if (isError) {
    return (
      <Container className="p-6">
        <Heading level="h2" className="mb-2">
          Could not read the plans
        </Heading>
        <Text size="small" className="text-ui-fg-subtle">
          {(error as Error).message}
        </Text>
      </Container>
    )
  }

  return (
    <div className="flex flex-col gap-y-5">
      <div className="flex items-start justify-between">
        <div>
          <Heading level="h1">Plans</Heading>
          <Text size="small" className="text-ui-fg-subtle">
            What each plan includes. A saved plan takes effect on every merchant on it
            immediately — no restart, and no deploy.
          </Text>
        </div>
        <Button
          variant="secondary"
          size="small"
          onClick={() => refetch()}
          isLoading={isFetching}
          aria-label="Refresh"
        >
          <ArrowPath />
        </Button>
      </div>

      {isLoading || !data ? (
        <Container className="text-ui-fg-muted p-8 text-center">Loading plans…</Container>
      ) : (
        data.tiers.map((tier) => (
          <PlanCard
            key={tier.tier}
            tier={tier}
            catalog={data.catalog}
            onSaved={invalidate}
          />
        ))
      )}
    </div>
  )
}
