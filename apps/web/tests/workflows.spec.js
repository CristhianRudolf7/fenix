import { test, expect } from "@playwright/test";
import { preview } from "vite";
test.use({ timezoneId: "UTC" });
let server, url;
test.beforeAll(async () => {
  server = await preview({ preview: { host: "127.0.0.1", port: 0 } });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  await new Promise((resolve) => server?.httpServer.close(resolve));
});
const flow = {
  id: "university",
  name: "Entregar informe",
  description: "Drive → navegador → comprobante",
  version: 3,
  project_id: null,
  nodes: [
    {
      id: "drive",
      label: "Buscar informe",
      kind: "connector",
      depends_on: [],
      arguments: {
        connection: "google1",
        operation: "drive_search",
        arguments: { q: "trashed = false" },
      },
    },
    {
      id: "campus",
      label: "Entregar en campus",
      kind: "browser",
      depends_on: ["drive"],
      arguments: {},
      steps: [
        {
          id: "open",
          tool: "browser_open",
          arguments: { url: "https://campus.example" },
        },
        { id: "observe", tool: "browser_observe", arguments: {} },
      ],
    },
    {
      id: "notify",
      label: "Notificar",
      kind: "notify",
      depends_on: ["campus"],
      arguments: { text: "Documento entregado" },
    },
  ],
};
async function fixture(page) {
  const requests = [],
    runs = [],
    connections = [];
  const schedules = [
    {
      id: "once",
      name: "Entrega puntual",
      content: "Revisa el informe",
      mode: "task",
      kind: "once",
      at: "2026-10-02T13:00:00Z",
      status: "active",
    },
    {
      id: "weekly",
      name: "Resumen semanal",
      content: "Resume mis pendientes",
      mode: "task",
      kind: "cron",
      cron: "0 18 * * 5",
      status: "active",
      next_run: "2026-10-02T23:00:00Z",
    },
    {
      id: "delivery",
      name: "Entregar informe",
      content: "Entrega de mi documento",
      mode: "workflow",
      workflow_id: "university",
      kind: "cron",
      cron: "0 8 * * 1",
      status: "active",
    },
  ];
  await page.route("**/api/**", async (route) => {
    const req = route.request(),
      path = new URL(req.url()).pathname;
    const body = req.postDataJSON();
    requests.push({ path, method: req.method(), body });
    let data = [];
    if (path === "/api/auth/session")
      data = { email: "alice@example.test", user_id: "alice" };
    else if (path === "/api/status")
      data = {
        ready: true,
        session_id: "alice",
        event_cursor: 10,
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
    } else if (path === "/api/schedules") {
      if (req.method() === "POST") {
        const { action, ...value } = body;
        const i = schedules.findIndex((s) => s.id === value.id);
        if (action === "create")
          schedules.push({ ...value, id: "created", status: "active" });
        else if (i >= 0)
          schedules[i] = {
            ...schedules[i],
            ...value,
            status: action === "pause" ? "paused" : "active",
          };
        data = value;
      } else data = schedules;
    } else if (path === "/api/workflows") data = [flow];
    else if (path === "/api/workflows/university") data = flow;
    else if (path === "/api/workflows/university/runs") data = runs;
    else if (path === "/api/workflows/university/run") {
      const run = {
        id: "run1",
        task_id: "task1",
        workflow: flow,
        status: body.test_mode ? "paused" : "running",
        created_at: new Date().toISOString(),
        test_mode: body.test_mode,
        nodes: {
          drive: {
            status: "completed",
            attempts: 1,
            output: { files: [{ id: "report", name: "Informe" }] },
          },
          campus: { status: "pending", attempts: 0, output: null },
          notify: { status: "pending", attempts: 0, output: null },
        },
      };
      runs.splice(0, runs.length, run);
      data = run;
    } else if (path === "/api/integrations")
      data = { connections: [{ id: "whatsapp", state: "disconnected" }] };
    else if (path === "/api/connectors") {
      if (req.method() === "POST") {
        const c = {
          id: "connection1",
          provider: body.provider,
          name: body.name,
          state: "unverified",
          allow_write: body.allow_write,
        };
        connections.push(c);
        data = c;
      } else
        data = {
          catalog: [
            {
              id: "google",
              name: "Google Workspace",
              category: "office",
              auth: "oauth",
              services: ["Drive", "Gmail", "Calendar"],
              operations: ["drive_search", "drive_export"],
            },
            {
              id: "github",
              name: "GitHub",
              category: "development",
              auth: "token",
              operations: ["repositories", "issues"],
            },
            {
              id: "email",
              name: "Email · IMAP / SMTP",
              category: "communication",
              auth: "mail",
              operations: ["mail_list", "mail_send"],
            },
            {
              id: "sqlite",
              name: "SQLite",
              category: "data",
              auth: "local",
              operations: ["query"],
            },
            {
              id: "pdf",
              name: "PDF",
              category: "documents",
              auth: "builtin",
              tools: ["document_read"],
            },
          ],
          connections,
        };
    } else if (path === "/api/connectors/connection1/test") {
      connections[0].state = "connected";
      data = connections[0];
    } else if (path === "/api/messages" && req.method() === "POST")
      data = { task_id: "task-chat" };
    await route.fulfill({ json: data });
  });
  return { requests, schedules };
}
test("schedules preserve prompts and show a navigable workflow with browser actions", async ({
  page,
}, info) => {
  const mock = await fixture(page);
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(`${url}schedules`);
  await expect(
    page.getByRole("heading", { name: "Tareas programadas" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Resumen semanal" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Abrir flujo" }).click();
  await expect(
    page.getByRole("region", { name: "Diagrama del flujo" }),
  ).toBeVisible();
  await page
    .locator(".flow-node")
    .filter({ hasText: "Entregar en campus" })
    .click();
  await expect(page.locator(".flow-browser-steps")).toContainText(
    "browser_open",
  );
  await page.getByRole("button", { name: "Acercar" }).click();
  await expect(page.locator(".flow-zoom")).toContainText("115%");
  await page.getByRole("button", { name: "Probar sin efectos" }).click();
  await expect(page.locator(".flow-run-strip")).toContainText("En pausa");
  expect(mock.requests.find((r) => r.path.endsWith("/run"))?.body).toEqual({
    test_mode: true,
  });
  await page.screenshot({ path: info.outputPath("workflow-desktop.png") });
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator(".flow-inspector")).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth + 1,
    ),
  ).toBe(true);
  await page.screenshot({
    path: info.outputPath("workflow-mobile.png"),
    fullPage: true,
  });
  await page.route("**/api/workflows/university", (route) =>
    route.fulfill({ json: { ...flow, version: 4 } }),
  );
  await page.getByRole("button", { name: "Ver versión actual" }).click();
  await expect(page.locator(".flow-heading .eyebrow")).toContainText("v4");
  await expect(page.locator(".flow-node.completed")).toHaveCount(0);
  await page.getByRole("button", { name: /Ejecuciones/ }).click();
  await page.locator(".flow-history-row").click();
  await expect(page.locator(".flow-heading .eyebrow")).toContainText("v3");
  await expect(page.locator(".flow-node.completed")).toHaveCount(1);
});
test("create a simple scheduled prompt and request a flow in the conversation", async ({
  page,
}) => {
  const mock = await fixture(page);
  await page.goto(`${url}schedules`);
  await page.getByRole("button", { name: "Nueva tarea" }).click();
  const dialog = page.getByRole("dialog");
  await dialog.getByLabel("Nombre", { exact: true }).fill("Revisar pendientes");
  await dialog
    .getByLabel("Instrucciones")
    .fill("Resume las tareas pendientes de la semana.");
  await dialog.getByRole("button", { name: "Guardar programación" }).click();
  await expect(dialog).toBeHidden();
  expect(
    mock.schedules.some(
      (s) => s.name === "Revisar pendientes" && s.mode === "task",
    ),
  ).toBe(true);
  await page
    .getByRole("textbox", { name: "Describe una automatización" })
    .fill("Busca mi informe en Drive y súbelo al campus");
  await page.getByRole("button", { name: "Crear con Fénix" }).click();
  await expect(page).toHaveURL(/\/chat$/);
  expect(
    mock.requests.some(
      (r) =>
        r.path === "/api/messages" && r.body?.text.includes("workflow_manage"),
    ),
  ).toBe(true);
});
test("integration catalog connects a token without exposing it in account cards", async ({
  page,
}, info) => {
  const mock = await fixture(page);
  await page.goto(`${url}integrations`);
  await expect(
    page.getByRole("heading", { name: "Google Workspace", exact: true }),
  ).toBeVisible();
  const card = page.locator(".connector-card").filter({
    has: page.getByRole("heading", { name: "GitHub", exact: true }),
  });
  await card.getByRole("button", { name: /Conectar$/ }).click();
  const dialog = page.getByRole("dialog");
  await dialog.getByLabel("Token de acceso").fill("fixture-only-token");
  await dialog.getByRole("button", { name: "Guardar y comprobar" }).click();
  await expect(dialog).toBeHidden();
  await expect(card).toContainText("Conectada");
  await expect(card).not.toContainText("fixture-only-token");
  expect(
    mock.requests.find(
      (r) => r.path === "/api/connectors" && r.method === "POST",
    )?.body.credentials.access_token,
  ).toBe("fixture-only-token");
  await page.screenshot({
    path: info.outputPath("connections.png"),
    fullPage: true,
  });
});

test("editing a one-time task preserves its Peru time even in a different browser timezone", async ({
  page,
}) => {
  const mock = await fixture(page);
  await page.goto(`${url}schedules`);
  const card = page.locator(".schedule-card").filter({
    has: page.getByRole("heading", { name: "Entrega puntual", exact: true }),
  });
  await card.getByRole("button", { name: "Configurar", exact: true }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByLabel("Fecha y hora")).toHaveValue(
    "2026-10-02T08:00",
  );
  await dialog.getByRole("button", { name: "Guardar programación" }).click();
  await expect(dialog).toBeHidden();
  const saved = mock.schedules.find((s) => s.id === "once");
  expect(new Date(saved.at).toISOString()).toBe("2026-10-02T13:00:00.000Z");
});
