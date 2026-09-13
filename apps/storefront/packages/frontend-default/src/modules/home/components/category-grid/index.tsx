import LocalizedClientLink from "@modules/common/components/localized-client-link"

// ----------------------------------------------------------------------------
// Section 3: Curated Quick Rail Items
// ----------------------------------------------------------------------------
const QUICK_RAIL_ITEMS = [
  {
    title: "Scarlet Zardozi Bridal Lehenga",
    subtitle: "Pure Raw Silk with Gold Dabka",
    image: "https://images.unsplash.com/photo-1610030469983-98e550d6193c?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Midnight Blue Velvet Anarkali",
    subtitle: "Heavy Tilla Embroidery & Organza Dupatta",
    image: "https://images.unsplash.com/photo-1583391733975-f2d3284f676b?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Ivory & Gold Groom Sherwani",
    subtitle: "Hand-Embroidered Chanderi Silk",
    image: "https://images.unsplash.com/photo-1594938298603-c8148c4dae35?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Crimson Banarasi Heritage Saree",
    subtitle: "Real Zari Katan Silk Weave",
    image: "https://images.unsplash.com/photo-1617627143750-d86bc21e42bb?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
]

// ----------------------------------------------------------------------------
// Section 4: Tri-Card Editorial Trio
// ----------------------------------------------------------------------------
const EDITORIAL_TRIO = [
  {
    title: "Velvet Heritage Menswear",
    subtitle: "Hand-Tailored Bandhgalas & Jackets",
    image: "https://images.unsplash.com/photo-1507679799987-c73779587ccf?q=80&w=800&auto=format&fit=crop",
    href: "/store",
    accent: "bg-red-950/20",
  },
  {
    title: "The Meadow Romance Edit",
    subtitle: "Floral Threadwork & Pastel Tulle",
    image: "https://images.unsplash.com/photo-1485230895905-ec40ba36b9bc?q=80&w=800&auto=format&fit=crop",
    href: "/store",
    accent: "bg-stone-900/10",
  },
  {
    title: "Rose Gold Bridal Splendor",
    subtitle: "Metallic Sequins & Swarvoski Details",
    image: "https://images.unsplash.com/photo-1515372039744-b8f02a3ae446?q=80&w=800&auto=format&fit=crop",
    href: "/store",
    accent: "bg-amber-900/10",
  },
]

// ----------------------------------------------------------------------------
// Section 5: "SHOP BY STYLE" (8 Silhouettes in 4x2 Grid)
// ----------------------------------------------------------------------------
const STYLE_SILHOUETTES = [
  {
    title: "Bridal Lehengas",
    count: "Couture Edit",
    image: "https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Shararas & Ghararas",
    count: "Festive Prêt",
    image: "https://images.unsplash.com/photo-1515886657613-9f3515b0c78f?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Anarkalis & Flared Suits",
    count: "Heritage Silks",
    image: "https://images.unsplash.com/photo-1490481651871-ab68de25d43d?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Classic Sarees",
    count: "Handloom Banarasi",
    image: "https://images.unsplash.com/photo-1610030469983-98e550d6193c?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Velvet Formals",
    count: "Winter Soirée",
    image: "https://images.unsplash.com/photo-1583391733975-f2d3284f676b?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Festive Kurtas & Sets",
    count: "Ready to Wear",
    image: "https://images.unsplash.com/photo-1509631179647-0177331693ae?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Pastel Florals",
    count: "Day Wedding",
    image: "https://images.unsplash.com/photo-1539571696357-5a69c17a67c6?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Haldi & Mehendi",
    count: "Vibrant Celebrations",
    image: "https://images.unsplash.com/photo-1434389677669-e08b4cac3105?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
]

export default function CategoryGrid() {
  return (
    <div className="w-full space-y-16 sm:space-y-24 py-12 sm:py-16">
      {/* ------------------------------------------------------------------------ */}
      {/* Section 3: Curated Quick Rail (Best Sellers / New Arrivals)              */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
          {/* Promo Card on Left */}
          <div className="lg:col-span-3 rounded-2xl bg-stone-950 text-white p-6 sm:p-8 flex flex-col justify-between shadow-md border border-stone-800 relative overflow-hidden">
            <div className="absolute top-0 right-0 w-48 h-48 bg-amber-500/10 rounded-full blur-2xl pointer-events-none" />
            <div className="space-y-3 relative z-10">
              <span className="text-[10px] font-mono uppercase tracking-widest text-amber-300">
                Curated Spotlight
              </span>
              <h3 className="text-2xl sm:text-3xl font-serif font-bold leading-tight">
                New Arrivals &amp; Best Sellers
              </h3>
              <p className="text-xs text-stone-300 font-light leading-relaxed">
                Most coveted couture ensembles from the atelier runway.
              </p>
            </div>
            <div className="pt-6 relative z-10">
              <LocalizedClientLink
                href="/store"
                className="inline-block w-full py-3 px-4 bg-white text-stone-950 text-center rounded-xl text-xs font-semibold uppercase tracking-wider hover:bg-stone-100 transition-colors shadow-sm"
              >
                View Complete Rail
              </LocalizedClientLink>
            </div>
          </div>

          {/* 4 Cards on Right */}
          <div className="lg:col-span-9 grid grid-cols-2 sm:grid-cols-4 gap-4">
            {QUICK_RAIL_ITEMS.map((item, idx) => (
              <LocalizedClientLink
                key={idx}
                href={item.href}
                className="group flex flex-col rounded-2xl overflow-hidden bg-white border border-brand-hairline shadow-sm hover:shadow-md transition-all"
              >
                <div className="relative aspect-[3/4] overflow-hidden bg-stone-100">
                  <img
                    src={item.image}
                    alt={item.title}
                    className="w-full h-full object-cover object-top group-hover:scale-105 transition-transform duration-500"
                    loading="lazy"
                  />
                  <span className="absolute top-2.5 right-2.5 w-7 h-7 rounded-full bg-white/80 backdrop-blur-sm text-stone-900 flex items-center justify-center text-xs opacity-0 group-hover:opacity-100 transition-opacity shadow-sm">
                    ♡
                  </span>
                </div>
                <div className="p-3 flex flex-col space-y-1">
                  <h4 className="text-xs font-semibold text-stone-900 line-clamp-1 group-hover:text-amber-700 transition-colors">
                    {item.title}
                  </h4>
                  <p className="text-[10px] text-stone-500 font-light line-clamp-1">
                    {item.subtitle}
                  </p>
                </div>
              </LocalizedClientLink>
            ))}
          </div>
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 4: Tri-Card Campaign Editorial (3 Vertical Columns)              */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {EDITORIAL_TRIO.map((item, idx) => (
            <LocalizedClientLink
              key={idx}
              href={item.href}
              className="group flex flex-col rounded-2xl overflow-hidden bg-white border border-brand-hairline shadow-sm hover:shadow-xl transition-all"
            >
              <div className="relative aspect-[4/5] overflow-hidden bg-stone-900">
                <img
                  src={item.image}
                  alt={item.title}
                  className="w-full h-full object-cover object-top group-hover:scale-105 transition-transform duration-700"
                  loading="lazy"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-60 group-hover:opacity-80 transition-opacity" />
                <div className="absolute bottom-4 inset-x-4 flex justify-between items-end text-white">
                  <span className="text-xs font-semibold uppercase tracking-wider">
                    Explore
                  </span>
                  <span className="text-sm transform group-hover:translate-x-1 transition-transform">
                    →
                  </span>
                </div>
              </div>
              <div className="p-4 sm:p-5 flex flex-col space-y-1 bg-brand-surface">
                <h3 className="text-base sm:text-lg font-serif font-bold text-stone-900 group-hover:text-amber-700 transition-colors">
                  {item.title}
                </h3>
                <p className="text-xs text-stone-500 font-light">
                  {item.subtitle}
                </p>
              </div>
            </LocalizedClientLink>
          ))}
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 5: "SHOP BY STYLE" (8-Card 4x2 Grid)                             */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container">
        <div className="text-center mb-10 sm:mb-12 space-y-2">
          <span className="text-xs font-mono uppercase tracking-[0.25em] text-stone-500 block">
            Wardrobe Silhouettes
          </span>
          <h2 className="text-3xl sm:text-4xl font-serif font-bold tracking-tight text-stone-900">
            SHOP BY STYLE
          </h2>
          <div className="w-12 h-0.5 bg-amber-500/60 mx-auto mt-3" />
        </div>

        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 sm:gap-6">
          {STYLE_SILHOUETTES.map((style, idx) => (
            <LocalizedClientLink
              key={idx}
              href={style.href}
              className="group relative flex flex-col rounded-2xl overflow-hidden aspect-[3/4] bg-stone-950 shadow-sm hover:shadow-lg transition-all"
            >
              <img
                src={style.image}
                alt={style.title}
                className="w-full h-full object-cover object-top group-hover:scale-105 transition-transform duration-700 opacity-90 group-hover:opacity-100"
                loading="lazy"
              />
              <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/20 to-transparent" />
              <div className="absolute bottom-3 sm:bottom-4 inset-x-3 sm:inset-x-4 text-center text-white">
                <h4 className="text-xs sm:text-sm font-serif font-bold tracking-wide group-hover:text-amber-200 transition-colors">
                  {style.title}
                </h4>
                <span className="text-[10px] text-stone-300 font-mono tracking-wider uppercase block mt-0.5 opacity-80">
                  {style.count}
                </span>
              </div>
            </LocalizedClientLink>
          ))}
        </div>
      </section>
    </div>
  )
}
