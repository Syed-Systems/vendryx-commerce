"use client"

import { useRef } from "react"
import gsap from "gsap"
import { ScrollTrigger } from "gsap/ScrollTrigger"
import { useGSAP } from "@gsap/react"
import styles from "./hero.module.css"
import {
  BezelRing,
  CaseBody,
  DialFace,
  Movement,
  Strap,
  WatchHands,
} from "./watch-parts"

gsap.registerPlugin(ScrollTrigger)

/**
 * PORT of The Matrix Hours' exploded timepiece.
 *
 * The watch is 3D: six SVG layers inside a `perspective: 1400px` scene, moved
 * along Y and Z and tilted on X and Y. Not WebGL — no canvas is involved — but
 * genuinely three-dimensional, and the reason the hero reads the way it does.
 *
 * The timeline is the source's, phase for phase: explode over the first 35% of
 * a pinned 150% scroll, fade the part labels in at 30%, hold, then reassemble
 * from 60%. Mouse parallax runs on `quickTo` with the per-layer depth factors,
 * so nearer layers travel further — which is what sells the depth.
 *
 * This needs GSAP, and that is why `gsap` and `@gsap/react` are storefront
 * dependencies now. A CSS scroll-driven animation was tried first and cannot do
 * it: `animation-timeline` cannot pin a section, and nothing in CSS tracks the
 * pointer. See docs/FRONTEND_MATRIX_CORE_DELTAS.md.
 */
const PARTS = [
  { name: "STRAP", Component: Strap, y: 200, z: -60, parallax: 0.2 },
  { name: "CASE", Component: CaseBody, y: 80, z: -20, parallax: 0.35 },
  { name: "MOVEMENT", Component: Movement, y: -10, z: 40, parallax: 0.5 },
  { name: "DIAL", Component: DialFace, y: -80, z: 80, parallax: 0.7 },
  { name: "HANDS", Component: WatchHands, y: -130, z: 130, parallax: 0.85 },
  { name: "BEZEL", Component: BezelRing, y: -180, z: 160, parallax: 1.0 },
]

export default function WatchScene() {
  const sceneRef = useRef<HTMLDivElement>(null)
  const layerRefs = useRef<(HTMLDivElement | null)[]>([])
  const labelRefs = useRef<(HTMLSpanElement | null)[]>([])

  useGSAP(
    () => {
      const mm = gsap.matchMedia()

      // The source gated the whole thing at 992px and on reduced motion. Both
      // matter: the exploded view needs the width, and a pinned scrub is
      // exactly the kind of motion someone disables it for.
      mm.add(
        {
          desktop:
            "(min-width: 992px) and (prefers-reduced-motion: no-preference)",
        },
        () => {
          const layers = layerRefs.current.filter(Boolean) as HTMLDivElement[]
          const labels = labelRefs.current.filter(Boolean) as HTMLSpanElement[]
          if (!layers.length) return

          // The pinned element is an ANCESTOR of this component, marked with
          // `data-hero-pin`. It cannot be passed as a selector string: `useGSAP`
          // scope resolves selectors *inside* the scope, so an ancestor never
          // matches and ScrollTrigger silently falls back to the tween target.
          // It is the grid, not the <section>, because the section is a flex
          // child and pin spacing cannot be reserved on one — see .pinWrap in
          // hero.module.css.
          const trigger = sceneRef.current?.closest("[data-hero-pin]")
          if (!trigger) return

          const tl = gsap.timeline({
            scrollTrigger: {
              trigger,
              start: "top top",
              // A function, not the literal "+=150%": the end is resolved on
              // every refresh, so it survives the layout shift when the webfonts
              // land. Measured once at build time it came out short, the
              // pin-spacer never reserved the scroll distance, and the next
              // section slid up over a still-fixed hero.
              end: () => "+=" + window.innerHeight * 1.5,
              pin: true,
              scrub: 1.2,
              anticipatePin: 1,
            },
          })

          // Phase 1 — explode.
          layers.forEach((layer, i) => {
            tl.to(
              layer,
              {
                y: PARTS[i].y,
                z: PARTS[i].z,
                rotationX: i % 2 === 0 ? 8 : -8,
                duration: 0.35,
                ease: "none",
              },
              0
            )
          })

          // Phase 2 — labels.
          labels.forEach((label) => {
            tl.to(label, { opacity: 1, x: 0, duration: 0.1, ease: "none" }, 0.3)
          })

          // Phase 3 — hold. Phase 4 — reassemble.
          layers.forEach((layer) => {
            tl.to(
              layer,
              { y: 0, z: 0, rotationX: 0, duration: 0.4, ease: "none" },
              0.6
            )
          })
          labels.forEach((label) => {
            tl.to(
              label,
              { opacity: 0, x: 10, duration: 0.1, ease: "none" },
              0.6
            )
          })

          // Mouse parallax — nearer layers travel further.
          const quickTos = layers.map((layer) => ({
            x: gsap.quickTo(layer, "x", { duration: 0.5, ease: "power2.out" }),
            rotY: gsap.quickTo(layer, "rotationY", {
              duration: 0.8,
              ease: "power2.out",
            }),
          }))

          const handleMouseMove = (e: MouseEvent) => {
            const cx = (e.clientX / window.innerWidth - 0.5) * 2
            quickTos.forEach((qt, i) => {
              const f = PARTS[i].parallax
              qt.x(cx * 30 * f)
              qt.rotY(cx * 8 * f)
            })
          }

          window.addEventListener("mousemove", handleMouseMove)

          // Webfonts land after first paint and reflow the hero. Without this
          // the pin keeps the geometry it measured against fallback metrics.
          const refresh = () => ScrollTrigger.refresh()
          document.fonts?.ready.then(refresh)

          return () => {
            window.removeEventListener("mousemove", handleMouseMove)
          }
        }
      )

      return () => mm.revert()
    },
    { scope: sceneRef }
  )

  return (
    <div className={styles.explodedScene} ref={sceneRef}>
      {PARTS.map((part, i) => (
        <div
          key={part.name}
          ref={(el) => {
            layerRefs.current[i] = el
          }}
          className={styles.partLayer}
          data-watch-part={part.name}
        >
          <part.Component />
          <span
            ref={(el) => {
              labelRefs.current[i] = el
            }}
            className={styles.partLabel}
          >
            <span className={styles.labelLine} />
            {part.name}
          </span>
        </div>
      ))}
    </div>
  )
}
