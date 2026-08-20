import { Metadata } from "next"
import FeaturedProducts from "@core/loaders/featured-products"
import Hero from "@theme/home/components/hero"
import BrandFeatures from "@theme/home/components/brand-features"
import CategoryGrid from "@theme/home/components/category-grid"
import LookbookStory from "@theme/home/components/lookbook-story"
import NewsletterSection from "@theme/home/components/newsletter-section"
import { listCollections } from "@lib/data/collections"
import { getRegion } from "@lib/data/regions"

export const metadata: Metadata = {
  title: "STHENO Atelier | Contemporary Luxury & Timeless Apparel",
  description:
    "Explore curated collections of conscious luxury apparel, tailored outerwear, and organic essentials crafted with enduring minimalism.",
}

export default async function Home(props: {
  params: Promise<{ countryCode: string }>
}) {
  const params = await props.params
  const { countryCode } = params

  const region = await getRegion(countryCode)
  const { collections } = await listCollections({
    fields: "id, handle, title",
  })

  if (!region) {
    return null
  }

  return (
    <div className="w-full flex flex-col">
      {/* Editorial Fashion Hero */}
      <Hero />

      {/* Trust & Brand Values Bar */}
      <BrandFeatures />

      {/* Curated Wardrobe Categories Grid */}
      <CategoryGrid />

      {/* Featured Products / Capsule Collections */}
      {collections && collections.length > 0 && (
        <section className="w-full bg-white">
          <ul className="flex flex-col">
            <FeaturedProducts collections={collections} region={region} />
          </ul>
        </section>
      )}

      {/* Brand Craft & Lookbook Narrative */}
      <LookbookStory />

      {/* VIP Club / Newsletter Capture */}
      <NewsletterSection />
    </div>
  )
}
