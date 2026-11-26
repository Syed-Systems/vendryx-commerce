"use server"

import { sdk } from "@lib/config"

export type StorefrontAppearance = {
  storefront_archetype: "baseline" | "editorial" | "dense_catalog"
  product_card_style: "standard" | "editorial" | "dense"
  product_image_aspect: "square" | "portrait" | "landscape"
}

export const DEFAULT_STOREFRONT_APPEARANCE: StorefrontAppearance = {
  storefront_archetype: "baseline",
  product_card_style: "standard",
  product_image_aspect: "portrait",
}

/** Reads tenant-scoped appearance at SSR; defaults are safe if the API is unavailable. */
export async function getStorefrontAppearance(host: string | null): Promise<StorefrontAppearance> {
  if (!host) return DEFAULT_STOREFRONT_APPEARANCE
  try {
    const { appearance } = await sdk.client.fetch<{
      appearance: Partial<StorefrontAppearance>
    }>("/store/appearance", {
      method: "GET",
      headers: { host },
      cache: "no-store",
    })
    return {
      storefront_archetype: appearance.storefront_archetype ?? "baseline",
      product_card_style: appearance.product_card_style ?? "standard",
      product_image_aspect: appearance.product_image_aspect ?? "portrait",
    }
  } catch {
    return DEFAULT_STOREFRONT_APPEARANCE
  }
}
