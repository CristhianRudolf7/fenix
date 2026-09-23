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

async function fixture(page, waitForTab = async () => {}) {
  const requests = [],
    commands = [];
  const profiles = [
    { name: "university", allow_agent: true },
    { name: "other", allow_agent: false },
  ];
  let session = {
    profile: "university",
    tab: "tab-1",
    temporary: false,
    width: 1280,
    height: 800,
    dpr: 1,
    rss_sum_mib: 110,
  };
  let frameRequests = 0;
  await page.route("**/api/**", async (route) => {
    const request = route.request(),
      path = new URL(request.url()).pathname;
    const body = request.method() === "POST" ? request.postDataJSON() : null;
    requests.push({ path, body });
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
        status: 200,
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    } else if (path === "/api/browser/status")
      data = { available: true, profiles, sessions: session ? [session] : [] };
    else if (path === "/api/browser/profiles") {
      const existing = profiles.find((p) => p.name === body.name);
      if (existing) existing.allow_agent = body.allow_agent;
      else profiles.push(body);
      data = body;
    } else if (path === "/api/browser/command") {
      commands.push(body);
      const { tool, args } = body;
      if (tool === "browser_tabs")
        data = {
          active: session?.tab,
          tabs: [
            {
              tab: "tab-1",
              title: "University",
              url: "http://example.test/login",
            },
            {
              tab: "tab-2",
              title: "Courses",
              url: "http://example.test/courses",
            },
          ],
        };
      if (tool !== "browser_tabs" || args.action) {
        const profile = profiles.find((p) => p.name === args.profile);
        if (profile) profile.allow_agent = false;
      }
      if (tool === "browser_resize") {
        session.width = args.width;
        session.height = args.height;
      }
      if (tool === "browser_tabs" && args.action === "select") {
        session.tab = args.tab;
        await waitForTab();
      }
      if (tool === "browser_close" && !args.tab) session = null;
    } else if (path === "/api/browser/frame") {
      frameRequests++;
      await route.fulfill({
        status: 200,
        contentType: "image/svg+xml",
        body: '<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="800"><rect width="1280" height="800" fill="white"/><text x="10" y="30">Synthetic login form</text></svg>',
      });
      return;
    } else if (path === "/api/browser/credentials") data = { saved: true };
    else if (path === "/api/browser/downloads")
      data = {
        files: [
          {
            id: "report",
            bytes: 123,
            url: "/api/browser/downloads/university/report",
          },
        ],
      };
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(data),
    });
  });
  await page.goto(`${url}browser`);
  await expect(
    page.getByAltText("Pantalla interactiva del navegador"),
  ).toBeVisible();
  return { requests, commands, profiles, frames: () => frameRequests };
}

test("owner handoff, Unicode/IME input, pinned tabs, resize and local vault", async ({
  page,
}) => {
  const { requests, commands, profiles } = await fixture(page);
  const image = page.getByAltText("Pantalla interactiva del navegador");
  await expect(page.getByText("Página actual:")).toContainText(
    "http://example.test/login",
  );
  await image.click({ position: { x: 70, y: 50 } });
  await expect
    .poll(() =>
      commands.some((c) => c.args.action === "click" && c.args.tab === "tab-1"),
    )
    .toBe(true);
  const keyboard = page.getByLabel("Teclado del navegador");
  await keyboard.fill("contraseña ñ 😀");
  await expect
    .poll(() =>
      commands.filter((c) => c.args.action === "type").map((c) => c.args.text),
    )
    .toEqual(["contraseña ñ 😀"]);
  await expect(keyboard).toHaveValue("");
  await keyboard.evaluate((element) => {
    element.dispatchEvent(
      new CompositionEvent("compositionstart", { bubbles: true }),
    );
    element.value = "大学";
    element.dispatchEvent(
      new InputEvent("input", {
        bubbles: true,
        data: "大学",
        isComposing: true,
      }),
    );
    element.dispatchEvent(
      new CompositionEvent("compositionend", { bubbles: true, data: "大学" }),
    );
    element.dispatchEvent(
      new InputEvent("input", { bubbles: true, data: "大学" }),
    );
  });
  await expect
    .poll(() => commands.filter((c) => c.args.text === "大学").length)
    .toBe(1);
  await keyboard.press("Control+a");
  await keyboard.press("Tab");
  await expect
    .poll(() =>
      commands.filter((c) => c.args.action === "key").map((c) => c.args.key),
    )
    .toEqual(["Control+a", "Tab"]);
  await expect.poll(() => profiles[0].allow_agent).toBe(false);
  await page.getByLabel("Pestaña", { exact: true }).selectOption("tab-2");
  await expect(page.getByLabel("Pestaña", { exact: true })).toHaveValue(
    "tab-2",
  );
  await expect(image).toBeVisible();
  await expect(page.getByText("Página actual:")).toContainText(
    "http://example.test/courses",
  );
  await image.click({ position: { x: 50, y: 40 } });
  await expect
    .poll(
      () => commands.filter((c) => c.args.action === "click").at(-1).args.tab,
    )
    .toBe("tab-2");
  await page.getByLabel("Tamaño de pantalla").selectOption("390,844");
  await page
    .getByRole("button", { name: "Redimensionar", exact: true })
    .click();
  await expect
    .poll(() => commands.find((c) => c.tool === "browser_resize")?.args)
    .toMatchObject({ width: 390, height: 844, tab: "tab-2" });
  await page.getByText("Contraseñas guardadas", { exact: true }).click();
  await page.getByLabel("Clave maestra").fill("synthetic-master-password");
  await page.getByLabel("Usuario", { exact: true }).fill("student");
  await page.getByLabel("Contraseña del sitio").fill("synthetic-secret");
  await page.getByRole("button", { name: "Guardar para este sitio" }).click();
  await expect
    .poll(() => requests.some((r) => r.path === "/api/browser/credentials"))
    .toBe(true);
  await expect(page.getByLabel("Clave maestra")).toHaveValue("");
  await expect(page.getByLabel("Contraseña del sitio")).toHaveValue("");
  expect(
    requests
      .filter((r) => JSON.stringify(r.body).includes("synthetic-secret"))
      .every((r) => r.path === "/api/browser/credentials"),
  ).toBe(true);
  expect(
    requests.find((r) => r.path === "/api/browser/credentials").body.tab,
  ).toBe("tab-2");
  await page.getByLabel("Permitir acceso al agente").check();
  await expect.poll(() => profiles[0].allow_agent).toBe(true);
  await page.getByRole("button", { name: "Descargas", exact: true }).click();
  await expect(
    page.getByRole("link", { name: "report", exact: true }),
  ).toHaveAttribute("href", "/api/browser/downloads/university/report");
  await page
    .getByRole("button", { name: "Cerrar perfil", exact: true })
    .click();
  await expect(image).toBeHidden();
});

test("switching profiles clears the remote screen and leaving stops polling", async ({
  page,
}) => {
  const { frames, commands } = await fixture(page);
  await page.getByLabel("Perfil del navegador").selectOption("other");
  await expect(
    page.getByAltText("Pantalla interactiva del navegador"),
  ).toBeHidden();
  await expect(page.getByLabel("Teclado del navegador")).toBeDisabled();
  expect(commands.filter((c) => c.tool === "browser_act")).toHaveLength(0);
  await page.getByRole("link", { name: "Conversación", exact: true }).click();
  const before = frames();
  await page.waitForTimeout(1300);
  expect(frames()).toBe(before);
});

test("a handoff queued for the previous tab does not leave the permission control stuck", async ({
  page,
}) => {
  let release;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const { commands, requests } = await fixture(page, () => gate);
  await page
    .getByRole("button", { name: "Tomar control", exact: true })
    .click();
  const access = page.getByLabel("Permitir acceso al agente");
  await expect(access).not.toBeChecked();
  await page.getByLabel("Pestaña", { exact: true }).selectOption("tab-2");
  await expect
    .poll(() =>
      commands.some(
        (c) => c.tool === "browser_tabs" && c.args.action === "select",
      ),
    )
    .toBe(true);
  await access.check();
  await expect(access).toBeDisabled();
  release();
  await expect(access).toBeEnabled();
  await expect(access).not.toBeChecked();
  expect(
    requests.filter(
      (r) => r.path === "/api/browser/profiles" && r.body.allow_agent,
    ),
  ).toHaveLength(0);
});
