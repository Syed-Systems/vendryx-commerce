const path = require("path")

let uiPreset
try {
  uiPreset = require("@medusajs/ui-preset")
} catch {
  uiPreset = null
}

/** @type {import('tailwindcss').Config} */
module.exports = {
  presets: uiPreset ? [uiPreset] : [],
  darkMode: "class",
  content: [
    path.join(__dirname, "index.html"),
    path.join(__dirname, "src/**/*.{js,ts,jsx,tsx}"),
    path.join(__dirname, "node_modules/@medusajs/ui/dist/**/*.{js,jsx,ts,tsx}"),
    path.join(__dirname, "../../node_modules/@medusajs/ui/dist/**/*.{js,jsx,ts,tsx}"),
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
