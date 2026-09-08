const checkEnvVariables = require("./check-env-variables")
const path = require("path")

checkEnvVariables()

/**
 * P27-08 — which frontend this process binds.
 *
 * ADR-013 keeps the binding to one seam. tsconfig's `@frontend/*` path is that
 * seam for typecheck; this is the same seam for the bundler, made switchable so
 * two frontends can be served from one tree at the same time (different port,
 * different distDir). One process still binds exactly one frontend.
 */
const FRONTEND = process.env.STHENO_FRONTEND || "default"
const FRONTEND_MODULES = path.resolve(__dirname, `packages/frontend-${FRONTEND}/src/modules`)
const CORE = path.resolve(__dirname, "packages/storefront-core/src")

// Turbopack does not read the `webpack` hook below, so it would silently serve
// a different frontend under a themed build. Refuse rather than lie about the frontend.
if (FRONTEND !== "default" && process.env.TURBOPACK) {
  throw new Error(
    `STHENO_FRONTEND=${FRONTEND} is bound through the webpack resolver; run without --turbopack.`
  )
}

/**
 * Object-storage host allowed for <Image> optimization of merchant media.
 */
const S3_HOSTNAME = process.env.STHENO_S3_HOSTNAME
const S3_PATHNAME = process.env.STHENO_S3_PATHNAME

/**
 * @type {import('next').NextConfig}
 */
const nextConfig = {
  reactStrictMode: true,
  // ADR-013: core and frontend are workspace source packages, not built artifacts.
  transpilePackages: ["@stheno/storefront-core", `@stheno/frontend-${FRONTEND}`],
  // Parallel frontends share the source tree; they must not share the build dir.
  distDir: FRONTEND === "default" ? ".next" : `.next-${FRONTEND}`,
  webpack: (config, { webpack }) => {
    // Rewrite the request rather than aliasing it: Next registers its own
    // tsconfig-paths resolver on the same hook as resolve.alias and wins, which
    // silently served frontend-default out of a matrix build. This runs before
    // resolution, so the tsconfig mapping never gets a say.
    config.plugins.push(
      new webpack.NormalModuleReplacementPlugin(/^@(frontend|modules)\//, (r) => {
        const rest = r.request.slice(r.request.indexOf("/") + 1)
        // ADR-013 §4: checkout and routing are core's, whatever the frontend is.
        const isCore =
          r.request.startsWith("@modules/") &&
          (rest.startsWith("checkout/") ||
            rest === "common/components/localized-client-link")
        r.request = path.join(
          isCore ? path.join(CORE, "modules") : FRONTEND_MODULES,
          rest
        )
      })
    )
    return config
  },
  // Development only. In production this writes a line per server-side fetch on every
  // render — and managed log ingestion is billed per GB, so it scales with traffic rather
  // than with anything useful. `fullUrl` also puts query strings into log storage.
  // `next build` sets NODE_ENV=production, so a production build simply omits the key.
  ...(process.env.NODE_ENV !== "production" && {
    logging: { fetches: { fullUrl: true } },
  }),
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  images: {
    remotePatterns: [
      {
        protocol: "http",
        hostname: "localhost",
      },
      {
        protocol: "https",
        hostname: "images.unsplash.com",
      },
      {
        protocol: "https",
        hostname: "plus.unsplash.com",
      },
      // Medusa's demo and test buckets were listed here by the fork. They served no
      // merchant image and allowed <Image> to load from buckets we do not control.
      // Removed — a remote pattern is an allowlist, and dead entries only widen it.
      {
        protocol: "https",
        hostname: "**.amazonaws.com",
      },
      ...(S3_HOSTNAME && S3_PATHNAME
        ? [
            {
              protocol: "https",
              hostname: S3_HOSTNAME,
              pathname: S3_PATHNAME,
            },
          ]
        : []),
    ],
  },
}

module.exports = nextConfig
