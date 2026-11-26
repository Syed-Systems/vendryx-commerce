import type { SocialChannel, StorefrontAccentId } from "../lib/marketing-logic"

export interface DashboardOrderRow {
  id: string
  displayId: string
  customer: string
  phone: string
  timeAgo: string
  source: "Instagram DM" | "WhatsApp" | "Storefront · COD" | "Storefront · bKash"
  itemSummary: string
  amountNum: number
  amountFormatted: string
  status: "pending" | "completed"
  courier: "Pathao" | "Steadfast" | "RedX"
  area: string
}

export const INITIAL_DASHBOARD_ORDERS: DashboardOrderRow[] = [
  {
    id: "ord-1048",
    displayId: "#1048",
    customer: "Nadia Rahman",
    phone: "01711-842910",
    timeAgo: "2 mins ago",
    source: "Instagram DM",
    itemSummary: "Jamdani Linen Overshirt · Olive (M)",
    amountNum: 3450,
    amountFormatted: "৳3,450",
    status: "pending",
    courier: "Pathao",
    area: "Dhanmondi 27, Dhaka",
  },
  {
    id: "ord-1047",
    displayId: "#1047",
    customer: "Farhan Chowdhury",
    phone: "01819-330412",
    timeAgo: "11 mins ago",
    source: "WhatsApp",
    itemSummary: "Leather Weekender Bag · Espresso",
    amountNum: 6800,
    amountFormatted: "৳6,800",
    status: "pending",
    courier: "Steadfast",
    area: "Uttara Sector 7, Dhaka",
  },
  {
    id: "ord-1046",
    displayId: "#1046",
    customer: "Tasnim Kabir",
    phone: "01912-650821",
    timeAgo: "38 mins ago",
    source: "Storefront · COD",
    itemSummary: "Raw Silk Kurta Set · Ivory (L) × 2",
    amountNum: 5200,
    amountFormatted: "৳5,200",
    status: "completed",
    courier: "Pathao",
    area: "Banani Block E, Dhaka",
  },
  {
    id: "ord-1045",
    displayId: "#1045",
    customer: "Arefin Hossain",
    phone: "01674-902145",
    timeAgo: "1 hr ago",
    source: "Storefront · bKash",
    itemSummary: "Cotton Oversized Jacket · Charcoal (L)",
    amountNum: 4900,
    amountFormatted: "৳4,900",
    status: "completed",
    courier: "RedX",
    area: "Agrabad, Chattogram",
  },
]

export interface SocialChatScenario {
  channel: SocialChannel
  label: string
  iconPath: string
  badgeLabel: string
  buyerName: string
  buyerMessage: string
  assistantReply: string
  summary: {
    orderCode: string
    product: string
    option: string
    deliveryArea: string
    paymentMethod: string
    stockStatus: string
    total: string
  }
}

export const SOCIAL_SCENARIOS: Record<SocialChannel, SocialChatScenario> = {
  whatsapp: {
    channel: "whatsapp",
    label: "WhatsApp",
    iconPath: "/branding/social/whatsapp.svg",
    badgeLabel: "Pre-filled WhatsApp Order + AI Reply",
    buyerName: "Tanvir Ahmed",
    buyerMessage:
      "Hi, I want to order Jamdani Linen Overshirt (Olive, Size M) × 1 — Uttara Sector 7, Dhaka. Ref #W9P2",
    assistantReply:
      "Got it! 1 Olive Linen Overshirt (M) is in stock at ৳2,450 + ৳60 Inside Dhaka delivery. We've saved your order #W9P2 and sent it to the store team for quick confirmation.",
    summary: {
      orderCode: "#W9P2",
      product: "Jamdani Linen Overshirt",
      option: "Olive · Size M (× 1)",
      deliveryArea: "Uttara Sector 7, Dhaka",
      paymentMethod: "Cash on Delivery",
      stockStatus: "1 item held for 15 mins",
      total: "৳2,510",
    },
  },
  facebook: {
    channel: "facebook",
    label: "Facebook & Messenger",
    iconPath: "/branding/social/facebook.svg",
    badgeLabel: "AI Assistant (Powered by OKF)",
    buyerName: "Sadia Islam",
    buyerMessage:
      "Vaiya ei olive linen shirt ta M size available? Dhanmondi 27 e COD pathano jabe? 01711-842910",
    assistantReply:
      "Yes! Olive Linen Overshirt (Size M) is available for ৳2,450. Delivery to Dhanmondi 27 is ৳60 (24h via Pathao). I've held 1 piece for you — reply YES to confirm your order.",
    summary: {
      orderCode: "#F8K3",
      product: "Jamdani Linen Overshirt",
      option: "Olive · Size M (× 1)",
      deliveryArea: "Dhanmondi 27, Dhaka",
      paymentMethod: "Cash on Delivery · Verified phone",
      stockStatus: "1 item held for 15 mins",
      total: "৳2,510",
    },
  },
  instagram: {
    channel: "instagram",
    label: "Instagram",
    iconPath: "/branding/social/instagram.svg",
    badgeLabel: "Story Reply to Order",
    buyerName: "@maliha.studio",
    buyerMessage:
      "Saw the Espresso Weekender bag on your story — can I pay with bKash and get delivery to Banani tomorrow?",
    assistantReply:
      "Yes, the Espresso Leather Weekender is ready to ship! Total with Banani delivery is ৳6,860. Here is your quick bKash payment link—once paid, we'll book Pathao for tomorrow.",
    summary: {
      orderCode: "#G4T7",
      product: "Leather Weekender Bag",
      option: "Espresso · Standard (× 1)",
      deliveryArea: "Banani Block E, Dhaka",
      paymentMethod: "bKash (0% Vendryx fee)",
      stockStatus: "1 item held for 15 mins",
      total: "৳6,860",
    },
  },
  tiktok: {
    channel: "tiktok",
    label: "TikTok",
    iconPath: "/branding/social/tiktok.svg",
    badgeLabel: "Live Video Order Capture",
    buyerName: "@rafi.kicks",
    buyerMessage:
      "Size 42 charcoal runner lagbe 2 pair, Sylhet sadar delivery koto porbe?",
    assistantReply:
      "We have 2 pairs of Charcoal Runner (Size 42) available right now! Delivery to Sylhet Sadar is ৳160 via RedX. Share your phone number or pay via SSLCommerz to lock both pairs.",
    summary: {
      orderCode: "#T6N2",
      product: "Charcoal Runner Sneaker",
      option: "Charcoal · Size 42 (× 2)",
      deliveryArea: "Sylhet Sadar (Outside Dhaka)",
      paymentMethod: "SSLCommerz or Cash on Delivery",
      stockStatus: "2 pairs held · no overselling",
      total: "৳7,560",
    },
  },
}

export interface StorefrontBrandPreset {
  id: StorefrontAccentId
  name: string
  domain: string
  bg: string
  border: string
  text: string
  muted: string
  accent: string
  accentText: string
  tierLabel: string
}

export const STOREFRONT_BRAND_PRESETS: Record<StorefrontAccentId, StorefrontBrandPreset> = {
  vendryx: {
    id: "vendryx",
    name: "Charcoal & Orange",
    domain: "dhakaloom.vendryx.shop",
    bg: "#0F1115",
    border: "rgba(255,252,248,0.10)",
    text: "#FFFCF8",
    muted: "#94A3B8",
    accent: "#FF7A00",
    accentText: "#080A0C",
    tierLabel: "Included Free",
  },
  emerald: {
    id: "emerald",
    name: "Studio Dark",
    domain: "atelier-dhaka.com",
    bg: "#111618",
    border: "rgba(255,252,248,0.10)",
    text: "#FFFCF8",
    muted: "#94A3B8",
    accent: "#16A34A",
    accentText: "#FFFFFF",
    tierLabel: "Custom Domain · Pro",
  },
  ivory: {
    id: "ivory",
    name: "Warm White",
    domain: "vervewear.co",
    bg: "#FFFCF8",
    border: "rgba(15,17,21,0.12)",
    text: "#0F1115",
    muted: "#64748B",
    accent: "#FF7A00",
    accentText: "#FFFFFF",
    tierLabel: "Custom Storefront · Pro",
  },
}

export const RELIABILITY_PILLARS = [
  {
    number: "01",
    title: "Never oversell during busy drops",
    description:
      "The moment a customer starts checking out, Vendryx holds that item for 15 minutes so two shoppers never buy your last piece.",
    detail: "Automatic stock hold & release",
  },
  {
    number: "02",
    title: "Never double-charge a buyer",
    description:
      "Even if a customer's mobile data drops mid-payment, Vendryx checks the payment safely so they are never charged twice.",
    detail: "Protected bKash, card & COD checkout",
  },
  {
    number: "03",
    title: "Your store data stays private",
    description:
      "Your products, customer phone numbers, and payment keys stay strictly isolated and encrypted—never shared with other stores.",
    detail: "Encrypted keys · Private store space",
  },
  {
    number: "04",
    title: "Fast on every phone",
    description:
      "Built for speed so your store opens quickly on 3G and 4G mobile networks—keeping shoppers from abandoning their cart.",
    detail: "Mobile-first storefront & dashboard",
  },
] as const

export const PAYMENT_PROVIDERS = [
  {
    name: "bKash",
    logo: "/branding/providers/bkash.svg",
    note: "Instant mobile wallet payments straight to you",
  },
  {
    name: "SSLCommerz",
    logo: "/branding/providers/sslcommerz.svg",
    note: "Local cards, bank transfers & mobile wallets",
  },
  {
    name: "Stripe",
    logo: "/branding/providers/stripe.svg",
    note: "Accept international cards from anywhere",
  },
  {
    name: "Nagad",
    logo: "/branding/providers/nagad.svg",
    note: "Direct Nagad merchant checkout support",
  },
] as const

export const COURIER_PROVIDERS = [
  { name: "Pathao", logo: "/branding/providers/pathao.svg" },
  { name: "Steadfast", logo: "/branding/providers/steadfast.svg" },
  { name: "RedX", logo: "/branding/providers/redx.svg" },
] as const

export const SPREADSHEET_MAPPINGS = [
  { sellerCol: "Item Name", vendryxField: "Product Name", status: "Matched" },
  { sellerCol: "Color / Size", vendryxField: "Variants & Sizes", status: "Grouped" },
  { sellerCol: "Price (BDT)", vendryxField: "Selling Price", status: "Ready" },
  { sellerCol: "Extra Notes", vendryxField: "Ignored cleanly", status: "Safe skip" },
  { sellerCol: "Publish to Store", vendryxField: "Live Storefront", status: "Automatic" },
] as const
