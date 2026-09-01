export type DemoProduct = {
  id: string
  title: string
  price: number
  stock: number
  category: string
  status: "Published" | "Draft"
  description: string
}

export type DemoOrder = {
  id: string
  customer: string
  email: string
  phone: string
  items: Array<{ title: string; quantity: number; price: number }>
  total: number
  status: "Received" | "In Packing" | "Shipped" | "Completed"
  placedAt: string
}

export type DemoSocialMessage = {
  id: string
  channel: "WhatsApp" | "Instagram"
  customer: string
  phone: string
  message: string
  proposedItem: string
  price: number
  converted: boolean
}

export type DemoSession = {
  id: string
  createdAt: number
  updatedAt: number
  storeName: string
  accent: string
  announcement: string
  heroTitle: string
  ctaLabel: string
  currency: string
  products: DemoProduct[]
  orders: DemoOrder[]
  conversations: DemoSocialMessage[]
}

const prefix = "stheno-demo-session:"
const maxAgeMs = 2 * 60 * 60 * 1000

const createId = () => {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) return crypto.randomUUID()
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`
}

export function createDemoSession(initial?: Partial<DemoSession>): DemoSession {
  const now = Date.now()
  const session: DemoSession = {
    id: createId(),
    createdAt: now,
    updatedAt: now,
    storeName: initial?.storeName ?? "Stheno Studio",
    accent: "#FF6B00",
    announcement: "A considered collection, prepared with care.",
    heroTitle: "Small details. A store that feels like yours.",
    ctaLabel: "Shop the collection",
    currency: "BDT",
    products: [
      {
        id: "PRD-78201",
        title: "Canvas Market Tote",
        price: 850,
        stock: 24,
        category: "Accessories",
        status: "Published",
        description: "Heavyweight 16oz organic canvas tote with reinforced box-stitched handles and interior pocket.",
      },
      {
        id: "PRD-78202",
        title: "Ceramic Pour-Over Dripper",
        price: 1200,
        stock: 15,
        category: "Living",
        status: "Published",
        description: "Hand-thrown stoneware dripper with spiral interior ribs for optimal extraction balance.",
      },
      {
        id: "PRD-78203",
        title: "Linen Studio Apron",
        price: 1650,
        stock: 8,
        category: "Apparel",
        status: "Published",
        description: "Cross-back washed European flax linen apron with dual deep side pockets for workshop utility.",
      },
      {
        id: "PRD-78204",
        title: "Brass Ruler & Pen Rest",
        price: 950,
        stock: 30,
        category: "Stationery",
        status: "Published",
        description: "Solid milled brass ruler with metric graduations and a machined pen rest groove.",
      },
    ],
    orders: [
      {
        id: "ORD-9402",
        customer: "Ayesha Rahman",
        email: "ayesha.r@gmail.com",
        phone: "+880 1711-234567",
        items: [{ title: "Canvas Market Tote", quantity: 1, price: 850 }],
        total: 850,
        status: "Received",
        placedAt: "12 mins ago",
      },
      {
        id: "ORD-9401",
        customer: "Tanvir Hasan",
        email: "tanvir.h@outlook.com",
        phone: "+880 1819-345678",
        items: [{ title: "Ceramic Pour-Over Dripper", quantity: 1, price: 1200 }],
        total: 1200,
        status: "In Packing",
        placedAt: "45 mins ago",
      },
      {
        id: "ORD-9400",
        customer: "Farhana Islam",
        email: "farhana.i@gmail.com",
        phone: "+880 1912-456789",
        items: [{ title: "Linen Studio Apron", quantity: 1, price: 1650 }],
        total: 1650,
        status: "Completed",
        placedAt: "3 hours ago",
      },
    ],
    conversations: [
      {
        id: "MSG-101",
        channel: "WhatsApp",
        customer: "Nafis Anjum",
        phone: "+880 1722-998877",
        message: "Hi! Can I order 2 Canvas Market Totes delivered to Banani, Dhaka?",
        proposedItem: "Canvas Market Tote",
        price: 1700,
        converted: false,
      },
      {
        id: "MSG-102",
        channel: "Instagram",
        customer: "Mehnaz Chowdhury",
        phone: "+880 1933-445566",
        message: "Hello, is the Ceramic Pour-Over Dripper still available? Want to buy one.",
        proposedItem: "Ceramic Pour-Over Dripper",
        price: 1200,
        converted: false,
      },
    ],
  }
  sessionStorage.setItem(`${prefix}${session.id}`, JSON.stringify(session))
  return session
}

export function readDemoSession(id: string): DemoSession | null {
  try {
    const raw = sessionStorage.getItem(`${prefix}${id}`)
    if (!raw) return null
    const session = JSON.parse(raw) as DemoSession
    if (!session?.id || session.id !== id || Date.now() - session.createdAt > maxAgeMs) {
      sessionStorage.removeItem(`${prefix}${id}`)
      return null
    }
    return session
  } catch {
    return null
  }
}

export function updateDemoSession(
  id: string,
  updates: Partial<Omit<DemoSession, "id" | "createdAt" | "updatedAt">>,
): DemoSession | null {
  const current = readDemoSession(id)
  if (!current) return null
  const next = { ...current, ...updates, updatedAt: Date.now() }
  sessionStorage.setItem(`${prefix}${id}`, JSON.stringify(next))
  return next
}

export function addDemoOrder(
  id: string,
  order: Omit<DemoOrder, "id" | "placedAt">,
): DemoOrder | null {
  const current = readDemoSession(id)
  if (!current) return null
  const orderId = `ORD-${Math.floor(9400 + current.orders.length + Math.random() * 100)}`
  const newOrder: DemoOrder = {
    ...order,
    id: orderId,
    placedAt: "Just now",
  }
  const nextOrders = [newOrder, ...current.orders]
  updateDemoSession(id, { orders: nextOrders })
  return newOrder
}

export function convertSocialToOrder(id: string, messageId: string): DemoOrder | null {
  const current = readDemoSession(id)
  if (!current) return null
  const msg = current.conversations.find((c) => c.id === messageId)
  if (!msg || msg.converted) return null

  const orderId = `ORD-${Math.floor(9400 + current.orders.length + Math.random() * 100)}`
  const newOrder: DemoOrder = {
    id: orderId,
    customer: msg.customer,
    email: `${msg.customer.toLowerCase().replace(/\s+/g, ".")}@customer.com`,
    phone: msg.phone,
    items: [{ title: msg.proposedItem, quantity: 1, price: msg.price }],
    total: msg.price,
    status: "Received",
    placedAt: "Just now (from " + msg.channel + ")",
  }

  const updatedConvs = current.conversations.map((c) => (c.id === messageId ? { ...c, converted: true } : c))
  const nextOrders = [newOrder, ...current.orders]
  updateDemoSession(id, { orders: nextOrders, conversations: updatedConvs })
  return newOrder
}

export function clearDemoSession(id: string) {
  sessionStorage.removeItem(`${prefix}${id}`)
}
