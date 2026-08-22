"use client"

import { useState } from "react"

import { REF_PARAM_KEY } from "@stheno/storefront-core"

/**
 * Social checkout (PRODUCT.md §5.2–5.4).
 *
 * The habit-forming free-tier path: the buyer messages the seller, a draft lands in the
 * admin's Social queue, and staff convert it after talking to them.
 *
 * Two platform facts drive the whole design and are worth stating, because getting them
 * wrong is silent:
 *
 * 1. **Only WhatsApp pre-fills message text from a plain deep link** (`wa.me/?text=`).
 * Messenger (`m.me`) and Instagram do not, and injecting text into Messenger needs a
 * Facebook App, a bot, page tokens and app review — an external dependency with
 * approval risk that breaks the free tier's zero-marginal-cost rule. So those two copy
 * to the clipboard and open the app.
 * 2. **`Ref` is the only link between the chat and the queued draft.** It comes from the
 * server with the draft, never from the client, so the code in the buyer's message is
 * provably the code on the order.
 */

export interface SocialCheckoutLine {
  variant_id: string
  title: string
  sku?: string
  options?: string[]
  quantity: number
  unit_price: string
  line_total: string
}

export interface SocialCheckoutProps {
  lines: SocialCheckoutLine[]
  total: string
  currencyCode: string
  regionId?: string
  salesChannelId?: string
  /** Digits only, no `+` or spaces — `wa.me` rejects anything else. */
  whatsappNumber?: string
  instagramHandle?: string
  messengerHandle?: string
  backendUrl: string
}

type Draft = { ref_code: string; message: string; order_id: string }

export const SocialCheckout: React.FC<SocialCheckoutProps> = ({
  lines,
  total,
  currencyCode,
  regionId,
  salesChannelId,
  whatsappNumber,
  instagramHandle,
  messengerHandle,
  backendUrl,
}) => {
  const [draft, setDraft] = useState<Draft | null>(null)
  const [pending, setPending] = useState(false)
  const [copied, setCopied] = useState(false)
  const [error, setError] = useState<string | null>(null)

  /**
   * Creates the draft once and reuses it, so a buyer who tries WhatsApp and then
   * Instagram does not queue the same order twice under two different codes.
   */
  const ensureDraft = async (): Promise<Draft | null> => {
    if (draft) {
      return draft
    }
    setPending(true)
    setError(null)
    try {
      const ref =
        typeof window !== "undefined"
          ? new URLSearchParams(window.location.search).get(REF_PARAM_KEY)
          : null

      const response = await fetch(`${backendUrl}/store/social-drafts`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          items: lines,
          total,
          currency_code: currencyCode,
          region_id: regionId,
          sales_channel_id: salesChannelId,
          source_ref: ref,
        }),
      })

      if (!response.ok) {
        const body = await response.json().catch(() => ({}))
        throw new Error(body.message || "Could not start the order.")
      }

      const created: Draft = (await response.json()).social_draft
      setDraft(created)
      return created
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not start the order.")
      return null
    } finally {
      setPending(false)
    }
  }

  const openWhatsApp = async () => {
    const created = await ensureDraft()
    if (!created || !whatsappNumber) {
      return
    }
    window.open(
      `https://wa.me/${whatsappNumber}?text=${encodeURIComponent(
        created.message
      )}`,
      "_blank",
      "noopener,noreferrer"
    )
  }

  const copyAndOpen = async (appUrl: string) => {
    const created = await ensureDraft()
    if (!created) {
      return
    }
    // `m.me` and Instagram cannot be pre-filled, so the message goes to the clipboard
    // and the buyer pastes it. Copy first: opening the app can move focus away and
    // clipboard writes need the document focused.
    try {
      await navigator.clipboard.writeText(created.message)
      setCopied(true)
      setTimeout(() => setCopied(false), 4000)
    } catch {
      // A denied clipboard permission must not block the handoff — the message is
      // shown below either way.
    }
    window.open(appUrl, "_blank", "noopener,noreferrer")
  }

  return (
    <div
      className="flex flex-col gap-y-3 rounded-none border p-4"
      data-testid="social-checkout"
    >
      <div>
        <h3 className="text-base-semi">Order by message</h3>
        <p className="text-small-regular text-ui-fg-subtle">
          Send us your order and we will confirm it in chat.
        </p>
      </div>

      <div className="flex flex-col gap-2 sm:flex-row">
        {whatsappNumber && (
          <button
            type="button"
            onClick={openWhatsApp}
            disabled={pending}
            className="w-full rounded-none bg-[#25D366] px-4 py-2 text-white disabled:opacity-60"
            data-testid="social-whatsapp"
          >
            {pending ? "Preparing…" : "WhatsApp"}
          </button>
        )}

        {instagramHandle && (
          <button
            type="button"
            onClick={() =>
              copyAndOpen(`https://instagram.com/${instagramHandle}`)
            }
            disabled={pending}
            className="w-full rounded-none border px-4 py-2 disabled:opacity-60"
            data-testid="social-instagram"
          >
            Copy &amp; open Instagram
          </button>
        )}

        {messengerHandle && (
          <button
            type="button"
            onClick={() => copyAndOpen(`https://m.me/${messengerHandle}`)}
            disabled={pending}
            className="w-full rounded-none border px-4 py-2 disabled:opacity-60"
            data-testid="social-messenger"
          >
            Copy &amp; open Messenger
          </button>
        )}
      </div>

      {copied && (
        <p
          className="text-small-regular text-ui-fg-interactive"
          data-testid="social-copied"
        >
          Order copied — paste it into the chat.
        </p>
      )}

      {error && (
        <p
          className="text-small-regular text-rose-600"
          data-testid="social-error"
        >
          {error}
        </p>
      )}

      {draft && (
        <div className="flex flex-col gap-y-1" data-testid="social-draft">
          <p className="text-small-regular text-ui-fg-subtle">
            Your reference:{" "}
            <span className="font-mono font-semibold" data-testid="social-ref">
              #{draft.ref_code}
            </span>
          </p>
          {/* Shown as well as sent: if the clipboard was denied or the app did not open,
 the buyer can still read the code out. */}
          <pre className="text-small-regular whitespace-pre-wrap rounded bg-ui-bg-subtle p-3">
            {draft.message}
          </pre>
        </div>
      )}
    </div>
  )
}

export default SocialCheckout
