import React from "react"
import { SignIn, useAuth } from "@clerk/clerk-react"
import { useNavigate } from "react-router-dom"
import { vendryxClerkAppearance } from "../../../providers/clerk-provider"

interface ClerkLoginProps {
  redirectTo?: string
}

export const ClerkLogin: React.FC<ClerkLoginProps> = ({
  redirectTo = "/orders",
}) => {
  const navigate = useNavigate()
  const { isSignedIn } = useAuth()

  React.useEffect(() => {
    if (isSignedIn) {
      navigate(redirectTo, { replace: true })
    }
  }, [isSignedIn, navigate, redirectTo])

  return (
    <div className="flex w-full flex-col items-center justify-center">
      <SignIn
        appearance={vendryxClerkAppearance}
        routing="hash"
        fallbackRedirectUrl={redirectTo}
      />
    </div>
  )
}
