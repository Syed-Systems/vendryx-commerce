// Route tree lives in @stheno/storefront-core, per ADR-013 decision 4.
// Next requires route files on disk in the app directory, so the app shell
// re-exports core's implementation. This file is a manifest entry, not logic.
export * from "@core/app/[countryCode]/(main)/layout"
export { default } from "@core/app/[countryCode]/(main)/layout"
