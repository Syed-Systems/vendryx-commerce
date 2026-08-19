import react from "@vitejs/plugin-react"
import { defineConfig, loadEnv } from "vite"

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "")
  return {
    plugins: [react()],
    define: {
      __BACKEND_URL__: JSON.stringify(
        env.VITE_STHENO_BACKEND_URL || "http://127.0.0.1:9000"
      ),
    },
    server: { port: 5174 },
  }
})
