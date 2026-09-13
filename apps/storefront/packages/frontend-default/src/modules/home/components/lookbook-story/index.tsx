import LocalizedClientLink from "@modules/common/components/localized-client-link"

// ----------------------------------------------------------------------------
// Section 6: "NEW IN COUTURE" (6 Items in 3x2 Grid)
// ----------------------------------------------------------------------------
const COUTURE_PIECES = [
  {
    title: "Ochre Zari Bridal Lehenga",
    subtitle: "Heavy Gotapatti Work on Pure Silk",
    image: "https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Mustard Mirror-Work Gown",
    subtitle: "Architectural Silhouette & Shish Mahal Embroidery",
    image: "https://images.unsplash.com/photo-1515886657613-9f3515b0c78f?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Crimson Velvet Royal Lehenga",
    subtitle: "Antique Zardozi & Hand-Cut Sequins",
    image: "https://images.unsplash.com/photo-1610030469983-98e550d6193c?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Blush Organza Resham Ensemble",
    subtitle: "Scalloped Borders & Pastel Floral Tulle",
    image: "https://images.unsplash.com/photo-1485230895905-ec40ba36b9bc?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Ice Blue Threadwork Anarkali",
    subtitle: "Chiffon Dupatta & Pearl Embellishments",
    image: "https://images.unsplash.com/photo-1515372039744-b8f02a3ae446?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Lavender Tulle Reception Gown",
    subtitle: "Silver Dabka & Crystal Motifs",
    image: "https://images.unsplash.com/photo-1539571696357-5a69c17a67c6?q=80&w=800&auto=format&fit=crop",
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
    image: "https://images.unsplash.com/photo-1610030469983-98e550d6193c?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Midnight Velvet Bandhgala Suit",
    tag: "Bespoke Cut",
    image: "https://images.unsplash.com/photo-1583391733975-f2d3284f676b?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Lilac Floral Organza Ensemble",
    tag: "Day Wedding",
    image: "https://images.unsplash.com/photo-1485230895905-ec40ba36b9bc?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
]

// ----------------------------------------------------------------------------
// Section 10: Accessories Small Tiles
// ----------------------------------------------------------------------------
const ACCESSORY_TILES = [
  {
    title: "Embroidered Potlis & Clutches",
    image: "https://images.unsplash.com/photo-1544816155-12df9643f363?q=80&w=600&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Royal Emerald Cocktails",
    image: "https://images.unsplash.com/photo-1599643478518-a784e5dc4c8f?q=80&w=600&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Handcrafted Zardozi Mojaris",
    image: "https://images.unsplash.com/photo-1560343090-f0409e92791a?q=80&w=600&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Heritage Pashmina & Silk Stoles",
    image: "https://images.unsplash.com/photo-1522335789203-aabd1fc54bc9?q=80&w=600&auto=format&fit=crop",
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
    image: "https://images.unsplash.com/photo-1594938298603-c8148c4dae35?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Midnight Raw Silk Kurta Set",
    subtitle: "Churidar & Hand-Embroidered Bundi",
    image: "https://images.unsplash.com/photo-1507679799987-c73779587ccf?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
  {
    title: "Champagne Embroidered Achkan",
    subtitle: "Tussar Silk with Antique Metallic Buttons",
    image: "https://images.unsplash.com/photo-1617627143750-d86bc21e42bb?q=80&w=800&auto=format&fit=crop",
    href: "/store",
  },
]

export default function LookbookStory() {
  return (
    <div className="w-full flex flex-col space-y-16 sm:space-y-24 my-12 sm:my-16">
      {/* ------------------------------------------------------------------------ */}
      {/* Section 6: "NEW IN COUTURE" (Midnight 3x2 Grid with Frosted Pills)       */}
      {/* ------------------------------------------------------------------------ */}
      <section className="w-full bg-stone-950 text-white py-16 sm:py-24 rounded-2xl sm:rounded-3xl border border-stone-800 shadow-2xl overflow-hidden relative">
        <div className="content-container">
          <div className="text-center mb-12 sm:mb-16 space-y-2">
            <span className="text-xs font-mono uppercase tracking-[0.25em] text-amber-400 block">
              Atelier Runway
            </span>
            <h2 className="text-3xl sm:text-5xl font-serif font-bold tracking-wider uppercase text-white">
              NEW IN COUTURE
            </h2>
            <div className="w-12 h-0.5 bg-amber-400/80 mx-auto mt-3" />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8">
            {COUTURE_PIECES.map((piece, idx) => (
              <LocalizedClientLink
                key={idx}
                href={piece.href}
                className="group relative flex flex-col rounded-2xl overflow-hidden aspect-[3/4] bg-stone-900 border border-stone-800/80 shadow-lg"
              >
                <img
                  src={piece.image}
                  alt={piece.title}
                  className="w-full h-full object-cover object-top group-hover:scale-105 transition-transform duration-700 opacity-90 group-hover:opacity-100"
                  loading="lazy"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/20 to-transparent" />

                {/* Frosted Action Pill */}
                <div className="absolute bottom-5 inset-x-5 flex flex-col items-center text-center space-y-2">
                  <h4 className="text-sm sm:text-base font-serif font-bold text-white group-hover:text-amber-300 transition-colors">
                    {piece.title}
                  </h4>
                  <span className="w-full py-2 bg-white/15 hover:bg-white/25 backdrop-blur-md border border-white/20 text-white rounded-xl text-xs font-semibold uppercase tracking-wider transition-colors shadow-sm">
                    Shop Now
                  </span>
                </div>
              </LocalizedClientLink>
            ))}
          </div>
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 7: "CELEBRITY SPOTTING" (Split 2-Column)                         */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 sm:gap-8 items-stretch">
          {/* Left Solo Bridal Feature */}
          <LocalizedClientLink
            href="/store"
            className="lg:col-span-5 group relative rounded-2xl sm:rounded-3xl overflow-hidden bg-stone-100 border border-brand-hairline shadow-sm min-h-[420px] sm:min-h-[500px]"
          >
            <img
              src="https://images.unsplash.com/photo-1610030469983-98e550d6193c?q=80&w=800&auto=format&fit=crop"
              alt="Bridal Couture Portrait"
              className="w-full h-full object-cover object-top group-hover:scale-105 transition-transform duration-700"
              loading="lazy"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent" />
            <div className="absolute bottom-6 inset-x-6 text-white space-y-1">
              <span className="text-[10px] uppercase font-mono tracking-widest text-amber-300 block">
                Bridal Heritage
              </span>
              <h3 className="text-xl sm:text-2xl font-serif font-bold">
                The Royal Crimson Series
              </h3>
            </div>
          </LocalizedClientLink>

          {/* Right Celebrity Banner */}
          <LocalizedClientLink
            href="/store"
            className="lg:col-span-7 group relative rounded-2xl sm:rounded-3xl overflow-hidden bg-stone-900 border border-stone-800 shadow-sm min-h-[420px] sm:min-h-[500px] flex items-end"
          >
            <img
              src="https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?q=80&w=1200&auto=format&fit=crop"
              alt="Celebrity Spotting Showcase"
              className="absolute inset-0 w-full h-full object-cover object-center group-hover:scale-105 transition-transform duration-700 opacity-80"
              loading="lazy"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/30 to-transparent" />
            <div className="relative z-10 p-6 sm:p-10 text-white space-y-3 max-w-xl">
              <span className="inline-block px-3 py-1 rounded-full bg-white/15 backdrop-blur-md border border-white/20 text-amber-200 text-xs font-semibold tracking-widest uppercase">
                Red Carpet &amp; Receptions
              </span>
              <h2 className="text-3xl sm:text-5xl font-serif font-bold tracking-tight leading-tight">
                CELEBRITY SPOTTING
              </h2>
              <p className="text-xs sm:text-sm text-stone-200 font-light leading-relaxed">
                Adored by icons of cinema and royal heirs. Discover bespoke creations worn on international gala evenings.
              </p>
              <div className="pt-2">
                <span className="inline-flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-amber-300 group-hover:translate-x-1 transition-transform">
                  <span>View The Red Carpet Edit</span>
                  <span>→</span>
                </span>
              </div>
            </div>
          </LocalizedClientLink>
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 8: "BRIDAL EDIT" (Palace Pavilion + 3 Highlights)                */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container space-y-8">
        <div className="text-center space-y-2">
          <span className="text-xs font-mono uppercase tracking-[0.25em] text-stone-500 block">
            Couture Masterpieces
          </span>
          <h2 className="text-3xl sm:text-4xl font-serif font-bold tracking-tight text-stone-900">
            BRIDAL EDIT
          </h2>
          <div className="w-12 h-0.5 bg-amber-500/60 mx-auto mt-2" />
        </div>

        {/* Wide Palace Courtyard Feature with Floated Glass Card */}
        <div className="relative w-full rounded-2xl sm:rounded-3xl overflow-hidden bg-stone-900 border border-stone-800 shadow-xl aspect-[16/9] min-h-[380px] sm:min-h-[500px]">
          <img
            src="https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?q=80&w=1800&auto=format&fit=crop"
            alt="Palace Courtyard Bridal Showcase"
            className="absolute inset-0 w-full h-full object-cover object-center"
            loading="lazy"
          />
          <div className="absolute inset-0 bg-gradient-to-r from-black/50 via-transparent to-black/70" />

          {/* Floated White Architectural Card on Right */}
          <div className="absolute top-6 bottom-6 right-6 sm:right-10 w-full max-w-sm rounded-2xl bg-white/95 backdrop-blur-md p-6 sm:p-8 flex flex-col justify-between shadow-2xl border border-white/60">
            <div className="space-y-3">
              <span className="text-[10px] font-mono uppercase tracking-widest text-amber-700 font-semibold block">
                Atelier Heritage
              </span>
              <h3 className="text-2xl sm:text-3xl font-serif font-bold text-stone-900 leading-tight">
                The Grand Courtyard Pavilion
              </h3>
              <p className="text-xs text-stone-600 font-light leading-relaxed">
                Expansive flared lehengas, pure silk canopies, and micro-embroidered floral borders taking hundreds of artisan hours.
              </p>
            </div>
            <div className="pt-4 border-t border-stone-200">
              <LocalizedClientLink
                href="/store"
                className="w-full py-3 bg-stone-950 hover:bg-stone-800 text-white rounded-xl text-center text-xs font-semibold uppercase tracking-wider block transition-colors shadow-sm"
              >
                Schedule Bridal Consultation
              </LocalizedClientLink>
            </div>
          </div>
        </div>

        {/* 3 Bridal Cards Below */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
          {BRIDAL_HIGHLIGHTS.map((item, idx) => (
            <LocalizedClientLink
              key={idx}
              href={item.href}
              className="group flex flex-col rounded-2xl overflow-hidden bg-white border border-brand-hairline shadow-sm hover:shadow-lg transition-all"
            >
              <div className="relative aspect-[4/5] overflow-hidden bg-stone-100">
                <img
                  src={item.image}
                  alt={item.title}
                  className="w-full h-full object-cover object-top group-hover:scale-105 transition-transform duration-700"
                  loading="lazy"
                />
                <span className="absolute top-3 left-3 px-2.5 py-1 rounded-full bg-white/90 backdrop-blur-sm text-stone-900 text-[10px] font-semibold uppercase tracking-wider shadow-sm">
                  {item.tag}
                </span>
              </div>
              <div className="p-4 flex items-center justify-between">
                <h4 className="text-xs sm:text-sm font-serif font-bold text-stone-900 group-hover:text-amber-700 transition-colors">
                  {item.title}
                </h4>
                <span className="text-stone-400 group-hover:text-stone-900 transition-colors">
                  →
                </span>
              </div>
            </LocalizedClientLink>
          ))}
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 9: "BEYOND CENTURIES" (Cinematic Royal Twin Brides Banner)       */}
      {/* ------------------------------------------------------------------------ */}
      <section className="relative w-full rounded-2xl sm:rounded-3xl overflow-hidden bg-stone-950 text-white min-h-[400px] sm:min-h-[500px] flex items-center justify-center border border-stone-800 shadow-xl">
        <img
          src="https://images.unsplash.com/photo-1583391733975-f2d3284f676b?q=80&w=1800&auto=format&fit=crop"
          alt="Beyond Centuries Royal Banner"
          className="absolute inset-0 w-full h-full object-cover object-center opacity-65"
          loading="lazy"
        />
        <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/40 to-black/60" />

        <div className="content-container relative z-10 text-center py-16 flex flex-col items-center space-y-4 max-w-2xl">
          <span className="text-xs font-mono uppercase tracking-[0.25em] text-amber-300">
            Generations of Craft
          </span>
          <h2 className="text-3xl sm:text-5xl lg:text-6xl font-serif font-bold tracking-tight text-white leading-tight">
            Beyond Centuries. <br />
            <span className="italic font-normal text-stone-300">Timeless Dynasty.</span>
          </h2>
          <p className="text-xs sm:text-sm text-stone-200 font-light leading-relaxed">
            Heirloom silhouettes curated for those who carry heritage with effortless sovereign grace.
          </p>
          <div className="pt-2">
            <LocalizedClientLink
              href="/store"
              className="px-8 py-3 bg-white text-stone-950 hover:bg-stone-100 rounded-full text-xs font-semibold uppercase tracking-wider transition-colors shadow-lg"
            >
              Explore Heritage Vault
            </LocalizedClientLink>
          </div>
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 10: "ACCESSORIES & JEWELLERY" (Multi-Grid)                       */}
      {/* ------------------------------------------------------------------------ */}
      <section className="content-container space-y-6">
        <div className="text-center space-y-2">
          <span className="text-xs font-mono uppercase tracking-[0.25em] text-stone-500 block">
            Finishing Accents
          </span>
          <h2 className="text-3xl sm:text-4xl font-serif font-bold tracking-tight text-stone-900">
            JEWELLERY &amp; ORNAMENTS
          </h2>
          <div className="w-12 h-0.5 bg-amber-500/60 mx-auto mt-2" />
        </div>

        {/* 2 Big Portrait Showcases */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
          <LocalizedClientLink
            href="/store"
            className="group relative rounded-2xl overflow-hidden aspect-[4/5] bg-stone-900 shadow-md"
          >
            <img
              src="https://images.unsplash.com/photo-1599643478518-a784e5dc4c8f?q=80&w=800&auto=format&fit=crop"
              alt="Kundan & Polki Bridal Jewellery"
              className="w-full h-full object-cover object-center group-hover:scale-105 transition-transform duration-700"
              loading="lazy"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-transparent to-transparent" />
            <div className="absolute bottom-6 inset-x-6 text-white space-y-1">
              <span className="text-[10px] uppercase font-mono tracking-widest text-amber-300">
                Polki &amp; Antique Gold
              </span>
              <h3 className="text-xl font-serif font-bold">
                Royal Chokers &amp; Matha Pattis
              </h3>
            </div>
          </LocalizedClientLink>

          <LocalizedClientLink
            href="/store"
            className="group relative rounded-2xl overflow-hidden aspect-[4/5] bg-stone-900 shadow-md"
          >
            <img
              src="https://images.unsplash.com/photo-1539571696357-5a69c17a67c6?q=80&w=800&auto=format&fit=crop"
              alt="Festive Emerald Jewellery"
              className="w-full h-full object-cover object-center group-hover:scale-105 transition-transform duration-700"
              loading="lazy"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-transparent to-transparent" />
            <div className="absolute bottom-6 inset-x-6 text-white space-y-1">
              <span className="text-[10px] uppercase font-mono tracking-widest text-emerald-300">
                Zambian Emeralds
              </span>
              <h3 className="text-xl font-serif font-bold">
                Festive Statement Sets
              </h3>
            </div>
          </LocalizedClientLink>
        </div>

        {/* 4 Supporting Tiles */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
          {ACCESSORY_TILES.map((tile, idx) => (
            <LocalizedClientLink
              key={idx}
              href={tile.href}
              className="group flex flex-col rounded-xl overflow-hidden bg-white border border-brand-hairline shadow-sm hover:shadow-md transition-all"
            >
              <div className="relative aspect-square overflow-hidden bg-stone-100">
                <img
                  src={tile.image}
                  alt={tile.title}
                  className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                  loading="lazy"
                />
              </div>
              <div className="p-3 text-center">
                <span className="text-xs font-serif font-medium text-stone-900 group-hover:text-amber-700 transition-colors line-clamp-1">
                  {tile.title}
                </span>
              </div>
            </LocalizedClientLink>
          ))}
        </div>
      </section>

      {/* ------------------------------------------------------------------------ */}
      {/* Section 11: "MENSWEAR EDIT" (Midnight 3-Card Showcase)                   */}
      {/* ------------------------------------------------------------------------ */}
      <section className="w-full bg-stone-950 text-white py-16 sm:py-24 rounded-2xl sm:rounded-3xl border border-stone-800 shadow-2xl overflow-hidden">
        <div className="content-container space-y-10">
          <div className="text-center space-y-2">
            <span className="text-xs font-mono uppercase tracking-[0.25em] text-amber-400 block">
              Groom &amp; Groomsmen
            </span>
            <h2 className="text-3xl sm:text-4xl font-serif font-bold tracking-wider uppercase text-white">
              MENSWEAR EDIT
            </h2>
            <div className="w-12 h-0.5 bg-amber-400/80 mx-auto mt-2" />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 sm:gap-8">
            {MENSWEAR_PIECES.map((piece, idx) => (
              <LocalizedClientLink
                key={idx}
                href={piece.href}
                className="group flex flex-col rounded-2xl overflow-hidden bg-stone-900 border border-stone-800 shadow-md hover:shadow-xl transition-all"
              >
                <div className="relative aspect-[3/4] overflow-hidden bg-stone-950">
                  <img
                    src={piece.image}
                    alt={piece.title}
                    className="w-full h-full object-cover object-top group-hover:scale-105 transition-transform duration-700 opacity-90 group-hover:opacity-100"
                    loading="lazy"
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent" />
                </div>
                <div className="p-4 sm:p-5 flex flex-col space-y-1">
                  <h4 className="text-sm sm:text-base font-serif font-bold text-white group-hover:text-amber-300 transition-colors">
                    {piece.title}
                  </h4>
                  <p className="text-xs text-stone-400 font-light">
                    {piece.subtitle}
                  </p>
                </div>
              </LocalizedClientLink>
            ))}
          </div>
        </div>
      </section>
    </div>
  )
}
