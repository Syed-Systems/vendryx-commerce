import Image from "next/image"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

// ----------------------------------------------------------------------------
// Section 6: "NEW IN COUTURE" (6 Items in 3x2 Grid)
// ----------------------------------------------------------------------------
const COUTURE_PIECES = [
  {
    title: "Ochre Zari Bridal Lehenga",
    subtitle: "Heavy Gotapatti Work on Pure Silk",
    image: "/brand/editorial/couture-1.jpg",
    href: "/store",
  },
  {
    title: "Mustard Mirror-Work Gown",
    subtitle: "Architectural Silhouette & Shish Mahal Embroidery",
    image: "/brand/editorial/couture-2.jpg",
    href: "/store",
  },
  {
    title: "Crimson Velvet Royal Lehenga",
    subtitle: "Antique Zardozi & Hand-Cut Sequins",
    image: "/brand/editorial/couture-3.jpg",
    href: "/store",
  },
  {
    title: "Blush Organza Resham Ensemble",
    subtitle: "Scalloped Borders & Pastel Floral Tulle",
    image: "/brand/editorial/couture-4.jpg",
    href: "/store",
  },
  {
    title: "Ice Blue Threadwork Anarkali",
    subtitle: "Chiffon Dupatta & Pearl Embellishments",
    image: "/brand/editorial/couture-5.jpg",
    href: "/store",
  },
  {
    title: "Lavender Tulle Reception Gown",
    subtitle: "Silver Dabka & Crystal Motifs",
    image: "/brand/editorial/couture-6.jpg",
    href: "/store",
  },
]

// ----------------------------------------------------------------------------
// Section 8: Bridal Highlights (3 Cards)
// ----------------------------------------------------------------------------
const BRIDAL_HIGHLIGHTS = [
  {
    title: "Magenta Heritage Lehenga",
    tag: "Royal Atelier",
    image: "/brand/editorial/celeb-bridal.jpg",
    href: "/store",
  },
  {
    title: "Midnight Velvet Bandhgala Suit",
    tag: "Bespoke Cut",
    image: "/brand/editorial/celeb-redcarpet.jpg",
    href: "/store",
  },
  {
    title: "Lilac Floral Organza Ensemble",
    tag: "Day Wedding",
    image: "/brand/editorial/bridal-palace.jpg",
    href: "/store",
  },
]

// ----------------------------------------------------------------------------
// Section 10: Accessories Small Tiles
// ----------------------------------------------------------------------------
const ACCESSORY_TILES = [
  {
    title: "Embroidered Potlis & Clutches",
    image: "/brand/editorial/highlight-1.jpg",
    href: "/store",
  },
  {
    title: "Royal Emerald Cocktails",
    image: "/brand/editorial/highlight-2.jpg",
    href: "/store",
  },
  {
    title: "Handcrafted Zardozi Mojaris",
    image: "/brand/editorial/highlight-3.jpg",
    href: "/store",
  },
  {
    title: "Heritage Pashmina & Silk Stoles",
    image: "/brand/editorial/centuries.jpg",
    href: "/store",
  },
]

// ----------------------------------------------------------------------------
// Section 11: Menswear Collection (3 Cards)
// ----------------------------------------------------------------------------
const MENSWEAR_PIECES = [
  {
    title: "Royal Ivory Groom Sherwani",
    subtitle: "Chanderi Silk with Thread Embroidery",
    image: "/brand/editorial/jewellery-1.jpg",
    href: "/store",
  },
  {
    title: "Midnight Raw Silk Kurta Set",
    subtitle: "Churidar & Hand-Embroidered Bundi",
    image: "/brand/editorial/jewellery-2.jpg",
    href: "/store",
  },
  {
    title: "Champagne Embroidered Achkan",
    subtitle: "Tussar Silk with Antique Metallic Buttons",
    image: "/brand/editorial/accessory-1.jpg",
    href: "/store",
  },
]

/** Centred section heading. The accent rule reads the token, not `amber-500`. */
function SectionHeading({
  eyebrow,
  title,
  id,
  tone = "light",
}: {
  eyebrow: string
  title: string
  id: string
  tone?: "light" | "dark"
}) {
  return (
    <div className="text-center mb-10 sm:mb-14 space-y-3">
      <span
        className={`label-eyebrow font-mono ${tone === "dark" ? "text-brand-accent" : "text-[color:var(--on-dark-muted)]"}`}
      >
        {eyebrow}
      </span>
      <h2
        id={id}
        className="display-section font-heading font-bold text-[color:var(--on-dark)]"
      >
        {title}
      </h2>
      <div
        aria-hidden="true"
        className="w-12 h-0.5 mx-auto"
        style={{ background: "var(--brand-accent)" }}
      />
    </div>
  )
}

export default function LookbookStory() {
  return (
    <div className="w-full flex flex-col gap-12 sm:gap-16 py-10 sm:py-14">
      {/* ------------------------------------------------------------------------ */}
      {/* Section 6: New in Couture                                                    */}
      {/* ------------------------------------------------------------------------ */}
      <section className="w-full bg-[color:var(--band-dark)] text-[color:var(--on-dark)] py-16 sm:py-24" aria-labelledby="couture-heading">
        <div className="content-container">
          <SectionHeading
            eyebrow="Atelier Runway"
            title="New in Couture"
            id="couture-heading"
            tone="dark"
          />

          <ul className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8">
            {COUTURE_PIECES.map((piece) => (
              <li key={piece.title}>
                <LocalizedClientLink
                  href={piece.href}
                  className="group relative flex flex-col rounded-[var(--brand-radius-card)] overflow-hidden aspect-[3/4] bg-[color:var(--band-dark)] border text-[color:var(--on-dark-muted)]/80"
                >
                  <Image
                    src={piece.image}
                    alt={piece.title}
                    fill
                    loading="lazy"
                    quality={75}
                    sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
                    className="object-cover object-top group-hover:scale-105 transition-transform duration-700 opacity-90 group-hover:opacity-100"
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/85 via-[color:var(--band-dark)]/25 to-transparent" />

                  {/* The shop pill is now the shared glass material rather than a
                      locally invented `bg-[color:var(--brand-surface)]/15 backdrop-blur-md`. */}
                  <div className="absolute bottom-5 inset-x-5 flex flex-col items-center text-center gap-2">
                    <h3 className="text-sm sm:text-base font-heading font-bold text-[color:var(--on-dark)] group-hover:text-brand-accent transition-colors">
                      {piece.title}
                    </h3>
                    <span className="glass-dark glass-chip w-full py-2.5 block text-xs font-semibold uppercase tracking-wider text-[color:var(--on-dark)]">
                      Shop Now
                    </span>
                  </div>
                </LocalizedClientLink>
              </li>
            ))}
          </ul>
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 7: Celebrity Spotting                                                */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container" aria-labelledby="celebrity-heading">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 sm:gap-8 items-stretch">
          <LocalizedClientLink
            href="/store"
            className="lg:col-span-5 group relative rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--band-dark)] border text-[color:var(--on-dark-muted)] min-h-[420px] sm:min-h-[500px]"
          >
            <Image
              src="/brand/editorial/accessory-2.jpg"
              alt="Bridal couture portrait"
              fill
              loading="lazy"
              quality={75}
              sizes="(max-width: 1024px) 100vw, 42vw"
              className="object-cover object-top group-hover:scale-105 transition-transform duration-700"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/75 via-transparent to-transparent" />
            <div className="absolute bottom-6 inset-x-6 text-[color:var(--on-dark)] space-y-1">
              <span className="label-eyebrow text-brand-accent">Bridal Heritage</span>
              <h3 className="text-xl sm:text-2xl font-heading font-bold">
                The Royal Crimson Series
              </h3>
            </div>
          </LocalizedClientLink>

          <LocalizedClientLink
            href="/store"
            className="lg:col-span-7 group relative rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--band-dark)] min-h-[420px] sm:min-h-[500px] flex items-end"
          >
            <Image
              src="/brand/editorial/accessory-3.jpg"
              alt="Red carpet edit"
              fill
              loading="lazy"
              quality={75}
              sizes="(max-width: 1024px) 100vw, 58vw"
              className="object-cover object-center group-hover:scale-105 transition-transform duration-700 opacity-80"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/90 via-[color:var(--band-dark)]/35 to-transparent" />
            <div className="relative z-10 p-6 sm:p-10 text-[color:var(--on-dark)] space-y-4 max-w-xl">
              <span className="glass-dark glass-chip inline-block px-3 py-1.5 text-brand-accent text-xs font-semibold uppercase tracking-widest">
                Red Carpet &amp; Receptions
              </span>
              <h2 id="celebrity-heading" className="display-section font-heading font-bold">
                Celebrity Spotting
              </h2>
              <p className="body-copy text-[color:var(--on-dark)] font-light">
                Bespoke creations made for gala evenings.
              </p>
              <span className="inline-flex items-center gap-2 pt-2 text-xs font-semibold uppercase tracking-wider text-brand-accent group-hover:translate-x-1 transition-transform">
                View the red carpet edit
                <span aria-hidden="true">→</span>
              </span>
            </div>
          </LocalizedClientLink>
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 8: Bridal Edit                                                      */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container space-y-8" aria-labelledby="bridal-heading">
        <SectionHeading
          eyebrow="Couture Masterpieces"
          title="Bridal Edit"
          id="bridal-heading"
        />

        {/* Palace courtyard.

            Two defects fixed. `aspect-[16/9]` and `min-h-[500px]` were both on
            this element and they fight — the aspect ratio sets the box, the
            min-height overrides it, and the result is neither the shape nor the
            height that was intended. Now a single `min-h` with the image
            filling. And the consultation card is the glass material, which is
            the point of the overdrive pass: the same pane that carries the nav
            carries this. */}
        <div className="relative w-full rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--band-dark)] min-h-[380px] sm:min-h-[560px]">
          <Image
            src="/brand/editorial/accessory-4.jpg"
            alt="Palace courtyard bridal showcase"
            fill
            loading="lazy"
            quality={80}
            sizes="(max-width: 1440px) 100vw, 1440px"
            className="object-cover object-center"
          />
          <div className="absolute inset-0 bg-gradient-to-r from-[color:var(--band-dark)]/55 via-transparent to-[color:var(--band-dark)]/70" />

          <div className="glass absolute top-6 bottom-6 right-6 sm:right-10 w-[calc(100%-3rem)] sm:w-auto sm:max-w-sm rounded-[var(--brand-radius-card)] p-6 sm:p-8 flex flex-col justify-between">
            <div className="space-y-3">
              <span className="label-eyebrow" style={{ color: "var(--brand-accent)" }}>
                Atelier Heritage
              </span>
              <h3 className="text-2xl sm:text-3xl font-heading font-bold text-[color:var(--on-dark)] leading-tight">
                The Grand Courtyard Pavilion
              </h3>
              <p className="body-copy text-[color:var(--on-dark-muted)]">
                Expansive flared lehengas, pure silk canopies, and
                micro-embroidered floral borders.
              </p>
            </div>
            <div className="pt-4">
              <LocalizedClientLink
                href="/store"
                className="min-h-[44px] inline-flex w-full items-center justify-center py-3 bg-[color:var(--band-dark)] text-[color:var(--on-dark)] rounded-[var(--brand-radius-card)] text-xs font-semibold uppercase tracking-wider block transition-colors hover:bg-[color:var(--band-dark)]"
              >
                Explore Bridal
              </LocalizedClientLink>
            </div>
          </div>
        </div>

        <ul className="grid grid-cols-1 sm:grid-cols-3 gap-6">
          {BRIDAL_HIGHLIGHTS.map((item) => (
            <li key={item.title}>
              <LocalizedClientLink
                href={item.href}
                className="group flex flex-col h-full rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--brand-surface)] border text-[color:var(--on-dark-muted)] shadow-sm hover:shadow-lg transition-all"
              >
                <div className="relative aspect-[4/5] overflow-hidden bg-[color:var(--band-dark)]">
                  <Image
                    src={item.image}
                    alt={item.title}
                    fill
                    loading="lazy"
                    quality={75}
                    sizes="(max-width: 640px) 100vw, 33vw"
                    className="object-cover object-top group-hover:scale-105 transition-transform duration-700"
                  />
                  <span className="glass glass-chip absolute top-3 left-3 px-3 py-1.5 text-[color:var(--on-dark)] text-xs font-semibold uppercase tracking-wider">
                    {item.tag}
                  </span>
                </div>
                <div className="p-4 flex items-center justify-between gap-2">
                  <h4 className="text-sm font-heading font-bold text-[color:var(--on-dark)] group-hover:text-brand-accent transition-colors">
                    {item.title}
                  </h4>
                  <span aria-hidden="true" className="text-[color:var(--on-dark-muted)] group-hover:text-[color:var(--on-dark)] transition-colors">
                    →
                  </span>
                </div>
              </LocalizedClientLink>
            </li>
          ))}
        </ul>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 9: Beyond Centuries                                                  */}
      {/* ------------------------------------------------------------------------ */}
      <section className="relative w-full rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--band-dark)] text-[color:var(--on-dark)] min-h-[400px] sm:min-h-[520px] flex items-center justify-center" aria-labelledby="centuries-heading">
        <Image
          src="/brand/editorial/groom-1.jpg"
          alt="Heritage collection"
          fill
          loading="lazy"
          quality={75}
          sizes="(max-width: 1440px) 100vw, 1440px"
          className="object-cover object-center opacity-65"
        />
        <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/85 via-[color:var(--band-dark)]/45 to-[color:var(--band-dark)]/65" />

        <div className="content-container relative z-10 text-center py-16 flex flex-col items-center gap-4 max-w-3xl">
          <span className="label-eyebrow font-mono" style={{ color: "var(--brand-accent)" }}>
            Generations of Craft
          </span>
          <h2 id="centuries-heading" className="display-section font-heading font-bold text-[color:var(--on-dark)]">
            Beyond Centuries. <br />
            <span className="italic font-normal text-[color:var(--on-dark)]">Timeless Dynasty.</span>
          </h2>
          <p className="body-copy-lg measure-tight text-[color:var(--on-dark)] font-light">
            Heirloom silhouettes for those who carry heritage with grace.
          </p>
          <div className="pt-2">
            <LocalizedClientLink
              href="/store"
              className="glass-chip inline-flex min-h-[44px] items-center px-8 py-3 bg-[color:var(--brand-surface)] text-[color:var(--on-dark)] text-xs font-semibold uppercase tracking-wider transition-colors hover:bg-[color:var(--band-dark)]"
            >
              Explore the vault
            </LocalizedClientLink>
          </div>
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 10: Jewellery & Ornaments                                             */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container space-y-6" aria-labelledby="jewellery-heading">
        <SectionHeading
          eyebrow="Finishing Accents"
          title="Jewellery & Ornaments"
          id="jewellery-heading"
        />

        <ul className="grid grid-cols-1 sm:grid-cols-2 gap-6">
          {[
            {
              title: "Royal Chokers & Matha Pattis",
              eyebrow: "Polki & Antique Gold",
              image:
                "/brand/editorial/groom-2.jpg",
              href: "/store",
            },
            {
              title: "Festive Statement Sets",
              eyebrow: "Emerald Pieces",
              image:
                "/brand/editorial/groom-3.jpg",
              href: "/store",
            },
          ].map((item) => (
            <li key={item.title}>
              <LocalizedClientLink
                href={item.href}
                className="group relative flex flex-col rounded-[var(--brand-radius-card)] overflow-hidden aspect-[4/5] bg-[color:var(--band-dark)]"
              >
                <Image
                  src={item.image}
                  alt={item.title}
                  fill
                  loading="lazy"
                  quality={75}
                  sizes="(max-width: 640px) 100vw, 50vw"
                  className="object-cover object-center group-hover:scale-105 transition-transform duration-700"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/85 via-transparent to-transparent" />
                <div className="absolute bottom-6 inset-x-6 text-[color:var(--on-dark)] space-y-1">
                  <span className="label-eyebrow" style={{ color: "var(--brand-accent)" }}>
                    {item.eyebrow}
                  </span>
                  <h3 className="text-xl font-heading font-bold">{item.title}</h3>
                </div>
              </LocalizedClientLink>
            </li>
          ))}
        </ul>

        <ul className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          {ACCESSORY_TILES.map((tile) => (
            <li key={tile.title}>
              <LocalizedClientLink
                href={tile.href}
                className="group flex flex-col h-full rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--brand-surface)] border text-[color:var(--on-dark-muted)] shadow-sm hover:shadow-md transition-all"
              >
                <div className="relative aspect-square overflow-hidden bg-[color:var(--band-dark)]">
                  <Image
                    src={tile.image}
                    alt={tile.title}
                    fill
                    loading="lazy"
                    quality={70}
                    sizes="(max-width: 640px) 45vw, 25vw"
                    className="object-cover group-hover:scale-105 transition-transform duration-500"
                  />
                </div>
                <div className="p-3 text-center">
                  <span className="text-xs font-heading font-medium text-[color:var(--on-dark)] group-hover:text-brand-accent transition-colors line-clamp-1">
                    {tile.title}
                  </span>
                </div>
              </LocalizedClientLink>
            </li>
          ))}
        </ul>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 11: Menswear Edit                                                    */}
      {/* ------------------------------------------------------------------------ */}
      <section className="w-full bg-[color:var(--band-dark)] text-[color:var(--on-dark)] py-16 sm:py-24" aria-labelledby="menswear-heading">
        <div className="content-container space-y-10">
          <SectionHeading
            eyebrow="Groom & Groomsmen"
            title="Menswear Edit"
            id="menswear-heading"
            tone="dark"
          />

          <ul className="grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8">
            {MENSWEAR_PIECES.map((piece) => (
              <li key={piece.title}>
                <LocalizedClientLink
                  href={piece.href}
                  className="group flex flex-col h-full rounded-[var(--brand-radius-card)] overflow-hidden bg-[color:var(--band-dark)] border text-[color:var(--on-dark-muted)] hover:border-[color:color-mix(in_oklab,var(--brand-accent)_40%,transparent)] transition-colors"
                >
                  <div className="relative aspect-[3/4] overflow-hidden bg-[color:var(--band-dark)]">
                    <Image
                      src={piece.image}
                      alt={piece.title}
                      fill
                      loading="lazy"
                      quality={75}
                      sizes="(max-width: 640px) 100vw, 33vw"
                      className="object-cover object-top group-hover:scale-105 transition-transform duration-700 opacity-90 group-hover:opacity-100"
                    />
                    <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/80 via-transparent to-transparent" />
                  </div>
                  <div className="p-4 sm:p-5 flex flex-col gap-1">
                    <h4 className="text-sm sm:text-base font-heading font-bold text-[color:var(--on-dark)] group-hover:text-brand-accent transition-colors">
                      {piece.title}
                    </h4>
                    <p className="body-copy text-[color:var(--on-dark)]">
                      {piece.subtitle}
                    </p>
                  </div>
                </LocalizedClientLink>
              </li>
            ))}
          </ul>
        </div>
      </section>
    </div>
  )
}
