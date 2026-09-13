export default function NewsletterSection() {
  return (
    <section className="content-container my-16 sm:my-24">
      <div className="relative rounded-3xl bg-stone-100/80 overflow-hidden border border-brand-hairline p-8 sm:p-16 text-center shadow-sm">
        {/* Subtle decorative warm ambient glow */}
        <div className="absolute -top-24 -left-24 w-72 h-72 rounded-full bg-amber-500/10 blur-3xl pointer-events-none" />
        <div className="absolute -bottom-24 -right-24 w-72 h-72 rounded-full bg-rose-500/10 blur-3xl pointer-events-none" />

        <div className="relative z-10 max-w-2xl mx-auto space-y-4">
          <span className="text-xs uppercase tracking-[0.25em] text-stone-500 font-mono font-medium block">
            Private Client Register
          </span>

          <h2 className="text-3xl sm:text-4xl font-serif font-bold text-stone-900 tracking-tight">
            Join the Vendryx Atelier Salon
          </h2>

          <p className="text-sm text-stone-600 font-light leading-relaxed">
            Receive private invitations to seasonal runway debuts, private salon appointments, and archival couture previews.
          </p>

          <form className="flex flex-col sm:flex-row gap-3 max-w-md mx-auto pt-4" action="#" method="POST">
            <input
              type="email"
              placeholder="Enter your email address"
              required
              className="bg-white border border-stone-300 px-5 py-3.5 rounded-full text-xs text-stone-900 placeholder-stone-400 focus:outline-none focus:border-stone-950 flex-1 shadow-sm transition-colors"
            />
            <button
              type="button"
              className="bg-brand-primary text-white hover:opacity-90 px-8 py-3.5 rounded-full text-xs font-semibold uppercase tracking-wider shadow-md hover:shadow-lg active:scale-95 transition-all"
            >
              Request Access
            </button>
          </form>

          <p className="text-[11px] text-stone-400 font-light pt-2">
            By registering, you agree to our Terms of Service &amp; Privacy Policy. Discretion guaranteed.
          </p>
        </div>
      </div>
    </section>
  )
}
