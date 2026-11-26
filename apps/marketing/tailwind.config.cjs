/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Plus Jakarta Sans"', "system-ui", "-apple-system", "sans-serif"],
        mono: ['"JetBrains Mono"', "ui-monospace", "SFMono-Regular", "monospace"],
      },
      colors: {
        vendryx: {
          orange: "#FF7A00",
          "orange-hover": "#D95700",
          "orange-light": "#FFAF24",
          "orange-deep": "#F45605",
          black: "#080A0C",
          charcoal: "#0F1115",
          surface: "#1B1E24",
          "warm-white": "#FFFCF8",
          slate: "#64748B",
          "light-gray": "#E5E7EB",
        },
      },
      letterSpacing: {
        display: "-0.035em",
        heading: "-0.025em",
      },
    },
  },
  plugins: [],
}
