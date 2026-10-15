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

async function fixture(page, waitForTab = async () => {}, options = {}) {
  const requests = [],
    commands = [];
  const profiles = options.empty
    ? []
    : [
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
  if (options.empty) session = null;
  const tabs = () => [
    { tab: "tab-1", title: "University", url: "http://example.test/login" },
    { tab: "tab-2", title: "Courses", url: "http://example.test/courses" },
  ];
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
        model_health: {
          can_send: true,
          roles: {
            intelligent: { status: "available" },
            fast: { status: "available" },
          },
        },
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
    } else if (path === "/api/browser/view") {
      const name = body.profile || profiles[0]?.name || "personal";
      if (options.waitForOpen) await options.waitForOpen(name);
      if (!profiles.some((p) => p.name === name))
        profiles.push({ name, allow_agent: false });
      if (!session || session.profile !== name)
        session = {
          profile: name,
          tab: "tab-1",
          temporary: false,
          width: 1280,
          height: 800,
          dpr: 1,
        };
      data = { profile: name, active: session.tab, tabs: tabs() };
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
        body: `<svg xmlns="http://www.w3.org/2000/svg" width="${session.width}" height="${session.height}"><rect width="100%" height="100%" fill="white"/><text x="10" y="30">Synthetic login form</text></svg>`,
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
  await expect(page.getByLabel("Dirección web")).toHaveValue(
    "http://example.test/login",
  );
  await page.screenshot({
    path: test.info().outputPath("browser-desktop.png"),
  });
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
  await page.getByRole("tab", { name: "Courses", exact: true }).click();
  await expect(
    page.getByRole("tab", { name: "Courses", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await expect(image).toBeVisible();
  await expect(page.getByLabel("Dirección web")).toHaveValue(
    "http://example.test/courses",
  );
  await image.click({ position: { x: 50, y: 40 } });
  await expect
    .poll(
      () => commands.filter((c) => c.args.action === "click").at(-1).args.tab,
    )
    .toBe("tab-2");
  await page.setViewportSize({ width: 700, height: 900 });
  await expect
    .poll(
      () =>
        commands.filter((c) => c.tool === "browser_resize").at(-1)?.args.width,
    )
    .toBeLessThan(700);
  await expect
    .poll(
      () =>
        commands.filter((c) => c.tool === "browser_resize").at(-1)?.args.tab,
    )
    .toBe("tab-2");
  await expect(page.getByLabel("Tamaño de pantalla")).toHaveCount(0);
  await page
    .getByText("Sesiones guardadas y opciones", { exact: true })
    .click();
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
  await page
    .getByRole("button", { name: "Dejar continuar a Fénix", exact: true })
    .click();
  await expect.poll(() => profiles[0].allow_agent).toBe(true);
  await page.getByRole("button", { name: "Descargas", exact: true }).click();
  await expect(
    page.getByRole("link", { name: "report", exact: true }),
  ).toHaveAttribute("href", "/api/browser/downloads/university/report");
  await page
    .getByRole("button", { name: "Cerrar navegador", exact: true })
    .click();
  await expect(image).toBeHidden();
});

test("switching saved sessions clears stale input and leaving stops transmission", async ({
  page,
}) => {
  let release;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const { frames, commands } = await fixture(page, undefined, {
    waitForOpen: (name) => (name === "other" ? gate : Promise.resolve()),
  });
  await page
    .getByText("Sesiones guardadas y opciones", { exact: true })
    .click();
  await page
    .getByLabel("Sesión guardada", { exact: true })
    .selectOption("other");
  await expect(
    page.getByAltText("Pantalla interactiva del navegador"),
  ).toBeHidden();
  await expect(page.getByLabel("Teclado del navegador")).toBeDisabled();
  expect(commands.filter((c) => c.tool === "browser_act")).toHaveLength(0);
  release();
  await expect(
    page.getByAltText("Pantalla interactiva del navegador"),
  ).toBeVisible();
  await page.getByRole("link", { name: "Conversación", exact: true }).click();
  const before = frames();
  await page.waitForTimeout(1300);
  expect(frames()).toBe(before);
});

test("a handoff queued for the previous tab does not authorize a different page", async ({
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
  const access = page.getByRole("button", {
    name: "Dejar continuar a Fénix",
    exact: true,
  });
  await expect(access).toBeEnabled();
  await page.getByRole("tab", { name: "Courses", exact: true }).click();
  await expect
    .poll(() =>
      commands.some(
        (c) => c.tool === "browser_tabs" && c.args.action === "select",
      ),
    )
    .toBe(true);
  await access.click();
  await expect(page.locator(".browser-handoff")).toBeDisabled();
  release();
  await expect(access).toBeEnabled();
  expect(
    requests.filter(
      (r) => r.path === "/api/browser/profiles" && r.body.allow_agent,
    ),
  ).toHaveLength(0);
});

test("first visit opens a private browser automatically and adapts to mobile", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const { requests, commands, profiles } = await fixture(page, undefined, {
    empty: true,
  });
  expect(profiles).toEqual([{ name: "personal", allow_agent: false }]);
  expect(requests.filter((r) => r.path === "/api/browser/view")).toHaveLength(
    1,
  );
  expect(commands.some((c) => c.tool === "browser_open")).toBe(false);
  await page.screenshot({ path: test.info().outputPath("browser-mobile.png") });
  await expect(page.getByLabel("Nuevo perfil")).toHaveCount(0);
  await expect(page.getByLabel("Ancho", { exact: true })).toHaveCount(0);
  await expect
    .poll(
      () =>
        commands.filter((c) => c.tool === "browser_resize").at(-1)?.args.width,
    )
    .toBeLessThan(390);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.getByLabel("Dirección web").fill("example.test");
  await page.getByRole("button", { name: "Ir", exact: true }).click();
  await expect
    .poll(() => commands.find((c) => c.tool === "browser_open")?.args.url)
    .toBe("https://example.test");
  expect(
    requests.filter((r) => r.path === "/api/messages" && r.body),
  ).toHaveLength(0);
});

test("hidden views stop frame requests and reconnect when visible", async ({
  page,
}) => {
  const { frames } = await fixture(page);
  await page.evaluate(() => {
    Object.defineProperty(document, "hidden", {
      configurable: true,
      value: true,
    });
    document.dispatchEvent(new Event("visibilitychange"));
  });
  const before = frames();
  await page.waitForTimeout(1500);
  expect(frames()).toBe(before);
  await page.evaluate(() => {
    Object.defineProperty(document, "hidden", {
      configurable: true,
      value: false,
    });
    document.dispatchEvent(new Event("visibilitychange"));
  });
  await expect.poll(frames).toBeGreaterThan(before);
});

test("mouse dragging, touch scrolling and the wheel reach the displayed tab", async ({
  page,
}) => {
  const { commands } = await fixture(page);
  const image = page.getByAltText("Pantalla interactiva del navegador");
  const rect = await image.boundingBox();
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [{ x: rect.x + 50, y: rect.y + 150 }],
  });
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchMove",
    touchPoints: [{ x: rect.x + 50, y: rect.y + 50 }],
  });
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchEnd",
    touchPoints: [],
  });
  await cdp.detach();
  await expect
    .poll(() =>
      commands.some((c) => c.args.action === "scroll" && c.args.dy > 0),
    )
    .toBe(true);
  await image.dispatchEvent("wheel", {
    deltaY: 250,
    deltaMode: 0,
    clientX: rect.x + 50,
    clientY: rect.y + 50,
  });
  await expect
    .poll(() =>
      commands.some((c) => c.args.action === "scroll" && c.args.dy === 250),
    )
    .toBe(true);
  await expect(image).toBeVisible();
  const updated = await image.boundingBox();
  await page.mouse.move(updated.x + 30, updated.y + 40);
  await page.mouse.down();
  await page.mouse.move(updated.x + 100, updated.y + 60);
  await page.mouse.up();
  await expect
    .poll(() =>
      commands.some((c) => c.args.action === "drag" && c.args.tab === "tab-1"),
    )
    .toBe(true);
});

test("double clicking does not replay clicks and watching preserves agent access", async ({
  page,
}) => {
  const { commands, profiles } = await fixture(page);
  expect(commands.filter((c) => c.tool === "browser_resize")).toHaveLength(0);
  expect(profiles[0].allow_agent).toBe(true);
  const image = page.getByAltText("Pantalla interactiva del navegador");
  await image.dblclick({ position: { x: 80, y: 60 } });
  await expect
    .poll(() =>
      commands
        .filter((c) => c.args.action === "click")
        .map((c) => c.args.click_count),
    )
    .toEqual([1, 2]);
  expect(commands.some((c) => c.args.action === "double_click")).toBe(false);
  await expect.poll(() => profiles[0].allow_agent).toBe(false);
});
