import { CORE_LAYOUT_IDS } from "@medusajs/admin-shared"
import {
  ArrowUturnLeft,
  BuildingStorefront,
  Channels,
  CommandLine,
  CreditCard,
  CurrencyDollar,
  DocumentText,
  GlobeEurope,
  Key,
  Language,
  LockClosedSolid,
  MapPin,
  ReceiptPercent,
  ShieldCheck,
  Sparkles,
  Tag,
  TruckFast,
  User,
  Users,
} from "@medusajs/icons"
import { clx, Text } from "@medusajs/ui"
import { useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { Link, useLocation } from "react-router-dom"

import { useExtension } from "../../../providers/extension-provider"
import { LayoutComposer } from "../../layout-composer"
import { CUSTOMIZE_IDS } from "../../layout-composer/constants"
import { INavItem, NavItem } from "../nav-item"
import { Shell } from "../shell"
import { UserMenu } from "../user-menu"
import { useFeatureFlag } from "../../../providers/feature-flag-provider"
import { usePermissions } from "../../../providers/permissions-provider"

export const SettingsLayout = () => {
  return (
    <Shell>
      <SettingsSidebar />
    </Shell>
  )
}

const useSettingRoutes = (): INavItem[] => {
  const isTranslationsEnabled = useFeatureFlag("translation")
  const isRbacEnabled = useFeatureFlag("rbac")
  const isViewConfigEnabled = useFeatureFlag("view_configurations")
  const { hasPermission } = usePermissions()
  const { t } = useTranslation()

  const canReadRoles = isRbacEnabled && hasPermission("rbac_role:read")
  const canReadPolicies = isRbacEnabled && hasPermission("rbac_policy:read")

  return useMemo(
    () => [
      {
        label: t("store.domain"),
        to: "/settings/store",
        icon: <BuildingStorefront />,
      },
      {
        label: "Storefront Branding",
        to: "/settings/branding",
        icon: <Sparkles />,
      },
      {
        label: t("users.domain"),
        to: "/settings/users",
        icon: <Users />,
      },
      ...(canReadRoles
        ? [
            {
              label: t("roles.domain"),
              to: "/settings/roles",
              icon: <ShieldCheck />,
            },
          ]
        : []),
      ...(canReadPolicies
        ? [
            {
              label: t("policies.domain"),
              to: "/settings/policies",
              icon: <DocumentText />,
            },
          ]
        : []),
      {
        label: t("regions.domain"),
        to: "/settings/regions",
        icon: <GlobeEurope />,
      },
      {
        label: "Payment Providers",
        to: "/settings/payment-providers",
        icon: <CreditCard />,
      },
      {
        label: "Courier Logistics",
        to: "/settings/couriers",
        icon: <TruckFast />,
      },
      {
        label: "Domains & SSL",
        to: "/settings/domains",
        icon: <GlobeEurope />,
      },
      {
        label: t("taxRegions.domain"),
        to: "/settings/tax-regions",
        icon: <ReceiptPercent />,
      },
      {
        label: t("salesChannels.domain"),
        to: "/settings/sales-channels",
        icon: <Channels />,
      },
      {
        label: t("stockLocations.domain"),
        to: "/settings/locations",
        icon: <MapPin />,
      },
      {
        label: t("productTypes.domain"),
        to: "/settings/product-types",
        icon: <Tag />,
      },
      {
        label: t("productTags.domain"),
        to: "/settings/product-tags",
        icon: <Tag />,
      },
      {
        label: t("returnReasons.domain"),
        to: "/settings/return-reasons",
        icon: <ArrowUturnLeft />,
      },
      {
        label: t("refundReasons.domain"),
        to: "/settings/refund-reasons",
        icon: <CurrencyDollar />,
      },
      ...(isViewConfigEnabled
        ? [
            {
              label: t("propertyLabels.domain", "Property Labels"),
              to: "/settings/property-labels",
              icon: <DocumentText />,
            },
          ]
        : []),
      ...(isTranslationsEnabled
        ? [
            {
              label: t("translations.domain"),
              to: "/settings/translations",
              icon: <Language />,
            },
          ]
        : []),
    ],
    [
      t,
      isTranslationsEnabled,
      isViewConfigEnabled,
      canReadRoles,
      canReadPolicies,
    ]
  )
}

const useDeveloperRoutes = (): INavItem[] => {
  const { t } = useTranslation()

  return useMemo(
    () => [
      {
        label: t("apiKeyManagement.domain.publishable"),
        to: "/settings/publishable-api-keys",
        icon: <Key />,
      },
      {
        label: t("apiKeyManagement.domain.secret"),
        to: "/settings/secret-api-keys",
        icon: <LockClosedSolid />,
      },
      {
        label: t("workflowExecutions.domain"),
        to: "/settings/workflows",
        icon: <CommandLine />,
      },
    ],
    [t]
  )
}

const useMyAccountRoutes = (): INavItem[] => {
  const { t } = useTranslation()

  return useMemo(
    () => [
      {
        label: t("profile.domain"),
        to: "/settings/profile",
        icon: <User />,
      },
      {
        label: "Subscription & Billing",
        to: "/settings/billing",
        icon: <CreditCard />,
      },
    ],
    [t]
  )
}

/**
 * Ensure that the `from` prop is not another settings route, to avoid
 * the user getting stuck in a navigation loop.
 */
const getSafeFromValue = (from: string) => {
  if (from.startsWith("/settings")) {
    return "/orders"
  }

  return from
}

const toNavEntries = (items: INavItem[]) =>
  items.map((item) => (
    <LayoutComposer.Entry id={`settings-nav:${item.to}`} key={item.to}>
      <NavItem key={item.to} type="setting" {...item} />
    </LayoutComposer.Entry>
  ))

const SettingsSidebar = () => {
  const { getMenu } = useExtension()

  const routes = useSettingRoutes()
  const developerRoutes = useDeveloperRoutes()
  const myAccountRoutes = useMyAccountRoutes()
  const extensionRoutes = getMenu("settingsExtensions")

  return (
    <aside className="relative flex flex-1 flex-col justify-between overflow-y-auto h-full select-none">
      <div className="bg-[var(--vx-bg-sidebar,#F7F3ED)] dark:bg-[#141416] sticky top-0 z-[1] border-b border-ui-border-base/60">
        <Header />
      </div>
      <div className="flex flex-1 flex-col min-h-0">
        <div className="flex-1 overflow-y-auto overflow-x-hidden p-2">
          <LayoutComposer
            widgetsZonePrefix="settings.sidebar"
            preferredLayoutId={CORE_LAYOUT_IDS.SETTINGS_SIDEBAR}
            hasOutlet={false}
            disableWidgets
            customizeId={CUSTOMIZE_IDS.SETTINGS_SIDEBAR}
            controlSize="small"
            sections={{
              general: toNavEntries(routes),
              developer: toNavEntries(developerRoutes),
              myAccount: toNavEntries(myAccountRoutes),
              extensions: toNavEntries(extensionRoutes),
            }}
          />
        </div>
        <div className="sticky bottom-0 z-[1] border-t border-ui-border-base/60 bg-[var(--vx-bg-sidebar,#F7F3ED)] dark:bg-[#141416]">
          <UserSection />
        </div>
      </div>
    </aside>
  )
}

const Header = () => {
  const [from, setFrom] = useState("/orders")

  const { t } = useTranslation()
  const location = useLocation()

  useEffect(() => {
    if (location.state?.from) {
      setFrom(getSafeFromValue(location.state.from))
    }
  }, [location])

  return (
    <div className="p-2.5">
      <Link
        to={from}
        replace
        className={clx(
          "group transition-all duration-150 flex items-center gap-x-2.5 rounded-xl p-1.5 outline-none",
          "hover:bg-black/[0.04] dark:hover:bg-white/[0.06]",
          "focus-visible:shadow-borders-focus"
        )}
      >
        <div className="flex size-7 items-center justify-center rounded-lg bg-ui-bg-subtle border border-ui-border-base shadow-xs text-ui-fg-muted group-hover:text-ui-fg-base transition-colors shrink-0">
          <ArrowUturnLeft className="size-3.5" />
        </div>
        <div className="flex flex-col min-w-0">
          <Text leading="compact" weight="plus" size="small" className="text-[13px] font-semibold text-ui-fg-base">
            {t("app.nav.settings.header", "Settings")}
          </Text>
          <Text size="xsmall" leading="compact" className="text-xs text-ui-fg-muted">
            {t("app.nav.settings.return", "Back to store")}
          </Text>
        </div>
      </Link>
    </div>
  )
}

const UserSection = () => {
  return (
    <div>
      <UserMenu />
    </div>
  )
}
