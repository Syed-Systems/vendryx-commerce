import LocalizedClientLink from "@modules/common/components/localized-client-link"
import styles from "./hero.module.css"
import WatchScene from "./watch-scene"

/**
 * PORT of The Matrix Hours `HeroSection` — its stylesheet, its two-column grid,
 * its copy, and the timepiece that is the whole point of the page.
 *
 * The timepiece itself is in `watch-scene.tsx`, which is a client component
 * because the exploded view is a pinned GSAP scrub with pointer parallax. This
 * half stays on the server: copy, headings and links ship no JavaScript.
 */
const Hero = () => {
  return (
    <section className={styles.heroSection}>
      <div className={styles.pinWrap}>
        <div className={styles.gridContainer} data-hero-pin>
          <div className={styles.textContent}>
            <div className={styles.eyebrow}>SINCE 2026 · DHAKA, BANGLADESH</div>

            <h1 className={styles.heading}>
              Time <span className={styles.goldGradient}>Redefined.</span>
            </h1>

            <p className={styles.subtitle}>
              Curating authentic premium timepieces from world-renowned brands.
              Exquisite style, ultimate precision.
            </p>

            <div className={styles.ctaRow}>
              <LocalizedClientLink href="/store" className={styles.primaryCta}>
                Explore Collection
                <svg
                  className={styles.ctaIcon}
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="square"
                  aria-hidden
                >
                  <path d="M5 12h14M13 6l6 6-6 6" />
                </svg>
              </LocalizedClientLink>

              <a href="#story" className={styles.secondaryCta}>
                Our Story
              </a>
            </div>
          </div>

          <div className={styles.imageContent}>
            <WatchScene />
          </div>
        </div>
      </div>
    </section>
  )
}

export default Hero
