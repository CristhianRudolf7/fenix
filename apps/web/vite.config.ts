import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
const pdfRoot = dirname(
  createRequire(import.meta.url).resolve("pdfjs-dist/package.json"),
);
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    {
      name: "fenix-loading-mark",
      transformIndexHtml(html) {
        return html.replace(
          "<!-- fenix-loading-mark -->",
          readFileSync(
            new URL("./src/assets/favicon.svg", import.meta.url),
            "utf8",
          ),
        );
      },
    },
    {
      name: "fenix-pdf-assets",
      generateBundle() {
        this.emitFile({
          type: "asset",
          fileName: "assets/pdfjs/LICENSE",
          source: readFileSync(join(pdfRoot, "LICENSE")),
        });
        for (const directory of ["cmaps", "standard_fonts", "wasm", "iccs"]) {
          for (const file of readdirSync(join(pdfRoot, directory))) {
            if (file.startsWith("quickjs-")) continue; // No embedded PDF scripting runtime.
            this.emitFile({
              type: "asset",
              fileName: `assets/pdfjs/${directory}/${file}`,
              source: readFileSync(join(pdfRoot, directory, file)),
            });
          }
        }
      },
    },
  ],
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:4173",
        changeOrigin: true,
        configure(proxy) {
          proxy.on("proxyReq", (request) => request.removeHeader("origin"));
        },
      },
    },
  },
  build: { outDir: "dist", emptyOutDir: true },
});
