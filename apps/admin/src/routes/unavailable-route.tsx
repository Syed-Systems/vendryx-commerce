import { Button, Container, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"

const formatRouteLabel = (slug: string) =>
  slug
    .split(/[-_/]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ")

export const createUnavailableRoute = (label: string) => {
  const title = formatRouteLabel(label)
  const UnavailableRouteComponent = () => (
    <div className="mx-auto flex w-full max-w-[960px] flex-col gap-y-4 px-4 py-8 sm:px-6">
      <Container className="flex flex-col items-start gap-y-3 p-6">
        <Heading level="h1">{title}</Heading>
        <Text size="small" className="text-ui-fg-subtle max-w-xl">
          This section was omitted when the repository was imported. Active workspace modules
          including Overview, Analytics, Customers, Categories, Collections, Campaigns, Branding,
          Couriers, Domains, and Help remain available from the sidebar.
        </Text>
        <div className="flex flex-wrap items-center gap-2 pt-2">
          <Button size="small" variant="primary" asChild>
            <Link to="/">Back to Overview</Link>
          </Button>
          <Button size="small" variant="secondary" asChild>
            <Link to="/analytics">Open Analytics</Link>
          </Button>
          <Button size="small" variant="secondary" asChild>
            <Link to="/customers">Open Customers</Link>
          </Button>
        </div>
      </Container>
    </div>
  )

  return {
    Component: UnavailableRouteComponent,
    Breadcrumb: (_props?: any) => <span>{title}</span>,
    loader: async () => null,
    seo: (_match?: any) => ({ title }),
  }
}
