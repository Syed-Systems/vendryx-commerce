import Image from "next/image"
import LocalizedClientLink from "@modules/common/components/localized-client-link"

export default function Hero() {
  return (
    <div className="w-full">
      {/* -------------------------------------------------------------------------- */}
      {/* DEFAULT (STHENO) THEME HERO — loud couture                                   */}
      {/* -------------------------------------------------------------------------- */}
      {/* One dominant image, one statement, display-scale type. The previous split      */}
      {/* hero put a 3/4 card beside a paragraph and diluted both. Colour comes from    */}
      {/* --brand-accent rather than hardcoded amber, so switching tenant themes         */}
      {/* actually changes the accent.                                                   */}
      <section className="theme-hero theme-hero-default
        relative w-full overflow-hidden bg-[color:var(--band-dark)] text-[color:var(--on-dark)]
        min-h-[78svh] flex items-end
        mahal:hidden lustre:hidden matrix:hidden
      ">
        <div className="absolute inset-0">
          <Image
            src="/brand/editorial/hero-bridal.jpg"
            alt="Model in an evening silhouette from the current atelier edit"
            fill
            priority
            fetchPriority="high"
            quality={80}
            sizes="100vw"
            className="object-cover object-center"
          />
          {/* Two-stop scrim: enough to hold display type at 8xl over any frame of */}
          {/* the photograph, without flattening it into a grey rectangle.            */}
          <div className="absolute inset-0 bg-gradient-to-t from-[color:var(--band-dark)]/80 via-[color:var(--band-dark)]/35 to-[color:var(--band-dark)]/10" />
        </div>

        <div className="content-container relative z-10 py-16 sm:py-24 lg:py-32">
          <div className="max-w-[22ch] space-y-6 sm:space-y-8">
            <span className="label-eyebrow text-[color:var(--on-primary-muted)]">
              Spring / Summer — Atelier Edit
            </span>

            <h1 className="display-hero font-heading font-semibold uppercase tracking-[-0.02em]">
              Everyday
              <br />
              <span className="italic font-normal normal-case tracking-normal text-[color:var(--on-dark-muted)]">
                Elegance
              </span>
            </h1>

            <p className="body-copy-lg measure text-[color:var(--on-dark)]">
              Modern silhouettes cut for everyday comfort — quiet luxury, made to
              be lived in.
            </p>

            <div className="flex flex-wrap items-center gap-3 pt-2">
              <LocalizedClientLink
                href="/store"
                className="glass-chip inline-flex min-h-[44px] items-center gap-2 bg-[color:var(--brand-surface)] px-7 py-3 text-[color:var(--ink)] text-xs font-semibold uppercase tracking-[0.18em] hover:bg-[color:var(--brand-surface)] transition-colors"
              >
                Explore the collection
              </LocalizedClientLink>
              <LocalizedClientLink
                href="/collections/summer-collection"
                className="glass-dark glass-chip inline-flex min-h-[44px] items-center gap-2 px-7 py-3 text-xs font-semibold uppercase tracking-[0.18em] text-[color:var(--on-dark)] hover:bg-[color:var(--brand-surface)]/15 transition-colors"
              >
                The lookbook
              </LocalizedClientLink>
            </div>
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------------------------- */}
      {/* MAHAL THEME HERO                                                           */}
      {/* -------------------------------------------------------------------------- */}
      <section className="theme-hero theme-hero-mahal
        hidden mahal:flex
        relative w-full overflow-hidden rounded-[24px] bg-brand-primary border-[4px] border-[color:color-mix(in_oklab,var(--brand-accent)_20%,transparent)] min-h-[78svh] items-center justify-center
      ">
        <div className="absolute inset-0 bg-[url('https://www.transparenttextures.com/patterns/stardust.png')] opacity-10 mix-blend-overlay" />
        <div className="content-container relative z-10 grid grid-cols-1 md:grid-cols-2 gap-12 items-center">
          <div className="flex flex-col items-center md:items-start text-center md:text-start space-y-8">
            <h1 className="display-hero font-heading text-[color:var(--on-primary)]">
              Timeless <br />
              <span className="text-[color:var(--on-primary)] italic normal-case">Heritage</span>
            </h1>
            <p className="body-copy-lg measure text-[color:var(--on-primary-muted)]">
              Exquisite hand-woven fabrics and royal embellishments for life's most
              golden moments.
            </p>
            <LocalizedClientLink
              href="/store"
              className="min-h-[44px] inline-flex items-center px-10 py-4 bg-brand-accent text-[color:var(--on-accent)] rounded-[var(--brand-radius-button)] font-bold uppercase tracking-widest hover:shadow-lg hover:shadow-brand-accent/20 transition-all duration-300 hover:-translate-y-0.5"
            >
              Discover The Royal Edit
            </LocalizedClientLink>
          </div>
          <div className="relative h-[400px] md:h-[600px] w-full rounded-t-full overflow-hidden border-8 border-[color:color-mix(in_oklab,var(--brand-accent)_30%,transparent)] shadow-2xl">
            <Image
              src="/brand/editorial/hero-mahal.jpg"
              alt="Royal edit campaign portrait"
              fill
              quality={80}
              sizes="(max-width: 768px) 100vw, 50vw"
              className="object-cover"
            />
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------------------------- */}
      {/* LUSTRE THEME HERO                                                          */}
      {/* -------------------------------------------------------------------------- */}
      {/* `85vh` → `85svh`. With `vh`, mobile browser chrome collapsing mid-scroll */}
      {/* resized the hero under the reader's thumb.                                 */}
      <section className="theme-hero theme-hero-lustre
        hidden lustre:block
        relative w-full h-[85vh] h-[85svh] min-h-[600px] bg-brand-primary overflow-hidden
      ">
        <div className="absolute inset-0">
          <Image
            src="/brand/editorial/hero-lustre.jpg"
            alt="Lustre editorial"
            fill
            quality={80}
            sizes="100vw"
            className="object-cover object-top grayscale opacity-85"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-brand-background via-[color:color-mix(in_oklab,var(--brand-background)_20%,transparent)] to-transparent" />
          {/* A second scrim directly behind the type block.

              The headline used to carry `mix-blend-difference`, which inverts the
              glyph against whatever pixel sits under it — so a light patch of
              the photograph turned white display type black. Contrast has to be
              measured in the composited worst case, and no automated checker
              measures it that way: they read the container's computed background,
              which is transparent. Fixed by giving the copy its own opaque
              plate instead of relying on the blend to find a contrast. */}
          <div className="absolute inset-0 bg-[color:color-mix(in_oklab,var(--brand-background)_72%,transparent)]" />
        </div>

        <div className="absolute bottom-0 left-0 w-full p-8 md:p-16 flex flex-col items-start justify-end">
          <span className="text-[color:var(--on-primary)] font-mono text-sm tracking-[0.3em] uppercase mb-4 border-s-2 border-brand-accent ps-4">
            Vol. 04 — The Fall Collection
          </span>
          <h1 className="display-hero font-heading font-black uppercase tracking-tighter">
            Lustre <br />
            <span className="text-[color:var(--on-primary-muted)]">Studios</span>
          </h1>
          <div className="mt-8 flex gap-6 w-full max-w-2xl border-t border-brand-hairline pt-6">
            <LocalizedClientLink
              href="/store"
              className="tap text-brand-text font-mono text-sm uppercase tracking-widest hover:text-brand-accent transition-colors flex items-center gap-2"
            >
              Shop Collection <span aria-hidden="true">→</span>
            </LocalizedClientLink>
            <LocalizedClientLink
              href="/store?view=runway"
              className="tap text-[color:var(--on-primary-muted)] font-mono text-sm uppercase tracking-widest hover:text-[color:var(--on-primary)] transition-colors flex items-center gap-2"
            >
              View Campaign <span aria-hidden="true">→</span>
            </LocalizedClientLink>
          </div>
        </div>
      </section>

      {/* -------------------------------------------------------------------------- */}
      {/* MATRIX THEME HERO                                                          */}
      {/* -------------------------------------------------------------------------- */}
      <section className="theme-hero theme-hero-matrix
        hidden matrix:flex
        relative w-full h-[90vh] h-[90svh] min-h-[600px] bg-brand-background overflow-hidden border border-brand-primary items-center justify-center font-mono
      ">
        <div
          className="absolute inset-0 opacity-20 bg-cover bg-center mix-blend-screen"
          style={{
            backgroundImage:
              "url('/brand/editorial/hero-matrix.jpg')",
          }}
        />
        <div className="absolute inset-0 bg-[color:color-mix(in_oklab,var(--brand-background)_80%,transparent)]" />

        <div className="relative z-10 text-center flex flex-col items-center justify-center w-full px-4">
          <div className="inline-block px-4 py-1 mb-8 border border-brand-primary text-brand-primary text-xs uppercase tracking-[0.3em] motion-safe:animate-pulse">
            System Online // Protocol_01
          </div>
          <h1 className="display-hero text-brand-primary font-bold uppercase tracking-tighter mb-6">
            The <br /> <span className="text-brand-text">Matrix</span>
          </h1>
          {/* `--brand-secondary` for Matrix is `#008F11`, which on the `#0D0208`
              background measures about 3:1 — below the 4.5:1 AA floor for body
              text. `--ink-muted` is the token built for exactly this: it mixes
              the tenant's own text toward its own background, so Matrix gets
              `#00FF41` at 74% over `#0D0208` (~7.6:1) instead of a second, much
              darker green. `#008F11` stays for the accent strokes above, where it
              is not carrying a paragraph.

              Note this is deliberately NOT `--on-primary-muted`. Those are
              foregrounds for text sitting *on* `--brand-primary`, and Matrix's
              primary is the luminous green — so they resolve to near-black and
              would be dark-on-dark here. */}
          <p className="body-copy-lg measure text-[color:var(--ink-muted)] mx-auto mb-10">
            Tech-wear silhouettes engineered for the urban grid. Initialize your
            loadout.
          </p>
          <LocalizedClientLink
            href="/store"
            className="min-h-[44px] inline-flex items-center px-8 py-4 bg-brand-primary text-[color:var(--on-primary)] font-bold uppercase tracking-[0.2em] hover:bg-brand-text hover:text-[color:var(--on-primary)] transition-colors"
          >
            Execute Shop
          </LocalizedClientLink>
        </div>
      </section>
    </div>
  )
}
