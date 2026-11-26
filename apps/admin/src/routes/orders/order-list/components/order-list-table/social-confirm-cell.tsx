import { Button, toast } from "@medusajs/ui"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { sdk } from "../../../../../lib/client"
import { ordersQueryKeys } from "../../../../../hooks/api/orders"

export const SocialConfirmCell = ({ orderId }: { orderId: string }) => {
  const queryClient = useQueryClient()
  const confirm = useMutation({
    mutationFn: () =>
      sdk.client.fetch(`/admin/draft-orders/${orderId}/convert-to-order`, {
        method: "POST",
      }),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ordersQueryKeys.lists() }),
        queryClient.invalidateQueries({ queryKey: ordersQueryKeys.detail(orderId) }),
      ])
      toast.success("Social order confirmed")
    },
    onError: (error: Error) =>
      toast.error("Could not confirm social order", { description: error.message }),
  })

  return (
    <Button
      size="small"
      isLoading={confirm.isPending}
      onClick={(event) => {
        event.stopPropagation()
        confirm.mutate()
      }}
    >
      Confirm order
    </Button>
  )
}
