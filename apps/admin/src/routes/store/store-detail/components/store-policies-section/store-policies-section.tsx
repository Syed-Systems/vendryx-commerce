import {
  ArrowUpRightOnBox,
  DocumentText,
  PencilSquare,
} from "@medusajs/icons"
import {
  Badge,
  Button,
  Container,
  Heading,
  Input,
  Label,
  Text,
  toast,
} from "@medusajs/ui"
import { useQuery } from "@tanstack/react-query"
import { useEffect, useState } from "react"
import { sdk } from "../../../../../lib/client"

export interface StorePolicy {
  id: string
  title: string
  slug: string
  status: "published" | "draft"
  lastUpdated: string
  content: string
}

const DEFAULT_POLICIES: StorePolicy[] = [
  {
    id: "refund",
    title: "Refund & Return Policy",
    slug: "refund-policy",
    status: "published",
    lastUpdated: "Sep 1, 2026",
    content:
      "Customers may request returns within 7 days of delivery for damaged, defective, or incorrect items. Unopened items in original packaging are eligible for full refund or exchange.",
  },
  {
    id: "terms",
    title: "Terms of Service",
    slug: "terms-of-service",
    status: "published",
    lastUpdated: "Aug 15, 2026",
    content:
      "By accessing or ordering through this Vendryx storefront, customers agree to standard merchant terms, pricing, and fulfillment schedules.",
  },
  {
    id: "privacy",
    title: "Privacy Policy",
    slug: "privacy-policy",
    status: "published",
    lastUpdated: "Aug 15, 2026",
    content:
      "We collect customer shipping addresses and phone numbers solely for courier dispatch and order tracking. Customer information is never sold to third parties.",
  },
  {
    id: "shipping",
    title: "Shipping & Delivery Policy",
    slug: "shipping-policy",
    status: "published",
    lastUpdated: "Aug 20, 2026",
    content:
      "Orders within Dhaka Metro deliver within 24 to 48 hours. Nationwide orders via Pathao and Steadfast deliver within 3 to 5 business days.",
  },
]

export const StorePoliciesSection = () => {
  const [policies, setPolicies] = useState<StorePolicy[]>(DEFAULT_POLICIES)
  const [editingPolicy, setEditingPolicy] = useState<StorePolicy | null>(null)
  const [editTitle, setEditTitle] = useState("")
  const [editContent, setEditContent] = useState("")
  const [isSaving, setIsSaving] = useState(false)

  const { data, isLoading, refetch } = useQuery<StorePolicy[]>({
    queryKey: ["admin", "settings", "store_policies"],
    queryFn: async () => {
      try {
        const res = await sdk.client.fetch<{ setting: { value: StorePolicy[] } }>("/admin/settings/store_policies")
        if (Array.isArray(res?.setting?.value) && res.setting.value.length > 0) {
          return res.setting.value
        }
        return DEFAULT_POLICIES
      } catch {
        return DEFAULT_POLICIES
      }
    },
  })

  useEffect(() => {
    if (data) {
      setPolicies(data)
    }
  }, [data])

  const handleOpenEdit = (policy: StorePolicy) => {
    setEditingPolicy(policy)
    setEditTitle(policy.title)
    setEditContent(policy.content)
  }

  const handleSaveEdit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!editingPolicy) return

    const updatedPolicies = policies.map((p) =>
      p.id === editingPolicy.id
        ? {
            ...p,
            title: editTitle,
            content: editContent,
            lastUpdated: new Date().toLocaleDateString(undefined, { dateStyle: "medium" }),
          }
        : p
    )

    setIsSaving(true)
    try {
      await sdk.client.fetch("/admin/settings/store_policies", {
        method: "POST",
        body: {
          value: updatedPolicies,
          category: "store",
        },
      })
      setPolicies(updatedPolicies)
      setEditingPolicy(null)
      refetch()
      toast.success(`${editTitle} updated successfully`)
    } catch (err: any) {
      toast.error(err?.message || "Failed to save policy")
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <>
      <Container className="divide-y p-0">
        <div className="flex flex-col gap-y-1 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <Heading level="h2">Store Legal &amp; Customer Policies</Heading>
            <Text className="text-ui-fg-subtle" size="small">
              Customer-facing policies displayed in your storefront footer and checkout agreements.
            </Text>
          </div>
        </div>

        <div className="divide-y divide-ui-border-base">
          {policies.map((policy) => (
            <div
              key={policy.id}
              className="flex flex-col gap-y-3 px-6 py-4 sm:flex-row sm:items-center sm:justify-between"
            >
              <div className="flex items-start gap-x-3.5">
                <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-ui-border-base bg-ui-bg-subtle text-ui-fg-muted">
                  <DocumentText />
                </div>
                <div>
                  <div className="flex items-center gap-x-2">
                    <Text size="small" weight="plus" className="text-ui-fg-base">
                      {policy.title}
                    </Text>
                    <Badge color="green" size="2xsmall">
                      Published
                    </Badge>
                  </div>
                  <Text size="xsmall" className="text-ui-fg-subtle mt-0.5 max-w-xl line-clamp-1">
                    {policy.content}
                  </Text>
                  <Text size="xsmall" className="text-ui-fg-muted mt-1">
                    Slug: <code>/policies/{policy.slug}</code> • Last updated: {policy.lastUpdated}
                  </Text>
                </div>
              </div>

              <div className="flex items-center gap-x-2 shrink-0">
                <Button
                  size="small"
                  variant="secondary"
                  onClick={() => handleOpenEdit(policy)}
                  disabled={isLoading}
                >
                  <PencilSquare className="me-1.5 text-ui-fg-subtle" />
                  Edit policy
                </Button>
                <Button
                  size="small"
                  variant="transparent"
                  onClick={() => {
                    toast.info(`Opened public preview: /policies/${policy.slug}`)
                  }}
                  aria-label={`Preview ${policy.title}`}
                >
                  <ArrowUpRightOnBox className="h-4 w-4 text-ui-fg-subtle" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      </Container>

      {editingPolicy && (
        <div
          role="dialog"
          aria-labelledby="policy-modal-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-ui-bg-overlay/50 p-4 backdrop-blur-sm"
        >
          <div className="w-full max-w-lg rounded-xl border border-ui-border-base bg-ui-bg-base p-6 shadow-xl">
            <div className="flex items-center justify-between pb-4 border-b border-ui-border-base">
              <div>
                <Heading id="policy-modal-title" level="h2">Edit Store Policy</Heading>
                <Text size="xsmall" className="text-ui-fg-subtle">
                  Update customer policy text shown on your storefront
                </Text>
              </div>
              <Button
                variant="transparent"
                size="small"
                onClick={() => setEditingPolicy(null)}
                aria-label="Close"
              >
                ✕
              </Button>
            </div>

            <form onSubmit={handleSaveEdit} className="mt-4 flex flex-col gap-y-4">
              <div>
                <Label htmlFor="policy-title-input" className="text-ui-fg-subtle text-xs">
                  Policy Title
                </Label>
                <Input
                  id="policy-title-input"
                  value={editTitle}
                  onChange={(e) => setEditTitle(e.target.value)}
                  required
                />
              </div>

              <div>
                <Label htmlFor="policy-content-input" className="text-ui-fg-subtle text-xs">
                  Policy Content (Plain text or Markdown)
                </Label>
                <textarea
                  id="policy-content-input"
                  rows={6}
                  value={editContent}
                  onChange={(e) => setEditContent(e.target.value)}
                  className="w-full mt-1.5 rounded-lg border border-ui-border-base bg-ui-bg-field px-3 py-2 text-xs text-ui-fg-base focus:outline-none focus:ring-2 focus:ring-ui-fg-interactive"
                  required
                />
              </div>

              <div className="flex items-center justify-end gap-x-2 border-t border-ui-border-base pt-4">
                <Button
                  type="button"
                  variant="secondary"
                  size="small"
                  onClick={() => setEditingPolicy(null)}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  variant="primary"
                  size="small"
                  isLoading={isSaving}
                >
                  Save policy
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </>
  )
}
