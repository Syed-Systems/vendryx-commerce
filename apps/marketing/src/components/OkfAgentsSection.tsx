import {
  ArrowLeft,
  ArrowRight,
  Camera,
  CheckCheck,
  CheckCircle2,
  ChevronLeft,
  Image as ImageIcon,
  Info,
  Mic,
  MoreHorizontal,
  MoreVertical,
  Paperclip,
  Phone,
  PlusCircle,
  Send,
  Smile,
  Sparkles,
  ThumbsUp,
  Video,
} from "lucide-react"
import { useState } from "react"
import { SOCIAL_SCENARIOS, type SocialChatScenario } from "../data/marketing-content"
import type { SocialChannel } from "../lib/marketing-logic"

const CHANNEL_ORDER: SocialChannel[] = ["whatsapp", "facebook", "instagram", "tiktok"]

function WhatsAppChatBox({ scenario }: { scenario: SocialChatScenario }) {
  return (
    <div className="flex h-full flex-col overflow-hidden rounded-xl border border-[#2A3942] bg-[#0B141A] shadow-lg">
      {/* Authentic WhatsApp Dark Top Bar */}
      <div className="flex items-center justify-between bg-[#1F2C34] px-3.5 py-2.5">
        <div className="flex items-center gap-2.5 min-w-0">
          <ArrowLeft className="h-4 w-4 shrink-0 text-[#AEBAC1]" />
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[#00A884]/20 text-[13px] font-semibold text-[#00A884]">
            TA
          </div>
          <div className="min-w-0">
            <p className="truncate text-[14px] font-semibold text-[#E9EDEF]">
              {scenario.buyerName}
            </p>
            <p className="text-[11.5px] text-[#8696A0]">online</p>
          </div>
        </div>
        <div className="flex items-center gap-4 text-[#AEBAC1]">
          <Video className="h-4 w-4" />
          <Phone className="h-4 w-4" />
          <MoreVertical className="h-4 w-4" />
        </div>
      </div>

      {/* Authentic WhatsApp Dark Wallpaper Area */}
      <div
        className="flex flex-1 flex-col gap-3 px-4 py-4"
        style={{
          backgroundColor: "#0B141A",
          backgroundImage:
            "radial-gradient(rgba(255,255,255,0.03) 1px, transparent 1px), radial-gradient(rgba(255,255,255,0.025) 1px, transparent 1px)",
          backgroundSize: "24px 24px",
          backgroundPosition: "0 0, 12px 12px",
        }}
      >
        <div className="mx-auto rounded-lg bg-[#182229] px-3 py-1 text-[11px] font-medium text-[#8696A0]">
          Today
        </div>

        {/* Incoming Buyer Bubble */}
        <div className="max-w-[85%] self-start rounded-lg rounded-tl-none bg-[#202C33] px-3 py-2 text-[13.5px] leading-[1.45] text-[#E9EDEF] shadow-sm">
          <p>{scenario.buyerMessage}</p>
          <div className="mt-1 flex justify-end">
            <span className="text-[10.5px] text-[#8696A0]">10:42 AM</span>
          </div>
        </div>

        {/* Outgoing Store / AI Reply Bubble (WhatsApp Dark Green) */}
        <div className="max-w-[85%] self-end rounded-lg rounded-tr-none bg-[#005C4B] px-3 py-2 text-[13.5px] leading-[1.45] text-[#E9EDEF] shadow-sm">
          <p>{scenario.assistantReply}</p>
          <div className="mt-1 flex items-center justify-end gap-1">
            <span className="text-[10.5px] text-[#99BEB7]">10:42 AM</span>
            <CheckCheck className="h-3.5 w-3.5 text-[#53BDEB]" />
          </div>
        </div>
      </div>

      {/* Authentic WhatsApp Bottom Composer */}
      <div className="flex items-center gap-2 bg-[#0B141A] px-2.5 py-2">
        <div className="flex flex-1 items-center gap-2.5 rounded-full bg-[#2A3942] px-3.5 py-2 text-[13px] text-[#8696A0]">
          <Smile className="h-4 w-4 shrink-0" />
          <span className="flex-1 truncate">Message</span>
          <Paperclip className="h-4 w-4 shrink-0" />
          <Camera className="h-4 w-4 shrink-0" />
        </div>
        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[#00A884] text-[#0B141A]">
          <Mic className="h-4 w-4" />
        </div>
      </div>
    </div>
  )
}

function MessengerChatBox({ scenario }: { scenario: SocialChatScenario }) {
  return (
    <div className="flex h-full flex-col overflow-hidden rounded-xl border border-[#2D2F34] bg-[#000000] shadow-lg">
      {/* Authentic Messenger Dark Header */}
      <div className="flex items-center justify-between border-b border-[#242526] bg-[#1C1E21] px-3.5 py-2.5">
        <div className="flex items-center gap-2.5 min-w-0">
          <ChevronLeft className="h-5 w-5 shrink-0 text-[#0084FF]" />
          <div className="relative flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[#3A3B3C] text-[13px] font-semibold text-[#E4E6EB]">
            SI
            <span className="absolute bottom-0 right-0 h-2.5 w-2.5 rounded-full border-2 border-[#1C1E21] bg-[#31A24C]" />
          </div>
          <div className="min-w-0">
            <p className="truncate text-[14px] font-semibold text-[#E4E6EB]">
              {scenario.buyerName}
            </p>
            <p className="text-[11.5px] text-[#B0B3B8]">Active now</p>
          </div>
        </div>
        <div className="flex items-center gap-4 text-[#0084FF]">
          <Phone className="h-4 w-4" />
          <Video className="h-4 w-4" />
          <Info className="h-4 w-4" />
        </div>
      </div>

      {/* Authentic Messenger Message Area */}
      <div className="flex flex-1 flex-col gap-3 bg-[#18191A] px-4 py-4">
        <p className="text-center text-[11px] font-medium text-[#8A8D91]">Today 11:08 AM</p>

        {/* Incoming Buyer Message with Small Avatar */}
        <div className="flex max-w-[86%] items-end gap-2 self-start">
          <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#3A3B3C] text-[10px] font-semibold text-[#E4E6EB]">
            SI
          </div>
          <div className="rounded-[18px] rounded-bl-[4px] bg-[#303030] px-3.5 py-2.5 text-[13.5px] leading-[1.45] text-[#E4E6EB]">
            {scenario.buyerMessage}
          </div>
        </div>

        {/* Outgoing Messenger Blue Bubble */}
        <div className="flex max-w-[84%] flex-col items-end gap-1 self-end">
          <div
            className="rounded-[18px] rounded-br-[4px] px-3.5 py-2.5 text-[13.5px] leading-[1.45] text-white"
            style={{
              background: "linear-gradient(135deg, #0099FF 0%, #006AFF 100%)",
            }}
          >
            {scenario.assistantReply}
          </div>
          <span className="pr-1 text-[10.5px] text-[#8A8D91]">Seen</span>
        </div>
      </div>

      {/* Authentic Messenger Bottom Bar */}
      <div className="flex items-center gap-2.5 border-t border-[#242526] bg-[#1C1E21] px-3 py-2.5">
        <PlusCircle className="h-5 w-5 shrink-0 text-[#0084FF]" />
        <Camera className="h-5 w-5 shrink-0 text-[#0084FF]" />
        <ImageIcon className="h-5 w-5 shrink-0 text-[#0084FF]" />
        <div className="flex flex-1 items-center justify-between rounded-full bg-[#3A3B3C] px-3.5 py-1.5 text-[13px] text-[#B0B3B8]">
          <span>Aa</span>
          <Smile className="h-4 w-4 text-[#0084FF]" />
        </div>
        <ThumbsUp className="h-5 w-5 shrink-0 text-[#0084FF]" />
      </div>
    </div>
  )
}

function InstagramChatBox({ scenario }: { scenario: SocialChatScenario }) {
  return (
    <div className="flex h-full flex-col overflow-hidden rounded-xl border border-[#262626] bg-[#000000] shadow-lg">
      {/* Authentic Instagram DM Dark Header */}
      <div className="flex items-center justify-between border-b border-[#262626] bg-[#000000] px-3.5 py-2.5">
        <div className="flex items-center gap-2.5 min-w-0">
          <ChevronLeft className="h-5 w-5 shrink-0 text-[#F5F5F5]" />
          <div
            className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full p-[2px]"
            style={{
              background:
                "linear-gradient(45deg, #FEDA75 0%, #FA7E1E 30%, #D62976 65%, #962FBF 100%)",
            }}
          >
            <div className="flex h-full w-full items-center justify-center rounded-full bg-[#000000] text-[11px] font-semibold text-[#F5F5F5]">
              MS
            </div>
          </div>
          <div className="min-w-0">
            <p className="truncate text-[14px] font-semibold text-[#F5F5F5]">Maliha Studio</p>
            <p className="truncate text-[11.5px] text-[#A8A8A8]">{scenario.buyerName}</p>
          </div>
        </div>
        <div className="flex items-center gap-4 text-[#F5F5F5]">
          <Phone className="h-4 w-4" />
          <Video className="h-4 w-4" />
        </div>
      </div>

      {/* Authentic Instagram DM Body with Story Reply Context */}
      <div className="flex flex-1 flex-col gap-3 bg-[#000000] px-4 py-4">
        <div className="max-w-[85%] self-start">
          <p className="mb-1.5 pl-8 text-[11px] text-[#A8A8A8]">Replied to your story</p>
          <div className="mb-2 ml-8 flex w-fit items-center gap-2 rounded-xl border border-[#262626] bg-[#121212] px-2.5 py-1.5">
            <div className="h-8 w-6 shrink-0 rounded bg-gradient-to-b from-[#6B4423] to-[#2B1A10]" />
            <div className="text-[11px]">
              <p className="font-medium text-[#F5F5F5]">Espresso Weekender</p>
              <p className="text-[#A8A8A8]">৳6,800 · Story drop</p>
            </div>
          </div>
          <div className="flex items-end gap-2">
            <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#262626] text-[10px] font-semibold text-[#F5F5F5]">
              MS
            </div>
            <div className="rounded-[20px] bg-[#262626] px-3.5 py-2.5 text-[13.5px] leading-[1.45] text-[#F5F5F5]">
              {scenario.buyerMessage}
            </div>
          </div>
        </div>

        {/* Outgoing Instagram Purple/Violet Gradient Bubble */}
        <div className="flex max-w-[84%] flex-col items-end gap-1 self-end">
          <div
            className="rounded-[20px] px-3.5 py-2.5 text-[13.5px] leading-[1.45] text-white"
            style={{
              background: "linear-gradient(135deg, #6E42F5 0%, #9E2AF2 100%)",
            }}
          >
            {scenario.assistantReply}
          </div>
          <span className="pr-1 text-[10.5px] text-[#A8A8A8]">Seen just now</span>
        </div>
      </div>

      {/* Authentic Instagram Pill Input Bar */}
      <div className="bg-[#000000] px-3 py-2.5">
        <div className="flex items-center gap-2.5 rounded-full border border-[#363636] bg-[#121212] py-1.5 pl-1.5 pr-3.5">
          <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[#3797F0] text-white">
            <Camera className="h-3.5 w-3.5" />
          </div>
          <span className="flex-1 text-[13px] text-[#A8A8A8]">Message...</span>
          <Mic className="h-4 w-4 text-[#F5F5F5]" />
          <ImageIcon className="h-4 w-4 text-[#F5F5F5]" />
          <Smile className="h-4 w-4 text-[#F5F5F5]" />
        </div>
      </div>
    </div>
  )
}

function TikTokChatBox({ scenario }: { scenario: SocialChatScenario }) {
  return (
    <div className="flex h-full flex-col overflow-hidden rounded-xl border border-[#2B2B2E] bg-[#121212] shadow-lg">
      {/* Authentic TikTok Dark DM Header */}
      <div className="flex items-center justify-between border-b border-[#262629] bg-[#121212] px-3.5 py-2.5">
        <div className="flex items-center gap-2.5 min-w-0">
          <ChevronLeft className="h-5 w-5 shrink-0 text-white" />
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-[#FE2C55]/60 bg-[#252529] text-[12px] font-semibold text-white">
            RK
          </div>
          <div className="min-w-0">
            <p className="truncate text-[14px] font-semibold text-white">
              {scenario.buyerName}
            </p>
            <p className="text-[11px] text-[#A1A1AA]">Message from TikTok Live</p>
          </div>
        </div>
        <MoreHorizontal className="h-5 w-5 text-white" />
      </div>

      {/* Authentic TikTok DM Thread */}
      <div className="flex flex-1 flex-col gap-3 bg-[#121212] px-4 py-4">
        <div className="mx-auto flex items-center gap-1.5 text-[11px] text-[#8E8E93]">
          <span className="rounded bg-[#FE2C55] px-1.5 py-0.5 text-[9.5px] font-bold uppercase tracking-wide text-white">
            LIVE
          </span>
          <span>Today 8:14 PM</span>
        </div>

        {/* Incoming TikTok Buyer Bubble */}
        <div className="flex max-w-[86%] items-start gap-2 self-start">
          <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[#252529] text-[10px] font-semibold text-white">
            RK
          </div>
          <div className="rounded-2xl rounded-tl-sm bg-[#262629] px-3.5 py-2.5 text-[13.5px] leading-[1.45] text-white">
            {scenario.buyerMessage}
          </div>
        </div>

        {/* Outgoing TikTok Blue DM Bubble */}
        <div className="flex max-w-[84%] flex-col items-end gap-1 self-end">
          <div className="rounded-2xl rounded-tr-sm bg-[#1F9CF0] px-3.5 py-2.5 text-[13.5px] leading-[1.45] text-white">
            {scenario.assistantReply}
          </div>
          <span className="pr-1 text-[10.5px] text-[#8E8E93]">Delivered</span>
        </div>
      </div>

      {/* Authentic TikTok Bottom Composer */}
      <div className="flex items-center gap-2 border-t border-[#262629] bg-[#121212] px-3 py-2.5">
        <div className="flex flex-1 items-center justify-between rounded-full bg-[#262629] px-3.5 py-2 text-[13px] text-[#8E8E93]">
          <span>Send a message...</span>
          <Smile className="h-4 w-4 text-white/80" />
        </div>
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[#FE2C55] text-white">
          <Send className="h-3.5 w-3.5" />
        </div>
      </div>
    </div>
  )
}

export function OkfAgentsSection() {
  const [activeChannel, setActiveChannel] = useState<SocialChannel>("whatsapp")
  const [confirmedMap, setConfirmedMap] = useState<Record<string, boolean>>({})

  const scenario = SOCIAL_SCENARIOS[activeChannel]
  const isConfirmed = Boolean(confirmedMap[activeChannel])

  return (
    <section
      id="social-okf"
      className="scroll-mt-16 border-b border-[#FFFCF8]/[0.08] py-16 sm:py-20"
    >
      <div className="mx-auto max-w-[1200px] px-4 sm:px-6">
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_1.15fr] lg:items-end">
          <div>
            <h2 className="text-[28px] font-semibold leading-[1.12] tracking-[-0.03em] text-[#FFFCF8] sm:text-[36px]">
              Turn WhatsApp, Facebook, Instagram, and TikTok chats into ready-to-ship orders.
            </h2>
          </div>
          <p className="text-[15px] leading-[1.6] text-[#94A3B8]">
            Most store chatbots guess answers. Vendryx’s{" "}
            <span className="font-medium text-[#FFFCF8]">AI Store Assistant</span> uses our
            structured{" "}
            <span className="font-medium text-[#FFFCF8]">Open Knowledge Format (OKF)</span> so it
            always knows your exact products, sizes, live stock, and delivery rates—replying in
            Bengali, Banglish, or English and sending clean orders straight to your dashboard.
          </p>
        </div>

        {/* Channel Switcher using official /branding/social/*.svg icons */}
        <div
          className="mt-8 flex flex-wrap gap-2"
          role="tablist"
          aria-label="Social selling channels"
        >
          {CHANNEL_ORDER.map((channelKey) => {
            const item = SOCIAL_SCENARIOS[channelKey]
            const active = activeChannel === channelKey
            return (
              <button
                key={channelKey}
                type="button"
                role="tab"
                aria-selected={active}
                onClick={() => setActiveChannel(channelKey)}
                className={`inline-flex shrink-0 items-center gap-2 rounded-md border px-3.5 py-2 text-[13px] font-medium transition-colors ${
                  active
                    ? "border-[#FF7A00] bg-[#1B1E24] text-[#FFFCF8]"
                    : "border-[#FFFCF8]/[0.08] bg-[#0F1115] text-[#94A3B8] hover:border-[#FFFCF8]/20 hover:text-[#FFFCF8]"
                }`}
              >
                <img
                  src={item.iconPath}
                  alt=""
                  width={16}
                  height={16}
                  className={`h-4 w-4 shrink-0 brightness-0 invert transition-opacity ${
                    active ? "opacity-100" : "opacity-75"
                  }`}
                />
                <span className="whitespace-nowrap">{item.label}</span>
              </button>
            )
          })}
        </div>

        {/* Authentic Native Chat Window + Ready-to-Confirm Dashboard Order */}
        <div className="mt-5 grid grid-cols-1 gap-4 lg:grid-cols-2">
          {/* Left: Authentic App-Native Chat Window */}
          <div className="min-h-[380px]">
            {activeChannel === "whatsapp" && <WhatsAppChatBox scenario={scenario} />}
            {activeChannel === "facebook" && <MessengerChatBox scenario={scenario} />}
            {activeChannel === "instagram" && <InstagramChatBox scenario={scenario} />}
            {activeChannel === "tiktok" && <TikTokChatBox scenario={scenario} />}
          </div>

          {/* Right: Clean Order Summary Ready for Dashboard */}
          <div className="flex flex-col justify-between rounded-xl border border-[#FFFCF8]/[0.08] bg-[#0F1115] p-5">
            <div>
              <div className="flex items-center justify-between border-b border-[#FFFCF8]/[0.08] pb-3">
                <span className="text-[14px] font-semibold text-[#FFFCF8]">
                  Order Summary in Your Dashboard
                </span>
                <span
                  className={`font-mono text-[11px] font-medium ${
                    isConfirmed ? "text-[#4ADE80]" : "text-[#FFAF24]"
                  }`}
                >
                  {isConfirmed ? "Confirmed" : "Waiting for approval"}
                </span>
              </div>

              <dl className="mt-4 divide-y divide-[#FFFCF8]/[0.06] text-[13.5px]">
                <div className="flex items-center justify-between gap-4 py-2.5">
                  <dt className="text-[#94A3B8]">Order code</dt>
                  <dd className="font-mono font-semibold text-[#FFAF24]">
                    {scenario.summary.orderCode}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-4 py-2.5">
                  <dt className="text-[#94A3B8]">Product</dt>
                  <dd className="text-right font-medium text-[#FFFCF8]">
                    {scenario.summary.product}
                  </dd>
                </div>
                <div className="flex items-center justify-between gap-4 py-2.5">
                  <dt className="text-[#94A3B8]">Option</dt>
                  <dd className="text-right text-[#CBD5E1]">{scenario.summary.option}</dd>
                </div>
                <div className="flex items-center justify-between gap-4 py-2.5">
                  <dt className="text-[#94A3B8]">Delivery area</dt>
                  <dd className="text-right text-[#CBD5E1]">{scenario.summary.deliveryArea}</dd>
                </div>
                <div className="flex items-center justify-between gap-4 py-2.5">
                  <dt className="text-[#94A3B8]">Payment</dt>
                  <dd className="text-right text-[#CBD5E1]">{scenario.summary.paymentMethod}</dd>
                </div>
                <div className="flex items-center justify-between gap-4 py-2.5">
                  <dt className="text-[#94A3B8]">Stock check</dt>
                  <dd className="text-right text-[#CBD5E1]">{scenario.summary.stockStatus}</dd>
                </div>
                <div className="flex items-center justify-between gap-4 pt-3">
                  <dt className="font-semibold text-[#FFFCF8]">Order Total</dt>
                  <dd className="font-mono text-[17px] font-semibold tabular-nums text-[#FFFCF8]">
                    {scenario.summary.total}
                  </dd>
                </div>
              </dl>
            </div>

            <div className="mt-5 space-y-3">
              {isConfirmed ? (
                <div className="flex items-center justify-between rounded-md border border-[#4ADE80]/30 bg-[#4ADE80]/10 px-3.5 py-2.5 text-[13px] font-medium text-[#4ADE80]">
                  <span className="inline-flex items-center gap-2">
                    <CheckCircle2 className="h-4 w-4 shrink-0" />
                    Order confirmed and ready to hand to courier
                  </span>
                  <button
                    type="button"
                    onClick={() =>
                      setConfirmedMap((prev) => ({ ...prev, [activeChannel]: false }))
                    }
                    className="text-[12px] underline opacity-80 hover:opacity-100"
                  >
                    Reset
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  onClick={() =>
                    setConfirmedMap((prev) => ({ ...prev, [activeChannel]: true }))
                  }
                  className="inline-flex w-full items-center justify-center gap-2 rounded-md bg-[#FF7A00] px-4 py-2.5 text-[13.5px] font-semibold text-[#080A0C] transition-colors hover:bg-[#D95700]"
                >
                  Confirm order in 1 click
                  <ArrowRight className="h-4 w-4" />
                </button>
              )}

              <p className="flex items-center gap-2 text-[12px] text-[#94A3B8]">
                <Sparkles className="h-3.5 w-3.5 shrink-0 text-[#FF7A00]" />
                <span>
                  On the Free plan, WhatsApp &amp; Messenger orders still arrive in your Social
                  Commerce tab for 1-click approval.
                </span>
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
