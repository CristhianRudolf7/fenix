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
test("chat groups local days and offers quiet copy controls with a compact composer", async ({
  page,
  context,
}, info) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await page.addInitScript(() => {
    window.EventSource = class extends EventTarget {
      close() {}
    };
  });
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    let body = [];
    if (path === "/api/auth/session")
      body = { email: "test@example.com", user_id: "owner" };
    if (path === "/api/status")
      body = {
        ready: true,
        model_health: {
          can_send: true,
          roles: {
            intelligent: { status: "available" },
            fast: { status: "available" },
          },
        },
        session_id: "test",
        settings: { model: "test" },
        pending_requests: [],
        active_tasks: [],
        subagents: [],
        projects: [],
      };
    if (path === "/api/model-roles/fast")
      body = { configured: false, settings: {} };
    if (path === "/api/messages")
      body = [
        {
          id: "one",
          seq: 1,
          role: "user",
          text: "Hola",
          created_at: "2026-09-28T03:00:00Z",
        },
        {
          id: "two",
          seq: 2,
          role: "assistant",
          text: "**Hola**, ¿en qué te ayudo?",
          created_at: "2026-09-28T04:00:00Z",
        },
        {
          id: "three",
          seq: 3,
          role: "user",
          text: "Al día siguiente",
          created_at: "2026-09-28T05:01:00Z",
        },
      ];
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  await page.goto(url + "chat");
  await expect(page.locator("#dictation-controls")).toBeVisible();
  await expect(
    page.locator("#voice-root, .live-button, .voice-panel"),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Iniciar Live", exact: true }),
  ).toHaveCount(0);
  await expect(page.locator(".message-day")).toHaveCount(2);
  await expect(page.locator(".message-day").first()).toContainText(
    "27 de septiembre",
  );
  await expect(page.locator(".message-day").last()).toContainText(
    "28 de septiembre",
  );
  await expect(page.locator(".message-head time")).toHaveCount(0);
  await expect(page.locator(".message-actions time")).toHaveCount(3);
  await page.locator('[data-copy-message="two"]').click();
  await expect
    .poll(() => page.evaluate(() => navigator.clipboard.readText()))
    .toBe("**Hola**, ¿en qué te ayudo?");
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 844 });
    await page.locator("#chat-input").fill("Un mensaje corto");
    const input = await page.locator("#chat-input").boundingBox();
    const footer = await page.locator(".composer-footer").boundingBox();
    expect(input.height).toBeLessThanOrEqual(44);
    expect(footer.y - input.y - input.height).toBeLessThanOrEqual(8);
    const pinned = await page.locator(".composer-area").boundingBox();
    await page.locator("#chat-scroll").hover();
    await page.mouse.wheel(0, 12000);
    await page.waitForTimeout(300);
    const afterScroll = await page.locator(".composer-area").boundingBox();
    expect(afterScroll.y).toBeCloseTo(pinned.y, 0);
    expect(afterScroll.y + afterScroll.height).toBeCloseTo(844, 0);
    expect(
      await page
        .locator("#page")
        .evaluate((e) => e.scrollHeight - e.clientHeight),
    ).toBeLessThanOrEqual(1);
    const alignment = await page
      .locator(".message.assistant")
      .evaluate((e) => ({
        body: e.querySelector(".message-body").getBoundingClientRect().x,
        actions: e.querySelector(".message-actions").getBoundingClientRect().x,
      }));
    expect(alignment.actions).toBeCloseTo(alignment.body, 0);
    const userAlignment = await page
      .locator(".message.user")
      .last()
      .evaluate((e) => ({
        body: e.querySelector(".message-body").getBoundingClientRect().right,
        actions: e.querySelector(".message-actions").getBoundingClientRect()
          .right,
      }));
    expect(userAlignment.actions).toBeCloseTo(userAlignment.body, 0);
    await page.screenshot({ path: info.outputPath(`chat-${width}.png`) });
  }
  await page.route("**/api/messages", async (route) => {
    if (route.request().method() !== "POST") return route.fallback();
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({
        error: "No pude preparar una respuesta válida. Vuelve a intentarlo.",
      }),
    });
  });
  await page.locator("#chat-input").fill("Mi mensaje debe conservarse");
  await page
    .getByRole("button", { name: "Enviar mensaje", exact: true })
    .click();
  await expect(page.locator("#chat-input")).toHaveValue(
    "Mi mensaje debe conservarse",
  );
  await expect(page.locator("#message-list .message")).toHaveCount(3);
});
