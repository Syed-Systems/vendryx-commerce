import { Metadata } from "next"

import LoginTemplate from "@frontend/account/templates/login-template"

export const metadata: Metadata = {
  title: "Sign in",
  description: "Sign in to your Stheno account.",
}

export default function Login() {
  return <LoginTemplate />
}
