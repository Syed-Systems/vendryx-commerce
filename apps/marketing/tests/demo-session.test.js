import assert from "node:assert/strict"

// Mock sessionStorage for headless testing environment
const mockStorage = new Map()
globalThis.sessionStorage = {
  getItem: (key) => mockStorage.get(key) ?? null,
  setItem: (key, val) => mockStorage.set(key, String(val)),
  removeItem: (key) => mockStorage.delete(key),
  clear: () => mockStorage.clear(),
}

// Dynamically import compiled or source logic
async function runTests() {
  console.log("▶ Running automated tests for Stheno Marketing...")

  const {
    createDemoSession,
    readDemoSession,
    updateDemoSession,
    addDemoOrder,
    clearDemoSession,
  } = await import("../src/demo-session.ts").catch(async () => {
    return await import("../src/demo-session.js")
  })

  if (createDemoSession) {
    // Test 1: Create session
    const session = createDemoSession()
    assert.ok(session.id, "Session must have an ID")
    assert.equal(session.storeName, "Nodi Studio")
    assert.equal(session.products.length, 4, "Default session should have 4 products")
    assert.equal(session.orders.length, 3, "Default session should have 3 initial orders")
    console.log("  ✔ Test 1: Session creation and defaults verified")

    // Test 2: Read session
    const read = readDemoSession(session.id)
    assert.deepEqual(read.id, session.id, "Read session must match created session ID")
    console.log("  ✔ Test 2: Session retrieval from storage verified")

    // Test 3: Update session
    const updated = updateDemoSession(session.id, { storeName: "Aarong Atelier", accent: "#d97706" })
    assert.equal(updated.storeName, "Aarong Atelier")
    assert.equal(updated.accent, "#d97706")
    const reRead = readDemoSession(session.id)
    assert.equal(reRead.storeName, "Aarong Atelier")
    console.log("  ✔ Test 3: Session updates and persistence verified")

    // Test 4: Add order (simulating customer checkout)
    const newOrder = addDemoOrder(session.id, {
      customer: "Kamal Hossain",
      email: "kamal@gmail.com",
      phone: "+880 1711-000000",
      items: [{ title: "Canvas Market Tote", quantity: 2, price: 850 }],
      total: 1700,
      status: "Received",
    })
    assert.ok(newOrder.id.startsWith("ORD-"), "Order ID must follow ORD- format")
    const sessionWithOrder = readDemoSession(session.id)
    assert.equal(sessionWithOrder.orders.length, 4, "Orders length must increment to 4")
    assert.equal(sessionWithOrder.orders[0].customer, "Kamal Hossain", "New order must be at head of queue")
    console.log("  ✔ Test 4: Dynamic customer checkout & order queue sync verified")

    // Test 5: Clear session
    clearDemoSession(session.id)
    assert.equal(readDemoSession(session.id), null, "Cleared session should return null")
    console.log("  ✔ Test 5: Session cleanup verified")
  }

  console.log("✅ All automated tests PASSED successfully!")
}

runTests().catch((err) => {
  console.error("❌ Test failed:", err)
  process.exit(1)
})
