"use server"

import { sdk } from "@lib/config"
import medusaError from "@lib/util/medusa-error"
import { HttpTypes } from "@medusajs/types"
import { getCacheOptions } from "./cookies"

export const listRegions = async () => {
  const next = {
    ...(await getCacheOptions("regions")),
  }

  return sdk.client
    .fetch<{ regions: HttpTypes.StoreRegion[] }>(`/store/regions`, {
      method: "GET",
      next,
      cache: "force-cache",
    })
    .then(({ regions }) => regions)
    .catch(medusaError)
}

export const retrieveRegion = async (id: string) => {
  const next = {
    ...(await getCacheOptions(["regions", id].join("-"))),
  }

  return sdk.client
    .fetch<{ region: HttpTypes.StoreRegion }>(`/store/regions/${id}`, {
      method: "GET",
      next,
      cache: "force-cache",
    })
    .then(({ region }) => region)
    .catch(medusaError)
}

/**
 * Resolves the region serving a country code.
 *
 * There was a module-level `Map<countryCode, Region>` here, populated on first call and
 * never expired. Its key was the country code alone, so it held no notion of *whose*
 * regions it contained — and regions are tenant-owned (migration 20260819000006 made
 * `region.country` tenant-scoped precisely because they are not reference data). The
 * moment one storefront process serves more than one merchant (ADR-022), that map returns
 * one merchant's region — and its tenant-owned `region_id` — to another, which then flows
 * into cart creation through `getOrSetCart`.
 *
 * It is deleted rather than keyed by tenant, because it was never buying anything:
 * `listRegions()` is already `force-cache`d, so the map saved a cache read, not a network
 * call. Rebuilding the lookup per call over a handful of regions is not measurable, and it
 * cannot hold data across tenants because it does not outlive the call.
 */
export const getRegion = async (countryCode: string) => {
  try {
    const regions = await listRegions()

    if (!regions) {
      return null
    }

    const byCountry = new Map<string, HttpTypes.StoreRegion>()
    regions.forEach((region) => {
      region.countries?.forEach((c) => {
        byCountry.set(c?.iso_2 ?? "", region)
      })
    })

    return byCountry.get(countryCode || "us")
  } catch (e: any) {
    return null
  }
}
