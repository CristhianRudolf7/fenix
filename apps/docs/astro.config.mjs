import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

export default defineConfig({
  site: "https://fenix.crehlabs.com",
  output: "static",
  compressHTML: false,
  trailingSlash: "always",
  security: {
    csp: {
      directives: [
        "default-src 'self'",
        "connect-src 'self' https://cloudflareinsights.com",
        "img-src 'self' data:",
        "object-src 'none'",
        "base-uri 'self'",
      ],
      styleDirective: {
        resources: [{ resource: "'unsafe-inline'", kind: "attribute" }],
      },
      scriptDirective: { resources: ["'self'", "'wasm-unsafe-eval'", "https://static.cloudflareinsights.com"] },
    },
  },
  integrations: [
    starlight({
      title: "Fénix",
      description: "Conversa. Crea. Avanza. Documentación de Fénix.",
      defaultLocale: "root",
      locales: { root: { label: "Español", lang: "es" } },
      logo: { src: "./public/docs-assets/favicon.svg", replacesTitle: false },
      favicon: "/docs-assets/favicon.svg",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/CristhianRudolf7/fenix",
        },
      ],
      customCss: ["./src/styles/custom.css"],
      components: {
        PageTitle: "./src/components/PageTitle.astro",
        MarkdownContent: "./src/components/MarkdownContent.astro",
        ThemeProvider: "./src/components/ThemeProvider.astro",
      },
      head: [
        {
          tag: "meta",
          attrs: {
            property: "og:image",
            content: "https://fenix.crehlabs.com/docs-assets/social.png",
          },
        },
      ],
      sidebar: [
        {
          label: "Empieza aquí",
          items: [
            { label: "Introducción", slug: "docs/inicio" },
            { label: "Instalación", slug: "docs/instalacion" },
            { label: "Configuración inicial", slug: "docs/configuracion" },
          ],
        },
        {
          label: "Conoce Fénix",
          items: [
            { label: "Capacidades actuales", slug: "docs/capacidades" },
            { label: "Arquitectura", slug: "docs/arquitectura" },
            { label: "Despliegue", slug: "docs/despliegue" },
            { label: "Hoja de ruta", slug: "docs/hoja-de-ruta" },
            { label: "Cambios y versiones", slug: "docs/versiones" },
          ],
        },
        {
          label: "Profundiza",
          items: [{ label: "Referencias técnicas", slug: "docs/referencias" }],
        },
      ],
    }),
  ],
});
