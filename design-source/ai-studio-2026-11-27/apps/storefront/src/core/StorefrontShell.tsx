"use client"

import React, { useCallback, useEffect, useMemo, useState } from "react"
import {
  calculateLockedCheckout,
  type CartLineItem,
  type OrderReceipt,
} from "./checkout-engine"
import { LockedCheckoutModal } from "./LockedCheckoutModal"
import { PlatformAttributionFooter } from "./PlatformAttributionFooter"
import { RuntimeArchitectureDrawer } from "./RuntimeArchitectureDrawer"
import type { CatalogProduct, ResolvedTenantContext } from "./tenant-resolver"
import { resolveThemePack } from "./theme-registry"
import { buildScopedCssVariables } from "./theme-schema"

interface StorefrontShellProps {
  tenant: ResolvedTenantContext
}

export function StorefrontShell({ tenant }: StorefrontShellProps) {
  // Cart state is partitioned strictly by tenantId so switching stores never leaks items across tenants.
  const [cartsByTenant, setCartsByTenant] = useState<Record<string, CartLineItem[]>>({})
  const [cartOpen, setCartOpen] = useState(false)
  const [checkoutOpen, setCheckoutOpen] = useState(false)
  const [activeProduct, setActiveProduct] = useState<CatalogProduct | null>(null)
  const [lastReceipt, setLastReceipt] = useState<OrderReceipt | null>(null)

  // Reset modal states cleanly whenever tenant switches
  useEffect(() => {
    setCartOpen(false)
    setCheckoutOpen(false)
    setActiveProduct(null)
    setLastReceipt(null)
  }, [tenant.tenantId])

  const cartItems = cartsByTenant[tenant.tenantId] ?? []

  const cartTotals = useMemo(
    () => calculateLockedCheckout(cartItems, "inside_city", "stripe"),
    [cartItems]
  )

  const handleAddToCart = useCallback(
    (item: Omit<CartLineItem, "quantity">, qty = 1) => {
      const safeQty = Math.max(1, Math.trunc(qty))
      setCartsByTenant((prev) => {
        const current = prev[tenant.tenantId] ?? []
        const existingIndex = current.findIndex((line) => line.variantId === item.variantId)
        if (existingIndex >= 0) {
          const updated = [...current]
          updated[existingIndex] = {
            ...updated[existingIndex],
            quantity: updated[existingIndex].quantity + safeQty,
          }
          return { ...prev, [tenant.tenantId]: updated }
        }
        return {
          ...prev,
          [tenant.tenantId]: [...current, { ...item, quantity: safeQty }],
        }
      })
      setCartOpen(true)
    },
    [tenant.tenantId]
  )

  const handleUpdateQuantity = useCallback(
    (variantId: string, delta: number) => {
      setCartsByTenant((prev) => {
        const current = prev[tenant.tenantId] ?? []
        const next = current
          .map((line) =>
            line.variantId === variantId
              ? { ...line, quantity: line.quantity + delta }
              : line
          )
          .filter((line) => line.quantity > 0)
        return { ...prev, [tenant.tenantId]: next }
      })
    },
    [tenant.tenantId]
  )

  const handleRemoveItem = useCallback(
    (variantId: string) => {
      setCartsByTenant((prev) => {
        const current = prev[tenant.tenantId] ?? []
        return {
          ...prev,
          [tenant.tenantId]: current.filter((line) => line.variantId !== variantId),
        }
      })
    },
    [tenant.tenantId]
  )

  const handleProceedToCheckout = useCallback(() => {
    setCartOpen(false)
    setActiveProduct(null)
    setCheckoutOpen(true)
  }, [])

  const handleOrderComplete = useCallback(
    (receipt: OrderReceipt) => {
      setLastReceipt(receipt)
      setCartsByTenant((prev) => ({ ...prev, [tenant.tenantId]: [] }))
    },
    [tenant.tenantId]
  )

  const themePack = resolveThemePack(tenant.themeId)
  const ThemeComponent = themePack.Component
  const cssVars = useMemo(
    () => buildScopedCssVariables(tenant.themeConfig.tokens) as React.CSSProperties,
    [tenant.themeConfig.tokens]
  )

  return (
    <div
      style={cssVars}
      data-tenant-id={tenant.tenantId}
      data-theme-id={tenant.themeId}
      data-plan={tenant.plan}
      data-theme-mode={tenant.mode}
      className="min-h-screen bg-[var(--theme-canvas)] text-[var(--theme-ink)]"
    >
      {/* Active isolated theme pack — pure presentation */}
      <ThemeComponent
        tenant={tenant}
        cartItems={cartItems}
        cartTotals={cartTotals}
        cartOpen={cartOpen}
        activeProduct={activeProduct}
        onSelectProduct={setActiveProduct}
        onAddToCart={handleAddToCart}
        onUpdateQuantity={handleUpdateQuantity}
        onRemoveItem={handleRemoveItem}
        onToggleCart={setCartOpen}
        onProceedToCheckout={handleProceedToCheckout}
      />

      {/* Core-enforced footer attribution (rendered strictly for Free stores, null for Pro/Scale) */}
      <PlatformAttributionFooter tenant={tenant} />

      {/* Core-owned locked checkout modal */}
      <LockedCheckoutModal
        open={checkoutOpen}
        tenant={tenant}
        items={cartItems}
        onClose={() => setCheckoutOpen(false)}
        onOrderComplete={handleOrderComplete}
      />

      {/* Subtle toast after closing receipt if needed */}
      {lastReceipt && !checkoutOpen && (
        <div
          className="fixed bottom-4 left-4 z-40 border border-[var(--theme-border)] bg-[var(--theme-surface)] px-4 py-3 text-xs text-[var(--theme-ink)] shadow-lg rounded-[var(--theme-radius-card)]"
          role="status"
        >
          Order <span className="font-mono font-semibold">{lastReceipt.orderNumber}</span> confirmed ({lastReceipt.totalFormatted}).
          <button
            type="button"
            onClick={() => setLastReceipt(null)}
            className="ml-3 underline opacity-70 hover:opacity-100"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Discreet multi-tenant runtime switcher & security inspector */}
      <RuntimeArchitectureDrawer tenant={tenant} />
    </div>
  )
}
