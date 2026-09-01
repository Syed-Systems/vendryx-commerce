import assert from "node:assert/strict"
import {
  addDemoOrder,
  clearDemoSession,
  convertSocialToOrder,
  createDemoSession,
  readDemoSession,
  updateDemoSession,
} from "../src/demo-session"

// Mock sessionStorage for headless testing environment
const mockStorage = new Map<string, string>()
;(globalThis as any).sessionStorage = {
  getItem: (key: string) => mockStorage.get(key) ?? null,
  setItem: (key: string, val: string) => mockStorage.set(key, String(val)),
  removeItem: (key: string) => mockStorage.delete(key),
  clear: () => mockStorage.clear(),
}

console.log("▶ Running automated unit & workflow tests for Stheno Marketing...")

// Test 1: Create session
const session = createDemoSession()
assert.ok(session.id, "Session must have an ID")
assert.equal(session.storeName, "Satheno Studio")
assert.equal(session.products.length, 4, "Default session should have 4 products")
assert.equal(session.orders.length, 3, "Default session should have 3 initial orders")
assert.equal(session.conversations.length, 2, "Default session should have 2 social conversations")
console.log("  ✔ Test 1: Session creation and defaults verified")

// Test 2: Read session
const read = readDemoSession(session.id)
assert.ok(read)
assert.equal(read.id, session.id, "Read session must match created session ID")
console.log("  ✔ Test 2: Session retrieval from storage verified")

// Test 3: Update session
const updated = updateDemoSession(session.id, { storeName: "Aarong Atelier", accent: "#d97706" })
assert.ok(updated)
assert.equal(updated.storeName, "Aarong Atelier")
assert.equal(updated.accent, "#d97706")
const reRead = readDemoSession(session.id)
assert.ok(reRead)
assert.equal(reRead.storeName, "Aarong Atelier")
console.log("  ✔ Test 3: Session updates and persistence verified")

// Test 4: 1-Click Social Conversation to Order Conversion
const convertedOrder = convertSocialToOrder(session.id, "MSG-101")
assert.ok(convertedOrder, "Conversion must succeed")
assert.equal(convertedOrder.customer, "Nafis Anjum")
assert.equal(convertedOrder.total, 1700)
const sessionAfterConv = readDemoSession(session.id)
assert.ok(sessionAfterConv)
assert.equal(sessionAfterConv.orders.length, 4, "Orders length should be 4 after social conversion")
assert.equal(sessionAfterConv.conversations.find((c) => c.id === "MSG-101")?.converted, true, "Message must be marked converted")
console.log("  ✔ Test 4: 1-Click social conversation to order conversion verified")

// Test 5: Add customer store checkout order
const newOrder = addDemoOrder(session.id, {
  customer: "Kamal Hossain",
  email: "kamal@gmail.com",
  phone: "+880 1711-000000",
  items: [{ title: "Canvas Market Tote", quantity: 2, price: 850 }],
  total: 1700,
  status: "Received",
})
assert.ok(newOrder)
assert.ok(newOrder.id.startsWith("ORD-"), "Order ID must follow ORD- format")
const sessionWithOrder = readDemoSession(session.id)
assert.ok(sessionWithOrder)
assert.equal(sessionWithOrder.orders.length, 5, "Orders length must increment to 5")
assert.equal(sessionWithOrder.orders[0].customer, "Kamal Hossain", "New order must be at head of queue")
console.log("  ✔ Test 5: Dynamic customer checkout & order queue sync verified")

// Test 6: Clear session
clearDemoSession(session.id)
assert.equal(readDemoSession(session.id), null, "Cleared session should return null")
console.log("  ✔ Test 6: Session cleanup verified")

console.log("✅ All automated unit & workflow tests PASSED successfully!")
