import { Metadata } from "next"
import FeaturedProducts from "@core/loaders/featured-products"
import Hero from "@frontend/home/components/hero"
import BrandFeatures from "@frontend/home/components/brand-features"
import CategoryGrid from "@frontend/home/components/category-grid"
import LookbookStory from "@frontend/home/components/lookbook-story"
import NewsletterSection from "@frontend/home/components/newsletter-section"
import { listCollections } from "@lib/data/collections"
import { getRegion } from "@lib/data/regions"

export const metadata: Metadata = {
  title: "Vendryx | Demo Store",
  description:
    "Demo store environment powered by Vendryx Commerce.",
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
      <div className="w-full bg-ui-bg-subtle text-ui-fg-base text-center py-2 text-sm border-b border-ui-border-base">
        <strong>Demo Store Environment</strong> — powered by Vendryx Commerce
      </div>

      <Hero />

      <CategoryGrid />

      {collections && collections.length > 0 && (
        <section className="w-full bg-[color:var(--brand-surface)]">
          <ul className="flex flex-col">
            <FeaturedProducts collections={collections} region={region} />
          </ul>
        </section>
      )}
    </div>
  )
}
