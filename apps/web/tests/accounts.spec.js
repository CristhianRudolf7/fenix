import { test, expect } from "@playwright/test";
import { mkdtemp, mkdir, readFile, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawn } from "node:child_process";

const gatewayPort = process.env.FENIX_TEST_GATEWAY_PORT || "4192";
const runtimePort = process.env.FENIX_TEST_RUNTIME_PORT || "4193";
const gatewayURL = `http://127.0.0.1:${gatewayPort}`;
test.use({ baseURL: gatewayURL });
test.setTimeout(60_000);
let directory;
const processes = [];
function start(args, env = {}) {
  const child = spawn(
    process.env.FENIX_TEST_BINARY || resolve("../../target/debug/fenix"),
    args,
    {
      env: { ...process.env, ...env },
      stdio: "pipe",
    },
  );
  processes.push(child);
  child.stdout.resume();
  child.stderr.resume();
  return child;
}
test.beforeAll(async () => {
  directory = await mkdtemp(join(tmpdir(), "fenix-accounts-"));
  await mkdir(join(directory, "state"));
  await writeFile(
    join(directory, "state/web-search.json"),
    JSON.stringify({ enabled: false }),
  );
  const registry = join(directory, "accounts.json");
  const admin = start([
    "accounts",
    "create",
    registry,
    join(directory, "users"),
    "test@example.test",
    `http://127.0.0.1:${runtimePort}`,
  ]);
  admin.stdin.end("test-password\n");
  expect(await new Promise((resolve) => admin.once("exit", resolve))).toBe(0);
  const config = JSON.parse(await readFile(registry, "utf8"));
  config.accounts[0].runtime_port = Number(runtimePort);
  await writeFile(registry, JSON.stringify(config));
  start([
    "--port",
    runtimePort,
    "--data-dir",
    join(directory, "state"),
    "--skills-dir",
    join(directory, "skills"),
  ]);
  start(["gateway", registry], { FENIX_GATEWAY_PORT: gatewayPort });
  await expect
    .poll(
      async () => {
        try {
          return (await fetch(`${gatewayURL}/api/auth/session`)).status;
        } catch {
          return 0;
        }
      },
      { timeout: 15000 },
    )
    .toBe(401);
});
test.afterAll(async () => {
  for (const process of processes) {
    if (process.exitCode === null) {
      process.kill("SIGTERM");
      await new Promise((resolve) => process.once("exit", resolve));
    }
  }
  if (directory) await rm(directory, { recursive: true, force: true });
});
test("login, private APIs, no registration and logout", async ({
  page,
  request,
}) => {
  expect((await request.get("/api/messages")).status()).toBe(401);
  expect(
    (await request.post("/api/auth/register", { data: {} })).status(),
  ).toBe(404);
  expect((await request.get("/browser")).status()).toBe(200);
  await page.goto("/design");
  await expect(page.getByRole("heading", { name: "Fénix" })).toBeVisible();
  await page.getByLabel("Correo").fill("test@example.test");
  await page.getByLabel("Contraseña", { exact: true }).fill("incorrect");
  await page.getByRole("button", { name: "Ingresar", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("incorrectos", {
    timeout: 15000,
  });
  await page.getByLabel("Contraseña", { exact: true }).fill("test-password");
  await page.getByRole("button", { name: "Ingresar", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Conecta tu modelo" }),
  ).toBeVisible({ timeout: 15000 });
  await expect(page).toHaveURL(/\/design$/);
  await expect(
    page.getByRole("button", { name: "Salir / Sign out" }),
  ).toBeVisible();
  expect(
    await page.evaluate(async () => (await fetch("/api/auth/session")).status),
  ).toBe(200);
  await page.getByRole("button", { name: "Salir / Sign out" }).click();
  await expect(
    page.getByRole("button", { name: "Ingresar", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(async () => (await fetch("/api/messages")).status),
  ).toBe(401);
});

test("login retains preferences and stays usable on a phone", async ({
  page,
}, testInfo) => {
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Qué bueno verte." }),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.locator(".login-art").evaluate((image) => image.naturalWidth),
    )
    .toBeGreaterThan(0);
  await page.getByLabel("Contraseña", { exact: true }).fill("sample-password");
  await page
    .getByRole("button", { name: "Mostrar contraseña", exact: true })
    .click();
  await expect(page.locator("#login-password")).toHaveAttribute("type", "text");
  await page
    .getByRole("button", { name: "Ocultar contraseña", exact: true })
    .click();
  await expect(page.locator("#login-password")).toHaveAttribute(
    "type",
    "password",
  );
  await page.screenshot({
    path: testInfo.outputPath("login-desktop-light.png"),
    fullPage: true,
  });
  await page
    .getByRole("button", { name: "Cambiar modo claro y oscuro" })
    .click();
  await page.getByRole("button", { name: "Cambiar idioma" }).click();
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(
    page.getByRole("heading", { name: "Good to see you." }),
  ).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("button", { name: "Sign in", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.screenshot({
    path: testInfo.outputPath("login-mobile-dark.png"),
    fullPage: true,
  });
});

test("Integrations keeps its route through login and protects the pairing endpoint", async ({
  page,
}) => {
  expect(
    (await page.request.get("/api/integrations/whatsapp/qr")).status(),
  ).toBe(401);
  const response = await page.goto("/integrations");
  expect(response.status()).toBe(200);
  await page.getByLabel("Correo").fill("test@example.test");
  await page.getByLabel("Contraseña", { exact: true }).fill("test-password");
  await page.getByRole("button", { name: "Ingresar", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Conecta tu modelo" }),
  ).toBeVisible({ timeout: 15000 });
  await expect(page).toHaveURL(/\/integrations$/);
  const status = await page.request.get("/api/integrations");
  expect(status.status()).toBe(200);
  expect(status.headers()["cache-control"]).toContain("no-store");
  expect((await status.json()).connections[0].state).toBe("disconnected");
  const denied = await page.request.post("/api/integrations/whatsapp/connect", {
    headers: {
      origin: "https://evil.example",
      "content-type": "application/json",
    },
  });
  expect(denied.status()).toBe(403);
  const paused = await page.request.post("/api/integrations/whatsapp/pause", {
    headers: { "content-type": "application/json" },
  });
  expect(paused.status()).toBe(200);
  await page.getByRole("button", { name: "Salir / Sign out" }).click();
  await expect(
    page.getByRole("button", { name: "Ingresar", exact: true }),
  ).toBeVisible();
  expect(
    (await page.request.get("/api/integrations/whatsapp/qr")).status(),
  ).toBe(401);
});
