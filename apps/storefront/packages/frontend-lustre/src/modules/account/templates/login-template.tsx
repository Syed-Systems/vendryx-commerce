"use client"

import React, { useState } from "react"
import Login from "../components/login"
import Register from "../components/register"

export enum LOGIN_VIEW {
  SIGN_IN = "sign-in",
  REGISTER = "register",
}

/**
 * Lustre LoginTemplate (Slot 18).
 * Client authentication and onboarding portal view.
 */
const LoginTemplate: React.FC = () => {
  const [currentView, setCurrentView] = useState<LOGIN_VIEW>(LOGIN_VIEW.SIGN_IN)

  return (
    <div className="w-full flex justify-center py-12 px-4 sm:px-6">
      {currentView === LOGIN_VIEW.SIGN_IN ? (
        <Login setCurrentView={setCurrentView} />
      ) : (
        <Register setCurrentView={setCurrentView} />
      )}
    </div>
  )
}

export default LoginTemplate
