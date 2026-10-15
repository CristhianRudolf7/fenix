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
test("private catalog opens cards without requesting publication services", async ({
  page,
}) => {
  let enabled = true,
    removed = false,
    source =
      "---\ndescription: Preparar informes\n---\n# Informes\n\nPreparar informes.",
    revision = "original",
    writes = 0,
    failSave = false;
  const exchanges = [];
  await page.route("**/api/**", async (route) => {
    const request = route.request(),
      path = new URL(request.url()).pathname;
    let data = [];
    if (path.includes("skill-exchange")) exchanges.push(path);
    if (path === "/api/auth/session")
      data = { email: "test@example.test", user_id: "owner" };
    if (path === "/api/status")
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
    if (path === "/api/session")
      data = { id: "session", settings: { model: "test" } };
    if (path === "/api/events" || path === "/api/progress") {
      await route.fulfill({
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    }
    if (path === "/api/skills")
      data = removed
        ? []
        : [
            {
              name: "reports",
              description: "Prepare reports",
              version: "1.0",
              category: "office",
              enabled,
            },
          ];
    if (path === "/api/skill-categories")
      data = [{ id: "office", labels: { en: "Office", es: "Oficina" } }];
    if (path === "/api/skill-localizations")
      data = {
        cards: [
          {
            name: "reports",
            title: "Informes",
            description: "Preparar informes de oficina",
          },
        ],
      };
    if (path === "/api/skills/reports") {
      if (request.method() === "PUT") {
        enabled = request.postDataJSON().enabled;
        data = { enabled };
      } else data = { content: "# Informes\n\n**Preparar** informes." };
    }
    if (path === "/api/skills/reports/source") {
      if (request.method() === "GET" && new URL(request.url()).search)
        expect(new URL(request.url()).searchParams.get("language")).toBe("es");
      if (request.method() === "PUT") {
        if (failSave) {
          await route.fulfill({
            status: 409,
            json: { error: "The skill changed. Reopen it before saving." },
          });
          return;
        }
        await new Promise((resolve) => setTimeout(resolve, 200));
        expect(request.postDataJSON().revision).toBe(revision);
        expect(request.postDataJSON().language).toBe("es");
        source = request.postDataJSON().content;
        revision = `revision-${++writes}`;
      }
      if (request.method() === "DELETE") {
        expect(request.postDataJSON().revision).toBe(revision);
        removed = true;
      }
      data = { content: source, revision, language: "en" };
    }
    await route.fulfill({ json: data });
  });
  await page.goto(url + "skills");
  await expect(
    page.getByRole("heading", { name: "Informes", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("tab")).toHaveCount(0);
  await expect(
    page.getByText(/Marketplace|Publicaciones|Hacer público|Ver instrucciones/),
  ).toHaveCount(0);
  await page.locator(".skill-card").click();
  const dialog = page.getByRole("dialog", { name: "Informes" });
  await expect(dialog.locator("strong")).toHaveText("Preparar");
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await page.getByRole("checkbox", { name: "Activo" }).uncheck();
  await expect.poll(() => enabled).toBe(false);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await page
    .getByRole("button", { name: "Abrir Informes", exact: true })
    .focus();
  await page.keyboard.press("Enter");
  await expect(dialog).toBeVisible();
  await expect(
    dialog
      .locator(".skill-dialog-header")
      .getByRole("button", { name: "Editar", exact: true }),
  ).toBeVisible();
  await expect(
    dialog
      .locator(".skill-reader")
      .getByRole("button", { name: "Editar", exact: true }),
  ).toHaveCount(0);
  await dialog.getByRole("button", { name: "Cerrar", exact: true }).click();
  await page.setViewportSize({ width: 390, height: 844 });
  const editBounds = await page
    .locator(".skill-card")
    .getByRole("button", { name: "Editar", exact: true })
    .boundingBox();
  const deleteBounds = await page
    .locator(".skill-card")
    .getByRole("button", { name: "Eliminar", exact: true })
    .boundingBox();
  expect(Math.abs(editBounds.y - deleteBounds.y)).toBeLessThan(2);
  expect(editBounds.x + editBounds.width).toBeLessThanOrEqual(deleteBounds.x);
  await page.screenshot({ path: "/tmp/fenix-skills-actions-mobile.png" });
  await page
    .locator(".skill-card")
    .getByRole("button", { name: "Editar", exact: true })
    .click();
  const editor = page.getByRole("textbox", {
    name: "Instrucciones del skill",
    exact: true,
  });
  await expect(editor).toHaveValue(source);
  await editor.fill(source + "\n\nWrite a concise summary.");
  await expect(editor).toBeFocused();
  await page.waitForTimeout(1300);
  expect(writes).toBe(0);
  await dialog.getByRole("button", { name: "Cerrar", exact: true }).click();
  const warning = page.getByRole("alertdialog", {
    name: "¿Salir sin guardar?",
  });
  await expect(warning).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(warning).toHaveCount(0);
  await expect(editor).toHaveValue(source + "\n\nWrite a concise summary.");
  await dialog.getByRole("button", { name: "Cerrar", exact: true }).click();
  await warning
    .getByRole("button", { name: "Seguir editando", exact: true })
    .click();
  const save = dialog.getByRole("button", {
    name: "Guardar cambios",
    exact: true,
  });
  await expect(save).toHaveText("");
  await expect(save.locator("svg")).toHaveCount(1);
  await page.screenshot({ path: "/tmp/fenix-skills-editor-mobile.png" });
  failSave = true;
  await save.click();
  await expect(dialog.getByRole("alert")).toContainText("The skill changed");
  expect(writes).toBe(0);
  await expect(editor).toHaveValue(source + "\n\nWrite a concise summary.");
  failSave = false;
  await save.dblclick();
  await expect.poll(() => writes).toBe(1);
  await expect(save).toBeDisabled();
  await expect(
    page.getByText("Todos los cambios guardados", { exact: true }),
  ).toBeVisible();
  await editor.fill(source + "\nUnsaved draft");
  await page.keyboard.press("Escape");
  await expect(warning).toBeVisible();
  await warning
    .getByRole("button", { name: "Descartar cambios", exact: true })
    .click();
  await expect(dialog).toHaveCount(0);
  expect(writes).toBe(1);
  await page
    .locator(".skill-card")
    .getByRole("button", { name: "Editar", exact: true })
    .click();
  await expect(editor).toHaveValue(source);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Cerrar", exact: true })
    .click();
  await page
    .locator(".skill-card")
    .getByRole("button", { name: "Eliminar", exact: true })
    .click();
  await page.getByRole("button", { name: "Cancelar", exact: true }).click();
  expect(removed).toBe(false);
  await page
    .locator(".skill-card")
    .getByRole("button", { name: "Eliminar", exact: true })
    .click();
  await page
    .getByRole("button", { name: "Eliminar skill", exact: true })
    .click();
  await expect(page.locator(".skill-card")).toHaveCount(0);
  expect(removed).toBe(true);
  expect(exchanges).toEqual([]);
  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});
