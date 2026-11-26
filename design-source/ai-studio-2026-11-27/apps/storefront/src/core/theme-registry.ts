import type { StorefrontThemePack } from "./theme-contract"
import type { ThemeId } from "./theme-schema"
import { HomedineEcoTheme } from "../themes/homedine-eco/HomedineEcoTheme"
import { HydroVesselTheme } from "../themes/hydro-vessel/HydroVesselTheme"
import { MaisonCuirTheme } from "../themes/maison-cuir/MaisonCuirTheme"

/**
 * Runtime Multi-Theme Registry hosted inside the single unified Next.js storefront server.
 * Each theme is an isolated, audited presentation pack with its own CSS Module, bespoke SVG icons,
 * layout geometry, and typography — consuming ONLY locked storefront-core props and callbacks.
 */
export const THEME_REGISTRY: Record<ThemeId, StorefrontThemePack> = {
  "maison-cuir": {
    id: "maison-cuir",
    displayName: "Maison Cuir (Luxury Leather)",
    designLineage: "Inspired by Polène Paris, Métier London & Carl Friedrik",
    Component: MaisonCuirTheme,
  },
  "hydro-vessel": {
    id: "hydro-vessel",
    displayName: "Hydro Vessel (Bottle Brand)",
    designLineage: "Inspired by LARQ, Fellow, Hydro Flask & Owala",
    Component: HydroVesselTheme,
  },
  "homedine-eco": {
    id: "homedine-eco",
    displayName: "Homedine (Eco Kitchenware)",
    designLineage: "Inspired by Homedine Botanical Sustainable Kitchenware",
    Component: HomedineEcoTheme,
  },
}

export function resolveThemePack(themeId: ThemeId): StorefrontThemePack {
  return THEME_REGISTRY[themeId] ?? THEME_REGISTRY["maison-cuir"]
}
