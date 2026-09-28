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
const png = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=",
  "base64",
);
async function fixture(page) {
  const calls = [];
  let state = { id: "whatsapp", state: "disconnected", agent_access: false };
  let qrDelay = 0,
    failConnect = false;
  await page.route("**/api/**", async (route) => {
    const request = route.request(),
      path = new URL(request.url()).pathname;
    calls.push({ path, method: request.method(), body: request.postData() });
    if (
      request.method() === "POST" &&
      request.headers()["content-type"] !== "application/json"
    ) {
      await route.fulfill({
        status: 415,
        json: { error: "JSON content type required" },
      });
      return;
    }
    let data = [];
    if (path === "/api/auth/session")
      data = { email: "test@example.test", user_id: "owner" };
    else if (path === "/api/status")
      data = {
        ready: true,
        settings: { model: "test" },
        projects: [],
        pending: [],
        pending_requests: [],
        tasks: [],
        active_tasks: [],
      };
    else if (path === "/api/session")
      data = { id: "session", settings: { model: "test" } };
    else if (path === "/api/events" || path === "/api/progress") {
      await route.fulfill({
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    } else if (path === "/api/connectors")
      data = {
        catalog: [
          {
            id: "google",
            name: "Google Workspace",
            category: "office",
            auth: "oauth",
            services: ["Drive"],
          },
          {
            id: "whatsapp",
            name: "WhatsApp",
            category: "communication",
            auth: "existing",
            route: "/integrations",
            services: ["WhatsApp Web"],
          },
        ],
        connections: [],
      };
    else if (path === "/api/integrations") data = { connections: [state] };
    else if (path === "/api/integrations/whatsapp/connect") {
      if (failConnect) {
        await route.fulfill({
          status: 400,
          json: { error: "Browser session limit reached" },
        });
        return;
      }
      state = {
        ...state,
        state: state.session_saved ? "ready" : "awaiting_qr",
        profile: "integration-whatsapp",
        qr_available: !state.session_saved,
        agent_access: !!state.session_saved,
      };
      data = state;
    } else if (path === "/api/integrations/whatsapp/qr") {
      if (qrDelay) await new Promise((resolve) => setTimeout(resolve, qrDelay));
      await route
        .fulfill({
          contentType: "image/png",
          headers: { "cache-control": "no-store" },
          body: png,
        })
        .catch(() => {});
      return;
    } else if (path === "/api/integrations/whatsapp/pause") {
      state = {
        ...state,
        state: state.session_saved ? "saved" : "disconnected",
        qr_available: false,
      };
      data = { paused: true };
    } else if (path === "/api/integrations/whatsapp/forget") {
      state = { id: "whatsapp", state: "disconnected", agent_access: false };
      data = { forgotten: true };
    }
    await route.fulfill({ json: data });
  });
  await page.goto(`${url}integrations`);
  await expect(
    page.getByRole("heading", { name: "Integraciones", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Conectar WhatsApp" }),
  ).toBeEnabled();
  return {
    calls,
    setState: (value) => {
      state = { ...state, ...value };
    },
    delayQr: (value) => {
      qrDelay = value;
    },
    failConnect: (value) => {
      failConnect = value;
    },
  };
}

test("connect, scan, save, reopen and forget on a small screen without sending chat messages", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const f = await fixture(page);
  const qr = page.getByRole("img", { name: "QR para vincular WhatsApp" });
  await page.getByRole("button", { name: "Conectar WhatsApp" }).click();
  await expect(qr).toBeVisible();
  await expect(qr).toHaveAttribute("src", /^blob:/);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  f.setState({ state: "syncing", qr_available: false });
  await expect(page.locator(".connection-badge")).toHaveText(
    "Sincronizando tu sesión…",
  );
  await expect(qr).toHaveCount(0);
  f.setState({ state: "ready", session_saved: true, agent_access: true });
  await expect(page.locator(".connection-badge")).toHaveText("Sesión lista");
  await expect(
    page.getByText("Listo para Fénix.", { exact: false }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Cerrar navegador", exact: true })
    .click();
  await expect(page.locator(".connection-badge")).toHaveText(
    "Sesión guardada · navegador cerrado",
  );
  await page.reload();
  await page
    .getByRole("button", { name: "Gestionar conexión", exact: true })
    .click();
  await expect(page.locator(".connection-badge")).toHaveText("Sesión lista");
  await expect(qr).toHaveCount(0);
  await page
    .getByRole("button", { name: "Olvidar sesión", exact: true })
    .click();
  await expect(page.locator(".connection-badge")).toHaveText("Sin conectar");
  const mutations = f.calls.filter((c) => c.method === "POST");
  expect(mutations.map((c) => c.path)).toEqual(
    ["connect", "pause", "connect", "forget"].map(
      (a) => `/api/integrations/whatsapp/${a}`,
    ),
  );
  expect(mutations.every((c) => !c.body)).toBe(true);
});

test("closing while QR is still downloading cannot restore a stale pairing image", async ({
  page,
}) => {
  const f = await fixture(page);
  f.delayQr(1600);
  await page.getByRole("button", { name: "Conectar WhatsApp" }).click();
  await expect
    .poll(() => f.calls.some((c) => c.path.endsWith("/qr")))
    .toBe(true);
  await page
    .getByRole("button", { name: "Cerrar navegador", exact: true })
    .click();
  await expect(page.locator(".connection-badge")).toHaveText("Sin conectar");
  await page.waitForTimeout(2200);
  await expect(
    page.getByRole("img", { name: "QR para vincular WhatsApp" }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "Cerrar", exact: true }).click();
  await page.getByRole("link", { name: "Integraciones", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Integraciones", exact: true }),
  ).toBeVisible();
});

test("connect errors are visible and an explicit retry recovers", async ({
  page,
}) => {
  const f = await fixture(page);
  f.failConnect(true);
  await page.getByRole("button", { name: "Conectar WhatsApp" }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Browser session limit reached",
  );
  await expect(
    page.getByRole("button", { name: "Reintentar conexión" }),
  ).toBeEnabled();
  f.failConnect(false);
  await page.getByRole("button", { name: "Reintentar conexión" }).click();
  await expect(
    page.getByRole("img", { name: "QR para vincular WhatsApp" }),
  ).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("WhatsApp shares the searchable catalog and opens a dismissible pairing dialog", async ({
  page,
}) => {
  await fixture(page);
  const card = page
    .locator(".connector-grid .connector-card")
    .filter({
      has: page.getByRole("heading", { name: "WhatsApp", exact: true }),
    });
  await expect(card).toHaveCount(1);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page.getByPlaceholder("Google Drive, GitHub, PDF…").fill("WhatsApp");
  await expect(page.locator(".connector-card")).toHaveCount(1);
  await card.getByRole("button", { name: "Conectar WhatsApp" }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(
    page.getByRole("img", { name: "QR para vincular WhatsApp" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(card.getByRole("button")).toBeFocused();
});
