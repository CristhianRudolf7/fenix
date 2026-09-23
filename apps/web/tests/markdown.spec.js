import { test, expect } from "@playwright/test";
import { preview } from "vite";
let server, url;
// Exercise the built frontend, without development-time module compilation.
test.setTimeout(60_000);
test.beforeAll(async () => {
  server = await preview({ preview: { host: "127.0.0.1", port: 0 } });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  if (server)
    await new Promise((resolve) => {
      server.httpServer.close(resolve);
      server.httpServer.closeAllConnections();
    });
});

test("chat and progressive responses render safe Markdown on desktop and mobile", async ({
  page,
}) => {
  const markdown =
    "## Resultado\n\n**Listo** y *revisado*.\n\n- Primero\n- Segundo\n\n| Nombre | Estado |\n| --- | --- |\n| Informe | Correcto |\n\n```js\nconst literal = '**sin formato**';\n```\n\n[Referencia](https://example.com)\n\n[Inseguro](javascript:alert(1))\n\n<script>window.markdownAttack = true</script>\n\n![seguimiento](https://tracking.invalid/pixel)";
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    let body = [];
    if (path === "/api/auth/session")
      body = { email: "test@example.test", user_id: "test" };
    else if (path === "/api/status")
      body = {
        ready: true,
        settings: { model: "test" },
        projects: [],
        pending_requests: [],
        active_tasks: [],
        subagents: [],
      };
    else if (path === "/api/messages")
      body = [
        {
          id: "m1",
          seq: 1,
          role: "assistant",
          text: markdown,
          origin: "web",
          created_at: "2026-09-19T12:00:00Z",
        },
      ];
    else if (path === "/api/progress") {
      await route.fulfill({
        contentType: "text/event-stream",
        body: `event: progress\ndata: ${JSON.stringify({ phase: "generating", preview: "**Preparando** el siguiente paso.\n\n```rust\nlet value = 1;", message_id: "m2" })}\n\n`,
      });
      return;
    } else if (path === "/api/events") {
      await route.fulfill({
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    } else if (path === "/api/live/settings")
      body = { configured: false, settings: {} };
    else if (path.startsWith("/api/design/"))
      body = { designs: [], templates: [], total: 0, next_offset: null };
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  await page.goto(`${url}chat`, { waitUntil: "domcontentloaded" });
  const message = page.locator("#message-list .markdown-content");
  await expect(
    message.getByRole("heading", { name: "Resultado" }),
  ).toBeVisible();
  await expect(message.locator("strong")).toHaveText("Listo");
  await expect(message.locator("em")).toHaveText("revisado");
  await expect(message.locator("ul > li")).toHaveCount(2);
  await expect(message.locator("table tbody td")).toHaveText([
    "Informe",
    "Correcto",
  ]);
  await expect(message.locator("pre code")).toContainText("**sin formato**");
  await expect(
    message.getByRole("link", { name: "Referencia" }),
  ).toHaveAttribute("rel", "noopener noreferrer");
  await expect(
    message.locator("a[href^='javascript:'],script,img,iframe"),
  ).toHaveCount(0);
  expect(await page.evaluate(() => window.markdownAttack)).toBeUndefined();
  await expect(page.locator("#live-message strong")).toHaveText("Preparando");
  await expect(page.locator("#live-message pre code")).toContainText(
    "let value = 1;",
  );
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    message.getByRole("heading", { name: "Resultado" }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
