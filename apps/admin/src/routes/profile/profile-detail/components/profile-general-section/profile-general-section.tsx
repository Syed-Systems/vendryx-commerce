import { PencilSquare } from "@medusajs/icons"
import { HttpTypes } from "@medusajs/types"
import { Badge, Container, Heading, Text } from "@medusajs/ui"
import { useTranslation } from "react-i18next"
import { ActionMenu } from "../../../../../components/common/action-menu"
import { languages } from "../../../../../i18n/languages"

type ProfileGeneralSectionProps = {
  user: HttpTypes.AdminUser
}

export const ProfileGeneralSection = ({ user }: ProfileGeneralSectionProps) => {
  const { i18n, t } = useTranslation()

  const name = [user.first_name, user.last_name].filter(Boolean).join(" ")
  const initials = [user.first_name?.[0], user.last_name?.[0]]
    .filter(Boolean)
    .join("")
    .toUpperCase() || (user.email ? user.email.slice(0, 2).toUpperCase() : "VX")

  const createdDate = user.created_at
    ? new Date(user.created_at).toLocaleDateString(undefined, {
        dateStyle: "medium",
      })
    : null

  return (
    <Container className="divide-y p-0">
      <div className="flex items-center justify-between px-6 py-4">
        <div className="flex items-center gap-x-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-ui-tag-orange-bg border border-ui-tag-orange-border text-ui-tag-orange-text font-semibold text-base shadow-sm">
            {initials}
          </div>
          <div>
            <div className="flex items-center gap-x-2.5">
              <Heading level="h2">{name || "Store Operator"}</Heading>
              <Badge color="orange" size="2xsmall">
                Store Administrator
              </Badge>
            </div>
            <Text className="text-ui-fg-subtle" size="small">
              {user.email}
            </Text>
          </div>
        </div>
        <ActionMenu
          groups={[
            {
              actions: [
                {
                  label: t("actions.edit"),
                  to: "edit",
                  icon: <PencilSquare />,
                },
              ],
            },
          ]}
        />
      </div>

      <div className="grid grid-cols-2 items-center px-6 py-4">
        <Text size="small" leading="compact" weight="plus">
          {t("fields.name")}
        </Text>
        <Text size="small" leading="compact" className="text-ui-fg-base">
          {name || "-"}
        </Text>
      </div>

      <div className="grid grid-cols-2 items-center px-6 py-4">
        <Text size="small" leading="compact" weight="plus">
          {t("fields.email")}
        </Text>
        <Text size="small" leading="compact" className="text-ui-fg-base">
          {user.email}
        </Text>
      </div>

      <div className="grid grid-cols-2 items-center px-6 py-4">
        <Text size="small" leading="compact" weight="plus">
          {t("profile.fields.languageLabel")}
        </Text>
        <Text size="small" leading="compact" className="text-ui-fg-base">
          {languages.find((lang) => lang.code === i18n.language)?.display_name || "-"}
        </Text>
      </div>

      {createdDate && (
        <div className="grid grid-cols-2 items-center px-6 py-4">
          <Text size="small" leading="compact" weight="plus">
            Member Since
          </Text>
          <Text size="small" leading="compact" className="text-ui-fg-subtle">
            {createdDate}
          </Text>
        </div>
      )}
    </Container>
  )
}
