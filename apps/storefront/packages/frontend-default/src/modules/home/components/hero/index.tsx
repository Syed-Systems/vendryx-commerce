import LocalizedClientLink from "@modules/common/components/localized-client-link"

export default function Hero() {
  return (
    <div className="w-full flex flex-col space-y-4 sm:space-y-6">
      {/* -------------------------------------------------------------------------- */}
      {/* Section 1: Top Campaign Banner ("Elegant Every Moment")                     */}
      {/* Soft champagne & pistachio palette, editorial script typography             */}
      {/* -------------------------------------------------------------------------- */}
      <section className="relative w-full overflow-hidden rounded-2xl sm:rounded-3xl bg-gradient-to-r from-[#F4EFEA] via-[#E8EFE9] to-[#F7F3EE] text-stone-900 border border-brand-hairline shadow-sm min-h-[380px] sm:min-h-[460px] lg:min-h-[520px] flex items-center">
        <div className="content-container relative z-10 py-10 sm:py-16 grid grid-cols-1 md:grid-cols-12 gap-8 items-center">
          {/* Script Headline & Copy */}
          <div className="md:col-span-6 lg:col-span-7 flex flex-col items-start space-y-4 sm:space-y-5">
            <span className="text-xs uppercase tracking-[0.25em] text-stone-600 font-medium">
              Spring / Summer Festive Atelier
            </span>
            <div className="space-y-2">
              <h2 className="text-4xl sm:text-5xl lg:text-6xl font-serif italic text-stone-900 leading-[1.1]">
                Elegant Every <br />
                <span className="font-normal not-italic tracking-tight">Moment.</span>
              </h2>
              <p className="text-sm sm:text-base text-stone-600 font-light max-w-md leading-relaxed">
                Graceful pastels, gossamer handloom organza, and delicate botanical resham threadwork tailored for summer celebrations.
              </p>
            </div>
            <LocalizedClientLink
              href="/store"
              className="mt-2 px-7 py-3 bg-brand-primary text-white hover:opacity-90 active:scale-95 rounded-full text-xs font-semibold uppercase tracking-widest transition-all shadow-sm"
            >
              Explore Festive Edit
            </LocalizedClientLink>
          </div>

          {/* Editorial Model Image */}
          <div className="md:col-span-6 lg:col-span-5 relative flex justify-center">
            <div className="relative w-full max-w-sm aspect-[3/4] rounded-2xl overflow-hidden shadow-xl border border-white/60">
              <img
                src="https://images.unsplash.com/photo-1515886657613-9f3515b0c78f?q=80&w=1000&auto=format&fit=crop"
                alt="Elegant Every Moment Campaign Editorial"
                className="w-full h-full object-cover object-top hover:scale-105 transition-transform duration-700"
                loading="eager"
              />
              <div className="absolute inset-0 bg-gradient-to-t from-stone-900/40 via-transparent to-transparent pointer-events-none" />
            </div>
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------------------------- */}
      {/* Section 2: Major Campaign Panorama ("ICONS OF ELEGANCE")                    */}
      {/* Royal bridal & couture group shot in heritage palace architecture          */}
      {/* -------------------------------------------------------------------------- */}
      <section className="relative w-full overflow-hidden rounded-2xl sm:rounded-3xl bg-stone-950 text-white min-h-[500px] sm:min-h-[640px] lg:min-h-[720px] flex items-center justify-center border border-stone-800 shadow-xl">
        <div
          className="absolute inset-0 bg-cover bg-center opacity-70 scale-100 hover:scale-105 transition-transform duration-1000 ease-out"
          style={{
            backgroundImage: `url('https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?q=80&w=2070&auto=format&fit=crop')`,
          }}
        />
        <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/40 to-black/60 z-10" />

        <div className="content-container relative z-20 py-16 text-center flex flex-col items-center max-w-3xl space-y-4 sm:space-y-6">
          <span className="inline-block px-4 py-1 rounded-full bg-white/10 backdrop-blur-md border border-white/20 text-amber-200 text-xs font-semibold tracking-[0.25em] uppercase shadow-sm">
            The Haute Couture Collection
          </span>

          <h1 className="text-4xl sm:text-6xl lg:text-7xl font-serif tracking-wider uppercase text-white leading-tight font-bold drop-shadow-md">
            ICONS OF ELEGANCE
          </h1>

          <p className="text-sm sm:text-base text-stone-200 font-light max-w-xl leading-relaxed mx-auto">
            Regal velvet bandhgalas, intricately embroidered bridal lehengas, and heirloom zardozi craftsmanship celebrating timeless dynasties.
          </p>

          <div className="flex flex-wrap items-center justify-center gap-4 pt-2">
            <LocalizedClientLink
              href="/store"
              className="px-8 py-3.5 bg-white text-stone-950 hover:bg-stone-100 active:scale-95 rounded-full font-semibold text-xs uppercase tracking-widest transition-all shadow-lg"
            >
              Shop The Runway
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store?sortBy=created_at"
              className="px-8 py-3.5 bg-transparent border border-white/40 hover:border-white text-white hover:bg-white/10 active:scale-95 rounded-full font-semibold text-xs uppercase tracking-widest transition-all"
            >
              View Bridal Lookbook
            </LocalizedClientLink>
          </div>
        </div>
      </section>
    </div>
  )
}
