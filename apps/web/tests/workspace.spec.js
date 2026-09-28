import { test, expect } from "@playwright/test";
import { preview } from "vite";
let server, url;
test.beforeAll(async () => {
  server = await preview({ preview: { host: "127.0.0.1", port: 0 } });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  await new Promise((resolve) => {
    server.httpServer.close(resolve);
    server.httpServer.closeAllConnections();
  });
});
test("modal explorer, unified tabs, scoped automatic refresh and mobile preview", async ({
  page,
}, info) => {
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  let text = "# Research notes\n\nInitial content";
  const visual = {
    id: "slides",
    title: "Research presentation",
    kind: "presentation",
    revision: 1,
    content: {
      slides: [
        { title: "Overview", body: "Intro" },
        { title: "Findings", body: "Requested slide" },
      ],
    },
  };
  await page.addInitScript(() => {
    window.EventSource = class extends EventTarget {
      constructor(url) {
        super();
        if (url.includes("/api/events")) window.workspaceEvents = this;
      }
      close() {}
    };
  });
  await page.route("**/api/**", async (route) => {
    const u = new URL(route.request().url());
    let body = [];
    if (u.pathname === "/api/auth/session")
      body = { email: "alice@example.test", user_id: "alice" };
    else if (u.pathname === "/api/status")
      body = {
        ready: true,
        session_id: "alice-session",
        event_cursor: 40,
        settings: { model: "test" },
        projects: [],
        pending_requests: [],
        active_tasks: [],
        subagents: [],
      };
    else if (u.pathname === "/api/live/settings")
      body = { configured: false, settings: {} };
    else if (u.pathname === "/api/design/artifacts")
      body = { designs: [visual], next_offset: null };
    else if (u.pathname === "/api/design/artifacts/slides") body = visual;
    else if (u.pathname === "/api/documents")
      body = { documents: [], next_offset: null };
    else if (u.pathname === "/api/workspace/files")
      body = {
        entries:
          u.searchParams.get("path") === "research"
            ? [{ name: "notes.md", path: "research/notes.md", bytes: 200 }]
            : [{ name: "research", path: "research", directory: true }],
        next_offset: null,
      };
    else if (u.pathname === "/api/workspace/preview")
      body = {
        title: "notes.md",
        kind: "markdown",
        format: "md",
        text,
        sha256: "fixture",
      };
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(`${url}chat`);
  await page
    .getByRole("button", { name: "Abrir carpetas", exact: true })
    .click();
  const modal = page.getByRole("dialog", { name: "Archivos del proyecto" });
  await expect(modal).toBeVisible();
  await expect(modal.getByRole("combobox")).toHaveCount(0);
  await modal
    .locator(".workspace-file-row")
    .filter({ hasText: "research" })
    .dblclick();
  await expect(modal.locator(".workspace-file-row")).toContainText("notes.md");
  await page.screenshot({ path: info.outputPath("file-explorer-desktop.png") });
  await modal.locator(".workspace-file-row").dblclick();
  await expect(modal).toBeHidden();
  await expect(page.locator(".workspace-file-panel")).toContainText(
    "Initial content",
  );
  await expect(page.getByRole("tab", { name: "notes.md" })).toBeVisible();
  await expect(page.locator(".sidebar")).toBeHidden();
  await page
    .locator("#workspace-root")
    .getByRole("button", { name: "Abrir carpetas", exact: true })
    .click();
  await expect(modal.locator(".workspace-file-row")).toContainText("notes.md");
  await page.keyboard.press("Escape");
  await expect(modal).toBeHidden();
  await expect(page.locator(".workspace-file-panel")).toBeVisible();
  const emit = (target) =>
    page.evaluate(
      (data) =>
        window.workspaceEvents.dispatchEvent(
          new MessageEvent("activity", {
            data: JSON.stringify({ seq: 41, topic: "workspace.open", data }),
            lastEventId: "41",
          }),
        ),
      target,
    );
  await emit({
    kind: "design",
    id: "slides",
    title: "Research presentation",
    page: 2,
  });
  await expect(page.locator(".slides-view")).toContainText("Findings");
  await expect(page.getByRole("tab")).toHaveCount(2);
  await page.getByRole("tab", { name: "notes.md" }).click();
  await expect(page.locator(".workspace-file-panel")).toContainText(
    "Initial content",
  );
  text = "# Research notes\n\nUpdated by the agent";
  await emit({ kind: "file", path: "research/notes.md", title: "notes.md" });
  await expect(page.locator(".workspace-file-panel")).toContainText(
    "Updated by the agent",
  );
  await expect(page.getByRole("tab")).toHaveCount(2);
  await page.screenshot({
    path: info.outputPath("workspace-tabs-desktop.png"),
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator(".workspace-file-panel")).toBeVisible();
  const bounds = await page.locator(".workspace-file-panel").boundingBox();
  expect(bounds.x).toBe(0);
  expect(bounds.width).toBe(390);
  await page.screenshot({ path: info.outputPath("workspace-mobile.png") });
  await page
    .getByRole("button", { name: "Volver al chat", exact: true })
    .click();
  await expect(page.locator(".workspace-file-panel")).toBeHidden();
  expect(errors).toEqual([]);
});
