import { test, expect } from "@playwright/test";
import { createServer } from "node:http";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawn } from "node:child_process";
const port = 4197,
  baseURL = `http://127.0.0.1:${port}`;
test.use({ baseURL });
let directory, service, model;
test.beforeAll(async () => {
  directory = await mkdtemp(join(tmpdir(), "fenix-library-ui-"));
  await mkdir(join(directory, "data"));
  await writeFile(
    join(directory, "data", "web-search.json"),
    JSON.stringify({ enabled: false }),
  );
  model = createServer(async (req, res) => {
    for await (const _ of req) {
    }
    res.setHeader("content-type", "application/json");
    res.end(
      JSON.stringify({
        id: "library-test",
        status: "completed",
        output: [
          {
            type: "message",
            role: "assistant",
            content: [{ type: "output_text", text: "OK" }],
          },
        ],
        usage: { input_tokens: 1, output_tokens: 1, total_tokens: 2 },
      }),
    );
  });
  await new Promise((r) => model.listen(0, "127.0.0.1", r));
  service = spawn(
    process.env.FENIX_TEST_BINARY || resolve("../../target/debug/fenix"),
    [
      "--port",
      String(port),
      "--data-dir",
      join(directory, "data"),
      "--skills-dir",
      join(directory, "skills"),
    ],
    { stdio: "pipe" },
  );
  let logs = "";
  service.stdout.on("data", (b) => (logs += b));
  service.stderr.on("data", (b) => (logs += b));
  await expect
    .poll(
      async () => {
        if (service.exitCode !== null) throw new Error(logs);
        try {
          return (await fetch(`${baseURL}/api/status`)).status;
        } catch {
          return 0;
        }
      },
      { timeout: 30000 },
    )
    .toBe(200);
  const { default_settings: settings } = await (
    await fetch(`${baseURL}/api/status`)
  ).json();
  const response = await fetch(`${baseURL}/api/settings`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      ...settings,
      workspace: directory,
      base_url: `http://127.0.0.1:${model.address().port}`,
      api_key: "local-test-only",
    }),
  });
  expect(response.status, await response.text()).toBe(200);
  await fetch(`${baseURL}/api/documents/settings`, {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ enabled: false }),
  });
});
test.afterAll(async () => {
  if (service?.exitCode === null) {
    service.kill("SIGTERM");
    await new Promise((r) => service.once("exit", r));
  }
  if (model) await new Promise((r) => model.close(r));
  if (directory) await rm(directory, { recursive: true, force: true });
});
test("library uploads, searches, opens citations, preserves versions and attaches sources on mobile", async ({
  page,
}, testInfo) => {
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/documents");
  await expect(
    page.getByRole("heading", { name: "Biblioteca de documentos" }),
  ).toBeVisible();
  await page
    .getByLabel("Subir documentos", { exact: true })
    .setInputFiles({
      name: "Contrato.md",
      mimeType: "text/markdown",
      buffer: Buffer.from(
        "# Contrato\n\nLa cancelación requiere treinta días de aviso.\n",
      ),
    });
  await expect(page.locator(".library-card")).toHaveCount(1);
  let doc;
  await expect
    .poll(async () => {
      const r = await page.request.get("/api/documents");
      doc = (await r.json()).documents[0];
      return doc?.status;
    })
    .toBe("ready");
  await page.getByLabel("Buscar contenido de documentos").fill("cancelación");
  await page.getByRole("button", { name: "Buscar", exact: true }).click();
  await expect(page.locator(".library-hit")).toHaveCount(1);
  await page.locator(".library-hit").click();
  await expect(
    page.getByRole("complementary", { name: "Fuente del documento" }),
  ).toBeVisible();
  await expect(page.locator(".sidebar")).toBeHidden();
  await expect(page.locator(".library-panel")).toContainText("treinta días");
  await page.screenshot({
    path: testInfo.outputPath("library-source-desktop.png"),
    fullPage: true,
  });
  await page
    .getByRole("button", { name: "Volver al chat / cerrar fuente" })
    .click();
  const revision = await page.request.post("/api/documents/import", {
    data: {
      id: doc.id,
      expected_revision: 1,
      filename: "Contrato.md",
      data: Buffer.from(
        "# Contrato\n\nLa cancelación requiere sesenta días de aviso.",
      ).toString("base64"),
    },
  });
  expect(revision.ok()).toBe(true);
  await page.goto(`/documents?document=${doc.id}&revision=1`);
  await expect(page.getByLabel("Versión del documento")).toHaveValue("1");
  await expect(page.locator(".library-panel")).toContainText("treinta días");
  await page
    .getByRole("button", { name: "Volver al chat / cerrar fuente" })
    .click();
  await page.goto("/chat");
  await page.getByRole("button", { name: "Documentos", exact: true }).click();
  await expect(
    page.locator(".library-picker-list input[type=checkbox]"),
  ).toHaveCount(1);
  await page.locator(".library-picker-list input[type=checkbox]").check();
  await expect(page.locator(".library-source-chips")).toContainText(
    "Contrato.md",
  );
  await page.setViewportSize({ width: 390, height: 844 });
  await page
    .locator(".library-source-chips")
    .getByRole("button", { name: "Contrato.md", exact: true })
    .click();
  await expect(page.locator(".library-panel")).toBeVisible();
  const box = await page.locator(".library-panel").boundingBox();
  expect(Math.round(box.width)).toBe(390);
  await page.screenshot({
    path: testInfo.outputPath("library-source-mobile.png"),
    fullPage: true,
  });
  await page
    .getByRole("button", { name: "Volver al chat / cerrar fuente" })
    .click();
  await expect(page.locator("#chat-input")).toBeVisible();
  expect(errors).toEqual([]);
});
