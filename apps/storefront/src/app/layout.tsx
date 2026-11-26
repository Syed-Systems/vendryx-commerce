import type { Metadata } from "next"
import {
  Cormorant_Garamond,
  DM_Mono,
  DM_Sans,
  Hind_Siliguri,
  JetBrains_Mono,
  Outfit,
  Playfair_Display,
  Plus_Jakarta_Sans,
} from "next/font/google"
import "./globals.css"

const cormorant = Cormorant_Garamond({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  style: ["normal", "italic"],
  variable: "--font-cormorant",
  display: "swap",
})

const playfair = Playfair_Display({
  subsets: ["latin"],
  style: ["normal", "italic"],
  variable: "--font-playfair",
  display: "swap",
})

const jakarta = Plus_Jakarta_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  variable: "--font-jakarta",
  display: "swap",
})

const outfit = Outfit({
  subsets: ["latin"],
  weight: ["500", "600", "700"],
  variable: "--font-outfit",
  display: "swap",
})

const dmSans = DM_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  variable: "--font-dmsans",
  display: "swap",
})

const dmMono = DM_Mono({
  subsets: ["latin"],
  weight: ["400", "500"],
  variable: "--font-dm-mono",
  display: "swap",
})

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  variable: "--font-mono",
  display: "swap",
})

const hindSiliguri = Hind_Siliguri({
  subsets: ["bengali", "latin"],
  weight: ["400", "500", "600"],
  variable: "--font-bengali",
  display: "swap",
})

export const metadata: Metadata = {
  title: "KOVA — Built for Altitude | Insulated Stainless Vessels",
  description:
    "Unified multi-theme commerce runtime serving bespoke, isolated storefronts from a single Next.js server.",
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      className={`${cormorant.variable} ${playfair.variable} ${jakarta.variable} ${outfit.variable} ${dmSans.variable} ${dmMono.variable} ${jetbrainsMono.variable} ${hindSiliguri.variable}`}
    >
      <body>{children}</body>
    </html>
  )
}
