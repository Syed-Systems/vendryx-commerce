import Image from "next/image"
import React from "react"
import PlaceholderImage from "@modules/common/icons/placeholder-image"

export type ThumbnailProps = {
  thumbnail?: string | null
  images?: any[] | null
  size?: "small" | "medium" | "large" | "full" | "square"
  isFeatured?: boolean
  className?: string
  "data-testid"?: string
}

/**
 * Lustre Thumbnail.
 * Clean, restrained product image presentation with 16px surface radius and secondary crossfade.
 */
const Thumbnail: React.FC<ThumbnailProps> = ({
  thumbnail,
  images,
  size = "small",
  isFeatured,
  className = "",
  "data-testid": dataTestId,
}) => {
  const initialImage = thumbnail || images?.[0]?.url
  const hoverImage = images && images.length > 1 ? images[1]?.url : null

  const sizeClasses = {
    small: "w-20",
    medium: "w-56",
    large: "w-80",
    full: "w-full",
    square: "w-full",
  }[size]

  const aspectClass = size === "square" ? "aspect-square" : "aspect-[4/5]"

  return (
    <div
      data-testid={dataTestId}
      className={`relative overflow-hidden rounded-[16px] bg-[var(--bg-raised)] border border-[var(--hairline)] group/thumb lustre-product-hover ${aspectClass} ${sizeClasses} ${className}`}
    >
      {initialImage ? (
        <>
          <Image
            src={initialImage}
            alt="Product view"
            fill
            sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
            className={`object-cover object-center transition-opacity duration-300 ease-out ${
              hoverImage ? "group-hover/thumb:opacity-0" : ""
            }`}
            draggable={false}
          />
          {hoverImage && (
            <Image
              src={hoverImage}
              alt="Alternative product view"
              fill
              sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
              className="object-cover object-center opacity-0 group-hover/thumb:opacity-100 transition-opacity duration-300 ease-out"
              draggable={false}
            />
          )}
        </>
      ) : (
        <div className="absolute inset-0 flex items-center justify-center text-[var(--text-muted)]">
          <PlaceholderImage size={size === "small" ? "20" : "32"} />
        </div>
      )}
    </div>
  )
}

export default Thumbnail
