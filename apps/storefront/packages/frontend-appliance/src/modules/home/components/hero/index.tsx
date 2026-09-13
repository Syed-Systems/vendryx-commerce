import LocalizedClientLink from "@modules/common/components/localized-client-link"

const Hero = () => {
  return (
    <div className="relative w-full overflow-hidden bg-zinc-950 text-white min-h-[560px] sm:min-h-[660px] flex items-center border-b border-zinc-800/80">
      {/* Background Graphic & Tech Radial Illumination */}
      <div className="absolute inset-0 bg-gradient-to-b from-zinc-950 via-zinc-900/95 to-zinc-950 z-10" />
      <div 
        className="absolute inset-0 bg-cover bg-center opacity-30 mix-blend-luminosity scale-105 transform hover:scale-100 transition-transform duration-1000 ease-out"
        style={{
          backgroundImage: `url('https://images.unsplash.com/photo-1519389950473-47ba0277781c?q=80&w=2070&auto=format&fit=crop')`,
        }}
      />
      <div className="absolute -top-40 left-1/2 -translate-x-1/2 w-[700px] h-[350px] bg-blue-600/15 rounded-full blur-[120px] pointer-events-none z-10" />

      <div className="content-container relative z-20 py-16 sm:py-24 flex flex-col items-center text-center max-w-4xl mx-auto">
        {/* Eyebrow Pill */}
        <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-white/10 backdrop-blur-md border border-white/15 text-zinc-200 text-xs font-medium tracking-wide mb-6 shadow-sm">
          <span className="w-2 h-2 rounded-full bg-blue-400 animate-pulse" />
          Pro Architecture · Flagship 2026 Hardware Release
        </div>

        {/* Main Tech Headline */}
        <h1 className="text-4xl sm:text-6xl lg:text-7xl font-bold tracking-tight text-white leading-[1.08] mb-6">
          Engineered for Power. <br />
          <span className="text-transparent bg-clip-text bg-gradient-to-r from-zinc-200 via-zinc-400 to-zinc-500">
            Crafted with Pure Precision.
          </span>
        </h1>

        {/* Subtitle */}
        <p className="text-base sm:text-lg text-zinc-300 font-normal max-w-2xl leading-relaxed mb-8">
          Explore cutting-edge phones, high-performance ultrabooks, spatial audio headphones, and precision desk accessories engineered for professionals.
        </p>

        {/* CTA Buttons (Apple HIG Pill) */}
        <div className="flex flex-wrap items-center justify-center gap-4 w-full sm:w-auto">
          <LocalizedClientLink
            href="/store"
            className="px-8 py-3.5 bg-white text-zinc-950 hover:bg-zinc-100 active:scale-95 rounded-full font-semibold text-xs tracking-wider transition-all shadow-lg hover:shadow-blue-500/10 flex items-center justify-center gap-2"
          >
            <span>Explore All Devices</span>
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
            className="px-8 py-3.5 bg-white/5 border border-white/20 hover:border-white/40 text-white hover:bg-white/10 active:scale-95 rounded-full font-semibold text-xs tracking-wider transition-all flex items-center justify-center"
          >
            Latest Releases
          </LocalizedClientLink>
        </div>

        {/* Value Ticker Pills */}
        <div className="mt-14 pt-8 border-t border-zinc-800/80 grid grid-cols-1 sm:grid-cols-3 gap-6 w-full text-xs text-zinc-400">
          <div className="flex items-center justify-center gap-2.5">
            <span className="text-blue-400 text-sm">✦</span>
            <span>Factory Sealed & Verified Genuine</span>
          </div>
          <div className="flex items-center justify-center gap-2.5">
            <span className="text-blue-400 text-sm">✦</span>
            <span>2-Year Comprehensive Warranty</span>
          </div>
          <div className="flex items-center justify-center gap-2.5">
            <span className="text-blue-400 text-sm">✦</span>
            <span>14-Day Hassle-Free Returns</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Hero

