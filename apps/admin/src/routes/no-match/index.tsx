import { Button, Container, Heading, Text } from "@medusajs/ui"
import { Link } from "react-router-dom"

const NoMatch = () => {
  return (
    <div className="bg-ui-bg-subtle flex min-h-dvh w-full items-center justify-center p-4">
      <Container className="flex max-w-md flex-col items-center gap-y-3 p-6 text-center">
        <Heading level="h1">Page not found</Heading>
        <Text size="small" className="text-ui-fg-subtle">
          The page you requested does not exist in this workspace.
        </Text>
        <Button size="small" asChild className="mt-2">
          <Link to="/">Return to Dashboard</Link>
        </Button>
      </Container>
    </div>
  )
}

export const Component = NoMatch
