import { ComputerDesktop, ArrowRightOnRectangle, CheckCircle } from "@medusajs/icons"
import { Badge, Button, Container, Heading, Text, usePrompt } from "@medusajs/ui"
import { useMemo } from "react"
import { useNavigate } from "react-router-dom"
import { useLogout } from "../../../../hooks/api/auth"

const MobilePhoneIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <rect width="14" height="20" x="5" y="2" rx="2" ry="2"/>
    <path d="M12 18h.01"/>
  </svg>
)

export const ProfileSessionsSection = () => {
  const prompt = usePrompt()
  const navigate = useNavigate()
  const { mutateAsync: logout, isPending: isLoggingOut } = useLogout()

  const currentSession = useMemo(() => {
    const ua = typeof navigator !== "undefined" ? navigator.userAgent : ""
    const browser = /Chrome/i.test(ua)
      ? "Chrome"
      : /Firefox/i.test(ua)
      ? "Firefox"
      : /Safari/i.test(ua)
      ? "Safari"
      : /Edge/i.test(ua)
      ? "Edge"
      : "Browser"
    const isMobile = /Mobile|Android|iPhone|iPad/i.test(ua)
    const os = /Windows/i.test(ua)
      ? "Windows"
      : /Macintosh|Mac OS/i.test(ua)
      ? "macOS"
      : /Linux/i.test(ua)
      ? "Linux"
      : /Android/i.test(ua)
      ? "Android"
      : /iPhone|iPad/i.test(ua)
      ? "iOS"
      : "Desktop"

    return {
      device: `${os} ${isMobile ? "Mobile" : "Workstation"}`,
      browser: `${browser} • Active browser`,
      isMobile,
      lastActive: "Active now",
    }
  }, [])

  const handleSignOut = async () => {
    const confirmed = await prompt({
      title: "Sign Out",
      description: "Are you sure you want to end your active session on this device?",
      confirmText: "Sign out",
      cancelText: "Cancel",
      variant: "danger",
    })

    if (!confirmed) return

    try {
      await logout()
      navigate("/login")
    } catch {
      // Force navigation to login on error as fallback
      navigate("/login")
    }
  }

  return (
    <Container className="divide-y p-0">
      <div className="flex flex-col gap-y-2 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <Heading level="h2">Active Device Session</Heading>
          <Text className="text-ui-fg-subtle" size="small">
            Review the device and browser currently authenticated into your Vendryx merchant profile.
          </Text>
        </div>
        <Button
          size="small"
          variant="secondary"
          onClick={handleSignOut}
          isLoading={isLoggingOut}
          className="w-fit"
        >
          <ArrowRightOnRectangle className="me-1.5 text-ui-fg-subtle" />
          Sign out of device
        </Button>
      </div>

      <div className="divide-y divide-ui-border-base">
        <div className="flex flex-col gap-y-3 px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-start gap-x-3.5">
            <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-ui-border-base bg-ui-bg-subtle text-ui-fg-muted">
              {currentSession.isMobile ? <MobilePhoneIcon /> : <ComputerDesktop />}
            </div>
            <div>
              <div className="flex items-center gap-x-2">
                <Text size="small" weight="plus" className="text-ui-fg-base">
                  {currentSession.device}
                </Text>
                <Badge color="green" size="2xsmall" className="gap-x-1">
                  <CheckCircle className="h-3 w-3" />
                  Current session
                </Badge>
              </div>
              <Text size="xsmall" className="text-ui-fg-subtle mt-0.5">
                {currentSession.browser} • Authenticated session
              </Text>
            </div>
          </div>

          <div>
            <Text size="xsmall" className="text-ui-fg-muted italic">
              Active now
            </Text>
          </div>
        </div>
      </div>
    </Container>
  )
}
