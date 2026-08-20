// Routing and region resolution are core, per ADR-013 decision 4. Next requires
// middleware at the app root, so the app shell re-exports core's implementation
// rather than owning one.
export { middleware, config } from "@stheno/storefront-core/middleware"
