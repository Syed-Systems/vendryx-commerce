"use client"

import Image from "next/image"
import { useCallback, useRef, useState } from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import styles from "./product-card.module.css"

/**
 * PORT of The Matrix Hours `ProductCard`.
 *
 * Its markup, its CSS module, its interactions: pointer-tracked gold spotlight,
 * the tilt, the second-image crossfade on hover, the badge stack, the gold price
 * rail. `--spot-x` / `--spot-y` are the same custom properties the source card
 * drove, read by `.card::before` in the ported stylesheet.
 *
 * One deviation, and it is forced: the source card's slide-up rail was a quick
 * add-to-cart. A theme does not mutate the cart (ADR-013 decision 4), and the
 * listing gives it no add action to call — so the rail keeps its place and its
 * motion and navigates instead. See docs/FRONTEND_MATRIX_SLOTS.md.
 */
export default function MatrixProductCard({
  href,
  title,
  collection,
  initialImage,
  hoverImage,
  price,
  originalPrice,
  discountLabel,
  isFeatured,
  isSoldOut,
}: {
  href: string
  title: string
  collection: string
  initialImage?: string | null
  hoverImage?: string | null
  price?: string
  originalPrice?: string
  discountLabel?: string | null
  isFeatured?: boolean
  isSoldOut?: boolean
}) {
  const cardRef = useRef<HTMLDivElement>(null)
  const [isHovered, setIsHovered] = useState(false)
  const [spot, setSpot] = useState({ x: 50, y: 50 })

  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    const rect = cardRef.current?.getBoundingClientRect()
    if (!rect) return
    const x = ((e.clientX - rect.left) / rect.width) * 100
    const y = ((e.clientY - rect.top) / rect.height) * 100
    setSpot({ x, y })
    const rotY = ((x - 50) / 50) * 6
    const rotX = ((y - 50) / 50) * -4
    if (cardRef.current) {
      cardRef.current.style.transform = `perspective(600px) rotateX(${rotX}deg) rotateY(${rotY}deg) translateY(-4px)`
    }
  }, [])

  const handleMouseLeave = useCallback(() => {
    setIsHovered(false)
    if (cardRef.current) {
      cardRef.current.style.transform = ""
    }
  }, [])

  return (
    <div
      ref={cardRef}
      className={styles.card}
      data-testid="product-wrapper"
      onMouseEnter={() => setIsHovered(true)}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      style={
        {
          "--spot-x": `${spot.x}%`,
          "--spot-y": `${spot.y}%`,
        } as React.CSSProperties
      }
    >
      <LocalizedClientLink
        href={href}
        className={styles.cardLink}
        style={{ display: "block", textDecoration: "none", color: "inherit" }}
      >
        <div className={styles.imageContainer}>
          {initialImage && (
            <Image
              src={initialImage}
              alt={title}
              fill
              sizes="(max-width: 768px) 50vw, 25vw"
              className={`${styles.image} ${
                isHovered && hoverImage ? styles.imageHidden : ""
              }`}
            />
          )}
          {hoverImage && (
            <Image
              src={hoverImage}
              alt={`${title} alternate view`}
              fill
              sizes="(max-width: 768px) 50vw, 25vw"
              className={`${styles.image} ${styles.imageHover} ${
                isHovered ? styles.imageHoverVisible : ""
              }`}
            />
          )}

          <div className={styles.tagsOverlay}>
            {isSoldOut && (
              <span className={`${styles.badge} ${styles.soldOutBadge}`}>
                Sold Out
              </span>
            )}
            {!isSoldOut && isFeatured && (
              <span className={`${styles.badge} ${styles.featuredBadge}`}>
                Featured
              </span>
            )}
            {!isSoldOut && discountLabel && (
              <span className={`${styles.badge} ${styles.discountBadge}`}>
                {discountLabel}
              </span>
            )}
          </div>

          <div
            className={`${styles.quickActionBtn} ${
              isSoldOut ? styles.disabledBtn : ""
            }`}
          >
            {isSoldOut ? "Sold Out" : "View Reference"}
          </div>
        </div>

        <div className={styles.contentSection}>
          <div className={styles.brand}>{collection}</div>
          <h3 className={styles.name} title={title} data-testid="product-title">
            {title}
          </h3>
          <div className={styles.priceRow}>
            <span className={styles.price} data-testid="price">
              {price}
            </span>
            {originalPrice && originalPrice !== price && (
              <span
                className={styles.originalPrice}
                data-testid="original-price"
              >
                {originalPrice}
              </span>
            )}
          </div>
        </div>
      </LocalizedClientLink>
    </div>
  )
}
