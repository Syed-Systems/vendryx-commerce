import LocalizedClientLink from "@modules/common/components/localized-client-link"

export default function Hero() {
  return (
    <div className="w-full">
      {/* -------------------------------------------------------------------------- */}
      {/* DEFAULT (STHENO) THEME HERO                                                */}
      {/* -------------------------------------------------------------------------- */}
      <section className="theme-hero theme-hero-default
        flex flex-col space-y-4 sm:space-y-6
        group-data-[theme=mahal]:hidden group-data-[theme=lustre]:hidden group-data-[theme=matrix]:hidden
      ">
        <div className="relative w-full overflow-hidden rounded-card sm:rounded-3xl bg-gradient-to-r from-gray-100 via-gray-50 to-gray-100 text-brand-text border border-brand-hairline shadow-sm min-h-[380px] sm:min-h-[460px] lg:min-h-[520px] flex items-center">
          <div className="content-container relative z-10 py-10 sm:py-16 grid grid-cols-1 md:grid-cols-12 gap-8 items-center">
            <div className="md:col-span-6 lg:col-span-7 flex flex-col items-start space-y-4 sm:space-y-5">
              <span className="text-xs uppercase tracking-[0.25em] text-gray-500 font-medium">
                Spring / Summer Edit
              </span>
              <div className="space-y-2">
                <h2 className="text-4xl sm:text-5xl lg:text-6xl font-heading text-brand-text leading-[1.1]">
                  Everyday <br />
                  <span className="font-normal tracking-tight">Elegance.</span>
                </h2>
                <p className="text-sm sm:text-base text-gray-600 font-light max-w-md leading-relaxed">
                  Discover modern silhouettes crafted for everyday comfort and effortless style.
                </p>
              </div>
              <LocalizedClientLink
                href="/store"
                className="mt-2 px-7 py-3 bg-brand-primary text-brand-surface hover:opacity-90 active:scale-95 rounded-full text-xs font-semibold uppercase tracking-widest transition-all shadow-sm"
              >
                Explore Collection
              </LocalizedClientLink>
            </div>
            <div className="md:col-span-6 lg:col-span-5 relative flex justify-center">
              <div className="relative w-full max-w-sm aspect-[3/4] rounded-card overflow-hidden shadow-xl border border-white/60">
                <img
                  src="https://images.unsplash.com/photo-1515886657613-9f3515b0c78f?q=80&w=1000&auto=format&fit=crop"
                  alt="Everyday Elegance Campaign"
                  className="w-full h-full object-cover object-top hover:scale-105 transition-transform duration-700"
                  loading="eager"
                />
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------------------------- */}
      {/* MAHAL THEME HERO                                                           */}
      {/* -------------------------------------------------------------------------- */}
      <section className="theme-hero theme-hero-mahal
        hidden group-data-[theme=mahal]:flex
        relative w-full overflow-hidden rounded-[24px] bg-brand-primary border-[4px] border-brand-accent/20 min-h-[600px] lg:min-h-[700px] items-center justify-center
      ">
        <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/stardust.png')] opacity-10 mix-blend-overlay"></div>
        <div className="content-container relative z-10 grid grid-cols-1 md:grid-cols-2 gap-12 items-center">
          <div className="flex flex-col items-center md:items-start text-center md:text-left space-y-8 animate-fade-in-top">
            <h1 className="text-5xl md:text-7xl font-heading text-brand-surface leading-tight drop-shadow-md">
              Timeless <br/><span className="text-brand-accent italic">Heritage</span>
            </h1>
            <p className="text-brand-secondary text-lg max-w-md font-body leading-relaxed">
              Exquisite hand-woven fabrics and royal embellishments for life's most golden moments.
            </p>
            <LocalizedClientLink
              href="/store"
              className="px-10 py-4 bg-gradient-to-r from-brand-accent to-yellow-600 text-brand-primary rounded-full font-bold uppercase tracking-widest hover:shadow-lg hover:shadow-brand-accent/20 transition-all duration-300 hover:-translate-y-1"
            >
              Discover The Royal Edit
            </LocalizedClientLink>
          </div>
          <div className="relative h-[400px] md:h-[600px] w-full rounded-t-full overflow-hidden border-8 border-brand-accent/30 shadow-2xl animate-fade-in-right">
            <img 
              src="https://images.unsplash.com/photo-1583391733956-3750e0ff4e8b?q=80&w=1000&auto=format&fit=crop" 
              className="w-full h-full object-cover"
              alt="Royal Edit"
            />
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------------------------- */}
      {/* LUSTRE THEME HERO                                                          */}
      {/* -------------------------------------------------------------------------- */}
      <section className="theme-hero theme-hero-lustre
        hidden group-data-[theme=lustre]:block
        relative w-full h-[85vh] min-h-[600px] bg-brand-primary overflow-hidden
      ">
        <div className="absolute inset-0">
          <img 
            src="https://images.unsplash.com/photo-1618932260643-eee4a2f652a6?q=80&w=2000&auto=format&fit=crop"
            className="w-full h-full object-cover object-top grayscale opacity-60"
            alt="Lustre Editorial"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-brand-background via-brand-background/20 to-transparent"></div>
        </div>
        
        <div className="absolute bottom-0 left-0 w-full p-8 md:p-16 flex flex-col items-start justify-end">
          <span className="text-brand-accent font-mono text-sm tracking-[0.3em] uppercase mb-4 border-l-2 border-brand-accent pl-4">
            Vol. 04 — The Fall Collection
          </span>
          <h1 className="text-6xl md:text-9xl font-heading text-brand-text font-black uppercase tracking-tighter leading-none mix-blend-difference">
            LUSTRE <br/> <span className="text-brand-secondary">STUDIOS</span>
          </h1>
          <div className="mt-8 flex gap-6 w-full max-w-2xl border-t border-brand-hairline pt-6">
            <LocalizedClientLink
              href="/store"
              className="text-brand-text font-mono text-sm uppercase tracking-widest hover:text-brand-accent transition-colors flex items-center gap-2"
            >
              Shop Collection &rarr;
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store?view=runway"
              className="text-brand-secondary font-mono text-sm uppercase tracking-widest hover:text-brand-text transition-colors flex items-center gap-2"
            >
              View Campaign &rarr;
            </LocalizedClientLink>
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------------------------- */}
      {/* MATRIX THEME HERO                                                          */}
      {/* -------------------------------------------------------------------------- */}
      <section className="theme-hero theme-hero-matrix
        hidden group-data-[theme=matrix]:flex
        relative w-full h-[90vh] min-h-[600px] bg-brand-background overflow-hidden border border-brand-primary items-center justify-center font-mono
      ">
        <div className="absolute inset-0 bg-[url('https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?q=80&w=2000&auto=format&fit=crop')] opacity-20 bg-cover bg-center mix-blend-screen"></div>
        <div className="absolute inset-0 bg-brand-background/80"></div>
        
        <div className="relative z-10 text-center flex flex-col items-center justify-center w-full px-4">
          <div className="inline-block px-4 py-1 mb-8 border border-brand-primary text-brand-primary text-xs uppercase tracking-[0.3em] animate-pulse">
            SYSTEM_ONLINE // PROTOCOL_01
          </div>
          <h1 className="text-5xl md:text-8xl text-brand-primary font-bold uppercase tracking-tighter mb-6">
            THE <br/> <span className="text-brand-text">MATRIX</span>
          </h1>
          <p className="text-brand-secondary max-w-lg mx-auto text-sm md:text-base leading-relaxed mb-10">
            Tech-wear silhouettes engineered for the urban grid. Initialize your loadout.
          </p>
          <LocalizedClientLink
            href="/store"
            className="px-8 py-4 bg-brand-primary text-brand-surface font-bold uppercase tracking-[0.2em] hover:bg-brand-text hover:text-brand-surface transition-colors"
          >
            EXECUTE_SHOP
          </LocalizedClientLink>
        </div>
      </section>
    </div>
  )
}
