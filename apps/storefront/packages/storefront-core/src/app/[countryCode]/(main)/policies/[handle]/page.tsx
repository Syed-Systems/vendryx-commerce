import { Metadata } from "next"
import { notFound } from "next/navigation"
import { getRegion } from "@lib/data/regions"

export async function generateMetadata(props: {
  params: Promise<{ countryCode: string; handle: string }>
}): Promise<Metadata> {
  const params = await props.params
  const title = params.handle
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")

  return {
    title: `${title} | Store Policies`,
    description: `Read our official ${title.toLowerCase()} policy.`,
  }
}

export default async function PolicyPage(props: {
  params: Promise<{ countryCode: string; handle: string }>
}) {
  const params = await props.params
  const { countryCode, handle } = params
  const region = await getRegion(countryCode)

  if (!region) {
    notFound()
  }

  const title = handle
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")

  return (
    <div className="content-container py-16 max-w-4xl mx-auto px-4">
      <h1 className="text-3xl font-semibold mb-6">{title}</h1>
      <div className="prose max-w-none text-ui-fg-subtle leading-relaxed space-y-4">
        <p>
          Please review the terms and specifications for {title.toLowerCase()} applicable
          to your orders in this region.
        </p>
      </div>
    </div>
  )
}
