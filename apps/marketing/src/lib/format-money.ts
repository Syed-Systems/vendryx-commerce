export type Currency = "BDT" | "USD" | "EUR"

export const CURRENCY_SYMBOLS: Record<Currency, string> = { BDT: "৳", USD: "$", EUR: "€" }

export function formatMoney(bdtAmount: number, currency: Currency): string {
  if (currency === "USD") {
    const usd = bdtAmount / 120
    return `$${usd.toFixed(2)}`
  }
  if (currency === "EUR") {
    const eur = bdtAmount / 130
    return `€${eur.toFixed(2)}`
  }
  return `৳${bdtAmount.toLocaleString("en-BD")}`
}