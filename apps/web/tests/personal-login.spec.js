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
test("personal login has only email and password, with no registration", async ({
  page,
}) => {
  const requests = [];
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/api/auth/login") {
      requests.push(route.request().postDataJSON());
      return route.fulfill({
        status: 401,
        json: { error: "invalid_credentials" },
      });
    }
    return route.fulfill({ status: 401, json: {} });
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(url);
  await expect(
    page.getByRole("button", { name: "Ingresar", exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Google|Crear cuenta/ }),
  ).toHaveCount(0);
  await page.getByLabel("Correo", { exact: true }).fill("owner@example.test");
  await page.getByLabel("Contraseña", { exact: true }).fill("fixture-password");
  await page.getByRole("button", { name: "Ingresar", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Correo o contraseña incorrectos",
  );
  expect(requests).toEqual([
    { email: "owner@example.test", password: "fixture-password" },
  ]);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});
