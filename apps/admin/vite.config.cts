import { defineConfig, Plugin } from "vite"
import react from "@vitejs/plugin-react"
import { createMockApiState, handleMockApiRequest } from "./mock-api-handler.mjs"

const VIRTUAL_MEDUSA_MODULES: Record<string, string> = {
  "virtual:medusa/displays": "export default { displays: {} }",
  "virtual:medusa/forms": "export default { customFields: {} }",
  "virtual:medusa/i18n": "export default { resources: {} }",
  "virtual:medusa/layouts": "export default { layouts: [] }",
  "virtual:medusa/links": "export default { links: {} }",
  "virtual:medusa/menu-items": "export default { menuItems: [] }",
  "virtual:medusa/routes": "export default { routes: [] }",
  "virtual:medusa/widgets": "export default { widgets: [] }",
  "virtual:medusa/cell-renderers": "export {}",
  "virtual:medusa/search-entities": "export {}",
}

function medusaVirtualModulesPlugin(): Plugin {
  return {
    name: "stheno-medusa-virtual-modules",
    resolveId(id) {
      if (id in VIRTUAL_MEDUSA_MODULES) {
        return `\0${id}`
      }
      return null
    },
    load(id) {
      if (id.startsWith("\0virtual:medusa/")) {
        const key = id.slice(1)
        return VIRTUAL_MEDUSA_MODULES[key] ?? "export default {}"
      }
      return null
    },
  }
}

function readJsonBody(req: any): Promise<any> {
  return new Promise((resolve) => {
    const chunks: Buffer[] = []
    req.on("data", (chunk: Buffer) => chunks.push(chunk))
    req.on("end", () => {
      if (chunks.length === 0) {
        resolve(null)
        return
      }
      try {
        resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")))
      } catch {
        resolve(null)
      }
    })
    req.on("error", () => resolve(null))
  })
}

function sthenoMockApiPlugin(): Plugin {
  const state = createMockApiState()
  const attachMiddleware = (middlewares: any) => {
    middlewares.use(async (req: any, res: any, next: any) => {
      const rawUrl = req.url || "/"
      if (!rawUrl.startsWith("/admin/") && !rawUrl.startsWith("/auth/") && rawUrl !== "/admin") {
        return next()
      }

      const parsed = new URL(rawUrl, "http://localhost:3000")
      const body =
        req.method && !["GET", "HEAD"].includes(req.method.toUpperCase())
          ? await readJsonBody(req)
          : null

      const result = handleMockApiRequest(state, {
        method: req.method || "GET",
        pathname: parsed.pathname,
        searchParams: parsed.searchParams,
        body,
      })

      res.statusCode = result.status
      res.setHeader("Content-Type", "application/json")
      res.end(JSON.stringify(result.body))
    })
  }

  return {
    name: "stheno-mock-api",
    configureServer(server) {
      attachMiddleware(server.middlewares)
    },
    configurePreviewServer(server) {
      attachMiddleware(server.middlewares)
    },
  }
}

const backendUrl = process.env.VITE_STHENO_BACKEND_URL || ""
const storefrontUrl = process.env.VITE_STHENO_STOREFRONT_URL || "http://localhost:8000"

export default defineConfig({
  plugins: [
    react(),
    medusaVirtualModulesPlugin(),
    ...(backendUrl ? [] : [sthenoMockApiPlugin()]),
  ],
  define: {
    __BACKEND_URL__: JSON.stringify(backendUrl),
    __AUTH_TYPE__: JSON.stringify("jwt"),
    __JWT_TOKEN_STORAGE_KEY__: JSON.stringify("medusa_auth_token"),
    __STOREFRONT_URL__: JSON.stringify(storefrontUrl),
    __BASE__: JSON.stringify("/"),
    __MAX_UPLOAD_FILE_SIZE__: JSON.stringify(10 * 1024 * 1024),
  },
  server: {
    host: "0.0.0.0",
    port: 3000,
    strictPort: true,
    allowedHosts: true,
    hmr: process.env.DISABLE_HMR !== "true",
    proxy: backendUrl
      ? {
          "/admin": { target: backendUrl, changeOrigin: true },
          "/auth": { target: backendUrl, changeOrigin: true },
        }
      : undefined,
  },
  preview: {
    host: "0.0.0.0",
    port: 3000,
    strictPort: true,
    allowedHosts: true,
  },
  optimizeDeps: {
    entries: ["index.html"],
  },
})
