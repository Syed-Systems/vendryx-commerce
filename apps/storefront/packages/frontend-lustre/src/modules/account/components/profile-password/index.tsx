import { HttpTypes } from "@medusajs/types"

export type ProfilePasswordProps = {
  customer: HttpTypes.StoreCustomer
}

/**
 * Content-gated until core exposes a real password-update action.
 */
const ProfilePassword = (_props: ProfilePasswordProps) => null

export default ProfilePassword
