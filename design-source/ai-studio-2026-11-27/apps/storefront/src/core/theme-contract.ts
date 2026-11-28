import type { ComponentType } from "react"
import type { CartLineItem, CheckoutTotals } from "./checkout-engine"
import type { CatalogProduct, ResolvedTenantContext } from "./tenant-resolver"
import type { ThemeId } from "./theme-schema"

/**
 * Strict Pure-Presentation Contract for Storefront Themes.
 *
 * Security Invariants:
 * 1. Themes receive ONLY sanitized, tenant-scoped props and locked core callbacks.
 * 2. Themes NEVER call fetch(), read cookies/auth tokens, or render checkout forms.
 * 3. Every theme brings its own isolated CSS module, custom SVG icons, layout geometry, and typography.
 */
export interface StorefrontThemeProps {
  tenant: ResolvedTenantContext
  cartItems: CartLineItem[]
  cartTotals: CheckoutTotals
  cartOpen: boolean
  activeProduct: CatalogProduct | null
  onSelectProduct: (product: CatalogProduct | null) => void
  onAddToCart: (item: Omit<CartLineItem, "quantity">, qty?: number) => void
  onUpdateQuantity: (variantId: string, delta: number) => void
  onRemoveItem: (variantId: string) => void
  onToggleCart: (open: boolean) => void
  onProceedToCheckout: () => void
}

export interface StorefrontThemePack {
  id: ThemeId
  displayName: string
  designLineage: string
  Component: ComponentType<StorefrontThemeProps>
}
