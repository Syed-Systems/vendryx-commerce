import { createCipheriv, createDecipheriv, createHash, randomBytes } from "node:crypto"

const PREFIX = "v1"
const AAD = Buffer.from("stheno_cart_id:v1", "utf8")

function cookieKey(secret: string): Buffer {
  if (Buffer.byteLength(secret, "utf8") < 32) {
    throw new Error("COOKIE_SECRET must contain at least 32 bytes")
  }

  return createHash("sha256")
    .update("stheno-cart-cookie\0", "utf8")
    .update(secret, "utf8")
    .digest()
}

export function encryptCartId(cartId: string, secret: string): string {
  if (!/^[A-Za-z0-9_-]{1,255}$/.test(cartId)) {
    throw new Error("Cart ID has an invalid format")
  }

  const iv = randomBytes(12)
  const cipher = createCipheriv("aes-256-gcm", cookieKey(secret), iv)
  cipher.setAAD(AAD)
  const ciphertext = Buffer.concat([
    cipher.update(cartId, "utf8"),
    cipher.final(),
  ])

  return [
    PREFIX,
    iv.toString("base64url"),
    cipher.getAuthTag().toString("base64url"),
    ciphertext.toString("base64url"),
  ].join(".")
}

export function decryptCartId(value: string, secret: string): string | null {
  try {
    const [version, encodedIv, encodedTag, encodedCiphertext, ...extra] = value.split(".")
    if (
      version !== PREFIX ||
      !encodedIv ||
      !encodedTag ||
      !encodedCiphertext ||
      extra.length > 0
    ) {
      return null
    }

    const iv = Buffer.from(encodedIv, "base64url")
    const tag = Buffer.from(encodedTag, "base64url")
    const ciphertext = Buffer.from(encodedCiphertext, "base64url")
    if (iv.length !== 12 || tag.length !== 16 || ciphertext.length < 1 || ciphertext.length > 255) {
      return null
    }

    const decipher = createDecipheriv("aes-256-gcm", cookieKey(secret), iv)
    decipher.setAAD(AAD)
    decipher.setAuthTag(tag)
    const plaintext = Buffer.concat([
      decipher.update(ciphertext),
      decipher.final(),
    ]).toString("utf8")

    return /^[A-Za-z0-9_-]{1,255}$/.test(plaintext) ? plaintext : null
  } catch {
    return null
  }
}

export function encryptCartIdFromEnvironment(cartId: string): string {
  const secret = process.env.COOKIE_SECRET
  if (!secret) {
    throw new Error("COOKIE_SECRET is required to encrypt the cart cookie")
  }
  return encryptCartId(cartId, secret)
}

export function decryptCartIdFromEnvironment(value: string): string | null {
  for (const secret of [process.env.COOKIE_SECRET, process.env.COOKIE_SECRET_PREVIOUS]) {
    if (!secret) {
      continue
    }
    const cartId = decryptCartId(value, secret)
    if (cartId) {
      return cartId
    }
  }
  return null
}
