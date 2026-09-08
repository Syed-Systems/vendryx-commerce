import LocalizedClientLink from "@modules/common/components/localized-client-link"

export default function LookbookStory() {
  return (
    <section className="bg-zinc-950 text-white my-16 sm:py-24 py-16 overflow-hidden relative border-y border-zinc-850">
      <div className="content-container">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 lg:gap-16 items-center">
          {/* Visual Pair */}
          <div className="lg:col-span-6 grid grid-cols-2 gap-4 relative">
            <div className="relative rounded-2xl overflow-hidden aspect-[3/4] shadow-2xl">
              <div
                className="absolute inset-0 bg-cover bg-center hover:scale-105 transition-transform duration-700"
                style={{
                  backgroundImage: `url('https://images.unsplash.com/photo-1517336714731-489689fd1ca8?q=80&w=1000&auto=format&fit=crop')`,
                }}
              />
            </div>
            <div className="relative rounded-2xl overflow-hidden aspect-[3/4] shadow-2xl mt-8">
              <div
                className="absolute inset-0 bg-cover bg-center hover:scale-105 transition-transform duration-700"
                style={{
                  backgroundImage: `url('https://images.unsplash.com/photo-1511707171634-5f897ff02560?q=80&w=1000&auto=format&fit=crop')`,
                }}
              />
            </div>
            <div className="absolute -bottom-4 -left-4 bg-zinc-900 border border-zinc-800 text-white p-4 rounded-xl shadow-xl hidden sm:block">
              <span className="text-[10px] uppercase font-semibold tracking-widest block text-zinc-400">
                Architecture Standard
              </span>
              <span className="text-sm font-semibold">Pro Silicon 2026</span>
            </div>
          </div>

          {/* Editorial Content */}
          <div className="lg:col-span-6 space-y-6">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-blue-500/10 border border-blue-500/20 text-blue-400 text-xs font-semibold tracking-widest uppercase">
              The Engineering Standard
            </div>

            <h2 className="text-3xl sm:text-4xl lg:text-5xl font-semibold tracking-tight text-white leading-tight">
              An unyielding pursuit of thermal efficiency, tactile craft &amp; compute power.
            </h2>

            <p className="text-sm sm:text-base text-zinc-400 font-normal leading-relaxed">
              Every device in our catalog is engineered with aerospace-grade milled aluminum enclosures, liquid retina displays, and high-throughput unified memory architectures. Designed to maximize creative throughput, battery endurance, and daily tactile satisfaction.
            </p>

            <div className="grid grid-cols-2 gap-6 pt-4 border-t border-zinc-800 text-xs">
              <div>
                <span className="text-xl sm:text-2xl font-semibold text-white block">100%</span>
                <span className="text-zinc-400">Recycled Aerospace Aluminum Chassis</span>
              </div>
              <div>
                <span className="text-xl sm:text-2xl font-semibold text-white block">Up to 24h</span>
                <span className="text-zinc-400">Real-World Active Battery Life</span>
              </div>
            </div>

            <div className="pt-4">
              <LocalizedClientLink
                href="/store"
                className="inline-flex items-center gap-2 px-8 py-3.5 bg-white text-zinc-950 hover:bg-zinc-100 rounded-full font-semibold text-xs tracking-wider transition-all"
              >
                <span>Explore Technical Specs</span>
                <span>→</span>
              </LocalizedClientLink>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
