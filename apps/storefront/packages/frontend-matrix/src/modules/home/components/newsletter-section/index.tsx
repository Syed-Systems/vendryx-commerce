export default function NewsletterSection() {
  return (
    <section className="content-container my-16 sm:my-24">
      <div className="relative rounded-none bg-[#1a1a1a] overflow-hidden border border-white/10 p-8 sm:p-16 text-center">
        {/* Subtle decorative background circles */}
        <div className="absolute -top-24 -left-24 w-72 h-72 rounded-full bg-amber-500/10 blur-3xl pointer-events-none" />
        <div className="absolute -bottom-24 -right-24 w-72 h-72 rounded-full bg-emerald-500/10 blur-3xl pointer-events-none" />

        <div className="relative z-10 max-w-2xl mx-auto space-y-4">
          <span className="text-xs uppercase tracking-[0.25em] text-[#a0a0a0] font-semibold block">
            Exclusive Atelier Benefits
          </span>

          <h2 className="text-3xl sm:text-4xl font-[family-name:var(--font-plus-jakarta)] font-bold text-[#f5f5f0] tracking-tight">
            Unlock 10% Off Your First Order
          </h2>

          <p className="text-sm text-[#a0a0a0] font-light leading-relaxed">
            Join the STHENO community for early access to limited seasonal
            drops, private archive sales, and private styling previews.
          </p>

          <form
            className="flex flex-col sm:flex-row gap-3 max-w-md mx-auto pt-4"
            action="#"
            method="POST"
          >
            <input
              type="email"
              placeholder="Enter your email address"
              required
              className="bg-[#141414] border border-white/10 px-5 py-3.5 rounded-full text-xs text-[#f5f5f0] placeholder-[#666666] focus:outline-none focus:border-[#c9a96e] flex-1 transition-colors"
            />
            <button
              type="button"
              className="bg-zinc-950 text-white hover:bg-zinc-800 px-8 py-3.5 rounded-full text-xs font-semibold uppercase tracking-wider active:scale-95 transition-all"
            >
              Join Atelier
            </button>
          </form>

          <p className="text-[11px] text-zinc-400 font-light pt-2">
            By subscribing, you agree to our Privacy Policy. Unsubscribe at any
            time.
          </p>
        </div>
      </div>
    </section>
  )
}
