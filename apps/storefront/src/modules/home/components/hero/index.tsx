import LocalizedClientLink from "@modules/common/components/localized-client-link"

const Hero = () => {
  return (
    <div className="relative w-full overflow-hidden bg-zinc-950 text-white min-h-[580px] sm:min-h-[680px] flex items-center">
      {/* Background Graphic & Subtle Glow */}
      <div className="absolute inset-0 bg-gradient-to-r from-zinc-950 via-zinc-900/90 to-zinc-950/70 z-10" />
      <div 
        className="absolute inset-0 bg-cover bg-center opacity-40 mix-blend-luminosity scale-105 transform hover:scale-100 transition-transform duration-1000 ease-out"
        style={{
          backgroundImage: `url('https://images.unsplash.com/photo-1490481651871-ab68de25d43d?q=80&w=2070&auto=format&fit=crop')`,
        }}
      />

      <div className="content-container relative z-20 py-20 flex flex-col items-start justify-center max-w-4xl">
        {/* Season Tag */}
        <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-white/10 backdrop-blur-md border border-white/15 text-zinc-200 text-xs font-semibold tracking-widest uppercase mb-6 shadow-sm">
          <span className="w-2 h-2 rounded-full bg-amber-400" />
          Autumn / Winter 2026 • Exclusive Atelier Edit
        </div>

        {/* Main Editorial Headline */}
        <h1 className="text-4xl sm:text-6xl lg:text-7xl font-serif font-bold tracking-tight text-white leading-[1.1] mb-6">
          Timeless Silhouettes. <br className="hidden sm:inline" />
          <span className="italic font-normal text-zinc-300">Modern Conscious Craft.</span>
        </h1>

        {/* Subtitle */}
        <p className="text-base sm:text-lg text-zinc-300 font-light max-w-2xl leading-relaxed mb-8">
          Curated wardrobe essentials meticulously tailored from organic fabrics, refined textures, and enduring minimalist aesthetics.
        </p>

        {/* CTA Buttons */}
        <div className="flex flex-wrap items-center gap-4 w-full sm:w-auto">
          <LocalizedClientLink
            href="/store"
            className="px-8 py-3.5 bg-white text-zinc-950 hover:bg-zinc-100 active:scale-95 rounded-full font-semibold text-xs uppercase tracking-wider transition-all shadow-lg hover:shadow-white/10 flex items-center justify-center gap-2"
          >
            <span>Explore Collection</span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-4 w-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M14 5l7 7m0 0l-7 7m7-7H3"
              />
            </svg>
          </LocalizedClientLink>

          <LocalizedClientLink
            href="/store?sortBy=created_at"
            className="px-8 py-3.5 bg-transparent border border-zinc-500 hover:border-white text-white hover:bg-white/5 active:scale-95 rounded-full font-semibold text-xs uppercase tracking-wider transition-all flex items-center justify-center"
          >
            New Arrivals
          </LocalizedClientLink>
        </div>

        {/* Value Ticker Pills */}
        <div className="mt-14 pt-8 border-t border-zinc-800/80 grid grid-cols-2 sm:grid-cols-3 gap-6 w-full text-xs text-zinc-400">
          <div className="flex items-center gap-2.5">
            <span className="text-white text-sm">✦</span>
            <span>Ethically Sourced Wool & Cottons</span>
          </div>
          <div className="flex items-center gap-2.5">
            <span className="text-white text-sm">✦</span>
            <span>Complimentary Express Delivery</span>
          </div>
          <div className="hidden sm:flex items-center gap-2.5">
            <span className="text-white text-sm">✦</span>
            <span>30-Day Effortless Returns</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Hero
