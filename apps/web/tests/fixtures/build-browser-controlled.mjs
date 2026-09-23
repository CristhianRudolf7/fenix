import { build } from "vite";
import { fileURLToPath } from "node:url";
await build({
  configFile: false,
  define: { "process.env.NODE_ENV": JSON.stringify("production") },
  build: {
    outDir: fileURLToPath(
      new URL("../../../../target/browser-fixtures", import.meta.url),
    ),
    emptyOutDir: false,
    lib: {
      entry: fileURLToPath(
        new URL("./browser-controlled.tsx", import.meta.url),
      ),
      name: "BrowserControlledFixture",
      formats: ["iife"],
      fileName: () => "controlled.js",
    },
  },
});
