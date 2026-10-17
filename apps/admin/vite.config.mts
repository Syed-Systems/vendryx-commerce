import react from "@vitejs/plugin-react"
import { defineConfig, loadEnv, Plugin } from "vite"

function virtualMedusaPlugin(): Plugin {
  const virtualModules: Record<string, string> = {
    "virtual:medusa/displays": "export default { displays: {} };",
    "virtual:medusa/forms": "export default { customFields: {} };",
    "virtual:medusa/i18n": "export default { resources: {} };",
    "virtual:medusa/layouts": "export default { layouts: [] };",
    "virtual:medusa/menu-items": "export default { menuItems: [] };",
    "virtual:medusa/routes": "export default { routes: [] };",
    "virtual:medusa/widgets": "export default { widgets: [] };",
    "virtual:medusa/cell-renderers": "export default {};",
    "virtual:medusa/search-entities": "export default {};",
    "virtual:medusa/links": "export default { links: [] };",
  }

  return {
    name: "vite-plugin-virtual-medusa",
    enforce: "pre",
    resolveId(id) {
      if (id in virtualModules) {
        return "\0" + id
      }
    },
    load(id) {
      if (id.startsWith("\0virtual:medusa/")) {
        const key = id.slice(1)
        return virtualModules[key] || "export default {};"
      }
    },
  }
}

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "")

  const BASE = env.VITE_STHENO_BASE || "/"
  // Defaults to "" so the Medusa SDK issues same-origin requests and the proxy
  // below forwards them. That is load-bearing, not cosmetic: the engine resolves
  // the merchant from the Host header, and the proxy rewrites it from the
  // `stheno_tenant_host` cookie that the tenant picker sets. Calling the engine
  // directly meant every request arrived as Host: 127.0.0.1 -- always ten_pro --
  // so choosing Trial or Free signed in against the wrong tenant and returned
  // "Invalid credentials". Same-origin also removes the cross-origin dependency,
  // so the engine's CORS allowlist need not know this app's port.
  //
  // Empty rather than "/" on purpose: baseUrl "/" would join as "//auth/...".
  const BACKEND_URL = env.VITE_STHENO_BACKEND_URL ?? ""
  const STOREFRONT_URL = env.VITE_STHENO_STOREFRONT_URL || "http://localhost:8000"

  return {
    plugins: [
      virtualMedusaPlugin(),
      react(),
    ],
    define: {
      __BASE__: JSON.stringify(BASE),
      __BACKEND_URL__: JSON.stringify(BACKEND_URL),
      __STOREFRONT_URL__: JSON.stringify(STOREFRONT_URL),
      __AUTH_TYPE__: JSON.stringify("jwt"),
      __JWT_TOKEN_STORAGE_KEY__: JSON.stringify("medusa_auth_token"),
      __MAX_UPLOAD_FILE_SIZE__: JSON.stringify(10485760),
    },
    server: {
      port: 5173,
      host: "127.0.0.1",
      proxy: {
        "/auth": {
          target: "http://127.0.0.1:9000",
          changeOrigin: true,
          configure: (proxy) => {
            proxy.on("proxyReq", (proxyReq, req) => {
              const cookies = req.headers.cookie || ""
              const match = cookies.match(/stheno_tenant_host=([^;]+)/)
              const customHost =
                (req.headers["x-stheno-tenant-host"] as string) ||
                (match ? decodeURIComponent(match[1].trim()) : null)
              if (customHost) {
                proxyReq.setHeader("host", customHost)
              }
            })
          },
        },
        "/admin": {
          target: "http://127.0.0.1:9000",
          changeOrigin: true,
          configure: (proxy) => {
            proxy.on("proxyReq", (proxyReq, req) => {
              const cookies = req.headers.cookie || ""
              const match = cookies.match(/stheno_tenant_host=([^;]+)/)
              const customHost =
                (req.headers["x-stheno-tenant-host"] as string) ||
                (match ? decodeURIComponent(match[1].trim()) : null)
              if (customHost) {
                proxyReq.setHeader("host", customHost)
              }
            })
          },
        },
      },
    },
  }
})
