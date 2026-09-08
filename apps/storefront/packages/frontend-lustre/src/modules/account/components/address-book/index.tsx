import React from "react"
import { HttpTypes } from "@medusajs/types"
import AddAddress from "../address-card/add-address"
import EditAddress from "../address-card/edit-address-modal"

export type AddressBookProps = {
  customer: HttpTypes.StoreCustomer
  region: HttpTypes.StoreRegion
}

/**
 * Lustre AddressBook (Slot 8).
 * Address management grid for verified customer shipping destinations.
 */
const AddressBook: React.FC<AddressBookProps> = ({ customer, region }) => {
  const { addresses } = customer

  return (
    <div className="space-y-6">
      <div className="border-b border-[var(--hairline)] pb-4">
        <span className="text-xs font-semibold uppercase tracking-widest text-[var(--accent-sapphire)]">
          Account Registry
        </span>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          Delivery Addresses
        </h1>
        <p className="mt-1 text-xs text-[var(--text-secondary)]">
          Manage and configure multiple international and domestic delivery destinations.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <AddAddress region={region} addresses={addresses} />
        {addresses.map((address) => (
          <EditAddress
            key={address.id}
            region={region}
            address={address}
            isActive={address.is_default_shipping}
          />
        ))}
      </div>
    </div>
  )
}

export default AddressBook
