import { Metadata } from "next"
import { notFound } from "next/navigation"
import { getRegion } from "@lib/data/regions"

export const metadata: Metadata = {
  title: "About Us | Vendryx Commerce",
  description: "Learn more about our heritage, craft, and mission.",
}

export default async function AboutPage(props: {
  params: Promise<{ countryCode: string }>
}) {
  const params = await props.params
  const { countryCode } = params
  const region = await getRegion(countryCode)

  if (!region) {
    notFound()
  }

  return (
    <div className="content-container py-16 max-w-4xl mx-auto px-4">
      <h1 className="text-3xl font-semibold mb-6">About Us</h1>
      <div className="prose max-w-none text-ui-fg-subtle leading-relaxed space-y-4">
        <p>
          Welcome to our store. We craft enduring pieces with mindful attention to detail,
          materials, and ethics.
        </p>
        <p>
          Every item is designed to be cherished, bridging contemporary aesthetic with
          responsible production.
        </p>
      </div>
    </div>
  )
}
