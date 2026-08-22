import styles from "./editorial.module.css"

/**
 * PORT of The Matrix Hours `EditorialSection` — the three-pillar editorial band,
 * its numbered cards, its gold rule and its stylesheet.
 *
 * Dropped: the GSAP scrub that drew the rule and staggered the cards in. `gsap`
 * is not a storefront dependency and this theme does not add one for a reveal;
 * the CSS below carries the same entrance where the browser supports
 * scroll-driven animation, and renders finished where it does not.
 *
 * The copy is the source shop's own — authenticity, heritage, craftsmanship —
 * and it is claims about a merchant, not measured figures. It is placeholder
 * until a merchant writes its own. AGENTS.md §12's standing rule is why there
 * are no numbers in it.
 */
const PILLARS = [
  {
    number: "01",
    title: "Authenticity",
    body: "Every timepiece comes with original certification and manufacturer warranty. No replicas, no compromises.",
  },
  {
    number: "02",
    title: "Heritage",
    body: "We partner directly with heritage brands whose movements have kept time for generations.",
  },
  {
    number: "03",
    title: "Craftsmanship",
    body: "From hand-assembled movements to sapphire crystal faces — we only carry watches built to last lifetimes.",
  },
]

const BrandFeatures = () => {
  return (
    <section id="story" className={styles.section}>
      <div className={styles.lineWrapper}>
        <div className={styles.goldLine} />
      </div>

      <div className={styles.header}>
        <h2 className={styles.heading}>Built on Three Pillars</h2>
      </div>

      <div className={styles.track}>
        {PILLARS.map((pillar) => (
          <div key={pillar.number} className={styles.card}>
            <span className={styles.number}>{pillar.number}</span>
            <h3 className={styles.cardTitle}>{pillar.title}</h3>
            <p className={styles.cardBody}>{pillar.body}</p>
            <div className={styles.cardAccent} />
          </div>
        ))}
      </div>
    </section>
  )
}

export default BrandFeatures
