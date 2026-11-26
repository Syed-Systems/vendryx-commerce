import { HttpTypes } from "@medusajs/types"
import { TFunction } from "i18next"
import { useProducts } from "../../../../../hooks/api/products"
import {
  createTableAdapter,
  TableAdapter,
} from "../../../../../lib/table/table-adapters"
import { useMemo } from "react"
import { ProductActions } from "./product-list-table-actions"

// eslint-disable-next-line max-len
export function createProductTableAdapter({
  t,
}: {
  t: TFunction<"translation", undefined>
}): TableAdapter<HttpTypes.AdminProduct> {
  return createTableAdapter<HttpTypes.AdminProduct>({
    entity: "products",
    queryPrefix: "p",
    pageSize: 20,
    useData: (fields, params) => {
      const { products, count, isError, error, isLoading } = useProducts(
        {
          fields,
          ...params,
          is_giftcard: false, // Exclude gift cards from product list
        },
        {
          placeholderData: (previousData, previousQuery: any) => {
            // Only keep placeholder data if the fields haven't changed
            const prevFields =
              previousQuery?.[previousQuery?.length - 1]?.query?.fields
            if (prevFields && prevFields !== fields) {
              // Fields changed, don't use placeholder data
              return undefined
            }
            // Fields are the same, keep previous data for smooth transitions
            return previousData
          },
        }
      )
      return { data: products, count, isLoading, isError, error }
    },
    getRowHref: (row) => `/products/${row.id}`,
    renderRowActions: (row) => <ProductActions product={row} />,
    emptyState: {
      empty: {
        heading: t("products.list.noRecordsMessage"),
      },
      // Without this, a search matching nothing showed "Create your first
      // product to start selling" -- the empty-store message -- instead of
      // "no results", so a filtered-empty list looked like an empty store.
      filtered: {
        heading: t("general.noResultsTitle"),
        description: t("general.noResultsMessage"),
      },
    },
    transformColumns: (columns) => {
      const ALLOWED_FILTERS = [
        "id",
        "title",
        "handle",
        "status",
        "external_id",
        "created_at",
        "updated_at",
        "deleted_at",
        "collection.id",
        "type.id",
        "tags.id",
        "categories.id",
        "sales_channels.id",
        "variants.id",
      ]

      // Default visibility. The backend ships Product, Status and Actions
      // visible and hides the rest, so a fresh Products page shows two columns
      // and a merchant cannot tell price, stock or collection at a glance. The
      // reference dashboards (Ghost, Linear) lead with the densest table that
      // still reads, so the two highest-value hidden columns come forward.
      // Sales Channels stays hidden: most Vendryx merchants run one channel, and
      // surfacing it by default adds a column to every screen.
      const DEFAULT_VISIBLE = new Set(["product", "collection", "variants", "status"])

      return columns.map((column) => {
        const isFilterDisabled = !ALLOWED_FILTERS.includes(column.field)

        return {
          ...column,
          default_visible: DEFAULT_VISIBLE.has(column.field),
          filter: isFilterDisabled
            ? { ...column.filter, enabled: false }
            : column.filter,
        }
      })
    },
  })
}

/**
 * Hook to get the product table adapter with filters
 */
export function useProductTableAdapter({
  t,
}: {
  t: TFunction<"translation", undefined>
}): TableAdapter<HttpTypes.AdminProduct> {
  return useMemo(() => createProductTableAdapter({ t }), [t])
}
