import "server-only"
import { cookies as nextCookies } from "next/headers"
import {
  decryptCartIdFromEnvironment,
  encryptCartIdFromEnvironment,
} from "./cart-cookie-crypto"

export const getAuthHeaders = async (): Promise<
  { authorization: string } | {}
> => {
  try {
    const cookies = await nextCookies()
    const token = cookies.get("_stheno_jwt")?.value

    if (!token) {
      return {}
    }

    return { authorization: `Bearer ${token}` }
  } catch {
    return {}
  }
}

export const getCacheTag = async (tag: string): Promise<string> => {
  // Public catalog and navigation tags must NOT invoke nextCookies().
  // Doing so forces Next.js Server Components to dynamic SSR per request,
  // bypassing the Full Route Cache and breaking CDN edge caching (ADR-024).
  const isPublicCatalogTag = [
    "products",
    "categories",
    "collections",
    "regions",
    "locales",
    "variants",
    "shippingOptions",
  ].some((t) => tag.startsWith(t))

  if (isPublicCatalogTag) {
    return tag
  }

  try {
    const cookies = await nextCookies()
    const cacheId = cookies.get("_stheno_cache_id")?.value

    if (!cacheId) {
      return tag
    }

    return `${tag}-${cacheId}`
  } catch (error) {
    return tag
  }
}

export const getCacheOptions = async (
  tag: string
): Promise<{ tags: string[] } | {}> => {
  if (typeof window !== "undefined") {
    return {}
  }

  const cacheTag = await getCacheTag(tag)

  if (!cacheTag) {
    return {}
  }

  return { tags: [`${cacheTag}`] }
}

export const setAuthToken = async (token: string) => {
  const cookies = await nextCookies()
  cookies.set("_stheno_jwt", token, {
    maxAge: 60 * 60 * 24 * 7,
    httpOnly: true,
    sameSite: "strict",
    secure: process.env.NODE_ENV === "production",
  })
}

export const removeAuthToken = async () => {
  const cookies = await nextCookies()
  cookies.set("_stheno_jwt", "", {
    maxAge: -1,
  })
}

export const getCartId = async () => {
  const cookies = await nextCookies()
  const value = cookies.get("_stheno_cart_id")?.value
  if (!value) {
    return undefined
  }

  if (value.startsWith("v1.")) {
    return decryptCartIdFromEnvironment(value) ?? undefined
  }

  // Existing seven-day cookies remain readable through rollout, but malformed values
  // are never sent to the Store API as cart identifiers.
  return /^[A-Za-z0-9_-]{1,255}$/.test(value) ? value : undefined
}

export const setCartId = async (cartId: string) => {
  const cookies = await nextCookies()
  cookies.set("_stheno_cart_id", encryptCartIdFromEnvironment(cartId), {
    maxAge: 60 * 60 * 24 * 7,
    httpOnly: true,
    sameSite: "strict",
    secure: process.env.NODE_ENV === "production",
  })
}

export const removeCartId = async () => {
  const cookies = await nextCookies()
  cookies.set("_stheno_cart_id", "", {
    maxAge: -1,
  })
}
