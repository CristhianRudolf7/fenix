import { test, expect } from "@playwright/test";
import { preview } from "vite";
let server, url;
test.beforeAll(async () => {
  server = await preview({ preview: { host: "127.0.0.1", port: 0 } });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  await new Promise((resolve) => server?.httpServer.close(resolve));
});
async function fixture(page, existing = false) {
  const calls = [];
  let connections = existing
    ? [
        {
          id: "work",
          provider: "google",
          name: "Work account",
          allow_write: false,
          state: "partially_checked",
          settings: { services: ["drive", "docs"] },
          service_checks: {
            drive: { status: "passed", detail: "Read probe passed" },
            docs: { status: "not_checked", detail: "Requires a document ID" },
          },
        },
      ]
    : [];
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    const body = request.postDataJSON();
    calls.push({ path, method: request.method(), body });
    let data = [];
    if (path === "/api/auth/session")
      data = { email: "owner@example.test", user_id: "owner" };
    else if (path === "/api/status")
      data = {
        ready: true,
        settings: { model: "fixture" },
        projects: [],
        pending_requests: [],
        active_tasks: [],
        subagents: [],
      };
    else if (path === "/api/live/settings")
      data = { configured: false, settings: {} };
    else if (path === "/api/events" || path === "/api/progress") {
      await route.fulfill({
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    } else if (path === "/api/integrations")
      data = { connections: [{ id: "whatsapp", state: "disconnected" }] };
    else if (path === "/api/connectors" && request.method() === "POST") {
      data = {
        id: body.id || "work",
        name: body.name,
        provider: body.provider,
        allow_write: body.allow_write,
        settings: body.settings,
        state: "unverified",
      };
      connections = [data];
    } else if (path === "/api/connectors")
      data = {
        catalog: [
          {
            id: "google",
            name: "Google Workspace",
            category: "office",
            auth: "oauth",
            services: ["Drive", "Docs", "Gmail", "Tasks", "Meet"],
            service_options: [
              { id: "drive", name: "Drive" },
              { id: "docs", name: "Docs" },
              { id: "gmail", name: "Gmail" },
              { id: "tasks", name: "Tasks" },
              { id: "meet", name: "Meet" },
            ],
            operations: ["workspace_schema", "request", "mail_compose"],
          },
        ],
        connections,
      };
    else if (path === "/api/connectors/work/test") {
      connections[0] = {
        ...connections[0],
        state: "needs_attention",
        service_checks: {
          gmail: { status: "failed", detail: "Provider returned HTTP 403" },
          tasks: { status: "passed", detail: "Read probe passed" },
        },
      };
      data = connections[0];
    }
    await route.fulfill({ json: data });
  });
  await page.goto(`${url}integrations`);
  return calls;
}

test("Google selection sends only chosen services and displays partial diagnostics on mobile", async ({
  page,
}, info) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const calls = await fixture(page);
  const card = page.locator(".connector-card").filter({
    has: page.getByRole("heading", { name: "Google Workspace", exact: true }),
  });
  await card.getByRole("button", { name: /Conectar$/ }).click();
  const dialog = page.getByRole("dialog");
  await expect(
    dialog.getByRole("checkbox", { name: "Drive", exact: true }),
  ).toBeChecked();
  await dialog.getByRole("checkbox", { name: "Drive", exact: true }).uncheck();
  await expect(
    dialog.getByRole("button", { name: "Continuar a la autorización" }),
  ).toBeDisabled();
  await dialog.getByRole("checkbox", { name: "Gmail", exact: true }).check();
  await dialog.getByRole("checkbox", { name: "Tasks", exact: true }).check();
  await dialog.getByLabel("Método de conexión").selectOption("token");
  await dialog.getByLabel(/^Token de acceso/).fill("fixture-only-secret");
  await page.screenshot({
    path: info.outputPath("google-services-mobile.png"),
    fullPage: true,
  });
  await dialog.getByRole("button", { name: "Guardar y comprobar" }).click();
  await expect(dialog).toBeHidden();
  const saved = calls.find(
    (c) => c.path === "/api/connectors" && c.method === "POST",
  );
  expect(saved.body.settings.services).toEqual(["gmail", "tasks"]);
  expect(saved.body.allow_write).toBe(false);
  await card.getByText("Comprobación por servicio", { exact: true }).click();
  await expect(card).toContainText("Lectura comprobada");
  await expect(card).toContainText("Necesita atención");
  await expect(card).not.toContainText("fixture-only-secret");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: info.outputPath("google-checks-mobile.png"),
    fullPage: true,
  });
});

test("existing service choices are preserved and document checks are visibly incomplete", async ({
  page,
}) => {
  await fixture(page, true);
  const card = page.locator(".connector-card").filter({
    has: page.getByRole("heading", { name: "Google Workspace", exact: true }),
  });
  await expect(card).toContainText("Comprobada parcialmente");
  await card.getByText("Comprobación por servicio", { exact: true }).click();
  await expect(card).toContainText("Falta comprobar un documento concreto");
  await card.getByRole("button", { name: "Configurar", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(
    dialog.getByRole("checkbox", { name: "Drive", exact: true }),
  ).toBeChecked();
  await expect(
    dialog.getByRole("checkbox", { name: "Docs", exact: true }),
  ).toBeChecked();
  await expect(
    dialog.getByRole("checkbox", { name: "Gmail", exact: true }),
  ).not.toBeChecked();
  await expect(
    dialog.getByRole("checkbox", { name: "Meet", exact: true }),
  ).not.toBeChecked();
});
