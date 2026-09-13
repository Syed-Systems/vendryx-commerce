/**
 * Vendryx Marketing Configuration
 *
 * Defines environment-aware connection URLs between the public marketing site
 * and the merchant operations workspace (Admin Dashboard).
 */

export const ADMIN_URL =
  (typeof import.meta !== "undefined" && import.meta.env?.VITE_ADMIN_URL) ||
  "http://localhost:5173"

export const ADMIN_LOGIN_URL = `${ADMIN_URL}/login`
export const ADMIN_SIGNUP_URL = `${ADMIN_URL}/signup`
