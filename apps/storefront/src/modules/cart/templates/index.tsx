import ItemsTemplate from "./items"
import Summary from "./summary"
import EmptyCartMessage from "../components/empty-cart-message"
import SignInPrompt from "../components/sign-in-prompt"
import Divider from "@modules/common/components/divider"
import { HttpTypes } from "@medusajs/types"

const CartTemplate = ({
  cart,
  customer,
}: {
  cart: HttpTypes.StoreCart | null
  customer: HttpTypes.StoreCustomer | null
}) => {
  return (
    <div className="py-8 sm:py-16">
      <div className="content-container" data-testid="cart-container">
        {/* Cart Header */}
        <div className="mb-8 pb-4 border-b border-zinc-200">
          <span className="text-xs uppercase tracking-[0.2em] text-zinc-500 font-semibold block mb-1">
            Review Selection
          </span>
          <h1 className="text-3xl font-serif font-bold text-zinc-950">
            Your Shopping Bag
          </h1>
        </div>

        {cart?.items?.length ? (
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-12 items-start">
            <div className="lg:col-span-8 flex flex-col bg-white gap-y-6">
              {!customer && (
                <>
                  <SignInPrompt />
                  <Divider />
                </>
              )}
              <ItemsTemplate cart={cart} />
            </div>
            <div className="lg:col-span-4 lg:sticky lg:top-28">
              {cart && cart.region && (
                <Summary cart={cart as any} />
              )}
            </div>
          </div>
        ) : (
          <div>
            <EmptyCartMessage />
          </div>
        )}
      </div>
    </div>
  )
}

export default CartTemplate
