import React, { createContext, useContext, useMemo } from "react"
import { ClerkProvider, useUser, useClerk } from "@clerk/clerk-react"

// Environment variable for Clerk Publishable Key
export const CLERK_PUBLISHABLE_KEY =
  (typeof import.meta !== "undefined" &&
    (import.meta.env?.VITE_CLERK_PUBLISHABLE_KEY ||
      (import.meta.env as any)?.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY)) ||
  "pk_test_c21pbGluZy1wZWFjb2NrLTUyNzkuY2xlcmsuYWNjb3VudHMuZGV2JA"

// Vendryx brand appearance configuration for Clerk components
export const vendryxClerkAppearance = {
  variables: {
    colorPrimary: "#C2410C", // --vx-brand-700
    colorText: "#18181B", // --vx-fg-primary
    colorTextSecondary: "#5F6368", // --vx-fg-secondary
    colorBackground: "#FFFFFF",
    colorInputBackground: "#FCFBF9",
    colorInputBorder: "#E6E0D8",
    fontFamily:
      "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif",
    borderRadius: "0.625rem", // 10px
  },
  elements: {
    card: {
      boxShadow: "0 4px 16px rgba(24, 24, 27, 0.05)",
      border: "1px solid #E6E0D8",
      borderRadius: "0.875rem", // 14px
      backgroundColor: "#FFFFFF",
    },
    headerTitle: {
      color: "#18181B",
      fontWeight: "700",
      fontSize: "1.25rem",
    },
    headerSubtitle: {
      color: "#5F6368",
      fontSize: "0.875rem",
    },
    formButtonPrimary: {
      backgroundColor: "#C2410C",
      fontSize: "0.875rem",
      fontWeight: "600",
      "&:hover": {
        backgroundColor: "#9A3412",
      },
    },
    footerActionLink: {
      color: "#C2410C",
      fontWeight: "600",
    },
  },
}

interface ClerkAuthContextValue {
  isClerkEnabled: boolean
  publishableKey: string
}

interface SafeClerkUserState {
  user: any
  isSignedIn: boolean
  isLoaded: boolean
  signOut: () => Promise<void>
}

const defaultSafeClerkUser: SafeClerkUserState = {
  user: null,
  isSignedIn: false,
  isLoaded: true,
  signOut: async () => {},
}

const ClerkAuthContext = createContext<ClerkAuthContextValue>({
  isClerkEnabled: Boolean(CLERK_PUBLISHABLE_KEY),
  publishableKey: CLERK_PUBLISHABLE_KEY,
})

const ClerkSafeUserContext =
  createContext<SafeClerkUserState>(defaultSafeClerkUser)

export const useClerkAuth = () => useContext(ClerkAuthContext)
export const useSafeClerk = () => useContext(ClerkSafeUserContext)

const ClerkInnerConsumer: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const { user, isLoaded, isSignedIn } = useUser()
  const clerk = useClerk()

  const safeState = useMemo<SafeClerkUserState>(
    () => ({
      user,
      isSignedIn: Boolean(isSignedIn),
      isLoaded: Boolean(isLoaded),
      signOut: async () => {
        await clerk.signOut()
      },
    }),
    [user, isLoaded, isSignedIn, clerk]
  )

  return (
    <ClerkSafeUserContext.Provider value={safeState}>
      {children}
    </ClerkSafeUserContext.Provider>
  )
}

export interface ClerkAuthProviderProps {
  children: React.ReactNode
}

export const ClerkAuthProvider: React.FC<ClerkAuthProviderProps> = ({
  children,
}) => {
  const isClerkEnabled = Boolean(CLERK_PUBLISHABLE_KEY)

  const contextValue = useMemo(
    () => ({
      isClerkEnabled,
      publishableKey: CLERK_PUBLISHABLE_KEY,
    }),
    [isClerkEnabled]
  )

  if (!isClerkEnabled) {
    return (
      <ClerkAuthContext.Provider value={contextValue}>
        <ClerkSafeUserContext.Provider value={defaultSafeClerkUser}>
          {children}
        </ClerkSafeUserContext.Provider>
      </ClerkAuthContext.Provider>
    )
  }

  return (
    <ClerkProvider
      publishableKey={CLERK_PUBLISHABLE_KEY}
      appearance={vendryxClerkAppearance}
    >
      <ClerkAuthContext.Provider value={contextValue}>
        <ClerkInnerConsumer>{children}</ClerkInnerConsumer>
      </ClerkAuthContext.Provider>
    </ClerkProvider>
  )
}
