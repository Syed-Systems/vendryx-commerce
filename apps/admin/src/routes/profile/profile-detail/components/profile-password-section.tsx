import { LockClosedSolid, Key } from "@medusajs/icons"
import { Button, Container, Heading, Input, Label, Text, toast } from "@medusajs/ui"
import { useState } from "react"
import { sdk } from "../../../../lib/client"

export const ProfilePasswordSection = () => {
  const [isOpen, setIsOpen] = useState(false)
  const [currentPassword, setCurrentPassword] = useState("")
  const [newPassword, setNewPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!currentPassword) {
      toast.error("Current password is required")
      return
    }

    if (newPassword.length < 8) {
      toast.error("New password must be at least 8 characters long")
      return
    }

    if (newPassword !== confirmPassword) {
      toast.error("New passwords do not match")
      return
    }

    setIsSubmitting(true)
    try {
      await sdk.client.fetch("/auth/user/emailpass/update", {
        method: "POST",
        body: {
          old_password: currentPassword,
          new_password: newPassword,
        },
      })
      setIsOpen(false)
      setCurrentPassword("")
      setNewPassword("")
      setConfirmPassword("")
      toast.success("Password updated successfully")
    } catch (err: any) {
      toast.error(err?.message || "Failed to update password. Please check your current password.")
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <>
      <Container className="divide-y p-0">
        <div className="flex items-center justify-between px-6 py-4">
          <div>
            <Heading level="h2">Account Security &amp; Password</Heading>
            <Text className="text-ui-fg-subtle" size="small">
              Ensure your merchant account is protected with a strong, rotated credentials policy.
            </Text>
          </div>
          <Button
            size="small"
            variant="secondary"
            onClick={() => setIsOpen(true)}
          >
            <Key className="me-1.5 text-ui-fg-subtle" />
            Change password
          </Button>
        </div>

        <div className="grid grid-cols-1 gap-y-4 px-6 py-4 sm:grid-cols-2">
          <div>
            <Text size="small" weight="plus" className="text-ui-fg-base">
              Password Status
            </Text>
            <Text size="xsmall" className="text-ui-fg-subtle mt-0.5">
              Secure 128-bit hash standard
            </Text>
          </div>
          <div className="flex items-center gap-x-2">
            <LockClosedSolid className="text-ui-tag-green-icon h-4 w-4" />
            <Text size="small" className="text-ui-fg-subtle">
              Active and protected with bcrypt / argon2 hash
            </Text>
          </div>
        </div>
      </Container>

      {isOpen && (
        <div
          role="dialog"
          aria-labelledby="change-password-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-ui-bg-overlay/50 p-4 backdrop-blur-sm"
        >
          <div className="w-full max-w-md rounded-xl border border-ui-border-base bg-ui-bg-base p-6 shadow-xl">
            <div className="flex items-center justify-between pb-4 border-b border-ui-border-base">
              <div>
                <Heading id="change-password-title" level="h2">Change Password</Heading>
                <Text size="xsmall" className="text-ui-fg-subtle">
                  Update your merchant login credentials
                </Text>
              </div>
              <Button
                variant="transparent"
                size="small"
                onClick={() => setIsOpen(false)}
                aria-label="Close"
              >
                ✕
              </Button>
            </div>

            <form onSubmit={handleSubmit} className="mt-4 flex flex-col gap-y-4">
              <div>
                <Label htmlFor="current-pw" className="text-ui-fg-subtle text-xs">
                  Current Password
                </Label>
                <Input
                  id="current-pw"
                  type="password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  placeholder="Enter current password"
                  autoComplete="current-password"
                  required
                />
              </div>

              <div>
                <Label htmlFor="new-pw" className="text-ui-fg-subtle text-xs">
                  New Password
                </Label>
                <Input
                  id="new-pw"
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder="Minimum 8 characters"
                  autoComplete="new-password"
                  required
                />
              </div>

              <div>
                <Label htmlFor="confirm-pw" className="text-ui-fg-subtle text-xs">
                  Confirm New Password
                </Label>
                <Input
                  id="confirm-pw"
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder="Re-enter new password"
                  autoComplete="new-password"
                  required
                />
              </div>

              <div className="mt-2 flex items-center justify-end gap-x-2 border-t border-ui-border-base pt-4">
                <Button
                  type="button"
                  variant="secondary"
                  size="small"
                  onClick={() => setIsOpen(false)}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  variant="primary"
                  size="small"
                  isLoading={isSubmitting}
                >
                  Update password
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </>
  )
}
