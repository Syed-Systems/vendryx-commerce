import React from "react"
import LocalizedClientLink from "@modules/common/components/localized-client-link"
import { ArrowUpRight } from "@phosphor-icons/react/dist/ssr"

export type InteractiveLinkProps = {
  href: string
  children?: React.ReactNode
  onClick?: () => void
  className?: string
}

/**
 * Lustre InteractiveLink.
 * Direct action link with directional arrow and subtle hover translation.
 */
const InteractiveLink: React.FC<InteractiveLinkProps> = ({
  href,
  children,
  onClick,
  className = "",
  ...props
}) => {
  return (
    <LocalizedClientLink
      href={href}
      onClick={onClick}
      className={`inline-flex items-center gap-1.5 text-sm font-medium text-[var(--accent-sapphire)] hover:text-[var(--accent-sapphire-hover)] transition-colors group lustre-btn-press ${className}`}
      {...props}
    >
      <span>{children}</span>
      <ArrowUpRight
        size={14}
        className="transition-transform duration-150 ease-out group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
        aria-hidden="true"
      />
    </LocalizedClientLink>
  )
}

export default InteractiveLink
