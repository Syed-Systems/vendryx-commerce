import { createContext } from "react"

export type ThemeOption = "light" | "dark" | "system"
export type ThemeValue = "light" | "dark"

type ThemeContextValue = {
  theme: ThemeOption
  /** The theme actually painted, with "system" already resolved against the OS. */
  resolvedTheme: ThemeValue
  setTheme: (theme: ThemeOption) => void
}

export const ThemeContext = createContext<ThemeContextValue | null>(null)
