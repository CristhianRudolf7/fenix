import { test, expect } from "@playwright/test";
import { preview } from "vite";

let server, url;
test.beforeAll(async () => {
  server = await preview({ preview: { host: "127.0.0.1", port: 0 } });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  await new Promise((done) => {
    server.httpServer.close(done);
    server.httpServer.closeAllConnections();
  });
});
test("retired browser UI redirects to chat without opening a browser session", async ({
  page,
}) => {
  const browserRequests = [];
  await page.addInitScript(() => {
    window.EventSource = class extends EventTarget {
      close() {}
    };
  });
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path.startsWith("/api/browser/")) browserRequests.push(path);
    let body = [];
    if (path === "/api/auth/session")
      body = { email: "test@example.com", installation: "personal" };
    if (path === "/api/status")
      body = {
        ready: true,
        session_id: "test",
        settings: { model: "fixture" },
        model_health: {
          can_send: true,
          roles: {
            intelligent: { status: "available" },
            fast: { status: "available" },
          },
        },
        pending_requests: [],
        active_tasks: [],
        subagents: [],
        projects: [],
      };
    await route.fulfill({ json: body });
  });
  await page.goto(new URL("/browser", url).href);
  await expect(page).toHaveURL(/\/chat$/);
  await expect(page.locator("#chat-input")).toBeVisible();
  await expect(page.locator('a[href="/browser"]')).toHaveCount(0);
  await expect(page.locator(".browser-workspace")).toHaveCount(0);
  await page.setViewportSize({ width: 390, height: 844 });
  await page
    .getByRole("button", { name: "Expandir menú", exact: true })
    .click();
  await expect(page.locator("#main-sidebar")).toBeVisible();
  await expect(page.locator('[data-page="browser"]')).toHaveCount(0);
  expect(browserRequests).toEqual([]);
});
