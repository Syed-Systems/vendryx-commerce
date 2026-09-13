import React from "react"
import { SignUp, useAuth } from "@clerk/clerk-react"
import { Link, useNavigate } from "react-router-dom"
import AvatarBox from "../../components/common/logo-box/avatar-box"
import {
  useClerkAuth,
  vendryxClerkAppearance,
} from "../../providers/clerk-provider"

export const SignUpPage: React.FC = () => {
  const navigate = useNavigate()
  const { isClerkEnabled } = useClerkAuth()
  const { isSignedIn } = useAuth()

  React.useEffect(() => {
    if (isSignedIn) {
      navigate("/orders", { replace: true })
    }
  }, [isSignedIn, navigate])

  return (
    <div className="bg-ui-bg-subtle flex min-h-dvh w-dvw flex-col items-center justify-center p-4">
      <div className="mb-6 flex flex-col items-center">
        <AvatarBox />
        <h1 className="text-ui-fg-base mt-2 text-xl font-bold">
          Create Storefront Owner Account
        </h1>
        <p className="text-ui-fg-subtle text-sm">
          Vendryx Commerce Workspace by Syed Systems
        </p>
      </div>

      {isClerkEnabled ? (
        <SignUp
          appearance={vendryxClerkAppearance}
          routing="hash"
          fallbackRedirectUrl="/orders"
          signInUrl="/login"
        />
      ) : (
        <div className="bg-ui-bg-base border-ui-border-base flex max-w-sm flex-col items-center rounded-xl border p-6 text-center shadow-sm">
          <p className="text-ui-fg-subtle text-sm">
            Clerk registration requires <code>VITE_CLERK_PUBLISHABLE_KEY</code>{" "}
            to be configured in your environment.
          </p>
          <Link
            to="/login"
            className="text-ui-fg-interactive mt-4 text-sm font-medium hover:underline"
          >
            Return to Sign In
          </Link>
        </div>
      )}

      <div className="mt-6 text-center text-xs text-ui-fg-muted">
        Already registered?{" "}
        <Link to="/login" className="text-ui-fg-interactive font-medium">
          Sign In
        </Link>
      </div>
    </div>
  )
}

export default SignUpPage
