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
test("unavailable models block sending, preserve drafts and recover after verification", async ({
  page,
}) => {
  let health = {
    can_send: false,
    roles: {
      intelligent: { status: "unavailable", reason: "authentication" },
      fast: { status: "available" },
    },
  };
  let sends = 0,
    checks = 0;
  await page.addInitScript(() => {
    window.EventSource = class extends EventTarget {
      close() {}
    };
  });
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    let body = [];
    if (path === "/api/auth/session")
      body = {
        email: "test@example.com",
        user_id: "owner",
        installation: "personal",
      };
    if (path === "/api/status")
      body = {
        ready: true,
        session_id: "test",
        settings: {
          provider: "openai",
          model: "fixture",
          workspace: "/tmp",
          coding: {},
          execution: {},
        },
        model_health: health,
        pending_requests: [],
        active_tasks: [],
        subagents: [],
        projects: [],
      };
    if (path === "/api/providers")
      body = [
        {
          id: "openai",
          default_model: "fixture",
          base_url: "https://api.example.invalid",
        },
      ];
    if (path === "/api/model-roles/fast")
      body = {
        configured: true,
        settings: { provider: "openai", model: "fixture-fast", base_url: null },
      };
    if (path === "/api/model-health") {
      checks++;
      health = {
        can_send: true,
        roles: {
          intelligent: { status: "available" },
          fast: { status: "available" },
        },
      };
      body = health;
    }
    if (path === "/api/messages" && route.request().method() === "POST") {
      sends++;
      body = { accepted: true };
    }
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  await page.goto(url);
  const send = page.locator('#chat-form button[type="submit"]');
  await expect(send).toBeDisabled();
  await expect(page.locator("#model-health-notice")).toContainText(
    "autorización",
  );
  await page.locator("#chat-input").fill("Keep this draft");
  await page.locator("#chat-input").press("Enter");
  expect(sends).toBe(0);
  await expect(page.locator("#chat-input")).toHaveValue("Keep this draft");
  await page
    .getByRole("button", { name: "Configuración", exact: true })
    .click();
  await expect(page.locator("#detail-dialog")).toBeVisible();
  await expect(page.locator("#model-form")).toBeVisible();
  await expect(page.locator("#fast-model-form")).toBeVisible();
  await page.locator("#close-dialog").click();
  await page
    .getByRole("button", { name: "Comprobar conexión", exact: true })
    .click();
  await expect(send).toBeEnabled();
  await expect(page.locator("#model-health-notice")).toBeHidden();
  expect(checks).toBe(1);
  await send.click();
  await expect.poll(() => sends).toBe(1);
  // Availability changes are received by the existing status poll, without losing a draft.
  await page.locator("#chat-input").fill("Second draft");
  health = {
    can_send: false,
    roles: {
      intelligent: { status: "checking" },
      fast: { status: "available" },
    },
  };
  await expect(send).toBeDisabled();
  await expect(page.locator("#chat-input")).toHaveValue("Second draft");
  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
