import LocalizedClientLink from "@modules/common/components/localized-client-link"

const categories = [
  {
    title: "Flagship Smartphones",
    subtitle: "Titanium Enclosures · Neural Engine",
    image: "https://images.unsplash.com/photo-1511707171634-5f897ff02560?q=80&w=1000&auto=format&fit=crop",
    href: "/categories/phones",
    span: "col-span-1 md:col-span-2 lg:col-span-2 aspect-[4/3] md:aspect-[16/9]",
  },
  {
    title: "Laptops & Ultrabooks",
    subtitle: "High-Performance Workstations",
    image: "https://images.unsplash.com/photo-1517336714731-489689fd1ca8?q=80&w=1000&auto=format&fit=crop",
    href: "/categories/laptops",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
  {
    title: "Spatial & Studio Audio",
    subtitle: "Active Noise Cancellation",
    image: "https://images.unsplash.com/photo-1505740420928-5e560c06d30e?q=80&w=1000&auto=format&fit=crop",
    href: "/categories/audio",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
  {
    title: "Smartwatches & Trackers",
    subtitle: "Biometric Precision & Health",
    image: "https://images.unsplash.com/photo-1523275335684-37898b6baf30?q=80&w=1000&auto=format&fit=crop",
    href: "/categories/wearables",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
  {
    title: "Power & Desk Setup",
    subtitle: "Fast Wireless Docks & Peripherals",
    image: "https://images.unsplash.com/photo-1527864550417-7fd91fc51a46?q=80&w=1000&auto=format&fit=crop",
    href: "/categories/accessories",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
]

export default function CategoryGrid() {
  return (
    <section className="content-container py-16 sm:py-24">
      {/* Section Header */}
      <div className="flex flex-col sm:flex-row sm:items-end justify-between mb-10 pb-4 border-b border-zinc-200 dark:border-zinc-800">
        <div>
          <span className="text-xs uppercase tracking-[0.2em] text-zinc-500 font-medium block mb-1">
            Hardware Categories
          </span>
          <h2 className="text-2xl sm:text-3xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
            Explore by Device Family
          </h2>
        </div>
        <LocalizedClientLink
          href="/store"
          className="text-xs font-semibold uppercase tracking-widest text-zinc-900 dark:text-zinc-100 hover:text-zinc-600 dark:hover:text-zinc-400 transition-colors mt-2 sm:mt-0 flex items-center gap-1 group"
        >
          <span>View All Hardware</span>
          <span className="group-hover:translate-x-1 transition-transform">→</span>
        </LocalizedClientLink>
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {categories.map((cat, idx) => (
          <LocalizedClientLink
            key={idx}
            href={cat.href}
            className={`group relative overflow-hidden rounded-2xl bg-zinc-900 shadow-sm transition-all duration-300 hover:shadow-xl ${cat.span} min-h-[280px] sm:min-h-[340px]`}
          >
            {/* Background Image */}
            <div
              className="absolute inset-0 bg-cover bg-center transition-transform duration-700 ease-out group-hover:scale-105"
              style={{ backgroundImage: `url('${cat.image}')` }}
            />
            {/* Gradient Overlay */}
            <div className="absolute inset-0 bg-gradient-to-t from-zinc-950/90 via-zinc-950/30 to-transparent transition-opacity group-hover:opacity-95" />

            {/* Content */}
            <div className="absolute inset-0 p-6 sm:p-8 flex flex-col justify-end text-white">
              <span className="text-[11px] font-medium tracking-widest uppercase text-zinc-400 mb-1">
                {cat.subtitle}
              </span>
              <div className="flex items-center justify-between">
                <h3 className="text-xl sm:text-2xl font-semibold tracking-tight text-white">
                  {cat.title}
                </h3>
                <span className="w-9 h-9 rounded-full bg-white/20 backdrop-blur-md flex items-center justify-center text-white transform translate-y-2 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-300">
                  →
                </span>
              </div>
            </div>
          </LocalizedClientLink>
        ))}
      </div>
    </section>
  )
}
