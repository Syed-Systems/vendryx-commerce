"use client"

import { useEffect, useState } from "react"

type DevTheme = { name: string; css: string }

export default function DevThemeSwitcher({ themes }: { themes: DevTheme[] }) {
  const [activeTheme, setActiveTheme] = useState("Default")

  useEffect(() => {
    const currentTheme = document.body.dataset.theme
    const matchingTheme = themes.find(
      (theme) => theme.name.toLowerCase().replace(/\s+/g, "-") === currentTheme
    )

    if (matchingTheme) {
      setActiveTheme(matchingTheme.name)
    }
  }, [themes])

  const handleThemeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const themeName = e.target.value
    setActiveTheme(themeName)

    const theme = themes.find((t) => t.name === themeName)
    if (theme) {
      const style = document.getElementById("stheno-brand-tokens")

      if (style) {
        style.textContent = theme.css
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
          {themes.map((theme) => (
            <option key={theme.name} value={theme.name}>
              {theme.name}
            </option>
          ))}
        </select>
      </div>
    </div>
  )
}
