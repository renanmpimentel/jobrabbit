import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Backend axum local (jobrabbit --web). Em dev, o Vite faz proxy de /api.
const API = process.env.VITE_API_BASE || "http://localhost:8787";

export default defineConfig({
  plugins: [react()],
  build: { outDir: "dist", emptyOutDir: true },
  server: {
    host: "0.0.0.0",
    port: 5173,
    proxy: {
      "/api": { target: API, changeOrigin: true, ws: false },
    },
  },
});
