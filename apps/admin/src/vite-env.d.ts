// / <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_STHENO_ADMIN_BACKEND_URL: string
  readonly VITE_STHENO_STOREFRONT_URL: string
  readonly VITE_STHENO_V2: "true" | "false"
  /** Merchant-facing help links. Unset hides the menu item rather than linking nowhere. */
  readonly VITE_DOCS_URL?: string
  readonly VITE_CHANGELOG_URL?: string
  /** Clerk Authentication publishable key for storefront owner sign-in */
  readonly VITE_CLERK_PUBLISHABLE_KEY?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
  readonly hot: {
    accept: () => void
  }
}

declare const __BACKEND_URL__: string | undefined
declare const __STOREFRONT_URL__: string | undefined
declare const __BASE__: string
declare const __AUTH_TYPE__: "session" | "jwt" | undefined
declare const __JWT_TOKEN_STORAGE_KEY__: string | undefined
declare const __MAX_UPLOAD_FILE_SIZE__: number | undefined
