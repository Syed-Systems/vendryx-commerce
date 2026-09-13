"use client"

import { useEffect, useState } from "react"
import {
  DEFAULT_BRANDING,
  MAHAL_BRANDING,
  LUSTRE_BRANDING,
  MATRIX_BRANDING,
  emitBrandingCss,
} from "@lib/branding"

const THEMES = [
  { name: "Default", data: DEFAULT_BRANDING },
  { name: "Mahal", data: MAHAL_BRANDING },
  { name: "Lustre", data: LUSTRE_BRANDING },
  { name: "Matrix", data: MATRIX_BRANDING },
]

export default function DevThemeSwitcher() {
  const [activeTheme, setActiveTheme] = useState("Default")

  useEffect(() => {
    const currentTheme = document.body.dataset.theme
    const matchingTheme = THEMES.find(
      (theme) => theme.name.toLowerCase().replace(/\s+/g, "-") === currentTheme
    )

    if (matchingTheme) {
      setActiveTheme(matchingTheme.name)
    }
  }, [])

  const handleThemeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const themeName = e.target.value
    setActiveTheme(themeName)

    const theme = THEMES.find((t) => t.name === themeName)
    if (theme) {
      const css = emitBrandingCss(theme.data)
      const style = document.getElementById("stheno-brand-tokens")

      if (style) {
        style.textContent = css
      }

      document.body.dataset.theme = theme.name.toLowerCase().replace(/\s+/g, "-")
    }
  }

  return (
    <div className="fixed bottom-4 right-4 z-[9999] bg-brand-surface border border-brand-hairline p-3 rounded-card shadow-2xl flex items-center gap-3 font-sans text-brand-text">
      <div className="flex flex-col">
        <span className="text-[10px] uppercase font-bold tracking-widest text-brand-secondary">
          Dev Theme
        </span>
        <select
          value={activeTheme}
          onChange={handleThemeChange}
          className="bg-transparent border-none outline-none font-medium text-sm cursor-pointer p-0 m-0 text-brand-primary"
        >
          {THEMES.map((theme) => (
            <option key={theme.name} value={theme.name}>
              {theme.name}
            </option>
          ))}
        </select>
      </div>
    </div>
  )
}
