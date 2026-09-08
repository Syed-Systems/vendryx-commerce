import { Badge, clx } from "@medusajs/ui"

export type StatusVariant = "active" | "warning" | "error" | "neutral" | "pending"

export interface StatusAccessoryProps {
  label: string
  variant?: StatusVariant
  className?: string
}

export const StatusAccessory = ({
  label,
  variant = "neutral",
  className,
}: StatusAccessoryProps) => {
  const badgeColor =
    variant === "active"
      ? "green"
      : variant === "warning" || variant === "pending"
      ? "orange"
      : variant === "error"
      ? "red"
      : "grey"

  return (
    <Badge size="2xsmall" color={badgeColor} className={clx("capitalize", className)}>
      {label}
    </Badge>
  )
}
