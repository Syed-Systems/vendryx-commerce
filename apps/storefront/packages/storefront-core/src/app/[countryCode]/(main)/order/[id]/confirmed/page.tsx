import { retrieveOrder, retrieveOrderPaymentProof } from "@lib/data/orders"
import { cookies as nextCookies } from "next/headers"
import OrderCompletedTemplate from "@frontend/order/templates/order-completed-template"
import { Metadata } from "next"
import { notFound } from "next/navigation"

type Props = {
  params: Promise<{ id: string }>
}
export const metadata: Metadata = {
  title: "Order Confirmed",
  description: "You purchase was successful",
}

export default async function OrderConfirmedPage(props: Props) {
  const params = await props.params
  const order = await retrieveOrder(params.id).catch(() => null)

  if (!order) {
    return notFound()
  }

  const cookies = await nextCookies()
  const isOnboarding = cookies.get("_stheno_onboarding")?.value === "true"
  const paymentProof = order.customer_id
    ? await retrieveOrderPaymentProof(order.id)
    : null

  return (
    <OrderCompletedTemplate
      order={order}
      isOnboarding={isOnboarding}
      paymentProof={paymentProof}
    />
  )
}
