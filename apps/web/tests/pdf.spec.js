import { test, expect } from "@playwright/test";
import { preview } from "vite";
import { readFile } from "node:fs/promises";

let server, url;
const policy =
  "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'";
test.setTimeout(60_000);
test.beforeAll(async () => {
  server = await preview({
    preview: {
      host: "127.0.0.1",
      port: 0,
      headers: {
        "Content-Security-Policy": policy,
        "X-Content-Type-Options": "nosniff",
      },
    },
  });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  if (server)
    await new Promise((resolve) => {
      server.httpServer.close(resolve);
      server.httpServer.closeAllConnections();
    });
});

async function setup(page, bytes) {
  const visual = {
    id: "pdf-result",
    title: "Fenix PDF sample",
    kind: "document",
    revision: 1,
    content: { asset_id: "pdf-test-asset", filename: "sample.pdf" },
  };
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    let body = [];
    if (path === "/api/auth/session")
      body = { email: "test@example.test", user_id: "test" };
    else if (path === "/api/status")
      body = {
        ready: true,
        model_health: {
          can_send: true,
          roles: {
            intelligent: { status: "available" },
            fast: { status: "available" },
          },
        },
        settings: { model: "test" },
        projects: [],
        pending_requests: [],
        active_tasks: [],
        subagents: [],
      };
    else if (path === "/api/model-roles/fast")
      body = { configured: false, settings: {} };
    else if (path === "/api/events" || path === "/api/progress") {
      await route.fulfill({
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    } else if (path === "/api/design/artifacts")
      body = { designs: [visual], total: 1, next_offset: null };
    else if (path === "/api/design/artifacts/pdf-result") body = visual;
    else if (path === "/api/design/assets/pdf-test-asset") {
      await route.fulfill({
        contentType: "application/pdf",
        headers: {
          "Cache-Control": "private, no-store",
          "Content-Security-Policy": "default-src 'none'; sandbox",
        },
        body: bytes,
      });
      return;
    }
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  await page.goto(`${url}chat`, { waitUntil: "domcontentloaded" });
  await page
    .locator(".chat-design-results button")
    .filter({ hasText: "Fenix PDF sample" })
    .click();
  return page.locator(".pdf-viewer");
}

test("PDF panel only offers pagination and zoom and fits mobile without active content", async ({
  page,
}, testInfo) => {
  const errors = [],
    external = [],
    dialogs = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("dialog", async (dialog) => {
    dialogs.push(dialog.message());
    await dialog.dismiss();
  });
  page.on("request", (request) => {
    if (request.url().includes("untrusted.invalid"))
      external.push(request.url());
  });
  const viewer = await setup(
    page,
    await readFile(new URL("fixtures/pdf-viewer.pdf", import.meta.url)),
  );
  await expect(viewer.locator("canvas")).toBeVisible();
  await expect(viewer.locator(".pdf-canvas-container")).toHaveAttribute(
    "aria-busy",
    "false",
  );
  await expect(viewer.getByRole("button")).toHaveCount(2);
  await expect(viewer.locator("select")).toHaveCount(1);
  await expect(viewer.locator("details, form")).toHaveCount(0);
  await expect(
    page.locator(".design-panel .design-toolbar, .design-source"),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Pedir cambios en el chat" }),
  ).toBeVisible();
  const ink = await viewer.locator("canvas").evaluate((canvas) => {
    const pixels = canvas
      .getContext("2d")
      .getImageData(0, 0, canvas.width, canvas.height).data;
    let dark = 0;
    for (let i = 0; i < pixels.length; i += 4)
      if (pixels[i] < 100 && pixels[i + 3] > 0) dark++;
    return dark;
  });
  expect(ink).toBeGreaterThan(10);
  await expect(
    viewer.getByRole("button", { name: "Página anterior" }),
  ).toBeDisabled();
  await viewer.getByRole("button", { name: "Página siguiente" }).click();
  await expect(viewer.getByLabel("Página PDF")).toHaveValue("2");
  await expect(viewer.locator("canvas")).toHaveAttribute(
    "aria-label",
    /Página 2$/,
  );
  await viewer.getByLabel("Página PDF").fill("3");
  await viewer.getByLabel("Página PDF").press("Enter");
  await expect(
    viewer.getByRole("button", { name: "Página siguiente" }),
  ).toBeDisabled();
  await viewer.getByRole("button", { name: "Página anterior" }).click();
  await expect(viewer.getByLabel("Página PDF")).toHaveValue("2");
  await viewer.getByLabel("Zoom del PDF", { exact: true }).selectOption("0.75");
  await expect(viewer.locator("canvas")).toHaveCSS("width", "300px");
  await viewer.getByLabel("Zoom del PDF", { exact: true }).selectOption("1.5");
  await expect(viewer.locator("canvas")).toHaveCSS("width", "600px");
  await viewer.getByLabel("Zoom del PDF", { exact: true }).selectOption("fit");
  await expect(viewer.locator(".pdf-canvas-container")).toHaveAttribute(
    "aria-busy",
    "false",
  );
  const toolbar = await viewer.locator(".pdf-toolbar").boundingBox();
  expect(toolbar.height).toBeLessThan(60);
  await page.screenshot({ path: testInfo.outputPath("pdf-desktop.png") });
  await page.setViewportSize({ width: 390, height: 844 });
  await viewer.getByLabel("Zoom del PDF", { exact: true }).selectOption("fit");
  await expect(viewer.locator(".pdf-canvas-container")).toHaveAttribute(
    "aria-busy",
    "false",
  );
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({ path: testInfo.outputPath("pdf-mobile.png") });
  expect(dialogs).toEqual([]);
  expect(external).toEqual([]);
  expect(errors).toEqual([]);
  await page.locator("#design-close").click();
  await expect(viewer).toHaveCount(0);
  await page
    .locator(".chat-design-results button")
    .filter({ hasText: "Fenix PDF sample" })
    .click();
  await expect(page.locator(".pdf-viewer canvas")).toBeVisible();
});

test("PDF password stays in the viewer and malformed files show an error", async ({
  page,
}) => {
  const viewer = await setup(
    page,
    await readFile(new URL("fixtures/pdf-password.pdf", import.meta.url)),
  );
  await viewer.locator('input[type="password"]').fill("wrong");
  await viewer.getByRole("button", { name: "Abrir", exact: true }).click();
  await expect(viewer.locator('input[type="password"]')).toBeVisible();
  await viewer.locator('input[type="password"]').fill("fenix-test-password");
  await viewer.getByRole("button", { name: "Abrir", exact: true }).click();
  await expect(viewer.locator("canvas")).toBeVisible();
  await expect(viewer.locator('input[type="password"]')).toHaveCount(0);
  await page.locator("#design-close").click();
  await page.route("**/api/design/assets/pdf-test-asset", (route) =>
    route.fulfill({ contentType: "application/pdf", body: "%PDF-1.4\nbroken" }),
  );
  await page
    .locator(".chat-design-results button")
    .filter({ hasText: "Fenix PDF sample" })
    .click();
  await expect(page.locator(".pdf-viewer [role=alert]")).toBeVisible();
});
