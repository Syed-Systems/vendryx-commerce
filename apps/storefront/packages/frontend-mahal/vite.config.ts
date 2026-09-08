import { defineConfig } from "vite"

export default defineConfig({
  cacheDir: ".vite-cache",
  server: { port: 8003, host: "127.0.0.1" },
  preview: { port: 8003, host: "127.0.0.1" },
  build: { sourcemap: false },
})
