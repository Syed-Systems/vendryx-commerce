import React from "react"

const features = [
  {
    icon: (
      <svg className="w-5 h-5 text-amber-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
      </svg>
    ),
    title: "Complimentary Express Delivery",
    desc: "Insured, carbon-neutral delivery on all qualifying orders worldwide.",
  },
  {
    icon: (
      <svg className="w-5 h-5 text-amber-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
    ),
    title: "Master Weaver Heritage",
    desc: "Authentic Zari and hand-embroidered silks from certified atelier workshops.",
  },
  {
    icon: (
      <svg className="w-5 h-5 text-amber-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
    ),
    title: "Effortless Exchanges & Returns",
    desc: "Complimentary return documentation included with every packaged piece.",
  },
  {
    icon: (
      <svg className="w-5 h-5 text-amber-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z" />
      </svg>
    ),
    title: "Personal Bridal Concierge",
    desc: "Dedicated styling consultants available for bespoke measurements and fitting advice.",
  },
]

export default function BrandFeatures() {
  return (
    <section className="border-y border-brand-hairline bg-stone-50/60 py-10 my-8 sm:my-12">
      <div className="content-container">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 sm:gap-8">
          {features.map((f, i) => (
            <div key={i} className="flex items-start gap-4 p-2">
              <div className="p-2.5 rounded-full bg-white border border-stone-200 shadow-sm shrink-0">
                {f.icon}
              </div>
              <div className="space-y-1">
                <h4 className="text-xs uppercase tracking-wider font-semibold text-stone-900 font-serif">
                  {f.title}
                </h4>
                <p className="text-xs text-stone-500 font-light leading-relaxed">
                  {f.desc}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
