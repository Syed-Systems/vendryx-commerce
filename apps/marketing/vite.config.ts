import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

export default defineConfig({
  plugins: [react()],
  server: { port: 5175 },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          three: ["three", "@react-three/fiber", "@react-three/drei"],
          motion: ["motion"],
          icons: ["@phosphor-icons/react"],
        },
      },
    },
    chunkSizeWarningLimit: 900,
  },
})
