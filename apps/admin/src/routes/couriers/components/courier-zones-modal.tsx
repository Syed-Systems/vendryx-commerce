import { Badge, Button, FocusModal, Heading, Input, Label, Text, toast } from "@medusajs/ui"
import { useState, useEffect } from "react"
import { CourierLogo } from "./courier-logo"
import { CourierProvider } from "./courier-credential-modal"

interface CourierZonesModalProps {
  courier: CourierProvider | null
  open: boolean
  onClose: () => void
  onSaveRates: (id: string, rates: { insideDhaka: number; subDhaka: number; outsideDhaka: number }) => void
}

export const CourierZonesModal = ({
  courier,
  open,
  onClose,
  onSaveRates,
}: CourierZonesModalProps) => {
  const [rates, setRates] = useState({
    insideDhaka: 60,
    subDhaka: 100,
    outsideDhaka: 130,
  })

  useEffect(() => {
    if (courier) {
      setRates(courier.rates)
    }
  }, [courier, open])

  if (!courier) return null

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault()
    onSaveRates(courier.id, rates)
    toast.success(`Zone delivery charges updated for ${courier.name}`)
    onClose()
  }

  return (
    <FocusModal open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <FocusModal.Content className="inset-auto left-1/2 top-1/2 max-h-[calc(100vh-32px)] w-[calc(100vw-32px)] max-w-[560px] -translate-x-1/2 -translate-y-1/2 rounded-2xl shadow-2xl border border-ui-border-base bg-ui-bg-base">
        <FocusModal.Header className="border-b border-ui-border-base p-6 bg-ui-bg-subtle/50">
          <div className="flex items-center gap-x-3.5">
            <CourierLogo id={courier.id} size="medium" />
            <div className="flex flex-col">
              <FocusModal.Title asChild>
                <Heading level="h2" className="text-lg font-bold text-ui-fg-base">
                  Delivery Zone Rates: {courier.name}
                </Heading>
              </FocusModal.Title>
              <Text size="xsmall" className="text-ui-fg-subtle">
                Configure standard customer shipping fees charged at checkout per geographic zone.
              </Text>
            </div>
          </div>
        </FocusModal.Header>

        <form onSubmit={handleSave} className="flex flex-1 flex-col overflow-hidden">
          <FocusModal.Body className="flex flex-col gap-y-5 overflow-y-auto p-6">
            <div className="grid grid-cols-1 gap-4">
              {/* Inside Dhaka */}
              <div className="flex flex-col gap-y-2 rounded-xl border p-4 bg-ui-bg-subtle/20">
                <div className="flex items-center justify-between">
                  <div className="flex flex-col">
                    <Label size="small" weight="plus">
                      Inside Dhaka Metro
                    </Label>
                    <Text size="xsmall" className="text-ui-fg-subtle">
                      Turnaround: 24 - 48 hours
                    </Text>
                  </div>
                  <Badge size="small" color="blue">
                    Zone 1
                  </Badge>
                </div>
                <div className="relative flex items-center">
                  <span className="absolute left-3 text-ui-fg-muted font-semibold">৳</span>
                  <Input
                    type="number"
                    min={0}
                    step={5}
                    className="pl-8"
                    value={rates.insideDhaka}
                    onChange={(e) =>
                      setRates((prev) => ({
                        ...prev,
                        insideDhaka: Number(e.target.value) || 0,
                      }))
                    }
                  />
                </div>
              </div>

              {/* Sub-Dhaka / Adjacent */}
              <div className="flex flex-col gap-y-2 rounded-xl border p-4 bg-ui-bg-subtle/20">
                <div className="flex items-center justify-between">
                  <div className="flex flex-col">
                    <Label size="small" weight="plus">
                      Dhaka Suburbs (Gazipur, Savar, Narayanganj, Keraniganj)
                    </Label>
                    <Text size="xsmall" className="text-ui-fg-subtle">
                      Turnaround: 48 - 72 hours
                    </Text>
                  </div>
                  <Badge size="small" color="blue">
                    Zone 2
                  </Badge>
                </div>
                <div className="relative flex items-center">
                  <span className="absolute left-3 text-ui-fg-muted font-semibold">৳</span>
                  <Input
                    type="number"
                    min={0}
                    step={5}
                    className="pl-8"
                    value={rates.subDhaka}
                    onChange={(e) =>
                      setRates((prev) => ({
                        ...prev,
                        subDhaka: Number(e.target.value) || 0,
                      }))
                    }
                  />
                </div>
              </div>

              {/* Outside Dhaka / Nationwide */}
              <div className="flex flex-col gap-y-2 rounded-xl border p-4 bg-ui-bg-subtle/20">
                <div className="flex items-center justify-between">
                  <div className="flex flex-col">
                    <Label size="small" weight="plus">
                      Outside Dhaka Nationwide (All Districts)
                    </Label>
                    <Text size="xsmall" className="text-ui-fg-subtle">
                      Turnaround: 3 - 5 business days
                    </Text>
                  </div>
                  <Badge size="small" color="blue">
                    Zone 3
                  </Badge>
                </div>
                <div className="relative flex items-center">
                  <span className="absolute left-3 text-ui-fg-muted font-semibold">৳</span>
                  <Input
                    type="number"
                    min={0}
                    step={5}
                    className="pl-8"
                    value={rates.outsideDhaka}
                    onChange={(e) =>
                      setRates((prev) => ({
                        ...prev,
                        outsideDhaka: Number(e.target.value) || 0,
                      }))
                    }
                  />
                </div>
              </div>
            </div>
          </FocusModal.Body>

          <FocusModal.Footer className="border-t p-4 flex items-center justify-end gap-x-2 bg-ui-bg-subtle/20">
            <Button type="button" variant="secondary" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" variant="primary">
              Save Zone Rates
            </Button>
          </FocusModal.Footer>
        </form>
      </FocusModal.Content>
    </FocusModal>
  )
}
