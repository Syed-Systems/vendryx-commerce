import { Button, Container, Heading, Input, Label, Text } from "@medusajs/ui"
import { useState } from "react"
import { useNavigate } from "react-router-dom"
import { sdk } from "../../lib/client"
import { queryClient } from "../../lib/query-client"

const Login = () => {
  const navigate = useNavigate()
  const [email, setEmail] = useState("owner@vendryx.store")
  const [password, setPassword] = useState("demo-password")
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setSubmitting(true)
    try {
      await sdk.auth.login("user", "emailpass", { email, password })
      await queryClient.invalidateQueries()
      navigate("/", { replace: true })
    } catch (err: any) {
      setError(err?.message || "Unable to sign in")
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="bg-ui-bg-subtle flex min-h-dvh w-full items-center justify-center p-4">
      <Container className="w-full max-w-sm p-6">
        <div className="mb-5 flex flex-col items-center gap-y-2 text-center">
          <img
            src="/brand/vendryx/vendryx-mark.png"
            alt="Vendryx"
            className="h-10 w-10 rounded-lg"
          />
          <Heading level="h1">Sign in to Vendryx</Heading>
          <Text size="small" className="text-ui-fg-subtle">
            Enter your merchant credentials to continue.
          </Text>
        </div>
        <form onSubmit={handleSubmit} className="flex flex-col gap-y-4">
          <div className="flex flex-col gap-y-1.5">
            <Label htmlFor="login-email">Email</Label>
            <Input
              id="login-email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>
          <div className="flex flex-col gap-y-1.5">
            <Label htmlFor="login-password">Password</Label>
            <Input
              id="login-password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>
          {error && (
            <Text size="xsmall" className="text-ui-fg-error">
              {error}
            </Text>
          )}
          <Button type="submit" isLoading={submitting} className="w-full">
            Continue
          </Button>
        </form>
      </Container>
    </div>
  )
}

export const Component = Login
