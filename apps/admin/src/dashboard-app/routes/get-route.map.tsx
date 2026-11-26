import { HttpTypes } from "@medusajs/types"
import { t } from "i18next"
import { Navigate, Outlet, RouteObject, UIMatch } from "react-router-dom"
import { ProtectedRoute } from "../../components/authentication/protected-route"
import { RoutePermissionGuard } from "../../components/authentication/route-permission-guard"
import { MainLayout } from "../../components/layout/main-layout"
import { PublicLayout } from "../../components/layout/public-layout"
import { SettingsLayout } from "../../components/layout/settings-layout"
import { ErrorBoundary } from "../../components/utilities/error-boundary"
import { HydrateFallback } from "../../components/utilities/hydrate-fallback"

const MERCHANT_WORKSPACE_ROUTES = new Set([
  "/",
  "/analytics",
  "/orders",
  "/orders/social",
  "/products",
  "/collections",
  "/categories",
  "/inventory",
  "/customers",
  "/promotions",
  "/price-lists",
  "/profile",
  "/help",
  "/release-notes",
  "/onboarding",
  "/campaigns",
  "/customer-groups",
  "/reservations",
  "/product-options",
  "/product-variants",
  "/reconciliation",
])

const MERCHANT_SETTINGS_ROUTES = new Set([
  "",
  "profile",
  "branding",
  "store",
  "users",
  "billing",
  "domains",
  "couriers",
  "help",
  "release-notes",
  "regions",
  "payment-providers",
  "tax-regions",
  "return-reasons",
  "refund-reasons",
  "sales-channels",
  "product-types",
  "product-tags",
  "locations",
  "property-labels",
  "translations",
  "publishable-api-keys",
  "secret-api-keys",
  "workflows",
  "roles",
  "policies",
  "shipping-option-types",
  "shipping-profiles",
  "api-key-management",
])


/**
 * The Medusa fork remains the implementation underneath this map, but a small merchant is not
 * offered every inherited control plane. This filtering happens before the browser router is
 * created, which means excluded URLs are genuinely unmounted and resolve through the public 404.
 * Entitlement-specific Pro route mounting still happens in the backend route composer.
 */
export function curateMerchantWorkspace(routes: RouteObject[]): RouteObject[] {
  const mainChildren = routes[0]?.children?.[0]?.children
  if (mainChildren) {
    routes[0] = {
      ...routes[0],
      children: [
        {
          ...routes[0].children![0],
          children: mainChildren.filter(
            (route) =>
              typeof route.path !== "string" ||
              !route.path.startsWith("/") ||
              MERCHANT_WORKSPACE_ROUTES.has(route.path)
          ),
        },
      ],
    } as RouteObject
  }

  const settings = routes[1]?.children?.[0]
  if (settings?.path === "/settings" && settings.children) {
    routes[1] = {
      ...routes[1],
      children: [
        {
          ...settings,
          children: settings.children.filter(
            (route) => route.index === true || MERCHANT_SETTINGS_ROUTES.has(route.path ?? "")
          ),
        },
      ],
    } as RouteObject
  }

  return routes
}

export function getRouteMap({
  settingsRoutes,
  coreRoutes,
}: {
  settingsRoutes: RouteObject[]
  coreRoutes: RouteObject[]
}) {
  return [
    {
      element: <ProtectedRoute />,
      errorElement: <ErrorBoundary />,
      hydrateFallbackElement: <HydrateFallback />,
      children: [
        {
          element: <MainLayout />,
          children: [
            {
              path: "/",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/home"),
            },
            {
              path: "/analytics",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/analytics"),
              handle: {
                breadcrumb: () => "Analytics & Reports",
              },
            },
            {
              path: "/onboarding",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("onboarding")),
              handle: {
                breadcrumb: () => "Store Setup Guide",
              },
            },
            {
              path: "/products",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("products.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                    },
                    {
                      path: "import",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                    },
                    {
                      path: "export",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                    },
                  ],
                },
                {
                  path: ":id",
                  errorElement: <ErrorBoundary />,
                  lazy: async () => {
                    const { Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products"))

                    return {
                      Component: Outlet,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminProductResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                      children: [
                        {
                          path: "edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "edit-variant",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-variants")),
                        },
                        {
                          path: "sales-channels",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "attributes",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "organization",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "shipping-profile",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "media",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "images/:image_id/variants",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "prices",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "variants/create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "stock",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "metadata/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "options/manage",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                      ],
                    },
                    {
                      path: "variants/:variant_id",
                      lazy: async () => {
                        const { Component, Breadcrumb, loader, seo } =
                          await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-variants"))

                        return {
                          Component,
                          loader,
                          handle: {
                            breadcrumb: (
                              // eslint-disable-next-line max-len
                              match: UIMatch<HttpTypes.AdminProductVariantResponse>
                            ) => <Breadcrumb {...match} />,
                            seo,
                          },
                        }
                      },
                      children: [
                        {
                          path: "edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-variants")),
                        },
                        {
                          path: "prices",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("products")),
                        },
                        {
                          path: "manage-items",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-variants")),
                        },
                        {
                          path: "media",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-variants")),
                        },
                        {
                          path: "metadata/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-variants")),
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "/product-options",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("productOptions.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options")),
                    },
                  ],
                },
                {
                  path: ":id",
                  errorElement: <ErrorBoundary />,
                  lazy: async () => {
                    const { Breadcrumb, loader } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options"))

                    return {
                      Component: Outlet,
                      loader,
                      handle: {
                        breadcrumb: (match: UIMatch<any>) => (
                          <Breadcrumb {...match} />
                        ),
                      },
                    }
                  },
                  children: [
                    {
                      path: "",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options")),
                      children: [
                        {
                          path: "edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options")),
                        },
                        {
                          path: "metadata/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options")),
                        },
                      ],
                    },
                    {
                      path: "values/:value_id",
                      lazy: async () => {
                        const { Component, Breadcrumb, loader } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options"))

                        return {
                          Component,
                          loader,
                          handle: {
                            breadcrumb: (
                              // eslint-disable-next-line max-len
                              match: UIMatch<HttpTypes.AdminProductOptionValueResponse>
                            ) => <Breadcrumb {...match} />,
                          },
                        }
                      },
                      children: [
                        {
                          path: "metadata/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-options")),
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "/categories",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("categories.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/categories/category-list"),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/categories/category-create"),
                    },
                    {
                      path: "organize",
                      lazy: () =>
                        import("../../routes/categories/category-organize"),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import(
                      "../../routes/categories/category-detail"
                    )

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminProductCategoryResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/categories/category-edit"),
                    },
                    {
                      path: "products",
                      lazy: () =>
                        import("../../routes/categories/category-products"),
                    },
                    {
                      path: "organize",
                      lazy: () =>
                        import("../../routes/categories/category-organize"),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/categories/categories-metadata"),
                    },
                  ],
                },
              ],
            },
            {
              path: "/orders",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("orders.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                  children: [
                    {
                      path: "export",
                      lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                  ],
                },
                {
                  path: "social",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                  handle: {
                    breadcrumb: () => "Social Commerce Orders",
                  },
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminOrderResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "fulfillment",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "returns/:return_id/receive",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "allocate-items",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: ":f_id/create-shipment",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "returns",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "claims",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "exchanges",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "edits",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "refund",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "transfer",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "email",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "shipping-address",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "billing-address",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                  ],
                },
              ],
            },
            {
              path: "/promotions",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("promotions.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("promotions")),
                },
                {
                  path: "create",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("promotions")),
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("promotions"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminPromotionResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("promotions")),
                    },
                    {
                      path: "add-to-campaign",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("promotions")),
                    },
                    {
                      path: ":ruleType/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("promotions")),
                    },
                  ],
                },
              ],
            },
            {
              path: "/campaigns",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("campaigns.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/campaigns/campaign-list"),
                  children: [],
                },
                {
                  path: "create",
                  lazy: () => import("../../routes/campaigns/campaign-create"),
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import(
                      "../../routes/campaigns/campaign-detail"
                    )

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminCampaignResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/campaigns/campaign-edit"),
                    },
                    {
                      path: "configuration",
                      lazy: () =>
                        import("../../routes/campaigns/campaign-configuration"),
                    },
                    {
                      path: "edit-budget",
                      lazy: () =>
                        import("../../routes/campaigns/campaign-budget-edit"),
                    },
                    {
                      path: "add-promotions",
                      lazy: () =>
                        import(
                          "../../routes/campaigns/add-campaign-promotions"
                        ),
                    },
                  ],
                },
              ],
            },
            {
              path: "/collections",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("collections.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/collections/collection-list"),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/collections/collection-create"),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import(
                      "../../routes/collections/collection-detail"
                    )

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminCollectionResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/collections/collection-edit"),
                    },
                    {
                      path: "products",
                      lazy: () =>
                        import(
                          "../../routes/collections/collection-add-products"
                        ),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/collections/collection-metadata"),
                    },
                  ],
                },
              ],
            },
            {
              path: "/price-lists",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("priceLists.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists")),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminPriceListResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists")),
                    },
                    {
                      path: "configuration",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists")),
                    },
                    {
                      path: "products/add",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists")),
                    },
                    {
                      path: "products/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("price-lists")),
                    },
                  ],
                },
              ],
            },
            {
              path: "/customers",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("customers.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/customers/customer-list"),
                  children: [
                    {
                      path: "create",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "customer:create" },
                      children: [
                        {
                          path: "",
                          lazy: () =>
                            import("../../routes/customers/customer-create"),
                        },
                      ],
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import(
                      "../../routes/customers/customer-detail"
                    )

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminCustomerResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "customer:update" },
                      children: [
                        {
                          path: "",
                          lazy: () =>
                            import("../../routes/customers/customer-edit"),
                        },
                      ],
                    },
                    {
                      path: "create-address",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "customer:update" },
                      children: [
                        {
                          path: "",
                          lazy: () =>
                            import(
                              "../../routes/customers/customer-create-address"
                            ),
                        },
                      ],
                    },
                    {
                      path: "add-customer-groups",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "customer:update" },
                      children: [
                        {
                          path: "",
                          lazy: () =>
                            import(
                              "../../routes/customers/customers-add-customer-group"
                            ),
                        },
                      ],
                    },
                    {
                      path: ":order_id/transfer",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("orders")),
                    },
                    {
                      path: "metadata/edit",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "customer:update" },
                      children: [
                        {
                          path: "",
                          lazy: () =>
                            import("../../routes/customers/customer-metadata"),
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "/customer-groups",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("customerGroups.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/customer-groups/customer-group-list"),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import(
                          "../../routes/customer-groups/customer-group-create"
                        ),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import(
                      "../../routes/customer-groups/customer-group-detail"
                    )

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminCustomerGroupResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import(
                          "../../routes/customer-groups/customer-group-edit"
                        ),
                    },
                    {
                      path: "add-customers",
                      lazy: () =>
                        import(
                          "../../routes/customer-groups/customer-group-add-customers"
                        ),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import(
                          "../../routes/customer-groups/customer-group-metadata"
                        ),
                    },
                  ],
                },
              ],
            },
            {
              path: "/reservations",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("reservations.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("reservations")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("reservations")),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("reservations"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminReservationResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("reservations")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("reservations")),
                    },
                  ],
                },
              ],
            },
            {
              path: "/inventory",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("inventory.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("inventory")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/inventory/inventory-create"),
                    },
                    {
                      path: "export",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("inventory")),
                    },
                    {
                      path: "stock",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("inventory")),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("inventory"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminInventoryItemResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import(
                          "../../routes/inventory/inventory-detail/components/edit-inventory-item"
                        ),
                    },
                    {
                      path: "attributes",
                      lazy: () =>
                        import(
                          "../../routes/inventory/inventory-detail/components/edit-inventory-item-attributes"
                        ),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("inventory")),
                    },
                    {
                      path: "locations",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("inventory")),
                    },
                    {
                      path: "locations/:location_id",
                      lazy: () =>
                        import(
                          "../../routes/inventory/inventory-detail/components/adjust-inventory"
                        ),
                    },
                  ],
                },
              ],
            },
            {
              path: "/help",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/help"),
              handle: {
                breadcrumb: () => "Help & Support",
              },
            },
            {
              path: "/release-notes",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("release-notes")),
              handle: {
                breadcrumb: () => "Release Notes",
              },
            },
            {
              path: "/profile",
              element: <Navigate to="/settings/profile" replace />,
            },
            {
              path: "/product-variants",
              element: <Navigate to="/products" replace />,
            },
            {
              path: "/reconciliation",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("reconciliation")),
              handle: {
                breadcrumb: () => "Courier COD Reconciliation",
              },
            },
            ...coreRoutes,
          ],
        },
      ],
    },
    {
      element: <ProtectedRoute />,
      errorElement: <ErrorBoundary />,
      hydrateFallbackElement: <HydrateFallback />,
      children: [
        {
          path: "/settings",
          handle: {
            breadcrumb: () => t("app.nav.settings.header"),
          },
          element: <SettingsLayout />,
          children: [
            {
              index: true,
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("settings")),
            },
            {
              path: "profile",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("profile")),
              handle: {
                breadcrumb: () => t("profile.domain"),
              },
              children: [
                {
                  path: "edit",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("profile")),
                },
              ],
            },
            {
              path: "ai",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/ai"),
              handle: {
                breadcrumb: () => "AI & Social Commerce",
              },
            },
            {
              path: "billing",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/billing"),
              handle: {
                breadcrumb: () => "Subscription & Billing",
              },
            },
            {
              path: "regions",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("regions.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("regions")),
                  children: [
                    {
                      path: "create",
                      lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("regions")),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("regions"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminRegionResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("regions")),
                    },
                    {
                      path: "countries/add",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("regions")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("regions")),
                    },
                  ],
                },
              ],
            },
            {
              path: "property-labels",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("propertyLabels.domain", "Property Labels"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("property-labels")),
                  children: [
                    {
                      path: ":entity/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("property-labels")),
                    },
                  ],
                },
              ],
            },
            {
              path: "branding",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/branding"),
              handle: {
                breadcrumb: () => "Branding & Appearance",
              },
            },
            {
              path: "store",
              errorElement: <ErrorBoundary />,
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("store")),
              handle: {
                breadcrumb: () => t("store.domain"),
              },
              children: [
                {
                  path: "edit",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("store")),
                },
                {
                  path: "currencies",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("store")),
                },
                {
                  path: "locales",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("store")),
                },
                {
                  path: "metadata/edit",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("store")),
                },
              ],
            },
            {
              path: "users",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("users.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("users")),
                  children: [
                    {
                      path: "invite",
                      element: <RoutePermissionGuard />,
                      handle: {
                        permissions: ["invite:create", "invite:read"],
                      },
                      children: [
                        {
                          path: "",
                          lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("users")),
                        },
                      ],
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("users"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminUserResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("users")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("users")),
                    },
                  ],
                },
              ],
            },
            {
              path: "roles",
              errorElement: <ErrorBoundary />,
              element: <RoutePermissionGuard />,
              handle: {
                breadcrumb: () => t("roles.domain"),
                permissions: "rbac_role:read",
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("roles")),
                  children: [
                    {
                      path: "create",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "rbac_role:create" },
                      children: [
                        {
                          path: "",
                          lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("roles")),
                        },
                      ],
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("roles"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminRbacRoleResponse>
                        ) => <Breadcrumb {...match} />,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "rbac_role:update" },
                      children: [
                        {
                          path: "",
                          lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("roles")),
                        },
                      ],
                    },
                    {
                      path: "add-users",
                      element: <RoutePermissionGuard />,
                      handle: {
                        permissions: ["user:update", "rbac_role:update"],
                      },
                      children: [
                        {
                          path: "",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("roles")),
                        },
                      ],
                    },
                    {
                      path: "permissions",
                      element: <RoutePermissionGuard />,
                      handle: { permissions: "rbac_role:update" },
                      children: [
                        {
                          path: "",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("roles")),
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "policies",
              errorElement: <ErrorBoundary />,
              element: <RoutePermissionGuard />,
              handle: {
                breadcrumb: () => t("policies.domain"),
                permissions: "rbac_policy:read",
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("policies")),
                  // TODO: V1: policy CRUD lives in code (`definePolicies`).Uncomment
                  // along with the matching list/detail action sites to
                  // re-enable dashboard CRUD.
                  // children: [
                  //   {
                  //     path: "create",
                  //     element: <RoutePermissionGuard />,
                  //     handle: { permissions: "rbac_policy:create" },
                  //     children: [
                  //       {
                  //         path: "",
                  //         lazy: () =>
                  //           import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("policies")),
                  //       },
                  //     ],
                  //   },
                  // ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("policies"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminRbacPolicyResponse>
                        ) => <Breadcrumb {...match} />,
                      },
                    }
                  },
                  // TODO: V1: `edit` child kept commented out — see note above.
                  // children: [
                  //   {
                  //     path: "edit",
                  //     element: <RoutePermissionGuard />,
                  //     handle: { permissions: "rbac_policy:update" },
                  //     children: [
                  //       {
                  //         path: "",
                  //         lazy: () =>
                  //           import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("policies")),
                  //       },
                  //     ],
                  //   },
                  // ],
                },
              ],
            },
            {
              path: "sales-channels",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("salesChannels.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("sales-channels")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("sales-channels")),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("sales-channels"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminSalesChannelResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("sales-channels")),
                    },
                    {
                      path: "add-products",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("sales-channels")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("sales-channels")),
                    },
                  ],
                },
              ],
            },
            {
              path: "locations",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("locations.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                },
                {
                  path: "create",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                },
                {
                  path: "shipping-profiles",
                  element: <Outlet />,
                  handle: {
                    breadcrumb: () => t("shippingProfile.domain"),
                  },
                  children: [
                    {
                      path: "",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-profiles")),
                      children: [
                        {
                          path: "create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-profiles")),
                        },
                      ],
                    },
                    {
                      path: ":shipping_profile_id",
                      lazy: async () => {
                        const { Component, Breadcrumb, loader, seo } =
                          await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-profiles"))

                        return {
                          Component,
                          loader,
                          handle: {
                            breadcrumb: (
                              // eslint-disable-next-line max-len
                              match: UIMatch<HttpTypes.AdminShippingProfileResponse>
                            ) => <Breadcrumb {...match} />,
                            seo,
                          },
                        }
                      },
                      children: [
                        {
                          path: "metadata/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-profiles")),
                        },
                      ],
                    },
                  ],
                },
                {
                  path: "shipping-option-types",
                  errorElement: <ErrorBoundary />,
                  element: <Outlet />,
                  handle: {
                    breadcrumb: () => t("shippingOptionTypes.domain"),
                  },
                  children: [
                    {
                      path: "",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-option-types")),
                      children: [
                        {
                          path: "create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-option-types")),
                        },
                      ],
                    },
                    {
                      path: ":id",
                      lazy: async () => {
                        const { Component, Breadcrumb, loader, seo } =
                          await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-option-types"))

                        return {
                          Component,
                          loader,
                          handle: {
                            breadcrumb: (
                              // eslint-disable-next-line max-len
                              match: UIMatch<HttpTypes.AdminShippingOptionTypeResponse>
                            ) => <Breadcrumb {...match} />,
                            seo,
                          },
                        }
                      },
                      children: [
                        {
                          path: "edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("shipping-option-types")),
                        },
                      ],
                    },
                  ],
                },
                {
                  path: ":location_id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminStockLocationResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                    },
                    {
                      path: "sales-channels",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                    },
                    {
                      path: "fulfillment-providers",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                    },
                    {
                      path: "fulfillment-set/:fset_id",
                      children: [
                        {
                          path: "service-zones/create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                        },
                        {
                          path: "service-zone/:zone_id",
                          children: [
                            {
                              path: "edit",
                              lazy: () =>
                                import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                            },
                            {
                              path: "areas",
                              lazy: () =>
                                import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                            },
                            {
                              path: "shipping-option",
                              children: [
                                {
                                  path: "create",
                                  lazy: () =>
                                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                                },
                                {
                                  path: ":so_id",
                                  children: [
                                    {
                                      path: "edit",
                                      lazy: () =>
                                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                                    },
                                    {
                                      path: "pricing",
                                      lazy: () =>
                                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("locations")),
                                    },
                                  ],
                                },
                              ],
                            },
                          ],
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "product-tags",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("productTags.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-tags")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-tags")),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-tags"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminProductTagResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-tags")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-tags")),
                    },
                  ],
                },
              ],
            },
            {
              path: "workflows",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("workflowExecutions.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("workflow-executions")),
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("workflow-executions"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          // eslint-disable-next-line max-len
                          match: UIMatch<HttpTypes.AdminWorkflowExecutionResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                },
              ],
            },
            {
              path: "product-types",
              errorElement: <ErrorBoundary />,
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("productTypes.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-types")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-types")),
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-types"))

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminProductTypeResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-types")),
                    },
                    {
                      path: "metadata/edit",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("product-types")),
                    },
                  ],
                },
              ],
            },
            {
              path: "api-key-management",
              element: <Navigate to="../publishable-api-keys" replace />,
            },
            {
              path: "publishable-api-keys",
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("apiKeyManagement.domain.publishable"),
              },
              children: [
                {
                  path: "",
                  element: <Outlet />,
                  children: [
                    {
                      path: "",
                      lazy: () =>
                        import(
                          "../../routes/api-key-management/api-key-management-list"
                        ),
                      children: [
                        {
                          path: "create",
                          lazy: () =>
                            import(
                              "../../routes/api-key-management/api-key-management-create"
                            ),
                        },
                      ],
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import(
                      "../../routes/api-key-management/api-key-management-detail"
                    )

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminApiKeyResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import(
                          "../../routes/api-key-management/api-key-management-edit"
                        ),
                    },
                    {
                      path: "sales-channels",
                      lazy: () =>
                        import(
                          "../../routes/api-key-management/api-key-management-sales-channels"
                        ),
                    },
                  ],
                },
              ],
            },
            {
              path: "secret-api-keys",
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("apiKeyManagement.domain.secret"),
              },
              children: [
                {
                  path: "",
                  element: <Outlet />,
                  children: [
                    {
                      path: "",
                      lazy: () =>
                        import(
                          "../../routes/api-key-management/api-key-management-list"
                        ),
                      children: [
                        {
                          path: "create",
                          lazy: () =>
                            import(
                              "../../routes/api-key-management/api-key-management-create"
                            ),
                        },
                      ],
                    },
                  ],
                },
                {
                  path: ":id",
                  lazy: async () => {
                    const { Component, Breadcrumb, loader, seo } = await import(
                      "../../routes/api-key-management/api-key-management-detail"
                    )

                    return {
                      Component,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminApiKeyResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "edit",
                      lazy: () =>
                        import(
                          "../../routes/api-key-management/api-key-management-edit"
                        ),
                    },
                  ],
                },
              ],
            },
            {
              path: "tax-regions",
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("taxRegions.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                    },
                  ],
                },
                {
                  path: ":id",
                  errorElement: <ErrorBoundary />,
                  lazy: async () => {
                    const { Breadcrumb, loader, seo } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions"))

                    return {
                      Component: Outlet,
                      loader,
                      handle: {
                        breadcrumb: (
                          match: UIMatch<HttpTypes.AdminTaxRegionResponse>
                        ) => <Breadcrumb {...match} />,
                        seo,
                      },
                    }
                  },
                  children: [
                    {
                      path: "",
                      lazy: async () => {
                        const { Component } = await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions"))

                        return {
                          Component,
                        }
                      },
                      children: [
                        {
                          path: "edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "provinces/create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "overrides/create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "overrides/:tax_rate_id/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "tax-rates/create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "tax-rates/:tax_rate_id/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                      ],
                    },
                    {
                      path: "provinces/:province_id",
                      lazy: async () => {
                        const { Component, Breadcrumb, loader, seo } =
                          await import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions"))

                        return {
                          Component,
                          loader,
                          handle: {
                            breadcrumb: (
                              match: UIMatch<HttpTypes.AdminTaxRegionResponse>
                            ) => <Breadcrumb {...match} />,
                            seo,
                          },
                        }
                      },
                      children: [
                        {
                          path: "tax-rates/create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "tax-rates/:tax_rate_id/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "overrides/create",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                        {
                          path: "overrides/:tax_rate_id/edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("tax-regions")),
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "payment-providers",
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("payment-providers")),
              handle: {
                breadcrumb: () => "Payment Providers",
              },
            },
            {
              path: "couriers",
              lazy: () => import("../../routes/couriers"),
              handle: {
                breadcrumb: () => "Courier Logistics",
              },
            },
            {
              path: "domains",
              lazy: () => import("../../routes/domains"),
              handle: {
                breadcrumb: () => "Domains & SSL",
              },
            },
            {
              path: "return-reasons",
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("returnReasons.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("return-reasons")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("return-reasons")),
                    },

                    {
                      path: ":id",
                      children: [
                        {
                          path: "edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("return-reasons")),
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "refund-reasons",
              element: <Outlet />,
              handle: {
                breadcrumb: () => t("refundReasons.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("refund-reasons")),
                  children: [
                    {
                      path: "create",
                      lazy: () =>
                        import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("refund-reasons")),
                    },

                    {
                      path: ":id",
                      children: [
                        {
                          path: "edit",
                          lazy: () =>
                            import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("refund-reasons")),
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              path: "translations",
              errorElement: <ErrorBoundary />,
              handle: {
                breadcrumb: () => t("translations.domain"),
              },
              children: [
                {
                  path: "",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("translations")),
                  children: [
                    {
                      path: "settings",
                      lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("translations")),
                    },
                  ],
                },
                {
                  path: "edit",
                  lazy: () =>
                    import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("translations")),
                },
                {
                  path: "add-locales",
                  lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("translations")),
                },
              ],
            },
            ...settingsRoutes.flatMap((r) => r?.children || []),
          ],
        },
      ],
    },
    {
      element: <PublicLayout />,
      children: [
        {
          errorElement: <ErrorBoundary />,
          children: [
            {
              path: "/login",
              lazy: () => import("../../routes/login"),
              handle: {
                breadcrumb: () => t("login.title"),
              },
            },
            {
              path: "/signup",
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("signup")),
              handle: {
                breadcrumb: () => "Sign up",
              },
            },
            {
              path: "/reset-password",
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("reset-password")),
              handle: {
                breadcrumb: () => t("resetPassword.title"),
              },
            },
            {
              path: "/invite",
              lazy: () => import("../../routes/unavailable-route").then((m) => m.createUnavailableRoute("invite")),
              handle: {
                breadcrumb: () => t("invite.title"),
              },
            },
            {
              path: "*",
              lazy: () => import("../../routes/no-match"),
            },
          ],
        },
      ],
    },
  ]
}
