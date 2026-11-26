import { Heading, Text } from "@medusajs/ui"
import TransferActions from "@frontend/order/components/transfer-actions"
import TransferImage from "@frontend/order/components/transfer-image"

export default async function TransferPage({
  params,
}: {
  params: { id: string; token: string }
}) {
  const { id, token } = params

  return (
    <div className="flex flex-col gap-y-4 items-start w-2/5 mx-auto mt-10 mb-20">
      <TransferImage />
      <div className="flex flex-col gap-y-6">
        <Heading level="h1" className="text-xl text-[color:var(--ink)]">
          Transfer request for order {id}
        </Heading>
        <Text className="text-[color:var(--ink-muted)]">
          You&#39;ve received a request to transfer ownership of your order ({id}).
          If you agree to this request, you can approve the transfer by clicking
          the button below.
        </Text>
        <div className="w-full h-px bg-[color:var(--brand-surface)]" />
        <Text className="text-[color:var(--ink-muted)]">
          If you accept, the new owner will take over all responsibilities and
          permissions associated with this order.
        </Text>
        <Text className="text-[color:var(--ink-muted)]">
          If you do not recognize this request or wish to retain ownership, no
          further action is required.
        </Text>
        <div className="w-full h-px bg-[color:var(--brand-surface)]" />
        <TransferActions id={id} token={token} />
      </div>
    </div>
  )
}
