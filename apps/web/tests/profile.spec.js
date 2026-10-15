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
async function fixture(page, { empty = false, fail = false } = {}) {
  let display_name = "Cristhian Egoavil";
  const passwords = [];
  await page.addInitScript(() => {
    window.EventSource = class extends EventTarget {
      close() {}
    };
  });
  await page.route("**/api/**", async (route) => {
    const request = route.request(),
      path = new URL(request.url());
    let body = [],
      status = 200;
    if (path.pathname === "/api/auth/session")
      body = { email: "cristhian@example.test", user_id: "test-owner" };
    else if (path.pathname === "/api/auth/profile") {
      if (request.method() === "PUT")
        display_name = request.postDataJSON().display_name;
      body = { email: "cristhian@example.test", display_name };
    } else if (path.pathname === "/api/auth/password") {
      passwords.push(request.postDataJSON());
      body = { changed: true, sign_in_required: true };
    } else if (path.pathname === "/api/status")
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
    else if (path.pathname === "/api/model-roles/fast")
      body = { configured: false, settings: {} };
    else if (path.pathname === "/api/account/usage") {
      if (fail) {
        status = 503;
        body = { error: "unavailable" };
      } else {
        const days = Number(path.searchParams.get("days"));
        const daily = Array.from({ length: days }, (_, i) => {
          const date = new Date(Date.UTC(2026, 8, 27 - days + 1 + i))
              .toISOString()
              .slice(0, 10),
            n = empty ? 0 : ((i % 4) + 1) * 1300;
          return {
            date,
            usage: {
              input_tokens: n,
              output_tokens: n / 10,
              total_tokens: n * 1.1,
              requests: empty ? 0 : 3,
              missing_usage: 0,
            },
          };
        });
        const totals = daily.reduce(
          (a, r) => {
            Object.keys(a).forEach((k) => (a[k] += r.usage[k]));
            return a;
          },
          {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            requests: 0,
            missing_usage: 0,
          },
        );
        body = { days, timezone: "America/Lima", daily, totals };
      }
    }
    await route.fulfill({
      status,
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  return { passwords };
}
test("profile has its own route, persists edits and provides real usage controls", async ({
  page,
}, info) => {
  await fixture(page);
  await page.goto(url + "activity");
  await expect(page.locator(".topbar .account-menu")).toHaveCount(0);
  await page
    .locator(".sidebar-bottom")
    .getByRole("button", { name: "Ver mi perfil" })
    .click();
  await expect(
    page.getByRole("button", { name: "Cerrar sesión", exact: true }),
  ).toBeVisible();
  await expect(page).toHaveURL(/\/profile$/);
  await expect(
    page.getByRole("heading", { name: "Mi perfil", exact: true }),
  ).toBeVisible();
  await expect(page.locator("dialog[open]")).toHaveCount(0);
  await page.getByLabel("Nombre visible").fill("Cristhian");
  await page.getByRole("button", { name: "Guardar cambios" }).click();
  await expect(page.locator(".profile-form [role=status]")).toContainText(
    "Tu perfil está actualizado",
  );
  await page.reload();
  await expect(page.getByLabel("Nombre visible")).toHaveValue("Cristhian");
  await page.getByRole("tab", { name: "Consumo" }).click();
  await page.getByLabel("Período de consumo").selectOption("7");
  await expect(page.locator(".profile-chart-bar")).toHaveCount(7);
  await page.getByText("Ver detalle por día").click();
  await expect(page.locator(".profile-data tbody tr")).toHaveCount(7);
  await page.screenshot({
    path: info.outputPath("profile-usage-desktop.png"),
    fullPage: true,
  });
  await page.goBack();
  await expect(page).toHaveURL(/\/activity$/);
});
test("responsive profile supports dark mode and validates password confirmation", async ({
  page,
}, info) => {
  const { passwords } = await fixture(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(url + "profile");
  await expect(page.getByLabel("Nombre visible")).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: info.outputPath("profile-mobile-light.png"),
    fullPage: true,
  });
  await page.evaluate(() => {
    document.documentElement.dataset.theme = "dark";
  });
  await page.getByRole("tab", { name: "Contraseña", exact: true }).click();
  await page
    .getByLabel("Contraseña actual", { exact: true })
    .fill("old-password");
  await page
    .getByLabel("Nueva contraseña", { exact: true })
    .fill("replacement-password");
  await page
    .getByLabel("Confirmar nueva contraseña", { exact: true })
    .fill("different-password");
  await page.getByRole("button", { name: "Actualizar contraseña" }).click();
  await expect(page.getByRole("alert")).toContainText("no coinciden");
  expect(passwords).toHaveLength(0);
  await page.screenshot({
    path: info.outputPath("profile-security-mobile-dark.png"),
    fullPage: true,
  });
  await page
    .getByLabel("Confirmar nueva contraseña", { exact: true })
    .fill("replacement-password");
  await page.getByRole("button", { name: "Actualizar contraseña" }).click();
  await expect(
    page.getByRole("button", { name: "Iniciar sesión", exact: true }),
  ).toBeVisible();
  expect(passwords).toHaveLength(1);
});
test("empty usage and service failure are distinct states", async ({
  page,
}) => {
  await fixture(page, { empty: true });
  await page.goto(url + "profile");
  await expect(page.getByText("Tu actividad empieza aquí")).toBeVisible();
  await page.unrouteAll();
  await fixture(page, { fail: true });
  await page.reload();
  await expect(page.getByRole("alert")).toContainText(
    "No pudimos cargar el consumo",
  );
  await expect(page.getByText("Tu actividad empieza aquí")).toHaveCount(0);
});
