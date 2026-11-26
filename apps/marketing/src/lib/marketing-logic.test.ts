import test from "node:test"
import assert from "node:assert/strict"
import {
  calculateCourierQuote,
  calculatePlanTermQuote,
  calculateProTermQuote,
  formatUsdCents,
  formatWholeBdt,
  validateOnboardingForm,
} from "./marketing-logic"

test("formatUsdCents and formatWholeBdt format currency without float drift", () => {
  assert.equal(formatUsdCents(2000), "$20")
  assert.equal(formatUsdCents(21600), "$216")
  assert.equal(formatUsdCents(54000), "$540")
  assert.equal(formatUsdCents(1995), "$19.95")
  assert.equal(formatWholeBdt(2200), "৳2,200")
})

test("calculateProTermQuote and calculatePlanTermQuote compute exact Pro & Scale totals", () => {
  const pro1 = calculateProTermQuote(1, "USD")
  assert.equal(pro1.totalUsdCents, 2000)
  assert.equal(pro1.discountPercent, 0)
  assert.equal(pro1.formattedTotal, "$20")
  assert.equal(pro1.includesFreeCustomStorefront, false)

  const pro12 = calculateProTermQuote(12, "USD")
  assert.equal(pro12.discountPercent, 10)
  assert.equal(pro12.discountUsdCents, 2400)
  assert.equal(pro12.totalUsdCents, 21600)
  assert.equal(pro12.effectiveMonthlyUsdCents, 1800)
  assert.equal(pro12.formattedMonthly, "$18")
  assert.equal(pro12.formattedTotal, "$216")
  assert.equal(pro12.formattedSavings, "$24")
  assert.equal(pro12.includesFreeCustomStorefront, true)

  const pro12Bdt = calculatePlanTermQuote("pro", 12, "BDT")
  assert.equal(pro12Bdt.payableBdt, 23760)
  assert.equal(pro12Bdt.formattedMonthly, "৳1,980")
  assert.equal(pro12Bdt.formattedTotal, "৳23,760")
  assert.equal(pro12Bdt.formattedSavings, "৳2,640")

  const scale1 = calculatePlanTermQuote("scale", 1, "USD")
  assert.equal(scale1.totalUsdCents, 5000)
  assert.equal(scale1.formattedMonthly, "$50")
  assert.equal(scale1.includesFreeCustomStorefront, false)

  const scale12Usd = calculatePlanTermQuote("scale", 12, "USD")
  assert.equal(scale12Usd.discountPercent, 10)
  assert.equal(scale12Usd.totalUsdCents, 54000)
  assert.equal(scale12Usd.formattedMonthly, "$45")
  assert.equal(scale12Usd.formattedTotal, "$540")
  assert.equal(scale12Usd.formattedSavings, "$60")
  assert.equal(scale12Usd.includesFreeCustomStorefront, true)

  const scale12Bdt = calculatePlanTermQuote("scale", 12, "BDT")
  assert.equal(scale12Bdt.payableBdt, 58320)
  assert.equal(scale12Bdt.formattedMonthly, "৳4,860")
  assert.equal(scale12Bdt.formattedTotal, "৳58,320")
  assert.equal(scale12Bdt.formattedSavings, "৳6,480")
})

test("calculateCourierQuote computes exact Pathao/Steadfast/RedX zone fees", () => {
  const inside = calculateCourierQuote({
    zone: "inside_dhaka",
    weightKg: 1,
    codAmountBdt: 2400,
  })
  assert.equal(inside.deliveryFeeBdt, 60)
  assert.equal(inside.codFeeBdt, 0)
  assert.equal(inside.netMerchantPayoutBdt, 2340)
  assert.equal(inside.recommendedCarrier, "Pathao")

  const outsideHeavy = calculateCourierQuote({
    zone: "outside_dhaka",
    weightKg: 3,
    codAmountBdt: 4000,
  })
  assert.equal(outsideHeavy.deliveryFeeBdt, 190)
  assert.equal(outsideHeavy.codFeeBdt, 40)
  assert.equal(outsideHeavy.netMerchantPayoutBdt, 3770)
  assert.equal(outsideHeavy.recommendedCarrier, "RedX")
})

test("validateOnboardingForm validates brand name and email format", () => {
  const bad = validateOnboardingForm({
    storeName: "A",
    email: "bad-email",
    channel: "whatsapp",
  })
  assert.equal(bad.valid, false)
  assert.ok(bad.errors.storeName)
  assert.ok(bad.errors.email)

  const good = validateOnboardingForm({
    storeName: "Dhaka Loom Co.",
    email: "founder@dhakaloom.com",
    channel: "facebook",
  })
  assert.equal(good.valid, true)
  assert.deepEqual(good.errors, {})
})
