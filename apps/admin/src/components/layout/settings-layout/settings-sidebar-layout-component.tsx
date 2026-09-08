import { clx } from "@medusajs/ui"
import { Fragment, ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { LayoutComponentProps } from "../../layout-composer/types"

function hasContent(node: ReactNode): boolean {
  return Array.isArray(node) ? node.length > 0 : Boolean(node)
}

const SettingsGroup = ({
  label,
  children,
}: {
  label?: string
  children: ReactNode
}) => {
  if (!hasContent(children)) {
    return null
  }

  return (
    <div className="space-y-0.5">
      {label && (
        <div className="px-2.5 pt-3 pb-1 text-[11px] font-semibold tracking-wider text-ui-fg-muted/70 uppercase select-none">
          {label}
        </div>
      )}
      <div className="flex flex-col gap-y-0.5 px-0.5">{children}</div>
    </div>
  )
}

const GroupDivider = () => (
  <div className="my-1.5 mx-2.5 border-t border-ui-border-base/40" />
)

export const SettingsSidebarLayoutComponent = ({
  sections,
  className,
}: LayoutComponentProps) => {
  const { t } = useTranslation()

  return (
    <div className={clx("flex flex-1 flex-col pb-6", className)}>
      <SettingsGroup label={t("app.nav.settings.general", "General")}>
        {sections["general"]}
      </SettingsGroup>
      {hasContent(sections["developer"]) && (
        <Fragment>
          <GroupDivider />
          <SettingsGroup label={t("app.nav.settings.developer", "Developer")}>
            {sections["developer"]}
          </SettingsGroup>
        </Fragment>
      )}
      {hasContent(sections["myAccount"]) && (
        <Fragment>
          <GroupDivider />
          <SettingsGroup label={t("app.nav.settings.myAccount", "Account")}>
            {sections["myAccount"]}
          </SettingsGroup>
        </Fragment>
      )}
      {hasContent(sections["extensions"]) && (
        <Fragment>
          <GroupDivider />
          <SettingsGroup label={t("app.nav.common.extensions", "Extensions")}>
            {sections["extensions"]}
          </SettingsGroup>
        </Fragment>
      )}
    </div>
  )
}
