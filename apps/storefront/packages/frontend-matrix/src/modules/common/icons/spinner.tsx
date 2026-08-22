import React from "react"
import { IconProps } from "types/icon"
import styles from "../styles/ui.module.css"

/**
 * Matrix spinner: a gold arc on a hairline ring. No Tailwind `animate-spin` —
 * the keyframe lives in the theme's own stylesheet so the theme does not depend
 * on core's utility set, and it slows rather than stops under reduced motion.
 */
const Spinner: React.FC<IconProps> = ({
  size = "16",
  color = "currentColor",
  ...attributes
}) => (
  <svg
    className={styles.spinner}
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    {...attributes}
  >
    <circle
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      strokeOpacity="0.15"
      strokeWidth="2"
    />
    <path
      d="M22 12a10 10 0 0 0-10-10"
      stroke={color === "currentColor" ? "var(--accent-gold)" : color}
      strokeWidth="2"
      strokeLinecap="square"
    />
  </svg>
)

export default Spinner
