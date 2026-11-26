import { InformationCircleSolid } from "@medusajs/icons"
import { Badge, Button, Container, Heading, Input, Label, Switch, Text, toast } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import { sdk } from "../../../lib/client"

export interface CodRules {
  enabled: boolean
  maxOrderAmount: number
  dhakaMetroOnly: boolean
  requirePhoneConfirmation: boolean
  instructionsNote: string
}

const DEFAULT_COD_RULES: CodRules = {
  enabled: true,
  maxOrderAmount: 50000,
  dhakaMetroOnly: false,
  requirePhoneConfirmation: true,
  instructionsNote: "Please keep exact cash ready upon delivery. Inspect your package with the courier representative before accepting.",
}

export const CodRulesCard = () => {
  const [initialRules, setInitialRules] = useState<CodRules>(DEFAULT_COD_RULES)
  const [rules, setRules] = useState<CodRules>(DEFAULT_COD_RULES)
  const [isSaving, setIsSaving] = useState(false)

  const { data, isLoading, refetch } = useQuery<CodRules>({
    queryKey: ["admin", "settings", "cod_rules"],
    queryFn: async () => {
      try {
        const res = await sdk.client.fetch<{ setting: { value: CodRules } }>("/admin/settings/cod_rules")
        if (res?.setting?.value) {
          return {
            ...DEFAULT_COD_RULES,
            ...res.setting.value,
          }
        }
        return DEFAULT_COD_RULES
      } catch {
        return DEFAULT_COD_RULES
      }
    },
  })

  useEffect(() => {
    if (data) {
      setInitialRules(data)
      setRules(data)
    }
  }, [data])

  const hasChanged = JSON.stringify(initialRules) !== JSON.stringify(rules)

  const handleSave = async () => {
    setIsSaving(true)
    try {
      await sdk.client.fetch("/admin/settings/cod_rules", {
        method: "POST",
        body: {
          value: rules,
          category: "payment",
        },
      })
      setInitialRules(rules)
      refetch()
      toast.success("Cash on Delivery (COD) rules updated successfully")
    } catch (err: any) {
      toast.error(err?.message || "Failed to save COD rules")
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <Container className="p-0 border border-ui-border-base rounded-xl overflow-hidden shadow-xs bg-ui-bg-base">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between border-b border-ui-border-base p-6 gap-4 bg-ui-bg-subtle/50">
        <div className="flex flex-col gap-y-1">
          <div className="flex items-center gap-x-2.5">
            <Heading level="h2" className="text-base font-semibold text-ui-fg-base">
              Cash on Delivery (COD) Rules
            </Heading>
            <Badge size="small" color={rules.enabled ? "green" : "grey"}>
              {rules.enabled ? "Active" : "Disabled"}
            </Badge>
          </div>
          <Text size="small" className="text-ui-fg-subtle">
            Configure order thresholds, geographic scope, and customer delivery instructions for offline settlements.
          </Text>
        </div>

        <div className="flex items-center gap-x-3">
          <Switch
            checked={rules.enabled}
            disabled={isLoading}
            onCheckedChange={(enabled) => setRules((prev) => ({ ...prev, enabled }))}
          />
          <Label size="small" weight="plus">
            {rules.enabled ? "COD Enabled" : "COD Disabled"}
          </Label>
        </div>
      </div>

      <div className="p-6 flex flex-col gap-y-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Max Order Value */}
          <div className="flex flex-col gap-y-2">
            <Label size="small" weight="plus">
              Maximum Order Amount (৳)
            </Label>
            <div className="relative flex items-center">
              <span className="absolute left-3 text-ui-fg-muted font-semibold">৳</span>
              <Input
                type="number"
                min={0}
                max={500000}
                step={500}
                className="pl-8"
                value={rules.maxOrderAmount || ""}
                onChange={(e) =>
                  setRules((prev) => ({
                    ...prev,
                    maxOrderAmount: Number(e.target.value) || 0,
                  }))
                }
                disabled={!rules.enabled || isLoading}
              />
            </div>
            <Text size="xsmall" className="text-ui-fg-muted">
              Orders above this amount will not be eligible for Cash on Delivery checkout.
            </Text>
          </div>

          {/* Dhaka Metro Scope */}
          <div className="flex flex-col justify-between p-4 rounded-lg border bg-ui-bg-subtle/50">
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-y-0.5">
                <Label size="small" weight="plus">
                  Dhaka Metro Exclusive
                </Label>
                <Text size="xsmall" className="text-ui-fg-muted">
                  Restrict COD availability strictly to addresses within Dhaka City.
                </Text>
              </div>
              <Switch
                checked={rules.dhakaMetroOnly}
                disabled={!rules.enabled || isLoading}
                onCheckedChange={(dhakaMetroOnly) =>
                  setRules((prev) => ({ ...prev, dhakaMetroOnly }))
                }
              />
            </div>
          </div>
        </div>

        {/* Require Phone Confirmation */}
        <div className="flex items-center justify-between p-4 rounded-lg border bg-ui-bg-subtle/50">
          <div className="flex flex-col gap-y-0.5">
            <Label size="small" weight="plus">
              Require Order Verification Call / SMS
            </Label>
            <Text size="xsmall" className="text-ui-fg-muted">
              Flag incoming COD orders as "Awaiting Verification" until merchant staff verifies buyer intent.
            </Text>
          </div>
          <Switch
            checked={rules.requirePhoneConfirmation}
            disabled={!rules.enabled || isLoading}
            onCheckedChange={(requirePhoneConfirmation) =>
              setRules((prev) => ({ ...prev, requirePhoneConfirmation }))
            }
          />
        </div>

        {/* Instructions Note */}
        <div className="flex flex-col gap-y-2">
          <Label size="small" weight="plus">
            Customer Checkout Note
          </Label>
          <textarea
            rows={2}
            className="w-full text-small rounded-md border border-ui-border-base bg-ui-bg-field px-3 py-2 text-ui-fg-base focus:border-ui-border-interactive focus:outline-hidden disabled:bg-ui-bg-disabled disabled:text-ui-fg-disabled resize-none"
            value={rules.instructionsNote}
            onChange={(e) =>
              setRules((prev) => ({ ...prev, instructionsNote: e.target.value }))
            }
            disabled={!rules.enabled || isLoading}
            placeholder="Delivery instructions shown to buyers choosing COD..."
          />
          <div className="flex items-center gap-x-1.5 text-ui-fg-muted">
            <InformationCircleSolid className="w-4 h-4 shrink-0 text-ui-fg-subtle" />
            <Text size="xsmall" className="text-ui-fg-subtle">
              Displayed verbatim on the storefront checkout payment step.
            </Text>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center justify-end gap-x-3 pt-4 border-t">
          <Button
            size="small"
            variant="secondary"
            disabled={!hasChanged || isSaving || isLoading}
            onClick={() => setRules(initialRules)}
          >
            Reset
          </Button>
          <Button
            size="small"
            variant="primary"
            disabled={!hasChanged || isSaving || isLoading}
            isLoading={isSaving}
            onClick={handleSave}
          >
            Save COD Rules
          </Button>
        </div>
      </div>
    </Container>
  )
}
