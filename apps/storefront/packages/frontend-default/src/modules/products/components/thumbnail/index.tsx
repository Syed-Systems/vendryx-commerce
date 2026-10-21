import { clx } from "@medusajs/ui"
import Image from "next/image"
import React from "react"
import PlaceholderImage from "@modules/common/icons/placeholder-image"

type ThumbnailProps = {
  thumbnail?: string | null
  images?: any[] | null
  size?: "small" | "medium" | "large" | "full" | "square"
  isFeatured?: boolean
  className?: string
  /**
   * The product title, for the image's alt text.
   *
   * The audit found this component hardcoding `alt="Thumbnail"` and
   * `alt="Thumbnail Hover"`, so a screen-reader user heard "Thumbnail" once per
   * product in a grid — the component named itself instead of the garment.
   */
  alt?: string
  "data-testid"?: string
}

const Thumbnail: React.FC<ThumbnailProps> = ({
  thumbnail,
  images,
  size = "small",
  isFeatured,
  className,
  alt = "Product image",
  "data-testid": dataTestid,
}) => {
  const initialImage = thumbnail || images?.[0]?.url
  const hoverImage = images && images.length > 1 ? images[1].url : null

  return (
    <div
      className={clx(
        "relative w-full overflow-hidden bg-[color:var(--brand-surface)]/90 rounded-[var(--brand-radius-card)] border border-[color:var(--brand-hairline)]/60 group/thumb transition-all duration-300",
        className,
        {
          "aspect-[3/4]": isFeatured || size !== "square",
          "aspect-[1/1]": size === "square",
          "w-[180px]": size === "small",
          "w-[290px]": size === "medium",
          "w-[440px]": size === "large",
          "w-full": size === "full",
        }
      )}
      data-testid={dataTestid}
    >
      {initialImage ? (
        <>
          <Image
            src={initialImage}
            alt={alt}
            className={clx(
              "absolute inset-0 object-cover object-center transition-all duration-500 ease-out",
              hoverImage
                ? "group-hover/thumb:opacity-0 group-hover/thumb:scale-105"
                : "group-hover/thumb:scale-105"
            )}
            draggable={false}
            quality={80}
            sizes="(max-width: 576px) 280px, (max-width: 768px) 360px, (max-width: 992px) 480px, 800px"
            fill
          />
          {/* The hover frame was rendered eagerly, so every grid doubled its image
              bytes on mount whether or not anyone hovered. It is now lazy, and it
              carries `priority={false}` explicitly because it is the second
              request competing with LCP for the first. */}
          {hoverImage ? (
            <Image
              src={hoverImage}
              alt={`${alt} — alternate view`}
              loading="lazy"
              className="absolute inset-0 object-cover object-center opacity-0 group-hover/thumb:opacity-100 group-hover/thumb:scale-105 transition-all duration-500 ease-out"
              draggable={false}
              quality={80}
              sizes="(max-width: 576px) 280px, (max-width: 768px) 360px, (max-width: 992px) 480px, 800px"
              fill
            />
          ) : null}
        </>
      ) : (
        /* A product with no image gets a placeholder and a label naming what is
           missing, rather than an empty box that reads as a broken image. */
        <div
          className="w-full h-full absolute inset-0 flex flex-col items-center justify-center gap-2 text-[color:var(--ink-faint)]"
          role="img"
          aria-label={`No image available for ${alt}`}
        >
          <PlaceholderImage size={size === "small" ? 16 : 24} />
        </div>
      )}
    </div>
  )
}

export default Thumbnail
