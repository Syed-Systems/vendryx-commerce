import LocalizedClientLink from "@modules/common/components/localized-client-link"

export default function LookbookStory() {
  return (
    <section className="bg-zinc-900 text-white my-16 sm:my-24 py-16 sm:py-24 overflow-hidden relative">
      <div className="content-container">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 lg:gap-16 items-center">
          {/* Visual Pair */}
          <div className="lg:col-span-6 grid grid-cols-2 gap-4 relative">
            <div className="relative rounded-none overflow-hidden aspect-[3/4] ">
              <div
                className="absolute inset-0 bg-cover bg-center hover:scale-105 transition-transform duration-700"
                style={{
                  backgroundImage: `url('https://images.unsplash.com/photo-1515886657613-9f3515b0c78f?q=80&w=1000&auto=format&fit=crop')`,
                }}
              />
            </div>
            <div className="relative rounded-none overflow-hidden aspect-[3/4] mt-8">
              <div
                className="absolute inset-0 bg-cover bg-center hover:scale-105 transition-transform duration-700"
                style={{
                  backgroundImage: `url('https://images.unsplash.com/photo-1485230895905-ec40ba36b9bc?q=80&w=1000&auto=format&fit=crop')`,
                }}
              />
            </div>
            <div className="absolute -bottom-4 -left-4 bg-[#141414] text-[#f5f5f0] p-4 rounded-none hidden sm:block">
              <span className="text-[10px] uppercase font-bold tracking-widest block text-zinc-400">
                Atelier Series
              </span>
              <span className="text-sm font-[family-name:var(--font-plus-jakarta)] font-bold">
                Lookbook 2026
              </span>
            </div>
          </div>

          {/* Editorial Content */}
          <div className="lg:col-span-6 space-y-6">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#141414]/10 text-amber-300 text-xs font-semibold tracking-widest uppercase">
              The Philosophy
            </div>

            <h2 className="text-3xl sm:text-4xl lg:text-5xl font-[family-name:var(--font-plus-jakarta)] font-bold tracking-tight text-white leading-tight">
              An unyielding pursuit of silhouette purity & enduring comfort.
            </h2>

            <p className="text-sm sm:text-base text-zinc-300 font-light leading-relaxed">
              Every garment in our collection begins with traceable natural
              fibers—fine spun virgin wool, combed Aegean cotton, and pure
              mulberry silk. Designed in our studio to transcend ephemeral trend
              cycles, each piece is built to be worn, cherished, and inherited.
            </p>

            <div className="grid grid-cols-2 gap-6 pt-4 border-t border-zinc-800 text-xs">
              <div>
                <span className="text-xl sm:text-2xl font-[family-name:var(--font-plus-jakarta)] font-bold text-white block">
                  100%
                </span>
                <span className="text-zinc-400">
                  Traceable Natural Raw Materials
                </span>
              </div>
              <div>
                <span className="text-xl sm:text-2xl font-[family-name:var(--font-plus-jakarta)] font-bold text-white block">
                  Zero-Waste
                </span>
                <span className="text-zinc-400">
                  Pattern Optimization & Craft
                </span>
              </div>
            </div>

            <div className="pt-4">
              <LocalizedClientLink
                href="/store"
                className="inline-flex items-center gap-2 px-8 py-3.5 bg-[#141414] text-[#f5f5f0] hover:bg-[#242424] rounded-full font-semibold text-xs uppercase tracking-wider transition-all"
              >
                <span>Read the Journal & Sourcing</span>
                <span>→</span>
              </LocalizedClientLink>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
