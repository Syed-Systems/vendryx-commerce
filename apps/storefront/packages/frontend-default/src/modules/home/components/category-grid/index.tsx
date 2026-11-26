import Image from "next/image"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

// ----------------------------------------------------------------------------
// Section 3: Curated Quick Rail Items
// ----------------------------------------------------------------------------
const QUICK_RAIL_ITEMS = [
  {
    title: "Scarlet Zardozi Bridal Lehenga",
    subtitle: "Pure Raw Silk with Gold Dabka",
    image: "/brand/editorial/rail-bridal.jpg",
    href: "/store",
  },
  {
    title: "Midnight Blue Velvet Anarkali",
    subtitle: "Heavy Tilla Embroidery & Organza Dupatta",
    image: "/brand/editorial/rail-anarkali.jpg",
    href: "/store",
  },
  {
    title: "Ivory & Gold Groom Sherwani",
    subtitle: "Hand-Embroidered Chanderi Silk",
    image: "/brand/editorial/rail-sherwani.jpg",
    href: "/store",
  },
  {
    title: "Crimson Banarasi Heritage Saree",
    subtitle: "Real Zari Katan Silk Weave",
    image: "/brand/editorial/rail-saree.jpg",
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
    image: "/brand/editorial/editorial-menswear.jpg",
    href: "/store",
  },
  {
    title: "The Meadow Romance Edit",
    subtitle: "Floral Threadwork & Pastel Tulle",
    image: "/brand/editorial/editorial-portrait.jpg",
    href: "/store",
  },
  {
    title: "Rose Gold Bridal Splendor",
    subtitle: "Metallic Sequins & Crystal Details",
    image: "/brand/editorial/editorial-jewellery.jpg",
    href: "/store",
  },
]

// ----------------------------------------------------------------------------
// Section 5: "SHOP BY STYLE" (8 Silhouettes in 4x2 Grid)
// ----------------------------------------------------------------------------
const STYLE_SILHOUETTES = [
  {
    title: "Bridal Lehengas",
    count: "Couture Edit",
    image: "/brand/editorial/style-bridal-1.jpg",
    href: "/store",
  },
  {
    title: "Shararas & Ghararas",
    count: "Festive Prêt",
    image: "/brand/editorial/style-bridal-2.jpg",
    href: "/store",
  },
  {
    title: "Anarkalis & Flared Suits",
    count: "Heritage Silks",
    image: "/brand/editorial/style-bridal-3.jpg",
    href: "/store",
  },
  {
    title: "Classic Sarees",
    count: "Handloom Banarasi",
    image: "/brand/editorial/style-anarkali.jpg",
    href: "/store",
  },
  {
    title: "Velvet Formals",
    count: "Winter Soirée",
    image: "/brand/editorial/style-saree.jpg",
    href: "/store",
  },
  {
    title: "Festive Kurtas & Sets",
    count: "Ready to Wear",
    image: "/brand/editorial/style-menswear-1.jpg",
    href: "/store",
  },
  {
    title: "Pastel Florals",
    count: "Day Wedding",
    image: "/brand/editorial/style-menswear-2.jpg",
    href: "/store",
  },
  {
    title: "Haldi & Mehendi",
    count: "Vibrant Celebrations",
    image: "/brand/editorial/style-sherwani.jpg",
    href: "/store",
  },
]

/** Shared heading for the two centred sections. */
function SectionHeading({
  eyebrow,
  title,
  id,
}: {
  eyebrow: string
  title: string
  id: string
}) {
  return (
    <div className="text-center mb-10 sm:mb-14 space-y-3">
      <span className="label-eyebrow text-[color:var(--ink-faint)] font-mono">{eyebrow}</span>
      <h2 id={id} className="display-section font-heading font-bold text-[color:var(--ink)]">
        {title}
      </h2>
      {/* The rule was `bg-amber-500/60`, a literal that no tenant theme could
          change. It reads the accent token now. */}
      <div
        aria-hidden="true"
        className="w-12 h-0.5 mx-auto"
        style={{ background: "var(--brand-accent)" }}
      />
    </div>
  )
}

export default function CategoryGrid() {
  return (
    <div className="w-full space-y-12 sm:space-y-16 py-8 sm:py-12">
      {/* ---------------------------------------------------------------------- */}
      {/* Section 3: Curated Quick Rail                                            */}
      {/* ---------------------------------------------------------------------- */}
      <section className="content-container" aria-labelledby="rail-heading">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
          <div className="lg:col-span-3 glass-dark rounded-[var(--brand-radius-card)] text-[color:var(--on-dark)] p-6 sm:p-8 flex flex-col justify-between border border-[color:var(--brand-hairline)]/10">
            <div className="space-y-3">
              <span className="label-eyebrow text-[color:var(--on-dark)]">Curated Spotlight</span>
              <h2 id="rail-heading" className="text-2xl sm:text-3xl font-heading font-bold leading-tight">
                New Arrivals &amp; Best Sellers
              </h2>
              <p className="body-copy text-[color:var(--on-dark)]">
                Most coveted couture ensembles from the atelier runway.
              </p>
            </div>
            <div className="pt-6">
              <LocalizedClientLink
                href="/store"
                className="glass-chip inline-flex min-h-[44px] w-full items-center justify-center px-4 py-3 bg-[color:var(--brand-surface)] text-[color:var(--ink)] text-xs font-semibold uppercase tracking-wider hover:bg-[color:var(--brand-surface)] transition-colors"
              >
                View Complete Rail
              </LocalizedClientLink>
            </div>
          </div>

          <ul className="lg:col-span-9 grid grid-cols-2 sm:grid-cols-4 gap-4">
            {QUICK_RAIL_ITEMS.map((item) => (
              <li key={item.title}>
                <LocalizedClientLink
                  href={item.href}
                  className="group flex flex-col h-full rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--brand-surface)] border border-[color:var(--brand-hairline)] shadow-sm hover:shadow-md transition-all"
                >
                  <div className="relative aspect-[3/4] overflow-hidden bg-[color:var(--brand-surface)]">
                    <Image
                      src={item.image}
                      alt={item.title}
                      fill
                      loading="lazy"
                      quality={75}
                      sizes="(max-width: 640px) 45vw, (max-width: 1024px) 22vw, 260px"
                      className="object-cover object-top group-hover:scale-105 transition-transform duration-500"
                    />
                  </div>
                  <div className="p-3 flex flex-col gap-1">
                    <h3 className="text-xs font-semibold text-[color:var(--ink)] line-clamp-1 group-hover:text-brand-accent transition-colors">
                      {item.title}
                    </h3>
                    <p className="text-xs text-[color:var(--ink-muted)] font-light line-clamp-1">
                      {item.subtitle}
                    </p>
                  </div>
                </LocalizedClientLink>
              </li>
            ))}
          </ul>
        </div>
      </section>

      {/* ---------------------------------------------------------------------- */}
      {/* Section 4: Tri-Card Campaign Editorial                                     */}
      {/* ---------------------------------------------------------------------- */}
      <section className="content-container" aria-label="Editorial campaigns">
        <ul className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {EDITORIAL_TRIO.map((item) => (
            <li key={item.title}>
              <LocalizedClientLink
                href={item.href}
                className="group flex flex-col h-full rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--brand-surface)] border border-[color:var(--brand-hairline)] shadow-sm hover:shadow-xl transition-all"
              >
                <div className="relative aspect-[4/5] overflow-hidden bg-[color:var(--band-dark)]">
                  <Image
                    src={item.image}
                    alt={item.title}
                    fill
                    loading="lazy"
                    quality={75}
                    sizes="(max-width: 768px) 100vw, 33vw"
                    className="object-cover object-top group-hover:scale-105 transition-transform duration-700"
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/70 via-transparent to-transparent opacity-70 group-hover:opacity-90 transition-opacity" />
                  <div className="absolute bottom-4 inset-x-4 flex justify-between items-end text-[color:var(--on-dark)]">
                    <span className="label-eyebrow">Explore</span>
                    <span aria-hidden="true" className="text-lg group-hover:translate-x-1 transition-transform">
                      →
                    </span>
                  </div>
                </div>
                <div className="p-4 sm:p-5 flex flex-col gap-1 bg-brand-surface">
                  <h3 className="text-base sm:text-lg font-heading font-bold text-[color:var(--ink)] group-hover:text-brand-accent transition-colors">
                    {item.title}
                  </h3>
                  <p className="body-copy text-[color:var(--ink-muted)]">
                    {item.subtitle}
                  </p>
                </div>
              </LocalizedClientLink>
            </li>
          ))}
        </ul>
      </section>

      {/* ---------------------------------------------------------------------- */}
      {/* Section 5: Shop by Style                                                  */}
      {/* ---------------------------------------------------------------------- */}
      <section className="content-container" aria-labelledby="style-heading">
        <SectionHeading eyebrow="Wardrobe Silhouettes" title="Shop by Style" id="style-heading" />

        <ul className="grid grid-cols-2 sm:grid-cols-4 gap-4 sm:gap-6">
          {STYLE_SILHOUETTES.map((style) => (
            <li key={style.title}>
              <LocalizedClientLink
                href={style.href}
                className="group relative flex flex-col rounded-[var(--brand-radius-card)] overflow-hidden aspect-[3/4] bg-[color:var(--band-dark)] shadow-sm hover:shadow-lg transition-all"
              >
                <Image
                  src={style.image}
                  alt={style.title}
                  fill
                  loading="lazy"
                  quality={75}
                  sizes="(max-width: 640px) 45vw, (max-width: 1024px) 22vw, 300px"
                  className="object-cover object-top group-hover:scale-105 transition-transform duration-700 opacity-90 group-hover:opacity-100"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/85 via-[color:var(--band-dark)]/20 to-transparent" />
                <div className="absolute bottom-3 sm:bottom-4 inset-x-3 sm:inset-x-4 text-center text-[color:var(--on-dark)]">
                  <h3 className="text-xs sm:text-sm font-heading font-bold tracking-wide group-hover:text-brand-accent transition-colors">
                    {style.title}
                  </h3>
                  <span className="text-xs text-[color:var(--on-dark)] font-mono uppercase block mt-1">
                    {style.count}
                  </span>
                </div>
              </LocalizedClientLink>
            </li>
          ))}
        </ul>
      </section>
    </div>
  )
}
