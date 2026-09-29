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
async function setup(page, enabled) {
  const requests = [];
  await page.route("**/api/**", async (route) => {
    const req = route.request(),
      path = new URL(req.url()).pathname;
    if (path === "/api/auth/session")
      return route.fulfill({ status: 401, json: {} });
    if (path === "/api/auth/google") {
      if (req.method() === "GET")
        return route.fulfill({ json: { enabled, registration: true } });
      requests.push(req.postDataJSON());
      return route.fulfill({
        status: 400,
        json: { error: "google_capacity_unavailable" },
      });
    }
    if (path === "/api/auth/login") {
      requests.push(req.postDataJSON());
      return route.fulfill({ status: 401, json: {} });
    }
    return route.fulfill({ json: [] });
  });
  await page.goto(url);
  return requests;
}
test("password access remains usable until Google is configured", async ({
  page,
}) => {
  const requests = await setup(page, false);
  await expect(
    page.getByRole("button", {
      name: "Iniciar sesión con Google",
      exact: true,
    }),
  ).toBeDisabled();
  await page
    .getByRole("textbox", { name: "Correo", exact: true })
    .fill("owner@example.test");
  await page.locator("#login-password").fill("example-password");
  await page.getByRole("button", { name: "Ingresar", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Correo o contraseña incorrectos",
  );
  expect(requests[0].email).toBe("owner@example.test");
});
test("registration is explicit, has no password fields, and handles unavailable capacity", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const requests = await setup(page, true);
  await page
    .getByRole("button", { name: "¿Nuevo en Fénix? Crear cuenta" })
    .click();
  await expect(page.locator("#login-password")).toHaveCount(0);
  await page
    .getByRole("button", { name: "Registrarse con Google", exact: true })
    .click();
  await expect(page.getByRole("alert")).toContainText(
    "no podemos abrir un nuevo espacio",
  );
  expect(requests).toEqual([{ register: true }]);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page
    .getByRole("button", { name: "¿Ya tienes una cuenta? Inicia sesión" })
    .click();
  await expect(page.locator("#login-password")).toBeVisible();
});
test("expired Google callbacks are explained without leaking technical details", async ({
  page,
}) => {
  await setup(page, true);
  await page.goto(url + "?auth_error=google_expired");
  await expect(page.getByRole("alert")).toHaveText(
    "Este acceso caducó. Vuelve a intentarlo.",
  );
  expect(new URL(page.url()).search).toBe("");
  await page.getByRole("button", { name: "Cambiar idioma" }).click();
  await expect(page.getByRole("alert")).toHaveText(
    "This sign-in expired. Please try again.",
  );
});
