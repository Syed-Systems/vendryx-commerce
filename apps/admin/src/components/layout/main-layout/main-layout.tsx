import {
  Buildings,
  BuildingStorefront,
  CogSixTooth,
  CreditCard,
  CurrencyDollar,
  DocumentText,
  EllipsisHorizontal,
  MagnifyingGlass,
  OpenRectArrowOut,
  ReceiptPercent,
  ShoppingCart,
  Sparkles,
  SquaresPlus,
  Tag,
  Users,
  ChartPie,
} from "@medusajs/icons"
import { CORE_LAYOUT_IDS } from "@medusajs/admin-shared"
import { Avatar, clx, DropdownMenu, Kbd, Text } from "@medusajs/ui"
import { useTranslation } from "react-i18next"
import { useQuery } from "@tanstack/react-query"
import { sdk } from "../../../lib/client"

import { useStore } from "../../../hooks/api/store"
import { LayoutComposer } from "../../layout-composer"
import { PermissionGuard } from "../../common/permission-guard"
import { Skeleton } from "../../common/skeleton"
import { INavItem, NavItem } from "../../layout/nav-item"
import { Shell } from "../../layout/shell"

import { Link, useLocation } from "react-router-dom"
import { useExtension } from "../../../providers/extension-provider"
import { useSearch } from "../../../providers/search-provider"
import { UserMenu } from "../user-menu"
import { useDocumentDirection } from "../../../hooks/use-document-direction"
import { CUSTOMIZE_IDS } from "../../layout-composer/constants"

declare const __STOREFRONT_URL__: string | undefined

export const MainLayout = () => {
  return (
    <Shell>
      <MainSidebar />
    </Shell>
  )
}

const MainSidebar = () => {
  return (
    <aside className="flex flex-1 flex-col justify-between overflow-y-auto h-full select-none">
      <div className="flex flex-1 flex-col min-h-0">
        <PermissionGuard resource="store" operation="read">
          <div className="bg-[var(--vx-bg-sidebar,#F7F3ED)] dark:bg-[#141416] sticky top-0 z-[1] border-b border-ui-border-base/60">
            <Header />
          </div>
        </PermissionGuard>
        <div className="flex-1 overflow-y-auto overflow-x-hidden">
          <SidebarRoutes />
        </div>
        <div className="mt-auto">
          <UtilitySection />
        </div>
        <div className="sticky bottom-0 z-[1]">
          <UserSection />
        </div>
      </div>
    </aside>
  )
}

const Header = () => {
  const { t } = useTranslation()
  const { store, isPending, isError, error } = useStore()
  const direction = useDocumentDirection()
  const name = store?.name || "Vendryx Store"
  const fallback = name.slice(0, 1).toUpperCase()

  const isLoaded = !isPending && !!store && !!name && !!fallback

  const { data: entitlements } = useQuery<{ tier: string }>({
    queryKey: ["admin", "entitlements"],
    queryFn: async () => {
      try {
        return await sdk.client.fetch<{ tier: string }>("/admin/entitlements")
      } catch {
        return { tier: "free" }
      }
    },
  })
  const tier = entitlements?.tier || "Pro"
  const storefrontUrl = typeof __STOREFRONT_URL__ !== "undefined" ? __STOREFRONT_URL__ : "http://localhost:8000"

  if (isError) {
    throw error
  }

  return (
    <div className="w-full p-2.5">
      <DropdownMenu dir={direction}>
        <DropdownMenu.Trigger
          disabled={!isLoaded}
          className={clx(
            "bg-transparent transition-all duration-150 grid w-full grid-cols-[28px_1fr_auto] items-center gap-x-2.5 rounded-xl p-1.5 outline-none text-start",
            "hover:bg-black/[0.04] dark:hover:bg-white/[0.06]",
            "data-[state=open]:bg-black/[0.05] dark:data-[state=open]:bg-white/[0.08]",
            "focus-visible:shadow-borders-focus"
          )}
        >
          {fallback ? (
            <div className="size-7 rounded-lg bg-ui-bg-subtle border border-ui-border-base flex items-center justify-center overflow-hidden shrink-0 shadow-xs">
              <Avatar variant="squared" size="xsmall" src="/brand/vendryx/vendryx-mark.svg" fallback={fallback} />
            </div>
          ) : (
            <Skeleton className="h-7 w-7 rounded-lg shrink-0" />
          )}
          <div className="flex flex-col overflow-hidden text-start min-w-0">
            <div className="flex items-center gap-x-1.5">
              {name ? (
                <Text
                  size="small"
                  weight="plus"
                  leading="compact"
                  className="truncate text-ui-fg-base font-semibold text-[13px]"
                >
                  {name}
                </Text>
              ) : (
                <Skeleton className="h-[10px] w-[110px]" />
              )}
            </div>
            <div className="flex items-center gap-x-1.5 mt-0.5">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 shrink-0" />
              <Text size="xsmall" leading="compact" className="text-ui-fg-muted text-[11px] truncate font-normal">
                Active Store
              </Text>
            </div>
          </div>
          <div className="flex items-center gap-x-1 shrink-0 ps-1">
            <span className="font-semibold text-[10px] px-1.5 py-0.5 uppercase tracking-wide rounded-full bg-ui-bg-base border border-ui-border-base text-ui-fg-subtle">
              {tier}
            </span>
            <EllipsisHorizontal className="text-ui-fg-muted size-3.5" />
          </div>
        </DropdownMenu.Trigger>
        {isLoaded && (
          <DropdownMenu.Content className="w-[244px] rounded-xl p-1.5 shadow-2xl border border-ui-border-base bg-ui-bg-base/95 backdrop-blur-xl">
            <div className="flex items-center gap-x-3 px-2.5 py-2">
              <div className="size-8 rounded-lg bg-ui-bg-subtle border border-ui-border-base flex items-center justify-center overflow-hidden shrink-0 shadow-xs">
                <Avatar variant="squared" size="small" src="/brand/vendryx/vendryx-mark.svg" fallback={fallback} />
              </div>
              <div className="flex flex-col overflow-hidden min-w-0">
                <Text
                  size="small"
                  weight="plus"
                  leading="compact"
                  className="truncate font-semibold text-[13px] text-ui-fg-base"
                >
                  {name}
                </Text>
                <div className="flex items-center gap-x-1.5 mt-0.5">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 shrink-0" />
                  <Text
                    size="xsmall"
                    leading="compact"
                    className="text-ui-fg-muted text-[11px]"
                  >
                    Merchant Admin · Active
                  </Text>
                </div>
              </div>
            </div>
            <DropdownMenu.Separator className="my-1 border-ui-border-base/60" />
            <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" asChild>
              <a href={storefrontUrl} target="_blank" rel="noopener noreferrer">
                <OpenRectArrowOut className="text-ui-fg-muted size-4" />
                <span>View Storefront</span>
              </a>
            </DropdownMenu.Item>
            <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" asChild>
              <Link to="/settings/store">
                <BuildingStorefront className="text-ui-fg-muted size-4" />
                <span>{t("app.nav.main.storeSettings", "Store Settings")}</span>
              </Link>
            </DropdownMenu.Item>
            <DropdownMenu.Separator className="my-1 border-ui-border-base/60" />
            <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" asChild>
              <Link to="/help">
                <DocumentText className="text-ui-fg-muted size-4" />
                <span>Help &amp; Guides</span>
              </Link>
            </DropdownMenu.Item>
            <DropdownMenu.Item className="gap-x-2.5 rounded-lg py-1.5 px-2.5 text-[13px]" asChild>
              <Link to="/release-notes">
                <Sparkles className="text-ui-fg-muted size-4" />
                <span>Release Notes</span>
              </Link>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        )}
      </DropdownMenu>
    </div>
  )
}

type NavSection = {
  id: string
  title?: string
  routes: (Omit<INavItem, "pathname">)[]
}

const useCoreSections = (): NavSection[] => {
  const { t } = useTranslation()

  return [
    {
      id: "workspace",
      routes: [
        {
          icon: <SquaresPlus />,
          label: "Overview",
          to: "/",
        },
      ],
    },
    {
      id: "commerce",
      title: "Commerce",
      routes: [
        {
          icon: <ShoppingCart />,
          label: t("orders.domain", "Orders"),
          to: "/orders",
          items: [
            {
              label: "Draft Orders",
              to: "/orders?status=draft",
            },
            {
              label: "Social Commerce",
              to: "/orders/social",
            },
          ],
        },
        {
          icon: <Users />,
          label: t("customers.domain", "Customers"),
          to: "/customers",
          items: [
            {
              label: t("customerGroups.domain", "Customer Groups"),
              to: "/customer-groups",
            },
          ],
        },
      ],
    },
    {
      id: "catalog",
      title: "Catalog",
      routes: [
        {
          icon: <Tag />,
          label: t("products.domain", "Products"),
          to: "/products",
          items: [
            {
              label: t("categories.domain", "Categories"),
              to: "/categories",
            },
            {
              label: t("collections.domain", "Collections"),
              to: "/collections",
            },
            {
              label: t("productOptions.domain", "Options"),
              to: "/product-options",
            },
          ],
        },
        {
          icon: <Buildings />,
          label: t("inventory.domain", "Inventory"),
          to: "/inventory",
          items: [
            {
              label: t("reservations.domain", "Reservations"),
              to: "/reservations",
            },
          ],
        },
      ],
    },
    {
      id: "marketing",
      title: "Marketing & Pricing",
      routes: [
        {
          icon: <ReceiptPercent />,
          label: t("promotions.domain", "Promotions"),
          to: "/promotions",
          items: [
            {
              label: t("campaigns.domain", "Campaigns"),
              to: "/campaigns",
            },
          ],
        },
        {
          icon: <CurrencyDollar />,
          label: t("priceLists.domain", "Price Lists"),
          to: "/price-lists",
        },
      ],
    },
    {
      id: "operations",
      title: "Operations & Finance",
      routes: [
        {
          icon: <ChartPie />,
          label: "Analytics",
          to: "/analytics",
        },
        {
          icon: <CreditCard />,
          label: "COD Reconciliation",
          to: "/reconciliation",
        },
      ],
    },
  ]
}

const Searchbar = () => {
  const { t } = useTranslation()
  const { toggleSearch } = useSearch()

  return (
    <div className="mb-2">
      <button
        onClick={toggleSearch}
        type="button"
        className={clx(
          "bg-black/[0.03] dark:bg-white/[0.05] border border-ui-border-base/80 hover:border-ui-border-strong",
          "text-ui-fg-muted hover:text-ui-fg-subtle flex h-8 w-full items-center gap-x-2 rounded-lg px-2.5 outline-none transition-all duration-150",
          "focus-visible:shadow-borders-focus cursor-pointer"
        )}
      >
        <MagnifyingGlass className="size-3.5 text-ui-fg-muted shrink-0" />
        <span className="flex-1 text-start text-[12px] font-normal text-ui-fg-muted">
          {t("app.search.label", "Search...")}
        </span>
        <Kbd className="text-[10px] px-1.5 py-0.5 text-ui-fg-muted bg-ui-bg-subtle border-ui-border-base shadow-2xs font-mono">
          ⌘K
        </Kbd>
      </button>
    </div>
  )
}

const SidebarRoutes = () => {
  const sections = useCoreSections()
  const { getMenu } = useExtension()

  const menuItems = getMenu("coreExtensions")

  menuItems.forEach((item) => {
    if (item.nested) {
      for (const section of sections) {
        const route = section.routes.find((r) => r.to === item.nested)
        if (route) {
          route.items?.push(item)
          break
        }
      }
    }
  })

  const extensionItems = menuItems.filter((item) => !item.nested)

  return (
    <nav className="py-1.5">
      <div className="px-3">
        <LayoutComposer
          widgetsZonePrefix="sidebar"
          preferredLayoutId={CORE_LAYOUT_IDS.SINGLE_COLUMN}
          hasOutlet={false}
          disableWidgets
          customizeId={CUSTOMIZE_IDS.MAIN_SIDEBAR}
          controlSize="small"
          layoutProps={{
            className: "space-y-1.5",
          }}
          sections={{
            main: (
              <>
                <LayoutComposer.Entry id="Searchbar">
                  <Searchbar />
                </LayoutComposer.Entry>

                {sections.map((section) => (
                  <div key={section.id} className="space-y-0.5">
                    {section.title && (
                      <div className="px-2.5 pt-2 pb-0.5 text-[10px] font-semibold tracking-wider text-ui-fg-muted/70 uppercase select-none">
                        {section.title}
                      </div>
                    )}
                    {section.routes.map((route) => (
                      <LayoutComposer.Entry id={`nav:${route.to}`} key={route.to}>
                        <NavItem key={route.to} {...route} />
                      </LayoutComposer.Entry>
                    ))}
                  </div>
                ))}

                {extensionItems.length > 0 && (
                  <div className="space-y-0.5">
                    <div className="px-2.5 pt-2 pb-0.5 text-[10px] font-semibold tracking-wider text-ui-fg-muted/70 uppercase select-none">
                      Extensions
                    </div>
                    {extensionItems.map((item) => (
                      <LayoutComposer.Entry id={`nav:${item.to}`} key={item.to}>
                        <NavItem
                          key={item.to}
                          to={item.to}
                          label={item.label}
                          icon={item.icon ? item.icon : <SquaresPlus />}
                          items={item.items}
                          translationNs={item.translationNs}
                          type="extension"
                        />
                      </LayoutComposer.Entry>
                    ))}
                  </div>
                )}
              </>
            ),
          }}
        />
      </div>
    </nav>
  )
}

const UtilitySection = () => {
  const location = useLocation()
  const { t } = useTranslation()

  return (
    <div className="flex flex-col px-3 py-2 border-t border-ui-border-base/50">
      <NavItem
        label={t("app.nav.settings.header", "Settings")}
        to="/settings"
        from={location.pathname}
        icon={<CogSixTooth />}
      />
    </div>
  )
}

const UserSection = () => {
  return (
    <div className="border-t border-ui-border-base/60 bg-[var(--vx-bg-sidebar,#F7F3ED)] dark:bg-[#141416]">
      <UserMenu />
    </div>
  )
}
