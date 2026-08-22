import LocalizedClientLink from "@modules/common/components/localized-client-link"
import styles from "../../styles/ui.module.css"

type InteractiveLinkProps = {
  href: string
  children?: React.ReactNode
  onClick?: () => void
}

/**
 * Matrix link treatment: mono, gold, underlined on hover by a border rather
 * than text-decoration, arrow slides right. Replaces @medusajs/ui's Text and
 * @medusajs/icons' ArrowUpRightMini.
 */
const InteractiveLink = ({
  href,
  children,
  onClick,
  ...props
}: InteractiveLinkProps) => (
  <LocalizedClientLink
    className={styles.link}
    href={href}
    onClick={onClick}
    {...props}
  >
    <span>{children}</span>
    <svg
      className={styles.linkArrow}
      width="14"
      height="14"
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
)

export default InteractiveLink
