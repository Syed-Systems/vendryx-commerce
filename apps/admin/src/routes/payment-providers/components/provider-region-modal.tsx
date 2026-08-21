import { Button, Checkbox, FocusModal, Heading, Label, Text, toast } from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useState, useEffect } from "react"
import { sdk } from "../../../lib/client"
import { PaymentProvider } from "./provider-credential-modal"

interface Region {
  id: string
  name: string
  currency_code: string
}

interface ProviderRegionModalProps {
  provider: PaymentProvider | null
  open: boolean
  onClose: () => void
  onSuccess: () => void
}

export const ProviderRegionModal = ({
  provider,
  open,
  onClose,
  onSuccess,
}: ProviderRegionModalProps) => {
  const [selectedRegions, setSelectedRegions] = useState<Set<string>>(new Set())
  const [isSubmitting, setIsSubmitting] = useState(false)

  const { data: regionsData, isLoading } = useQuery<{ regions: Region[] }>({
    queryKey: ["admin", "regions"],
    queryFn: () => sdk.client.fetch("/admin/regions"),
  })

  useEffect(() => {
    if (provider) {
      setSelectedRegions(new Set(provider.enabled_regions || []))
    }
  }, [provider])

  if (!provider) return null

  const handleToggle = (regionId: string) => {
    setSelectedRegions((prev) => {
      const next = new Set(prev)
      if (next.has(regionId)) {
        next.delete(regionId)
      } else {
        next.add(regionId)
      }
      return next
    })
  }

  const handleSave = async () => {
    setIsSubmitting(true)
    const initialRegions = new Set(provider.enabled_regions || [])
    const toEnable = Array.from(selectedRegions).filter((id) => !initialRegions.has(id))
    const toDisable = Array.from(initialRegions).filter((id) => !selectedRegions.has(id))

    try {
      for (const regionId of toEnable) {
        await sdk.client.fetch(`/admin/regions/${regionId}/payment-providers/${provider.id}`, {
          method: "POST",
        })
      }
      for (const regionId of toDisable) {
        await sdk.client.fetch(`/admin/regions/${regionId}/payment-providers/${provider.id}`, {
          method: "DELETE",
        })
      }

      toast.success(`Region availability updated for ${provider.name}`)
      onSuccess()
      onClose()
    } catch (err: any) {
      toast.error(err.message || "Failed to update region availability")
    } finally {
      setIsSubmitting(false)
    }
  }

  const regions = regionsData?.regions || []

  return (
    <FocusModal open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <FocusModal.Content className="inset-auto left-1/2 top-1/2 max-h-[calc(100vh-32px)] w-[calc(100vw-32px)] max-w-[560px] -translate-x-1/2 -translate-y-1/2">
        <FocusModal.Header>
          <div className="flex flex-col">
            <FocusModal.Title asChild>
              <Heading level="h2">Enable Regions for {provider.name}</Heading>
            </FocusModal.Title>
            <Text size="small" className="text-ui-fg-subtle">
              Choose which sales regions can checkout with this provider.
            </Text>
          </div>
        </FocusModal.Header>

        <FocusModal.Body className="flex flex-col gap-y-3 overflow-y-auto p-6">
          {isLoading ? (
            <Text size="small" className="text-ui-fg-subtle">
              Loading regions...
            </Text>
          ) : regions.length === 0 ? (
            <Text size="small" className="text-ui-fg-subtle">
              No regions configured yet. Create a region in Settings &gt; Regions first.
            </Text>
          ) : (
            regions.map((region) => {
              const isChecked = selectedRegions.has(region.id)
              return (
                <div
                  key={region.id}
                  onClick={() => handleToggle(region.id)}
                  className="flex cursor-pointer items-center justify-between rounded-lg border p-3.5 transition-colors hover:bg-ui-bg-subtle"
                >
                  <div className="flex items-center gap-x-3">
                    <Checkbox checked={isChecked} onCheckedChange={() => handleToggle(region.id)} />
                    <div className="flex flex-col">
                      <Label size="small" weight="plus">
                        {region.name}
                      </Label>
                      <Text size="xsmall" className="text-ui-fg-subtle">
                        Currency: {region.currency_code.toUpperCase()}
                      </Text>
                    </div>
                  </div>
                </div>
              )
            })
          )}
        </FocusModal.Body>

        <FocusModal.Footer>
          <div className="flex items-center justify-end gap-x-2">
            <Button type="button" variant="secondary" onClick={onClose} disabled={isSubmitting}>
              Cancel
            </Button>
            <Button type="button" variant="primary" onClick={handleSave} isLoading={isSubmitting}>
              Save Regions
            </Button>
          </div>
        </FocusModal.Footer>
      </FocusModal.Content>
    </FocusModal>
  )
}
