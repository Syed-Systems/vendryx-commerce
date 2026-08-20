import { retrieveCart } from "@lib/data/cart"
import CartDropdown from "@theme/layout/components/cart-dropdown"

/**
 * Was theme-default's cart-button. The read belongs in core (ADR-013 decision
 * 4); the markup it renders is still the theme's.
 */
export default async function CartButtonLoader() {
  const cart = await retrieveCart().catch(() => null)

  return <CartDropdown cart={cart} />
}
