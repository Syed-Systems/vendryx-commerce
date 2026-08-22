import LocalizedClientLink from "@modules/common/components/localized-client-link"

const categories = [
  {
    title: "Tailored Outerwear",
    subtitle: "Coats, Trench & Blazers",
    image: "https://images.unsplash.com/photo-1539571696357-5a69c17a67c6?q=80&w=1000&auto=format&fit=crop",
    href: "/store",
    span: "col-span-1 md:col-span-2 lg:col-span-2 aspect-[4/3] md:aspect-[16/9]",
  },
  {
    title: "Knitwear & Essentials",
    subtitle: "Organic Merino & Cashmere",
    image: "https://images.unsplash.com/photo-1434389677669-e08b4cac3105?q=80&w=1000&auto=format&fit=crop",
    href: "/store",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
  {
    title: "Dresses & Silks",
    subtitle: "Flowing Modern Silhouettes",
    image: "https://images.unsplash.com/photo-1515372039744-b8f02a3ae446?q=80&w=1000&auto=format&fit=crop",
    href: "/store",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
  {
    title: "Tailored Trousers",
    subtitle: "Pleated & Relaxed Fits",
    image: "https://images.unsplash.com/photo-1509631179647-0177331693ae?q=80&w=1000&auto=format&fit=crop",
    href: "/store",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
  {
    title: "Accessories & Leather",
    subtitle: "Handcrafted Minimal Essentials",
    image: "https://images.unsplash.com/photo-1522335789203-aabd1fc54bc9?q=80&w=1000&auto=format&fit=crop",
    href: "/store",
    span: "col-span-1 md:col-span-1 lg:col-span-1 aspect-[4/3] md:aspect-auto",
  },
]

export default function CategoryGrid() {
  return (
    <section className="content-container py-16 sm:py-24">
      {/* Section Header */}
      <div className="flex flex-col sm:flex-row sm:items-end justify-between mb-10 pb-4 border-b border-zinc-200">
        <div>
          <span className="text-xs uppercase tracking-[0.2em] text-zinc-500 font-medium block mb-1">
            Curated Edits
          </span>
          <h2 className="text-2xl sm:text-3xl font-serif font-bold text-zinc-950">
            Shop by Wardrobe Department
          </h2>
        </div>
        <LocalizedClientLink
          href="/store"
          className="text-xs font-semibold uppercase tracking-widest text-zinc-900 hover:text-zinc-600 transition-colors mt-2 sm:mt-0 flex items-center gap-1 group"
        >
          <span>View All Departments</span>
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
            <div className="absolute inset-0 bg-gradient-to-t from-zinc-950/85 via-zinc-950/30 to-transparent transition-opacity group-hover:opacity-90" />

            {/* Content */}
            <div className="absolute inset-0 p-6 sm:p-8 flex flex-col justify-end text-white">
              <span className="text-[11px] font-medium tracking-widest uppercase text-zinc-300 mb-1">
                {cat.subtitle}
              </span>
              <div className="flex items-center justify-between">
                <h3 className="text-xl sm:text-2xl font-serif font-bold text-white tracking-tight">
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
