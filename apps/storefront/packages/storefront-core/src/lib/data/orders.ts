"use server"

import { sdk } from "@lib/config"
import medusaError from "@lib/util/medusa-error"
import { getAuthHeaders, getCacheOptions } from "./cookies"
import { HttpTypes } from "@medusajs/types"
import { revalidateTag } from "next/cache"
import { getCacheTag } from "./cookies"

export type OrderPaymentProof = {
  id: string
  order_id: string
  payment_method: string
  transaction_id: string
  proof_asset_url: string | null
  status: "pending_verification" | "verified" | "rejected"
  submitted_at: string
}

export const retrieveOrderPaymentProof = async (orderId: string) => {
  const headers = await getAuthHeaders()

  return sdk.client
    .fetch<{ payment_proof: OrderPaymentProof | null }>(
      `/store/orders/${encodeURIComponent(orderId)}/payment-proof`,
      { method: "GET", headers, cache: "no-store" }
    )
    .then(({ payment_proof }) => payment_proof)
    .catch(() => null)
}

export const submitManualOrderPaymentProof = async (
  orderId: string,
  formData: FormData
) => {
  const headers = await getAuthHeaders()
  const payment_method = formData.get("payment_method")
  const transaction_id = formData.get("transaction_id")
  const proof_asset_url = formData.get("proof_asset_url")

  if (
    typeof payment_method !== "string" ||
    typeof transaction_id !== "string" ||
    (proof_asset_url !== null && typeof proof_asset_url !== "string")
  ) {
    return { success: false, error: "Enter a valid payment reference." }
  }

  try {
    const result = await sdk.client.fetch<{ id: string; status: string }>(
      `/store/orders/${encodeURIComponent(orderId)}/payment-proof`,
      {
        method: "POST",
        headers,
        body: { payment_method, transaction_id, proof_asset_url },
        cache: "no-store",
      }
    )
    revalidateTag(await getCacheTag("orders"))
    return {
      success: true,
      error: null,
      status: result.status,
    }
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : "Unable to submit payment proof.",
    }
  }
}

export const retrieveOrder = async (id: string) => {
  const headers = {
    ...(await getAuthHeaders()),
  }

  const next = {
    ...(await getCacheOptions("orders")),
  }

  return sdk.client
    .fetch<HttpTypes.StoreOrderResponse>(`/store/orders/${id}`, {
      method: "GET",
      query: {
        fields:
          "*payment_collections.payments,*payment_collections.payment_sessions,*items,*items.metadata,*items.variant,*items.product",
      },
      headers,
      next,
      cache: "force-cache",
    })
    .then(({ order }) => order)
    .catch((err) => medusaError(err))
}

export const listOrders = async (
  limit: number = 10,
  offset: number = 0,
  filters?: Record<string, any>
) => {
  const headers = {
    ...(await getAuthHeaders()),
  }

  const next = {
    ...(await getCacheOptions("orders")),
  }

  return sdk.client
    .fetch<HttpTypes.StoreOrderListResponse>(`/store/orders`, {
      method: "GET",
      query: {
        limit,
        offset,
        order: "-created_at",
        fields: "*items,+items.metadata,*items.variant,*items.product",
        ...filters,
      },
      headers,
      next,
      cache: "force-cache",
    })
    .then(({ orders }) => orders)
    .catch((err) => medusaError(err))
}

export const createTransferRequest = async (
  state: {
    success: boolean
    error: string | null
    order: HttpTypes.StoreOrder | null
  },
  formData: FormData
): Promise<{
  success: boolean
  error: string | null
  order: HttpTypes.StoreOrder | null
}> => {
  const id = formData.get("order_id") as string

  if (!id) {
    return { success: false, error: "Order ID is required", order: null }
  }

  const headers = await getAuthHeaders()

  return await sdk.store.order
    .requestTransfer(
      id,
      {},
      {
        fields: "id, email",
      },
      headers
    )
    .then(({ order }) => ({ success: true, error: null, order }))
    .catch((err) => ({ success: false, error: err.message, order: null }))
}

export const acceptTransferRequest = async (id: string, token: string) => {
  const headers = await getAuthHeaders()

  return await sdk.store.order
    .acceptTransfer(id, { token }, {}, headers)
    .then(({ order }) => ({ success: true, error: null, order }))
    .catch((err) => ({ success: false, error: err.message, order: null }))
}

export const declineTransferRequest = async (id: string, token: string) => {
  const headers = await getAuthHeaders()

  return await sdk.store.order
    .declineTransfer(id, { token }, {}, headers)
    .then(({ order }) => ({ success: true, error: null, order }))
    .catch((err) => ({ success: false, error: err.message, order: null }))
}
