import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Backend axum local (jobrabbit --web). Em dev, o Vite faz proxy de /api.
const API = process.env.VITE_API_BASE || "http://localhost:8787";

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      output: {
        // Rolldown (Vite 8) replaces object-form manualChunks with codeSplitting.
        // Each regex also captures the library's exclusive dependency family so
        // the split matches the old chunks; higher priority wins.
        codeSplitting: {
          groups: [
            {
              name: "vendor-motion",
              test: /node_modules[\\/](framer-motion|motion-dom|motion-utils)[\\/]/,
              priority: 20,
            },
            {
              name: "vendor-query",
              test: /node_modules[\\/]@tanstack[\\/]/,
              priority: 20,
            },
            {
              name: "vendor-markdown",
              test: /node_modules[\\/](react-markdown|remark-|rehype-|micromark|mdast-|unist-|unified|hast-|vfile|estree-util|character-entities|decode-named-character-reference|property-information|space-separated-tokens|comma-separated-tokens|trim-lines|html-url-attributes|style-to-object|inline-style-parser|bail|trough|zwitch|devlop|is-plain-obj|longest-streak|markdown-table|ccount)/,
              priority: 20,
            },
            {
              name: "vendor",
              test: /node_modules[\\/](react|react-dom|scheduler)[\\/]/,
              priority: 10,
            },
          ],
        },
      },
    },
  },
  server: {
    host: "0.0.0.0",
    port: 5173,
    proxy: {
      "/api": { target: API, changeOrigin: true, ws: false },
    },
  },
});
