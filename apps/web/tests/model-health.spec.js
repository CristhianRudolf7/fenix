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
  test.setTimeout(60_000);
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
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(url);
  const send = page.locator('#chat-form button[type="submit"]');
  await expect(send).toBeDisabled();
  await expect(page.locator("#model-health-notice")).toContainText(
    "autorización",
  );
  await expect(page.locator(".connection-notice-inner")).toHaveCSS(
    "display",
    "flex",
  );
  await page.screenshot({
    path: "/tmp/fenix-polish-desktop.png",
    fullPage: true,
  });
  await expect(page.locator("#chat-input")).toHaveAttribute("readonly", "");
  await expect(page.locator("#chat-input")).toBeDisabled();
  const compact = await page.locator("#chat-form").boundingBox();
  await page.mouse.click(
    compact.x + compact.width / 2,
    compact.y + compact.height / 2,
  );
  expect(
    await page
      .locator("#chat-form")
      .evaluate((form) => form.contains(document.activeElement)),
  ).toBe(false);
  expect((await page.locator("#chat-form").boundingBox()).height).toBeCloseTo(
    compact.height,
    0,
  );
  await page.keyboard.type("Cannot enter this");
  await expect(page.locator("#chat-input")).toHaveValue("");
  await page.keyboard.press("Enter");
  expect(sends).toBe(0);
  await expect(page.locator("#chat-input")).toHaveValue("");
  await expect(page.locator("#model-health-notice button")).toHaveCount(0);
  await expect(page.locator("#chat-form")).toHaveClass(/model-unavailable/);
  // Recovery comes from the automatic status poll, never a user-triggered probe.
  health = {
    can_send: true,
    roles: {
      intelligent: { status: "available" },
      fast: { status: "available" },
    },
  };
  await expect(send).toBeEnabled();
  await expect(page.locator("#chat-input")).toBeEnabled();
  await expect(page.locator("#model-health-notice")).toBeHidden();
  await page.screenshot({
    path: "/tmp/fenix-polish-healthy.png",
    fullPage: true,
  });
  expect(checks).toBe(0);
  await expect(page.locator("#chat-form")).not.toHaveClass(/model-unavailable/);
  await expect(page.locator("#chat-input")).not.toHaveAttribute("readonly");
  await page.locator("#chat-input").fill("Keep this draft");
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
  await expect(page.locator("#chat-input")).toHaveAttribute("readonly", "");
  await expect(page.locator("#chat-input")).toHaveValue("Second draft");
  await expect(page.locator("#model-health-notice")).toBeHidden();
  await expect(page.locator("#chat-form")).not.toHaveClass(/model-unavailable/);
  health = {
    can_send: false,
    roles: {
      intelligent: { status: "unavailable", reason: "connection" },
      fast: { status: "available" },
    },
  };
  await expect(page.locator("#model-health-notice")).toBeVisible();
  await expect(page.locator("#chat-form")).toHaveClass(/model-unavailable/);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect
    .poll(async () => {
      const noticeBox = await page
        .locator(".connection-notice-inner")
        .boundingBox();
      const formBox = await page.locator("#chat-form").boundingBox();
      return noticeBox.y + noticeBox.height - formBox.y;
    })
    .toBeLessThanOrEqual(1);
  await expect(page.locator("#chat-input")).toHaveCSS("opacity", "0.55");
  await page.screenshot({
    path: "/tmp/fenix-polish-mobile.png",
    fullPage: true,
  });
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
  await page
    .getByRole("button", { name: "Expandir menú", exact: true })
    .click();
  await page
    .locator(".mobile-brand")
    .getByRole("button", { name: "Cambiar modo claro y oscuro" })
    .click();
  await page
    .getByRole("button", { name: "Cerrar menú", exact: true })
    .last()
    .click();
  await page.screenshot({ path: "/tmp/fenix-polish-dark.png", fullPage: true });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
