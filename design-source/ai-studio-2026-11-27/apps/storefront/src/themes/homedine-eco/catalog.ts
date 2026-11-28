export interface HomedineSwatch {
  id: string
  name: string
  hex: string
  tintFilter?: string
}

export interface HomedineProduct {
  id: string
  name: string
  shortName: string
  category: string
  categoryId: string
  price: number
  priceMinor: number
  tag?: string
  image: string
  alt: string
  swatches: HomedineSwatch[]
  description: string
  sustainabilitySpecs: string[]
  rating: number
  reviewCount: number
}

export interface HomedineCategory {
  id: string
  prefix: string
  title: string
  subtitle: string
  image: string
  alt: string
  filterCategoryId: string
}

export interface HomedineSpotlight {
  id: string
  headingPlain: string
  headingItalic: string
  description: string
  image: string
  alt: string
  productId: string
  reverse?: boolean
}

export interface HomedineGalleryItem {
  id: string
  title: string
  caption: string
  image: string
  alt: string
  linkedProductId: string
  aspectVariant: "tall" | "square" | "offset"
}

export interface HomedineReview {
  id: string
  quote: string
  author: string
  role: string
  rating: number
  verifiedProduct: string
}

export const MOTION_TOKENS = {
  ease: {
    expoOut: "expo.out",
    power3Out: "power3.out",
    power2Out: "power2.out",
    none: "none",
  },
  duration: {
    reveal: 0.62,
    heroTitle: 0.9,
    counter: 1.4,
  },
  stagger: {
    tight: 0.05,
    cards: 0.065,
  },
  scrollTrigger: {
    defaultStart: "top 90%",
    earlyStart: "top 95%",
    parallaxScrub: 0.7,
  },
} as const

export const SCROLL_TOKENS = {
  duration: 1.15,
  wheelMultiplier: 0.95,
  touchMultiplier: 1.4,
  anchorOffset: -20,
  easing: (t: number) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
} as const

export const HOMEDINE_STATS = {
  ecoMaterialStat: 96,
  freeShippingThresholdMinor: 15000, // Synced with Vendryx locked checkout engine ($150)
} as const

export const ASSETS = {
  heroKitchen: "/media/homedine/hero_eco_kitchen_1795688655405.jpg",
  ecoBottle: "/media/homedine/product_eco_bottle_1795688656957.jpg",
  ceramicPots: "/media/homedine/product_ceramic_pots_1795688657805.jpg",
  sageKettle: "/media/homedine/product_sage_kettle_1795688657240.jpg",
  bambooUtensils: "/media/homedine/product_bamboo_utensils_1795688660630.jpg",
  lifestyleKitchen: "/media/homedine/lifestyle_sage_kitchen_1795688662182.jpg",
  catCeramicCup: "/media/homedine/cat_ceramic_cup_1795688662040.jpg",
  catTravelCutlery: "/media/homedine/cat_travel_cutlery_1795688662327.jpg",
  spotlightRiceCookers: "/media/homedine/spotlight_rice_cookers_1795688662608.jpg",
  spotlightEmeraldSteamer: "/media/homedine/spotlight_emerald_steamer_1795688664175.jpg",
  gallerySkilletToast: "/media/homedine/gallery_skillet_toast_1795688665023.jpg",
  galleryWoodenBoards: "/media/homedine/gallery_wooden_boards_1795688665729.jpg",
  coffeeCarafe: "/media/homedine/product_coffee_carafe_1795688667299.jpg",
  storageJars: "/media/homedine/product_storage_jars_1795688667137.jpg",
} as const

export const PRODUCTS: HomedineProduct[] = [
  {
    id: "eco-bottle-01",
    name: "Reusable drinkware for a greener lifestyle.",
    shortName: "FreshPitcher Thermal Bottle",
    category: "Drinkware",
    categoryId: "freshpitcher",
    price: 43.85,
    priceMinor: 4385,
    tag: "Promotion",
    image: ASSETS.ecoBottle,
    alt: "Matte forest-green insulated stainless steel reusable eco water bottle on studio backdrop",
    swatches: [
      { id: "pine", name: "Forest Pine", hex: "#4B6353" },
      {
        id: "mist",
        name: "Glacier Mist",
        hex: "#A5C1CC",
        tintFilter: "hue-rotate(52deg) saturate(0.85) brightness(1.06)",
      },
      {
        id: "slate",
        name: "River Slate",
        hex: "#76807C",
        tintFilter: "saturate(0.35) brightness(1.02)",
      },
    ],
    description:
      "Double-walled vacuum insulated bottle crafted from 90% recycled food-grade 18/8 stainless steel. Keeps drinks cold for 24 hours or warm for 12 hours without metallic aftertaste.",
    sustainabilitySpecs: ["90% Recycled Steel", "BPA & Phthalate Free", "Carbon-Neutral Finish"],
    rating: 4.9,
    reviewCount: 412,
  },
  {
    id: "ceramic-pots-02",
    name: "Non-toxic cookware for sustainable cooking.",
    shortName: "Terra & Olive Ceramic Pot Duo",
    category: "Cookware",
    categoryId: "cookware",
    price: 78.35,
    priceMinor: 7835,
    tag: "New",
    image: ASSETS.ceramicPots,
    alt: "Olive-green stock pot and terracotta saucepan non-toxic ceramic cookware set",
    swatches: [
      { id: "olive", name: "Chartreuse Olive", hex: "#9BA84B" },
      {
        id: "sage",
        name: "Muted Sage",
        hex: "#8FA48F",
        tintFilter: "hue-rotate(34deg) saturate(0.75)",
      },
      {
        id: "clay",
        name: "Warm Terracotta",
        hex: "#C7784C",
        tintFilter: "hue-rotate(-28deg) saturate(1.05)",
      },
    ],
    description:
      "Mineral-derived ceramic non-stick coating free of PTFE, PFOA, and lead. Engineered with recycled aluminum cores for fast, energy-saving heat distribution on induction and gas ranges.",
    sustainabilitySpecs: ["Zero PFAS / PTFE", "Recycled Aluminum Core", "Tempered Glass Lids"],
    rating: 4.9,
    reviewCount: 289,
  },
  {
    id: "sage-kettle-03",
    name: "Kettle & Toaster eco-friendly meals.",
    shortName: "Fluted Sage Bamboo Kettle",
    category: "Appliances",
    categoryId: "cupeco",
    price: 143.65,
    priceMinor: 14365,
    tag: "Customer favorite",
    image: ASSETS.sageKettle,
    alt: "Fluted matte sage-green electric kettle with natural bamboo wood base and handle",
    swatches: [
      { id: "sage", name: "Botanical Sage", hex: "#7B947E" },
      {
        id: "sand",
        name: "Oatmeal Sand",
        hex: "#B9AA95",
        tintFilter: "hue-rotate(-68deg) saturate(0.55) brightness(1.04)",
      },
      {
        id: "charcoal",
        name: "Basalt Moss",
        hex: "#5D6B62",
        tintFilter: "saturate(0.55) brightness(0.92)",
      },
    ],
    description:
      "Rapid-boil precision electric kettle pairing a fluted matte ceramic-finish exterior with FSC-certified natural bamboo accents and an automatic energy-saving shutoff.",
    sustainabilitySpecs: ["FSC-Certified Bamboo", "Low-Draw Eco Boil", "Plastic-Free Interior"],
    rating: 5.0,
    reviewCount: 634,
  },
  {
    id: "bamboo-utensils-04",
    name: "Bamboo Made Utensil Holder",
    shortName: "Ochre Crock & Bamboo Utensil Set",
    category: "Utensils",
    categoryId: "naturesip",
    price: 26.27,
    priceMinor: 2627,
    tag: "New",
    image: ASSETS.bambooUtensils,
    alt: "Mustard-ochre ceramic utensil crock filled with handcrafted natural bamboo cooking spoons",
    swatches: [
      { id: "ochre", name: "Harvest Ochre", hex: "#C8A939" },
      {
        id: "moss",
        name: "Deep Moss",
        hex: "#587462",
        tintFilter: "hue-rotate(92deg) saturate(0.7)",
      },
      {
        id: "stone",
        name: "Warm Clay",
        hex: "#A59E92",
        tintFilter: "saturate(0.3) brightness(1.03)",
      },
    ],
    description:
      "Six hand-finished organic bamboo kitchen tools housed in a kiln-glazed stoneware crock. Naturally antibacterial, gentle on ceramic cookware, and sealed with food-safe plant oil.",
    sustainabilitySpecs: [
      "100% Organic Moso Bamboo",
      "Lead-Free Stoneware Glaze",
      "Compostable Packaging",
    ],
    rating: 4.8,
    reviewCount: 318,
  },
  {
    id: "coffee-carafe-05",
    name: "Pour-Over Glass & Sage Coffee Carafe",
    shortName: "Botanical Pour-Over Carafe",
    category: "Drinkware",
    categoryId: "cupeco",
    price: 54.0,
    priceMinor: 5400,
    tag: "Customer favorite",
    image: ASSETS.coffeeCarafe,
    alt: "Borosilicate glass pour-over coffee carafe with bamboo collar and ribbed sage dripper",
    swatches: [
      { id: "sage", name: "Sage Ceramic", hex: "#7A927E" },
      {
        id: "sand",
        name: "Linen Cream",
        hex: "#D4CCBE",
        tintFilter: "saturate(0.4) brightness(1.06)",
      },
      {
        id: "espresso",
        name: "Roast Clay",
        hex: "#6B5648",
        tintFilter: "hue-rotate(-78deg) brightness(0.94)",
      },
    ],
    description:
      "Hand-blown borosilicate glass server paired with a heat-safe FSC bamboo collar and ribbed ceramic pour-over cone for clean, zero-capsule morning brews.",
    sustainabilitySpecs: ["Recyclable Borosilicate", "Natural Bamboo Collar", "Zero-Waste Brewing"],
    rating: 4.9,
    reviewCount: 196,
  },
  {
    id: "storage-jars-06",
    name: "Ribbed Borosilicate Pantry Jar Trio",
    shortName: "Airtight Bamboo-Lid Jar Set",
    category: "Storage",
    categoryId: "ecospoonery",
    price: 38.5,
    priceMinor: 3850,
    tag: "New",
    image: ASSETS.storageJars,
    alt: "Trio of ribbed clear glass pantry storage jars with natural bamboo lids",
    swatches: [
      { id: "natural", name: "Natural Bamboo", hex: "#C7A97E" },
      {
        id: "walnut",
        name: "Smoked Walnut",
        hex: "#7B6450",
        tintFilter: "brightness(0.93) contrast(1.05)",
      },
      {
        id: "sage-seal",
        name: "Sage Silicone",
        hex: "#889E8B",
        tintFilter: "hue-rotate(28deg) saturate(0.9)",
      },
    ],
    description:
      "Three fluted pantry vessels built to keep grains, coffee beans, and loose-leaf teas crisp with platinum-grade silicone seals and sustainably harvested bamboo lids.",
    sustainabilitySpecs: ["Lead-Free Fluted Glass", "Airtight Food-Safe Seal", "Plastic-Free Pantry"],
    rating: 4.9,
    reviewCount: 521,
  },
]

export const CATEGORIES: HomedineCategory[] = [
  {
    id: "cat-cupeco",
    prefix: "Explore",
    title: "CupEco",
    subtitle: "Hand-thrown stoneware & eco kettles",
    image: ASSETS.catCeramicCup,
    alt: "Handcrafted dark ceramic coffee cup resting on carved cedar pedestal",
    filterCategoryId: "cupeco",
  },
  {
    id: "cat-ecospoonery",
    prefix: "Explore",
    title: "EcoSpoonery",
    subtitle: "Reusable travel cutlery & pantry storage",
    image: ASSETS.catTravelCutlery,
    alt: "Reusable stainless steel and charcoal cutlery in a sage wheat-straw case",
    filterCategoryId: "ecospoonery",
  },
  {
    id: "cat-naturesip",
    prefix: "Explore",
    title: "NatureSip",
    subtitle: "Organic bamboo cooking tools & holders",
    image: ASSETS.bambooUtensils,
    alt: "Handcrafted bamboo cooking spoons and spatulas in ceramic holder",
    filterCategoryId: "naturesip",
  },
  {
    id: "cat-freshpitcher",
    prefix: "Explore",
    title: "FreshPitcher",
    subtitle: "Insulated bottles & zero-waste hydration",
    image: ASSETS.ecoBottle,
    alt: "Matte forest-green insulated thermal eco flask",
    filterCategoryId: "freshpitcher",
  },
  {
    id: "cat-cookware",
    prefix: "Explore",
    title: "EarthPot",
    subtitle: "Non-toxic ceramic stock pots & skillets",
    image: ASSETS.ceramicPots,
    alt: "Olive and terracotta ceramic non-toxic cooking pots",
    filterCategoryId: "cookware",
  },
  {
    id: "cat-brew",
    prefix: "Explore",
    title: "PureBrew",
    subtitle: "Hand-blown glass pour-over rituals",
    image: ASSETS.coffeeCarafe,
    alt: "Glass pour-over coffee carafe with bamboo collar",
    filterCategoryId: "cupeco",
  },
]

export const SPOTLIGHTS: HomedineSpotlight[] = [
  {
    id: "spotlight-bestsellers",
    headingPlain: "Best",
    headingItalic: "sellers",
    description:
      "A polished ceramic rice cooker rests on a rustic wooden surface, encircled by fresh herbs — a perfect blend of durability and nature for mindful everyday cooking.",
    image: ASSETS.spotlightRiceCookers,
    alt: "Three minimalist matte sage, cream, and blush ceramic rice cookers on sculptural stoneware pedestals",
    productId: "ceramic-pots-02",
    reverse: false,
  },
  {
    id: "spotlight-new-arrival",
    headingPlain: "New",
    headingItalic: "Arrival",
    description:
      "Our OasiSteam multi-cooker showcases a colorful array of fresh, seasonal vegetables — a vibrant celebration of zero-waste, planet-friendly kitchen practices.",
    image: ASSETS.spotlightEmeraldSteamer,
    alt: "Deep forest emerald-green ceramic electric steamer pot on sunlit table with roasted seasonal vegetables",
    productId: "sage-kettle-03",
    reverse: true,
  },
]

export const GALLERY_ITEMS: HomedineGalleryItem[] = [
  {
    id: "gal-1",
    title: "SizzlePro Non-Stick Pan",
    caption: "PFAS-free mineral ceramic skillet for crisp morning toast and slow sautés.",
    image: ASSETS.gallerySkilletToast,
    alt: "Deep teal non-toxic ceramic frying pan with toasted sourdough bread on marble counter",
    linkedProductId: "ceramic-pots-02",
    aspectVariant: "square",
  },
  {
    id: "gal-2",
    title: "Grain Slice Board Duo",
    caption: "Hand-carved Mediterranean olivewood boards finished in organic beeswax.",
    image: ASSETS.galleryWoodenBoards,
    alt: "Four sculptural olivewood cutting and serving boards on dark slate",
    linkedProductId: "bamboo-utensils-04",
    aspectVariant: "tall",
  },
  {
    id: "gal-3",
    title: "Bamboo Utensil Set",
    caption: "Plastic-free travel and prep utensils engineered for years of daily cooking.",
    image: ASSETS.catTravelCutlery,
    alt: "Reusable sage wheat-straw case with charcoal stainless cutlery",
    linkedProductId: "bamboo-utensils-04",
    aspectVariant: "offset",
  },
  {
    id: "gal-4",
    title: "Glow Pot Ceramic",
    caption: "Low-wattage ceramic cooking pots crafted for tender grains and broths.",
    image: ASSETS.spotlightRiceCookers,
    alt: "Minimalist matte sage and alabaster eco ceramic rice cookers",
    linkedProductId: "ceramic-pots-02",
    aspectVariant: "tall",
  },
  {
    id: "gal-5",
    title: "StoneSip Ceramic Cup",
    caption: "Small-batch wheel-thrown stoneware that retains warmth naturally.",
    image: ASSETS.catCeramicCup,
    alt: "Artisanal dark stoneware coffee cup on carved oak block",
    linkedProductId: "coffee-carafe-05",
    aspectVariant: "square",
  },
]

export const REVIEWS: HomedineReview[] = [
  {
    id: "rev-1",
    quote:
      "Homedine's glass jars are awesome for storage, and the bamboo utensils are perfect for daily use without scratching our pans.",
    author: "Jane Cooper",
    role: "Nutritionist",
    rating: 5,
    verifiedProduct: "Ribbed Borosilicate Pantry Jar Trio",
  },
  {
    id: "rev-2",
    quote:
      "Fantastic products and fast plastic-free delivery. My kitchen feels calmer, cleaner, and so much greener every morning!",
    author: "Darlene Robertson",
    role: "Culinary Instructor",
    rating: 5,
    verifiedProduct: "Fluted Sage Bamboo Kettle",
  },
  {
    id: "rev-3",
    quote:
      "Love Homedine's eco-style! The ceramic pots heat evenly on induction, wipe clean effortlessly, and look sculptural on the stove.",
    author: "Jacob Jones",
    role: "Food Blogger",
    rating: 5,
    verifiedProduct: "Non-toxic cookware for sustainable cooking.",
  },
  {
    id: "rev-4",
    quote:
      "Switching our studio prep station to Homedine bamboo tools and insulated bottles cut single-use waste immediately.",
    author: "Esther Howard",
    role: "Sous Chef",
    rating: 5,
    verifiedProduct: "Bamboo Made Utensil Holder",
  },
]

export const PILLARS = [
  {
    id: "natural-finish",
    italicLabel: "Natural",
    boldLabel: "Finish",
    detail: "Plant-based waxes and mineral glazes free of synthetic PTFE.",
  },
  {
    id: "eco-innovation",
    italicLabel: "Eco",
    boldLabel: "Innovation",
    detail: "Energy-efficient thermal vessels built for decades of daily meals.",
  },
  {
    id: "sustainable-materials",
    italicLabel: "Sustainable",
    boldLabel: "Materials",
    detail: "96% recycled steel, borosilicate glass, and FSC-certified bamboo.",
  },
] as const

export function getActiveSwatch(
  product: HomedineProduct,
  selectedSwatchId?: string
): HomedineSwatch {
  if (!selectedSwatchId) return product.swatches[0]
  return product.swatches.find((swatch) => swatch.id === selectedSwatchId) ?? product.swatches[0]
}

export function filterHomedineProducts(
  products: HomedineProduct[],
  query: string,
  categoryId: string | null = null
): HomedineProduct[] {
  const normalizedQuery = query.trim().toLowerCase()
  return products.filter((product) => {
    const matchesCategory =
      !categoryId || categoryId === "all" || product.categoryId === categoryId
    if (!matchesCategory) return false
    if (!normalizedQuery) return true
    return (
      product.name.toLowerCase().includes(normalizedQuery) ||
      product.shortName.toLowerCase().includes(normalizedQuery) ||
      product.category.toLowerCase().includes(normalizedQuery) ||
      product.description.toLowerCase().includes(normalizedQuery)
    )
  })
}

export function getNextCarouselIndex(
  current: number,
  totalItems: number,
  visibleCount: number
): number {
  if (totalItems <= visibleCount) return 0
  const maxStart = totalItems - visibleCount
  return current >= maxStart ? 0 : current + 1
}

export function getPrevCarouselIndex(
  current: number,
  totalItems: number,
  visibleCount: number
): number {
  if (totalItems <= visibleCount) return 0
  const maxStart = totalItems - visibleCount
  return current <= 0 ? maxStart : current - 1
}

export function getVisibleWindow<T>(items: T[], startIndex: number, visibleCount: number): T[] {
  if (items.length <= visibleCount) return items
  const safeStart = Math.max(0, Math.min(startIndex, items.length - visibleCount))
  return items.slice(safeStart, safeStart + visibleCount)
}

export function formatUsd(amount: number): string {
  return `$${amount.toFixed(2)}`
}
