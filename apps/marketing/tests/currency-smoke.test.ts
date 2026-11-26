import assert from "node:assert/strict"

const CURRENCY_SYMBOLS: Record<string, string> = { BDT: "৳", USD: "$", EUR: "€" }

function formatMoney(bdtAmount: number, currency: string): string {
  if (currency === "USD") return `$${(bdtAmount / 120).toFixed(2)}`
  if (currency === "EUR") return `€${(bdtAmount / 130).toFixed(2)}`
  return `৳${bdtAmount.toLocaleString("en-BD")}`
}

console.log("Running currency smoke tests...")

assert.ok(formatMoney(1000, "BDT").startsWith("৳"), "BDT should start with ৳")
assert.ok(!formatMoney(1000, "BDT").includes("BDT"), "BDT should not contain 'BDT' text")
assert.ok(formatMoney(1200, "USD").startsWith("$"), "USD should start with $")
assert.ok(!formatMoney(1200, "USD").includes("USD"), "USD should not contain 'USD' text")
assert.ok(formatMoney(1300, "EUR").startsWith("€"), "EUR should start with €")
assert.ok(!formatMoney(1300, "EUR").includes("EUR"), "EUR should not contain 'EUR' text")
assert.equal(CURRENCY_SYMBOLS.BDT, "৳")
assert.equal(CURRENCY_SYMBOLS.USD, "$")
assert.equal(CURRENCY_SYMBOLS.EUR, "€")

console.log("  All currency smoke tests passed")