import { describe, expect, it } from "vitest"
import {
  decryptCartId,
  encryptCartId,
} from "../../packages/storefront-core/src/lib/data/cart-cookie-crypto"

const secret = "0123456789abcdef0123456789abcdef"

describe("encrypted cart cookie values", () => {
  it("round trips IDs without storing the plaintext and uses a fresh nonce", () => {
    const first = encryptCartId("cart_01J9ABCDEF0123456789", secret)
    const second = encryptCartId("cart_01J9ABCDEF0123456789", secret)

    expect(first).not.toContain("cart_01J9ABCDEF0123456789")
    expect(first).not.toBe(second)
    expect(decryptCartId(first, secret)).toBe("cart_01J9ABCDEF0123456789")
  })

  it("rejects tampering, malformed values, and a different deployment key", () => {
    const encrypted = encryptCartId("cart_01J9ABCDEF0123456789", secret)
    const parts = encrypted.split(".")
    parts[3] = `${parts[3][0] === "A" ? "B" : "A"}${parts[3].slice(1)}`

    expect(decryptCartId(parts.join("."), secret)).toBeNull()
    expect(decryptCartId(encrypted, `${secret}different`)).toBeNull()
    expect(decryptCartId("v1.not-a-payload", secret)).toBeNull()
  })

  it("rejects weak keys and invalid IDs when encrypting", () => {
    expect(() => encryptCartId("cart_123", "short")).toThrow(/COOKIE_SECRET/)
    expect(() => encryptCartId("cart id", secret)).toThrow(/Cart ID/)
  })
})
