import { test, expect } from "@playwright/test";
import { createServer } from "node:http";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawn } from "node:child_process";

let directory, service, modelServer, modelURL;
let releasePreview;
let streamRequests = 0;
const completion = (text) => ({
  id: "browser-response",
  status: "completed",
  output: [
    {
      type: "message",
      role: "assistant",
      content: [{ type: "output_text", text }],
    },
  ],
  usage: { input_tokens: 10, output_tokens: 10, total_tokens: 20 },
});
test.describe.configure({ mode: "serial" });
test.beforeAll(async () => {
  directory = await mkdtemp(join(tmpdir(), "fenix-browser-"));
  modelServer = createServer(async (request, response) => {
    const chunks = [];
    for await (const chunk of request) chunks.push(chunk);
    const body = JSON.parse(Buffer.concat(chunks).toString());
    response.setHeader("content-type", "application/json");
    if (request.headers.authorization === "Bearer bad-key") {
      response.statusCode = 401;
      response.end(JSON.stringify({ error: "Private provider diagnostic" }));
      return;
    }
    if (body.model !== "gpt-5.6-luna" || body.reasoning?.effort !== "low") {
      response.statusCode = 400;
      response.end(JSON.stringify({ error: "Wrong model configuration" }));
      return;
    }
    const last = body.input.at(-1);
    if (last?.role === "user" && last.content.includes("stream-preview")) {
      streamRequests++;
      expect(body.stream).toBe(true);
      response.setHeader("content-type", "text/event-stream");
      const send = (event) =>
        response.write(`data: ${JSON.stringify(event)}\n\n`);
      const waiting = new Promise((resolve) => {
        releasePreview = resolve;
        response.once("close", resolve);
      });
      send({ type: "response.output_text.delta", delta: "Respuesta parcial " });
      send({ type: "response.output_text.delta", delta: "fake-brow" });
      await waiting;
      if (response.destroyed) return;
      send({ type: "response.output_text.delta", delta: "ser-key final." });
      send({
        type: "response.completed",
        response: completion("Respuesta parcial fake-browser-key final."),
      });
      response.end();
      return;
    }
    if (last?.role === "user" && last.content.includes("browser-check.txt")) {
      response.end(
        JSON.stringify({
          id: "browser-call",
          status: "completed",
          output: [
            {
              type: "function_call",
              call_id: "browser-file",
              name: "file_write",
              arguments: JSON.stringify({
                path: "browser-check.txt",
                content: "Verified from the browser.",
              }),
            },
          ],
        }),
      );
    } else
      response.end(
        JSON.stringify(
          completion(
            last?.type === "function_call_output"
              ? "Archivo creado y verificado."
              : "Hello from Fénix.",
          ),
        ),
      );
  });
  await new Promise((resolve) => modelServer.listen(0, "127.0.0.1", resolve));
  modelURL = `http://127.0.0.1:${modelServer.address().port}`;
  service = spawn(
    resolve("../../target/debug/fenix"),
    ["--port", "4189", "--data-dir", join(directory, "data")],
    { stdio: "pipe" },
  );
  let logs = "";
  service.stdout.on("data", (chunk) => {
    logs += chunk;
  });
  service.stderr.on("data", (chunk) => {
    logs += chunk;
  });
  await expect
    .poll(
      async () => {
        if (service.exitCode !== null) throw new Error(logs);
        try {
          return (await fetch("http://127.0.0.1:4189/api/status")).status;
        } catch {
          return 0;
        }
      },
      { timeout: 15_000 },
    )
    .toBe(200);
});
test.afterAll(async () => {
  if (service?.exitCode === null) {
    service.kill("SIGTERM");
    await new Promise((resolve) => service.once("exit", resolve));
  }
  if (modelServer) await new Promise((resolve) => modelServer.close(resolve));
  if (directory) await rm(directory, { recursive: true, force: true });
});

test("welcome validates inference, lists all providers and defaults to white even on a dark OS", async ({
  page,
}, testInfo) => {
  await page.emulateMedia({ colorScheme: "dark" });
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Conecta tu modelo" }),
  ).toBeVisible();
  await expect(page.locator("#provider option")).toHaveCount(17);
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await expect(page.locator("body")).toHaveCSS(
    "background-color",
    "rgb(255, 255, 255)",
  );
  await page.screenshot({
    path: testInfo.outputPath("welcome-light.png"),
    fullPage: true,
  });
  await page.locator("details summary").click();
  await page.locator("#base_url").fill(modelURL);
  await page.locator("#api_key").fill("bad-key");
  await page.getByRole("button", { name: "Verificar y comenzar" }).click();
  await expect(page.locator("#model-status")).toContainText("HTTP 401");
  await expect(
    page.getByRole("heading", { name: "Conecta tu modelo" }),
  ).toBeVisible();
  await page.locator("#api_key").fill("fake-browser-key");
  await page.locator("#workspace").fill(directory);
  await page.getByRole("button", { name: "Verificar y comenzar" }).click();
  await expect(page.locator("#chat-input")).toBeVisible();
  expect(
    JSON.stringify(await (await page.request.get("/api/status")).json()),
  ).not.toContain("fake-browser-key");
  expect(errors).toEqual([]);
});

test("chat uses textual approval and a continuous FIFO conversation", async ({
  page,
}, testInfo) => {
  await page.goto("/");
  await page
    .locator("#chat-input")
    .fill("Create a file called browser-check.txt");
  await page.locator("#chat-input").press("Enter");
  await expect(page.locator("#message-list")).toContainText(
    "Necesito tu aprobación",
  );
  expect(
    await page.getByRole("button", { name: /aprobar|rechazar/i }).count(),
  ).toBe(0);
  await page.locator("#chat-input").fill("Hello next");
  await page.locator("#chat-input").press("Enter");
  const status = await (await page.request.get("/api/status")).json();
  await expect
    .poll(
      async () =>
        (await (await page.request.get("/api/tasks")).json()).filter(
          (task) => task.status === "queued",
        ).length,
    )
    .toBe(1);
  await page.locator("#chat-input").fill(`aprobar ${status.pending.id}`);
  await page.locator("#chat-input").press("Enter");
  await expect(page.locator("#message-list")).toContainText(
    "Archivo creado y verificado.",
  );
  await expect(page.locator("#message-list")).toContainText(
    "Hello from Fénix.",
  );
  expect(await readFile(join(directory, "browser-check.txt"), "utf8")).toBe(
    "Verified from the browser.",
  );
  await page.reload();
  await expect(page.locator("#message-list")).toContainText(
    "Archivo creado y verificado.",
  );
  await page.screenshot({
    path: testInfo.outputPath("conversation-light.png"),
    fullPage: true,
  });
});

test("memory CRUD, skill states, schedules and activity work through the real local API", async ({
  page,
}) => {
  await page.goto("/");
  await page.locator('[data-page="memory"]').click();
  await page.locator("#memory-title").fill("Test preference");
  await page.locator("#memory-content").fill("Prefiero café peruano");
  await page.getByRole("button", { name: "Guardar recuerdo" }).click();
  await expect(page.locator("#memory-list")).toContainText(
    "Prefiero café peruano",
  );
  await page.locator("#memory-query").fill("cafe");
  await page.getByRole("button", { name: "Buscar", exact: true }).click();
  await expect(page.locator("#memory-list")).toContainText("Test preference");
  await page.getByRole("button", { name: "Editar", exact: true }).click();
  await page.locator("#memory-content").fill("Prefiero té verde");
  await page.getByRole("button", { name: "Guardar recuerdo" }).click();
  await expect(page.locator("#memory-list")).toContainText("Prefiero té verde");
  await page.getByRole("button", { name: "Eliminar recuerdo" }).click();
  await expect(page.locator("#memory-list")).toContainText(
    "Aún no hay recuerdos",
  );
  await page.locator('[data-page="skills"]').click();
  await expect(page.locator("[data-skill]")).toHaveCount(4);
  await page.locator('[data-skill="conversation-recap"]').uncheck();
  await expect(
    page.locator('[data-read-skill="conversation-recap"]'),
  ).toBeDisabled();
  await page.locator('[data-skill="conversation-recap"]').check();
  await page.locator('[data-read-skill="conversation-recap"]').click();
  await expect(page.locator("#detail-content")).toContainText(
    "Conversation recap",
  );
  await page.locator("#close-dialog").click();
  await page.locator('[data-page="schedules"]').click();
  await page.locator("#schedule-name").fill("Recurring check");
  await page.locator("#schedule-content").fill("Review the task list");
  await page.locator("#schedule-kind").selectOption("interval");
  await page.locator("#schedule-seconds").fill("3600");
  await page.getByRole("button", { name: "Guardar programación" }).click();
  await expect(page.locator(".record")).toContainText("Recurring check");
  await page.getByRole("button", { name: "Pausar", exact: true }).click();
  await expect(page.locator(".record .badge")).toHaveText("Pausada");
  await page.locator('[data-page="activity"]').click();
  await expect(page.locator(".table")).toContainText("Create a file");
  await page.getByRole("button", { name: "Inspeccionar" }).first().click();
  await expect(page.locator("#detail-content")).toContainText("model_response");
});

test("theme is an explicit persistent choice and the interface fits a phone", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await page.locator('.mobile-brand [data-action="theme"]').click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: testInfo.outputPath("conversation-mobile-dark.png"),
    fullPage: true,
  });
  await page.locator('[data-page="settings"]').click();
  await expect(page.locator("#telegram-form")).toBeVisible();
  await expect(page.locator("#api_key")).toHaveValue("");
  await page.locator('.mobile-brand [data-action="language"]').click();
  await expect(page.locator("#telegram-form")).toContainText("Optional");
});

test("streaming previews survive reload without repeating inference or duplicating history", async ({
  page,
}) => {
  await page.goto("/");
  const before = streamRequests;
  await page.locator("#chat-input").fill("stream-preview reload");
  await page.locator("#chat-input").press("Enter");
  try {
    await expect(page.locator("#live-message")).toContainText(
      "Respuesta parcial",
    );
    await expect(page.locator("#live-message")).not.toContainText("fake-brow");
    await expect(page.locator("#activity-strip")).toContainText(
      "Generando una respuesta",
    );
    await expect(page.locator("#message-list")).not.toContainText(
      "Respuesta parcial",
    );
    await page.reload();
    await expect(page.locator("#live-message")).toContainText(
      "Respuesta parcial",
    );
    expect(streamRequests - before).toBe(1);
  } finally {
    releasePreview?.();
  }
  await expect(page.locator("#live-message")).toBeEmpty();
  await expect(page.locator("#message-list")).toContainText(
    "Respuesta parcial [REDACTED] final.",
  );
  await expect(
    page
      .locator("#message-list .message-text")
      .filter({ hasText: "Respuesta parcial [REDACTED] final." }),
  ).toHaveCount(1);
  const history = await (await page.request.get("/api/messages")).json();
  expect(JSON.stringify(history)).not.toContain("fake-browser-key");
});

test("cancelling an in-flight stream clears the preview and does not retry inference", async ({
  page,
}) => {
  await page.goto("/");
  const before = streamRequests;
  await page.locator("#chat-input").fill("stream-preview cancel");
  await page.locator("#chat-input").press("Enter");
  await expect(page.locator("#live-message")).toContainText(
    "Respuesta parcial",
  );
  const status = await (await page.request.get("/api/status")).json();
  await page.locator("#chat-input").fill(`cancelar ${status.active_task}`);
  await page.locator("#chat-input").press("Enter");
  await expect
    .poll(
      async () =>
        (await (await page.request.get("/api/tasks")).json()).find(
          (task) => task.id === status.active_task,
        ).status,
    )
    .toBe("cancelled");
  await expect(page.locator("#live-message")).toBeEmpty();
  expect(streamRequests - before).toBe(1);
});
