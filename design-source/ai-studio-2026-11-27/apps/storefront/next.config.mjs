/** @type {import('next').NextConfig} */
const nextConfig = {
  // Strip X-Powered-By: Next.js HTTP header (security hygiene).
  // Free-plan visible "Powered by Vendryx" attribution is enforced separately by storefront-core.
  poweredByHeader: false,
  reactStrictMode: true,
  images: {
    // Serve verified static/tenant media directly without exposing shared /_next/image CPU cache DoS vector.
    unoptimized: true,
  },
  async headers() {
    return [
      {
        source: "/(.*)",
        headers: [
          { key: "Vary", value: "Host" },
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
        ],
      },
    ]
  },
}

export default nextConfig
