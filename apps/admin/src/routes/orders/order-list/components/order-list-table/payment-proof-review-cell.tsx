import { Button, Text, toast } from "@medusajs/ui"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { sdk } from "../../../../../lib/client"
import { ordersQueryKeys } from "../../../../../hooks/api/orders"

type PaymentProof = {
  id: string
  payment_method: string
  transaction_id: string
  proof_asset_url: string | null
  status: "pending_verification" | "verified" | "rejected"
  submitted_at: string
}

export const PaymentProofReviewCell = ({ orderId }: { orderId: string }) => {
  const [open, setOpen] = useState(false)
  const queryClient = useQueryClient()
  const proofQuery = useQuery({
    queryKey: ["order-payment-proof", orderId],
    enabled: open,
    queryFn: async () => {
      const response = await sdk.client.fetch<{
        payment_proof: PaymentProof | null
      }>(`/admin/orders/${orderId}/payment-proof`, { method: "GET" })
      return response.payment_proof
    },
  })
  const review = useMutation({
    mutationFn: (decision: "verified" | "rejected") =>
      sdk.client.fetch(`/admin/orders/payment-proofs/${proofQuery.data?.id}/review`, {
        method: "POST",
        body: { decision },
      }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ordersQueryKeys.lists() }),
        queryClient.invalidateQueries({ queryKey: ["order-payment-proof", orderId] }),
      ])
      toast.success("Payment proof reviewed")
    },
    onError: (error: Error) =>
      toast.error("Could not review payment proof", { description: error.message }),
  })

  return (
    <div onClick={(event) => event.stopPropagation()}>
      <Button size="small" variant="secondary" onClick={() => setOpen((value) => !value)}>
        {open ? "Hide proof" : "Review payment"}
      </Button>
      {open && (
        <div className="absolute z-20 mt-2 w-80 rounded-lg border bg-ui-bg-base p-4 shadow-elevation-flyout">
          {proofQuery.isLoading ? (
            <Text>Loading payment reference…</Text>
          ) : proofQuery.error ? (
            <Text role="alert">Could not load payment reference.</Text>
          ) : !proofQuery.data ? (
            <Text>No manual payment reference was submitted.</Text>
          ) : proofQuery.data.status !== "pending_verification" ? (
            <Text>Reference status: {proofQuery.data.status}.</Text>
          ) : (
            <div className="grid gap-3">
              <Text>
                {proofQuery.data.payment_method} · {proofQuery.data.transaction_id}
              </Text>
              {proofQuery.data.proof_asset_url && (
                <a
                  href={proofQuery.data.proof_asset_url}
                  target="_blank"
                  rel="noreferrer"
                  className="text-ui-fg-interactive underline"
                >
                  View payment attachment
                </a>
              )}
              <div className="flex gap-2">
                <Button
                  size="small"
                  isLoading={review.isPending}
                  disabled={review.isPending}
                  onClick={() => review.mutate("verified")}
                >
                  Verify payment
                </Button>
                <Button
                  size="small"
                  variant="secondary"
                  isLoading={review.isPending}
                  disabled={review.isPending}
                  onClick={() => review.mutate("rejected")}
                >
                  Reject
                </Button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
