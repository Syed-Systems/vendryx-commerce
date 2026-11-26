import {
  THEME_SCHEMA_VERSION,
  validateThemeConfig,
  type ThemeId,
  type ValidatedThemeConfig,
} from "./theme-schema"

export type TenantPlan = "free" | "pro" | "scale"
export type StoreCurrency = "USD" | "BDT"
export type ThemePublishMode = "published" | "draft"

export interface ProductVariantOption {
  id: string
  label: string
  colorName: string
  swatchHex: string
  sizeLabel?: string
  priceMinor: number
  inStock: boolean
  imagePath: string
}

export interface CatalogProduct {
  id: string
  handle: string
  category: string
  title: string
  subtitle: string
  description: string
  provenanceOrSpec: string
  details: { label: string; value: string }[]
  variants: ProductVariantOption[]
  accessoryAddon?: {
    variantId: string
    title: string
    subtitle: string
    priceMinor: number
  }
}

export type TenantSlug = "maison-vela" | "kova" | "homedine"

export interface TenantDefinition {
  tenantId: string
  slug: TenantSlug
  brandName: string
  tagline: string
  primaryDomain: string
  freeSubdomain: string
  allowedHosts: string[]
  defaultPlan: TenantPlan
  currency: StoreCurrency
  seoTitle: string
  seoDescription: string
  contactEmail: string
  publishedTheme: ValidatedThemeConfig
  draftTheme: ValidatedThemeConfig
  catalog: CatalogProduct[]
}

export interface ResolvedTenantContext {
  tenantId: string
  slug: TenantSlug
  brandName: string
  tagline: string
  resolvedHost: string
  canonicalOrigin: string
  plan: TenantPlan
  /** Core-enforced plan rule: Free stores always render "Powered by Vendryx"; Pro & Scale are white-label. */
  showPlatformBadge: boolean
  currency: StoreCurrency
  mode: ThemePublishMode
  themeId: ThemeId
  themeConfig: ValidatedThemeConfig
  cacheKey: string
  seoTitle: string
  seoDescription: string
  contactEmail: string
  catalog: CatalogProduct[]
}

const MAISON_PUBLISHED_THEME: ValidatedThemeConfig = {
  schemaVersion: THEME_SCHEMA_VERSION,
  themeId: "maison-cuir",
  versionTag: "v1.4.2-live",
  tokens: {
    canvas: "#F6F3EE",
    surface: "#EFEAE1",
    surfaceAlt: "#E7DFD3",
    ink: "#171513",
    inkMuted: "#686158",
    border: "#DBD3C6",
    accent: "#171513",
    accentInk: "#F6F3EE",
    heroBg: "#1F1B17",
    heroInk: "#FAF7F2",
    fontPair: "cormorant-jakarta",
    radiusCardPx: 20,
    radiusButtonPx: 999,
    headerStyle: "editorial-centered",
    cardAspect: "portrait-3-4",
    iconStrokeWeight: "hairline-1.1",
  },
  navigation: [
    { label: "Showcase", href: "#showcase" },
    { label: "Collection", href: "#collection" },
    { label: "Patina", href: "#patina" },
    { label: "Atelier", href: "#craft" },
  ],
  sections: [
    {
      id: "hero-autumn",
      type: "hero",
      headline: "Sculpted in vegetable-tanned Tuscan calfskin.",
      subcopy:
        "Architectural leather goods hand-stitched in small numbered batches—designed to soften and patinate over decades of travel.",
      ctaLabel: "Explore the Collection",
      ctaHref: "#collection",
      mediaPath: "/media/leather-hero.jpg",
      mediaAlt: "Maison Véla saddle-cognac calfskin weekender resting on a sunlit travertine plinth",
    },
    {
      id: "featured-bags",
      type: "featured_collection",
      heading: "Permanent Leather Objects",
      subcopy:
        "Full-grain hides cut from a single bend, edge-painted by hand, and fitted with brushed solid brass hardware.",
    },
    {
      id: "craft-tannery",
      type: "craft_story",
      heading: "Forty days in chestnut bark vats. Zero synthetic coating.",
      body: "Every Maison Véla silhouette begins at a third-generation family tannery in Santa Croce sull'Arno. Unlike chrome-tanned leather sealed under plastic pigment, our full-grain calfskin breathes, darkens in warm amber tones, and carries the quiet record of every journey.",
      originNote: "Vegetable-tanned Vachetta · Hand-burnished edges · Brushed solid brass",
      mediaPath: "/media/leather-craft.jpg",
      mediaAlt: "Close-up of espresso full-grain leather stitching and brass saddlery awl on travertine",
    },
  ],
  footerNote: "Maison Véla Atelier · Complimentary linen dust bag and insured courier delivery.",
}

const MAISON_DRAFT_THEME: ValidatedThemeConfig = {
  ...MAISON_PUBLISHED_THEME,
  versionTag: "v1.5.0-draft",
  tokens: {
    ...MAISON_PUBLISHED_THEME.tokens,
    canvas: "#F3EEE6",
    accent: "#7A4628",
    accentInk: "#FAF7F2",
  },
  sections: [
    {
      ...MAISON_PUBLISHED_THEME.sections[0],
      type: "hero",
      headline: "Autumn Atelier: Warm Cognac & Obsidian Box Calf.",
      subcopy:
        "Previewing Draft v1.5.0 — burnished saddle accents and unlined suede interiors crafted for quiet permanence.",
      ctaLabel: "Preview Autumn Pieces",
      ctaHref: "#collection",
      mediaPath: "/media/leather-hero.jpg",
      mediaAlt: "Maison Véla saddle-cognac calfskin weekender resting on a sunlit travertine plinth",
    },
    MAISON_PUBLISHED_THEME.sections[1],
    MAISON_PUBLISHED_THEME.sections[2],
  ],
}

const KOVA_PUBLISHED_THEME: ValidatedThemeConfig = {
  schemaVersion: THEME_SCHEMA_VERSION,
  themeId: "hydro-vessel",
  versionTag: "v2.4.0-live",
  tokens: {
    canvas: "#F3F6F4",
    surface: "#E8EEEA",
    surfaceAlt: "#DDE5E0",
    ink: "#111614",
    inkMuted: "#48544E",
    border: "#D2DCD6",
    accent: "#3D5740",
    accentInk: "#F3F6F4",
    heroBg: "#0C100E",
    heroInk: "#EFF3F1",
    fontPair: "outfit-dmsans",
    radiusCardPx: 12,
    radiusButtonPx: 999,
    headerStyle: "kinetic-split",
    cardAspect: "studio-4-3",
    iconStrokeWeight: "industrial-2.2",
  },
  navigation: [
    { label: "Showcase", href: "#showcase" },
    { label: "Thermal", href: "#thermal" },
    { label: "Specs", href: "#craft" },
    { label: "Configure", href: "#order" },
  ],
  sections: [
    {
      id: "hero-altitude",
      type: "hero",
      headline: "Built for altitude.",
      subcopy:
        "A copper-lined 18/8 stainless vessel built for long ridge days and everyday commutes—keeping drinks glacial cold or steaming without ever sweating in your bag.",
      ctaLabel: "Shop Flask",
      ctaHref: "#order",
      mediaPath: "/media/kova-hero-studio.jpg",
      mediaAlt: "KOVA Alpine Flask 950 ml in matte pine-sage powder coat with machined black chug lid",
    },
    {
      id: "featured-vessels",
      type: "featured_collection",
      heading: "Configure Your Vessel",
      subcopy:
        "Choose finish and capacity. Every flask ships plastic-free with a lifetime thermal seal guarantee.",
    },
    {
      id: "thermal-specs",
      type: "performance_specs",
      heading: "Double-wall vacuum shield.",
      subcopy:
        "Lab-tested vacuum insulation paired with food-grade 18/8 electropolished stainless steel—zero metallic aftertaste.",
      specs: [
        {
          metric: "24h Cold",
          label: "Copper-lined vacuum wall",
          detail: "Holds iced water crisp through full summer commutes and alpine trails.",
        },
        {
          metric: "12h Hot",
          label: "Zero exterior condensation",
          detail: "Keeps pour-over coffee steaming without warming your hand or pack.",
        },
        {
          metric: "18/8 Steel",
          label: "Pure taste interior",
          detail: "Electropolished medical-grade steel resists flavor carryover and odors.",
        },
        {
          metric: "100% Seal",
          label: "Throw-in-bag leakproof",
          detail: "Silicone gasket lock tested upside-down under cabin pressure.",
        },
      ],
    },
  ],
  footerNote: "Built for altitude · Lifetime thermal seal warranty.",
}

const KOVA_DRAFT_THEME: ValidatedThemeConfig = {
  ...KOVA_PUBLISHED_THEME,
  versionTag: "v2.5.0-draft",
  tokens: {
    ...KOVA_PUBLISHED_THEME.tokens,
    accent: "#284B63",
    accentInk: "#F3F6F4",
  },
  sections: [
    {
      ...KOVA_PUBLISHED_THEME.sections[0],
      type: "hero",
      headline: "Glacier Edition: Built for altitude.",
      subcopy:
        "Previewing Draft v2.5.0 — deep alpine slate accent styling with zero-condensation copper vacuum shielding.",
      ctaLabel: "Preview Glacier Edition",
      ctaHref: "#order",
      mediaPath: "/media/kova-hero-studio.jpg",
      mediaAlt: "KOVA Alpine Flask 950 ml in matte pine-sage powder coat with machined black chug lid",
    },
    KOVA_PUBLISHED_THEME.sections[1],
    KOVA_PUBLISHED_THEME.sections[2],
  ],
}

const HOMEDINE_PUBLISHED_THEME: ValidatedThemeConfig = {
  schemaVersion: THEME_SCHEMA_VERSION,
  themeId: "homedine-eco",
  versionTag: "v1.0.0-live",
  tokens: {
    canvas: "#FAFAF7",
    surface: "#FFFFFF",
    surfaceAlt: "#EFF1E7",
    ink: "#1A1F1C",
    inkMuted: "#4E5651",
    border: "#E2E5DC",
    accent: "#0E5437",
    accentInk: "#FFFFFF",
    heroBg: "#10251C",
    heroInk: "#FFFFFF",
    fontPair: "playfair-jakarta",
    radiusCardPx: 20,
    radiusButtonPx: 999,
    headerStyle: "organic-floating",
    cardAspect: "portrait-3-4",
    iconStrokeWeight: "botanical-1.75",
  },
  navigation: [
    { label: "Shop", href: "#bestsellers" },
    { label: "Bestsellers", href: "#spotlights" },
    { label: "Gallery", href: "#gallery" },
    { label: "About", href: "#about" },
  ],
  sections: [
    {
      id: "hero-homedine",
      type: "hero",
      headline: "Eco-Friendly Kitchenware for a greener home",
      subcopy:
        "Thoughtfully crafted, non-toxic kitchen essentials made from sustainable materials for calmer, planet-first everyday cooking.",
      ctaLabel: "Shop now",
      ctaHref: "#bestsellers",
      mediaPath: "/media/homedine/hero_eco_kitchen_1795688655405.jpg",
      mediaAlt: "Sunlit emerald-green eco-friendly modern kitchen with herbs and ceramic bowls",
    },
    {
      id: "bestsellers-homedine",
      type: "featured_collection",
      heading: "Bestselling Products",
      subcopy:
        "Non-toxic ceramic cookware, recycled steel drinkware, and FSC-certified bamboo utensils crafted for daily rituals.",
    },
    {
      id: "craft-homedine",
      type: "craft_story",
      heading: "We craft kitchenware you can trust for years to come.",
      body: "Each piece is thoughtfully made with 96% recycled steel, borosilicate glass, mineral ceramic glazes free of PTFE, and FSC-certified bamboo.",
      originNote: "Natural Finish · Eco Innovation · Sustainable Materials",
      mediaPath: "/media/homedine/lifestyle_sage_kitchen_1795688662182.jpg",
      mediaAlt: "Serene contemporary sage-green architectural kitchen with white stone island",
    },
  ],
  footerNote: "Homedine Kitchenware · Plastic-free shipping · Certified non-toxic materials.",
}

const HOMEDINE_DRAFT_THEME: ValidatedThemeConfig = {
  ...HOMEDINE_PUBLISHED_THEME,
  versionTag: "v1.1.0-draft",
  tokens: {
    ...HOMEDINE_PUBLISHED_THEME.tokens,
    accent: "#166544",
    accentInk: "#FAFAF7",
  },
  sections: [
    {
      ...HOMEDINE_PUBLISHED_THEME.sections[0],
      type: "hero",
      headline: "Harvest Edition: Eco-Friendly Kitchenware for a greener home",
      subcopy:
        "Previewing Draft v1.1.0 — small-batch olivewood boards, fluted stoneware kettles, and zero-PFAS mineral cookware.",
      ctaLabel: "Explore Harvest Preview",
      ctaHref: "#bestsellers",
      mediaPath: "/media/homedine/hero_eco_kitchen_1795688655405.jpg",
      mediaAlt: "Sunlit emerald-green eco-friendly modern kitchen with herbs and ceramic bowls",
    },
    HOMEDINE_PUBLISHED_THEME.sections[1],
    HOMEDINE_PUBLISHED_THEME.sections[2],
  ],
}

export const TENANT_DIRECTORY: Record<TenantSlug, TenantDefinition> = {
  "maison-vela": {
    tenantId: "ten_maison_vela_01",
    slug: "maison-vela",
    brandName: "MAISON VÉLA",
    tagline: "Architectural Leather Goods",
    primaryDomain: "maisonvela.com",
    freeSubdomain: "maisonvela.vendryx.shop",
    allowedHosts: ["maisonvela.com", "www.maisonvela.com", "maisonvela.vendryx.shop"],
    defaultPlan: "pro",
    currency: "USD",
    seoTitle: "Maison Véla — Architectural Leather Goods",
    seoDescription:
      "Hand-stitched full-grain Tuscan calfskin totes, saddle bags, and weekenders crafted for permanence.",
    contactEmail: "concierge@maisonvela.com",
    publishedTheme: MAISON_PUBLISHED_THEME,
    draftTheme: MAISON_DRAFT_THEME,
    catalog: [
      {
        id: "mv-tote-01",
        handle: "atelier-structured-tote",
        category: "Totes",
        title: "Le Cabas Structuré",
        subtitle: "Pebbled Tuscan Calfskin · Brushed Brass Clasp",
        description:
          "Cut from full-grain vegetable-tanned calfskin with hand-folded gussets and an unlined suede interior pocket that fits a 14-inch laptop cleanly.",
        provenanceOrSpec: "Santa Croce sull'Arno Tannery · Hand-painted edges",
        details: [
          { label: "Dimensions", value: "34 cm × 25 cm × 13 cm" },
          { label: "Leather", value: "Full-grain vegetable-tanned calfskin" },
          { label: "Hardware", value: "Solid brushed brass turn-clasp" },
          { label: "Interior", value: "14\" laptop sleeve + zip card pocket" },
        ],
        variants: [
          {
            id: "mv-tote-cognac",
            label: "Saddle Cognac",
            colorName: "Saddle Cognac",
            swatchHex: "#8F4F2B",
            priceMinor: 48500,
            inStock: true,
            imagePath: "/media/leather-tote.jpg",
          },
          {
            id: "mv-tote-obsidian",
            label: "Noir Box Calf",
            colorName: "Noir Box Calf",
            swatchHex: "#1C1B1A",
            priceMinor: 48500,
            inStock: true,
            imagePath: "/media/leather-crossbody.jpg",
          },
          {
            id: "mv-tote-espresso",
            label: "Dark Espresso",
            colorName: "Dark Espresso",
            swatchHex: "#3E2A21",
            priceMinor: 51000,
            inStock: true,
            imagePath: "/media/leather-hero.jpg",
          },
        ],
      },
      {
        id: "mv-saddle-02",
        handle: "nocturne-saddle-crossbody",
        category: "Crossbody",
        title: "La Besace Nocturne",
        subtitle: "Smooth Box Calfskin · Adjustable Bridle Strap",
        description:
          "A crisp architectural half-moon silhouette with tonal saddle stitching, magnetic concealed flap closure, and an adjustable solid-brass pin buckle.",
        provenanceOrSpec: "Hand-stitched in small batches · Numbered dust bag",
        details: [
          { label: "Dimensions", value: "23 cm × 18 cm × 7.5 cm" },
          { label: "Strap Drop", value: "44 cm – 56 cm adjustable" },
          { label: "Leather", value: "Smooth matte box calfskin" },
          { label: "Closure", value: "Concealed magnetic tab + brass stud" },
        ],
        variants: [
          {
            id: "mv-saddle-noir",
            label: "Obsidian Noir",
            colorName: "Obsidian Noir",
            swatchHex: "#1A1A1A",
            priceMinor: 39000,
            inStock: true,
            imagePath: "/media/leather-crossbody.jpg",
          },
          {
            id: "mv-saddle-cognac",
            label: "Warm Cognac",
            colorName: "Warm Cognac",
            swatchHex: "#95532E",
            priceMinor: 39000,
            inStock: true,
            imagePath: "/media/leather-tote.jpg",
          },
        ],
      },
      {
        id: "mv-weekender-03",
        handle: "voyageur-48h-weekender",
        category: "Travel",
        title: "Le Voyageur 48H",
        subtitle: "Drum-Dyed Saddle Hide · Cabin-Approved",
        description:
          "Built for two-day architectural travel with rolled hand-sewn handles, brass roller buckle strap, and reinforced base corners that hold their structure.",
        provenanceOrSpec: "Cabin compliant · Detachable shoulder pad included",
        details: [
          { label: "Dimensions", value: "49 cm × 29 cm × 22 cm" },
          { label: "Capacity", value: "32 Liters · International cabin fit" },
          { label: "Leather", value: "Drum-dyed full-grain saddle hide" },
          { label: "Lining", value: "Heavy organic cotton herringbone twill" },
        ],
        variants: [
          {
            id: "mv-weekender-saddle",
            label: "Heritage Saddle",
            colorName: "Heritage Saddle",
            swatchHex: "#8B4B24",
            priceMinor: 74000,
            inStock: true,
            imagePath: "/media/leather-hero.jpg",
          },
          {
            id: "mv-weekender-espresso",
            label: "Espresso Grain",
            colorName: "Espresso Grain",
            swatchHex: "#3C281E",
            priceMinor: 76000,
            inStock: true,
            imagePath: "/media/leather-craft.jpg",
          },
        ],
      },
    ],
  },
  kova: {
    tenantId: "ten_kova_hydro_02",
    slug: "kova",
    brandName: "KOVA",
    tagline: "Insulated Everyday Vessels",
    primaryDomain: "kovahydration.com",
    freeSubdomain: "kova.vendryx.shop",
    allowedHosts: ["kovahydration.com", "www.kovahydration.com", "kova.vendryx.shop"],
    defaultPlan: "pro",
    currency: "USD",
    seoTitle: "KOVA — Copper-Lined Insulated Bottles & Tumblers",
    seoDescription:
      "Sweat-free matte powder-coated stainless steel water bottles and commuter tumblers built to stay ice-cold for 24 hours.",
    contactEmail: "hello@kovahydration.com",
    publishedTheme: KOVA_PUBLISHED_THEME,
    draftTheme: KOVA_DRAFT_THEME,
    catalog: [
      {
        id: "kv-apex-01",
        handle: "kova-alpine-flask",
        category: "Flagship Vessel",
        title: "KOVA Alpine Flask",
        subtitle: "24h Cold · 12h Hot · Copper-Lined 18/8 Steel",
        description:
          "A copper-lined 18/8 stainless vessel built for long ridge days and everyday commutes—keeping drinks glacial cold or steaming without ever sweating in your bag.",
        provenanceOrSpec: "18/8 Pro-Grade Steel · Leakproof Chug Lid · Lifetime Seal Warranty",
        details: [
          { label: "Insulation", value: "24h Glacial Cold / 12h Steaming Hot" },
          { label: "Material", value: "18/8 electropolished double-wall steel" },
          { label: "Finish", value: "Micro-textured matte mineral powder coat" },
          { label: "Empty Weight", value: "520 g (950 ml)" },
        ],
        variants: [
          {
            id: "kv-alpine-500",
            label: "500 ml",
            colorName: "Alpine Sage",
            swatchHex: "#48634B",
            sizeLabel: "500 ml",
            priceMinor: 3800,
            inStock: true,
            imagePath: "/media/kova-hero-studio.jpg",
          },
          {
            id: "kv-alpine-750",
            label: "750 ml",
            colorName: "Alpine Sage",
            swatchHex: "#48634B",
            sizeLabel: "750 ml",
            priceMinor: 4400,
            inStock: true,
            imagePath: "/media/kova-hero-studio.jpg",
          },
          {
            id: "kv-alpine-950",
            label: "950 ml",
            colorName: "Alpine Sage",
            swatchHex: "#48634B",
            sizeLabel: "950 ml",
            priceMinor: 4800,
            inStock: true,
            imagePath: "/media/kova-hero-studio.jpg",
          },
        ],
        accessoryAddon: {
          variantId: "kv-addon-boot-01",
          title: "Silicone Protective Base Boot",
          subtitle: "Quiet desk landings & granite impact shield",
          priceMinor: 900,
        },
      },
      {
        id: "kv-commuter-02",
        handle: "kova-flip-commuter-tumbler",
        category: "Coffee & Cold Brew",
        title: "KOVA Flip Commuter",
        subtitle: "Splash-Proof Sip Lid · Fits 99% Cup Holders",
        description:
          "Engineered for one-handed sips on the morning drive with a ceramic-smooth steel interior that preserves espresso crema and iced matcha without metallic taste.",
        provenanceOrSpec: "Tapered 72mm Base · One-Hand Flip Lock",
        details: [
          { label: "Insulation", value: "16h Cold / 8h Hot" },
          { label: "Base Diameter", value: "72 mm universal car cup-holder fit" },
          { label: "Lid", value: "Locking flip-sip silicone gasket" },
          { label: "Cleaning", value: "Top-rack dishwasher safe lid & body" },
        ],
        variants: [
          {
            id: "kv-tumbler-coral-500",
            label: "Canyon Coral · 500ml",
            colorName: "Canyon Coral",
            swatchHex: "#DF6747",
            sizeLabel: "500ml",
            priceMinor: 3400,
            inStock: true,
            imagePath: "/media/bottle-coral.jpg",
          },
          {
            id: "kv-tumbler-cobalt-500",
            label: "Tidepool Cobalt · 500ml",
            colorName: "Tidepool Cobalt",
            swatchHex: "#1449B8",
            sizeLabel: "500ml",
            priceMinor: 3400,
            inStock: true,
            imagePath: "/media/bottle-cobalt.jpg",
          },
          {
            id: "kv-tumbler-obsidian-750",
            label: "Obsidian Matte · 750ml",
            colorName: "Obsidian Matte",
            swatchHex: "#1D2228",
            sizeLabel: "750ml",
            priceMinor: 4000,
            inStock: true,
            imagePath: "/media/bottle-obsidian.jpg",
          },
        ],
        accessoryAddon: {
          variantId: "kv-addon-straw-02",
          title: "Stainless Straw + Brush Kit",
          subtitle: "Soft silicone tip + flexible cleaner",
          priceMinor: 800,
        },
      },
      {
        id: "kv-trail-03",
        handle: "kova-trail-1l-wide-mouth",
        category: "1L Wide-Mouth",
        title: "KOVA Trail 1L",
        subtitle: "Knurled Grip Band · High-Flow Chug Neck",
        description:
          "Maximum all-day hydration with a machined knurled grip band for wet hands, rapid ice-cube fill wide mouth, and heavy-gauge dent-resistant steel walls.",
        provenanceOrSpec: "30h Ice Retention · Knurled Tactile Collar",
        details: [
          { label: "Capacity", value: "1,000 ml (34 oz)" },
          { label: "Insulation", value: "30h Ice Cold / 14h Hot" },
          { label: "Grip", value: "360° anodized knurled band" },
          { label: "Cap", value: "Machined stainless insulated screw cap" },
        ],
        variants: [
          {
            id: "kv-trail-obsidian-1l",
            label: "Obsidian Matte · 1L",
            colorName: "Obsidian Matte",
            swatchHex: "#1D2228",
            sizeLabel: "1L",
            priceMinor: 4900,
            inStock: true,
            imagePath: "/media/bottle-obsidian.jpg",
          },
          {
            id: "kv-trail-cobalt-1l",
            label: "Tidepool Cobalt · 1L",
            colorName: "Tidepool Cobalt",
            swatchHex: "#1449B8",
            sizeLabel: "1L",
            priceMinor: 4900,
            inStock: true,
            imagePath: "/media/bottle-cobalt.jpg",
          },
          {
            id: "kv-trail-coral-750",
            label: "Canyon Coral · 750ml",
            colorName: "Canyon Coral",
            swatchHex: "#DF6747",
            sizeLabel: "750ml",
            priceMinor: 4300,
            inStock: true,
            imagePath: "/media/bottle-coral.jpg",
          },
        ],
        accessoryAddon: {
          variantId: "kv-addon-cap-03",
          title: "Quick-Flip Sport Straw Lid",
          subtitle: "Interchangeable wide-mouth cap",
          priceMinor: 1200,
        },
      },
    ],
  },
  homedine: {
    tenantId: "ten_homedine_eco_03",
    slug: "homedine",
    brandName: "Homedine",
    tagline: "Eco-Friendly Kitchenware",
    primaryDomain: "homedine.shop",
    freeSubdomain: "homedine.vendryx.shop",
    allowedHosts: ["homedine.shop", "www.homedine.shop", "homedine.vendryx.shop"],
    defaultPlan: "pro",
    currency: "USD",
    seoTitle: "Homedine — Eco-Friendly Kitchenware for a Greener Home",
    seoDescription:
      "Non-toxic ceramic cookware, recycled stainless drinkware, borosilicate storage jars, and FSC-certified bamboo kitchen essentials.",
    contactEmail: "hello@homedine.shop",
    publishedTheme: HOMEDINE_PUBLISHED_THEME,
    draftTheme: HOMEDINE_DRAFT_THEME,
    catalog: [
      {
        id: "eco-bottle-01",
        handle: "freshpitcher-thermal-bottle",
        category: "Drinkware",
        title: "FreshPitcher Thermal Bottle",
        subtitle: "90% Recycled Steel · 24h Cold / 12h Warm",
        description:
          "Double-walled vacuum insulated bottle crafted from 90% recycled food-grade 18/8 stainless steel. Keeps drinks cold for 24 hours or warm for 12 hours without metallic aftertaste.",
        provenanceOrSpec: "90% Recycled Steel · BPA & Phthalate Free · Carbon-Neutral Finish",
        details: [
          { label: "Material", value: "90% recycled 18/8 stainless steel" },
          { label: "Thermal", value: "24h Cold / 12h Warm" },
          { label: "Finish", value: "Carbon-neutral botanical matte coat" },
        ],
        variants: [
          {
            id: "eco-bottle-01::pine",
            label: "Forest Pine",
            colorName: "Forest Pine",
            swatchHex: "#4B6353",
            priceMinor: 4385,
            inStock: true,
            imagePath: "/media/homedine/product_eco_bottle_1795688656957.jpg",
          },
          {
            id: "eco-bottle-01::mist",
            label: "Glacier Mist",
            colorName: "Glacier Mist",
            swatchHex: "#A5C1CC",
            priceMinor: 4385,
            inStock: true,
            imagePath: "/media/homedine/product_eco_bottle_1795688656957.jpg",
          },
          {
            id: "eco-bottle-01::slate",
            label: "River Slate",
            colorName: "River Slate",
            swatchHex: "#76807C",
            priceMinor: 4385,
            inStock: true,
            imagePath: "/media/homedine/product_eco_bottle_1795688656957.jpg",
          },
        ],
      },
      {
        id: "ceramic-pots-02",
        handle: "terra-olive-ceramic-pot-duo",
        category: "Cookware",
        title: "Terra & Olive Ceramic Pot Duo",
        subtitle: "Zero PFAS / PTFE · Recycled Aluminum Core",
        description:
          "Mineral-derived ceramic non-stick coating free of PTFE, PFOA, and lead. Engineered with recycled aluminum cores for fast, energy-saving heat distribution on induction and gas ranges.",
        provenanceOrSpec: "Zero PFAS / PTFE · Recycled Aluminum Core · Tempered Glass Lids",
        details: [
          { label: "Coating", value: "Mineral ceramic free of PTFE/PFOA" },
          { label: "Compatibility", value: "Induction, gas, and electric" },
          { label: "Core", value: "Recycled aluminum heat spreader" },
        ],
        variants: [
          {
            id: "ceramic-pots-02::olive",
            label: "Chartreuse Olive",
            colorName: "Chartreuse Olive",
            swatchHex: "#9BA84B",
            priceMinor: 7835,
            inStock: true,
            imagePath: "/media/homedine/product_ceramic_pots_1795688657805.jpg",
          },
          {
            id: "ceramic-pots-02::sage",
            label: "Muted Sage",
            colorName: "Muted Sage",
            swatchHex: "#8FA48F",
            priceMinor: 7835,
            inStock: true,
            imagePath: "/media/homedine/product_ceramic_pots_1795688657805.jpg",
          },
          {
            id: "ceramic-pots-02::clay",
            label: "Warm Terracotta",
            colorName: "Warm Terracotta",
            swatchHex: "#C7784C",
            priceMinor: 7835,
            inStock: true,
            imagePath: "/media/homedine/product_ceramic_pots_1795688657805.jpg",
          },
        ],
      },
      {
        id: "sage-kettle-03",
        handle: "fluted-sage-bamboo-kettle",
        category: "Appliances",
        title: "Fluted Sage Bamboo Kettle",
        subtitle: "FSC-Certified Bamboo · Low-Draw Eco Boil",
        description:
          "Rapid-boil precision electric kettle pairing a fluted matte ceramic-finish exterior with FSC-certified natural bamboo accents and an automatic energy-saving shutoff.",
        provenanceOrSpec: "FSC-Certified Bamboo · Low-Draw Eco Boil · Plastic-Free Interior",
        details: [
          { label: "Accents", value: "FSC-certified natural bamboo handle & base" },
          { label: "Interior", value: "100% plastic-free stainless chamber" },
          { label: "Shutoff", value: "Automatic low-draw eco cut-off" },
        ],
        variants: [
          {
            id: "sage-kettle-03::sage",
            label: "Botanical Sage",
            colorName: "Botanical Sage",
            swatchHex: "#7B947E",
            priceMinor: 14365,
            inStock: true,
            imagePath: "/media/homedine/product_sage_kettle_1795688657240.jpg",
          },
          {
            id: "sage-kettle-03::sand",
            label: "Oatmeal Sand",
            colorName: "Oatmeal Sand",
            swatchHex: "#B9AA95",
            priceMinor: 14365,
            inStock: true,
            imagePath: "/media/homedine/product_sage_kettle_1795688657240.jpg",
          },
          {
            id: "sage-kettle-03::charcoal",
            label: "Basalt Moss",
            colorName: "Basalt Moss",
            swatchHex: "#5D6B62",
            priceMinor: 14365,
            inStock: true,
            imagePath: "/media/homedine/product_sage_kettle_1795688657240.jpg",
          },
        ],
      },
      {
        id: "bamboo-utensils-04",
        handle: "ochre-crock-bamboo-utensil-set",
        category: "Utensils",
        title: "Ochre Crock & Bamboo Utensil Set",
        subtitle: "100% Organic Moso Bamboo · Lead-Free Stoneware",
        description:
          "Six hand-finished organic bamboo kitchen tools housed in a kiln-glazed stoneware crock. Naturally antibacterial, gentle on ceramic cookware, and sealed with food-safe plant oil.",
        provenanceOrSpec: "100% Organic Moso Bamboo · Lead-Free Stoneware Glaze",
        details: [
          { label: "Tools", value: "6 hand-oiled Moso bamboo utensils" },
          { label: "Holder", value: "Kiln-glazed lead-free stoneware crock" },
          { label: "Packaging", value: "100% home-compostable pulp wrap" },
        ],
        variants: [
          {
            id: "bamboo-utensils-04::ochre",
            label: "Harvest Ochre",
            colorName: "Harvest Ochre",
            swatchHex: "#C8A939",
            priceMinor: 2627,
            inStock: true,
            imagePath: "/media/homedine/product_bamboo_utensils_1795688660630.jpg",
          },
          {
            id: "bamboo-utensils-04::moss",
            label: "Deep Moss",
            colorName: "Deep Moss",
            swatchHex: "#587462",
            priceMinor: 2627,
            inStock: true,
            imagePath: "/media/homedine/product_bamboo_utensils_1795688660630.jpg",
          },
          {
            id: "bamboo-utensils-04::stone",
            label: "Warm Clay",
            colorName: "Warm Clay",
            swatchHex: "#A59E92",
            priceMinor: 2627,
            inStock: true,
            imagePath: "/media/homedine/product_bamboo_utensils_1795688660630.jpg",
          },
        ],
      },
      {
        id: "coffee-carafe-05",
        handle: "botanical-pour-over-carafe",
        category: "Drinkware",
        title: "Botanical Pour-Over Carafe",
        subtitle: "Recyclable Borosilicate · Natural Bamboo Collar",
        description:
          "Hand-blown borosilicate glass server paired with a heat-safe FSC bamboo collar and ribbed ceramic pour-over cone for clean, zero-capsule morning brews.",
        provenanceOrSpec: "Recyclable Borosilicate · Natural Bamboo Collar · Zero-Waste Brewing",
        details: [
          { label: "Server", value: "Hand-blown thermal borosilicate glass" },
          { label: "Dripper", value: "Ribbed stoneware pour-over cone" },
          { label: "Collar", value: "Removable FSC bamboo heat wrap" },
        ],
        variants: [
          {
            id: "coffee-carafe-05::sage",
            label: "Sage Ceramic",
            colorName: "Sage Ceramic",
            swatchHex: "#7A927E",
            priceMinor: 5400,
            inStock: true,
            imagePath: "/media/homedine/product_coffee_carafe_1795688667299.jpg",
          },
          {
            id: "coffee-carafe-05::sand",
            label: "Linen Cream",
            colorName: "Linen Cream",
            swatchHex: "#D4CCBE",
            priceMinor: 5400,
            inStock: true,
            imagePath: "/media/homedine/product_coffee_carafe_1795688667299.jpg",
          },
          {
            id: "coffee-carafe-05::espresso",
            label: "Roast Clay",
            colorName: "Roast Clay",
            swatchHex: "#6B5648",
            priceMinor: 5400,
            inStock: true,
            imagePath: "/media/homedine/product_coffee_carafe_1795688667299.jpg",
          },
        ],
      },
      {
        id: "storage-jars-06",
        handle: "airtight-bamboo-lid-jar-set",
        category: "Storage",
        title: "Airtight Bamboo-Lid Jar Set",
        subtitle: "Lead-Free Fluted Glass · Plastic-Free Pantry",
        description:
          "Three fluted pantry vessels built to keep grains, coffee beans, and loose-leaf teas crisp with platinum-grade silicone seals and sustainably harvested bamboo lids.",
        provenanceOrSpec: "Lead-Free Fluted Glass · Airtight Food-Safe Seal · Plastic-Free Pantry",
        details: [
          { label: "Vessels", value: "Set of 3 ribbed borosilicate glass jars" },
          { label: "Lids", value: "FSC bamboo with platinum silicone seal" },
          { label: "Care", value: "Dishwasher-safe glass jars" },
        ],
        variants: [
          {
            id: "storage-jars-06::natural",
            label: "Natural Bamboo",
            colorName: "Natural Bamboo",
            swatchHex: "#C7A97E",
            priceMinor: 3850,
            inStock: true,
            imagePath: "/media/homedine/product_storage_jars_1795688667137.jpg",
          },
          {
            id: "storage-jars-06::walnut",
            label: "Smoked Walnut",
            colorName: "Smoked Walnut",
            swatchHex: "#7B6450",
            priceMinor: 3850,
            inStock: true,
            imagePath: "/media/homedine/product_storage_jars_1795688667137.jpg",
          },
          {
            id: "storage-jars-06::sage-seal",
            label: "Sage Silicone",
            colorName: "Sage Silicone",
            swatchHex: "#889E8B",
            priceMinor: 3850,
            inStock: true,
            imagePath: "/media/homedine/product_storage_jars_1795688667137.jpg",
          },
        ],
      },
    ],
  },
}

const VALID_HOSTNAME_PATTERN = /^[a-z0-9.-]{1,253}$/

export function normalizeAndValidateHost(rawHost: string | null | undefined): string | null {
  if (!rawHost) return null
  const firstHost = rawHost.split(",")[0]?.trim().toLowerCase() ?? ""
  const withoutPort = firstHost.replace(/:\d+$/, "")
  if (!withoutPort || !VALID_HOSTNAME_PATTERN.test(withoutPort) || withoutPort.includes("..")) {
    return null
  }
  return withoutPort
}

export function isRegisteredTenantHost(hostname: string): boolean {
  return Object.values(TENANT_DIRECTORY).some((t) => t.allowedHosts.includes(hostname))
}

export function buildTenantCacheKey(params: {
  tenantId: string
  route: string
  currency: StoreCurrency
  themeVersion: string
}): string {
  const safeRoute = params.route.replace(/[^a-zA-Z0-9/_-]/g, "") || "/"
  return `sf:${params.tenantId}:${params.themeVersion}:${params.currency}:${safeRoute}`
}

/**
 * Generates a host-only Set-Cookie header string.
 * Intentionally omits `Domain=` so Free subdomains (*.vendryx.shop) NEVER share cookies across tenants.
 */
export function buildHostOnlyCookie(
  name: string,
  value: string,
  maxAgeSeconds = 60 * 60 * 24 * 14
): string {
  const cleanName = name.replace(/[^a-zA-Z0-9_-]/g, "")
  const encodedValue = encodeURIComponent(value)
  return `${cleanName}=${encodedValue}; Path=/; Max-Age=${Math.max(0, Math.trunc(maxAgeSeconds))}; HttpOnly; Secure; SameSite=Lax`
}

export function resolveTenantRequest(params: {
  rawHost?: string | null
  storeQuery?: string | null
  planQuery?: string | null
  modeQuery?: string | null
  route?: string
}): ResolvedTenantContext {
  const normalizedHost = normalizeAndValidateHost(params.rawHost)

  let matchedTenant: TenantDefinition | undefined
  if (normalizedHost) {
    matchedTenant = Object.values(TENANT_DIRECTORY).find((t) =>
      t.allowedHosts.includes(normalizedHost)
    )
  }

  // In local / AI Studio preview environments (localhost, 127.0.0.1, *.run.app),
  // default to the flagship KOVA bottle storefront unless ?store=maison-vela or ?store=homedine is requested.
  if (!matchedTenant) {
    if (params.storeQuery === "maison-vela") {
      matchedTenant = TENANT_DIRECTORY["maison-vela"]
    } else if (params.storeQuery === "homedine") {
      matchedTenant = TENANT_DIRECTORY.homedine
    } else {
      matchedTenant = TENANT_DIRECTORY.kova
    }
  }

  const effectivePlan: TenantPlan =
    params.planQuery === "free"
      ? "free"
      : params.planQuery === "scale"
        ? "scale"
        : params.planQuery === "pro"
          ? "pro"
          : normalizedHost && normalizedHost.endsWith(".vendryx.shop")
            ? "free"
            : matchedTenant.defaultPlan

  const mode: ThemePublishMode = params.modeQuery === "draft" ? "draft" : "published"
  const rawTheme = mode === "draft" ? matchedTenant.draftTheme : matchedTenant.publishedTheme
  const validated = validateThemeConfig(rawTheme, matchedTenant.publishedTheme)

  const resolvedHost =
    effectivePlan === "free" ? matchedTenant.freeSubdomain : matchedTenant.primaryDomain

  const cacheKey = buildTenantCacheKey({
    tenantId: matchedTenant.tenantId,
    route: params.route ?? "/",
    currency: matchedTenant.currency,
    themeVersion: validated.config.versionTag,
  })

  return {
    tenantId: matchedTenant.tenantId,
    slug: matchedTenant.slug,
    brandName: matchedTenant.brandName,
    tagline: matchedTenant.tagline,
    resolvedHost,
    canonicalOrigin: `https://${resolvedHost}`,
    plan: effectivePlan,
    showPlatformBadge: effectivePlan === "free",
    currency: matchedTenant.currency,
    mode,
    themeId: validated.config.themeId,
    themeConfig: validated.config,
    cacheKey,
    seoTitle: matchedTenant.seoTitle,
    seoDescription: matchedTenant.seoDescription,
    contactEmail: matchedTenant.contactEmail,
    catalog: matchedTenant.catalog,
  }
}
