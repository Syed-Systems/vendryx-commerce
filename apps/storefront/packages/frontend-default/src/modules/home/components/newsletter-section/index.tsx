export default function NewsletterSection() {
  return (
    <section className="content-container my-16 sm:my-24">
      <div className="relative rounded-3xl bg-[#F5F5F7] dark:bg-[#161617] overflow-hidden border border-zinc-200/80 dark:border-zinc-800/80 p-8 sm:p-16 text-center">
        {/* Subtle decorative background circles */}
        <div className="absolute -top-24 -left-24 w-72 h-72 rounded-full bg-blue-500/10 blur-3xl pointer-events-none" />
        <div className="absolute -bottom-24 -right-24 w-72 h-72 rounded-full bg-indigo-500/10 blur-3xl pointer-events-none" />

        <div className="relative z-10 max-w-2xl mx-auto space-y-4">
          <span className="text-xs uppercase tracking-[0.25em] text-zinc-500 font-semibold block">
            Hardware Launch Briefings
          </span>

          <h2 className="text-3xl sm:text-4xl font-semibold text-zinc-950 dark:text-zinc-50 tracking-tight">
            Priority Access to Flagship Drops
          </h2>

          <p className="text-sm text-zinc-600 dark:text-zinc-400 font-normal leading-relaxed">
            Join the VENDRYX Insider network for early notifications on flagship hardware releases, architecture whitepapers, and firmware updates.
          </p>

          <form className="flex flex-col sm:flex-row gap-3 max-w-md mx-auto pt-4" action="#" method="POST">
            <input
              type="email"
              placeholder="Enter your professional email"
              required
              className="bg-white dark:bg-zinc-900 border border-zinc-300 dark:border-zinc-700 px-5 py-3.5 rounded-full text-xs text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 focus:outline-none focus:border-blue-600 flex-1 shadow-sm transition-colors"
            />
            <button
              type="button"
              className="bg-zinc-950 dark:bg-white text-white dark:text-zinc-950 hover:bg-zinc-800 dark:hover:bg-zinc-200 px-8 py-3.5 rounded-full text-xs font-semibold tracking-wider shadow-md hover:shadow-lg active:scale-95 transition-all"
            >
              Get Priority Access
            </button>
          </form>

          <p className="text-[11px] text-zinc-400 font-light pt-2">
            By subscribing, you agree to our Hardware Warranty & Privacy Terms. Unsubscribe at any time.
          </p>
        </div>
      </div>
    </section>
  )
}
