import path from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { viteSingleFile } from "vite-plugin-singlefile";

/**
 * Builds the viewer as a fully self-contained HTML file (JS/CSS inlined).
 * The Rust `export_analysis_html` command include_str!'s the result and
 * substitutes the report JSON at export time.
 */
export default defineConfig({
  plugins: [react(), tailwindcss(), viteSingleFile()],
  clearScreen: false,
  build: {
    outDir: "dist-viewer",
    emptyOutDir: true,
    cssCodeSplit: false,
    assetsInlineLimit: 100_000_000,
    rollupOptions: {
      input: path.resolve(__dirname, "viewer.html"),
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
