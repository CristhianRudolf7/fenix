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
    [
      "--port",
      "4189",
      "--data-dir",
      join(directory, "data"),
      "--skills-dir",
      join(directory, "skills"),
    ],
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

test("background tasks and simultaneous textual requests update without leaving the Tasks view", async ({
  page,
}, testInfo) => {
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  let finished = false;
  await page.route("**/api/status", async (route) => {
    const response = await route.fetch();
    const status = await response.json();
    status.subagents = [
      {
        id: "child-task-1234567890",
        parent_task_id: "parent-task-1234567890",
        spec: {
          objective: "Check the local project",
          expected_output: "A verified report with file references",
          context: "",
          constraints: "",
        },
        status: finished ? "completed" : "waiting",
        attempts: finished ? 2 : 1,
        result: finished
          ? {
              status: "completed",
              summary: "Verified <report>",
              evidence: "file.txt:2",
            }
          : null,
        created_at: "2026-09-14T09:00:00Z",
        updated_at: "2026-09-14T09:02:00Z",
      },
    ];
    status.pending_requests = finished
      ? []
      : [
          {
            id: "approval-child-123",
            task_id: "child-task-1234567890",
            kind: "approval",
            question: "Write the report?",
            action: null,
          },
          {
            id: "question-child-456",
            task_id: "another-child",
            kind: "question",
            question: "Which file?",
            action: null,
          },
        ];
    await route.fulfill({ response, json: status });
  });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/");
  await page.locator('[data-page="activity"]').click();
  const panel = page.locator("#background-subagents");
  await expect(panel).toContainText("Check the local project");
  await expect(panel).toContainText("A verified report with file references");
  await expect(panel).toContainText("1 / 3");
  await expect(panel).toContainText("parent-task-1234567890");
  await expect(page.locator("#task-pending")).toContainText(
    "aprobar approval-child-123",
  );
  await expect(page.locator("#task-pending")).toContainText(
    "responder question-child-456",
  );
  await expect(
    page.getByRole("button", { name: /aprobar|rechazar/i }),
  ).toHaveCount(0);
  await page.screenshot({
    path: testInfo.outputPath("background-tasks-desktop-light.png"),
    fullPage: true,
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator('.mobile-brand [data-action="theme"]').click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: testInfo.outputPath("background-tasks-mobile-dark.png"),
    fullPage: true,
  });
  finished = true;
  await expect(panel).toContainText("Completada");
  await expect(panel).toContainText("2 / 3");
  await expect(page.locator("#task-pending")).toBeHidden();
  await panel.locator("summary").click();
  await expect(panel.locator("pre")).toContainText("file.txt:2");
  await expect(panel.locator("report")).toHaveCount(0);
  await page.locator('.mobile-brand [data-action="language"]').click();
  await expect(page.locator("#background-title")).toHaveText(
    "Background tasks",
  );
  await expect(panel).toContainText("Completed");
  expect(errors).toEqual([]);
});

test("projects keep separate memories and referenced replies in one conversation", async ({
  page,
}, testInfo) => {
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await page.locator('[data-page="projects"]').click();
  await page.locator("#project-name").fill("Project Atlas");
  await page.locator("#project-aliases").fill("atlas, office launch");
  await page.locator("#project-description").fill("Prepare the office launch");
  await page.locator("#project-workspace").fill(directory);
  await page
    .locator("#project-brief")
    .fill("Objective: launch the office\nConstraints: use SQLite");
  await page
    .getByRole("button", { name: "Guardar proyecto", exact: true })
    .click();
  await expect(page.locator("#project-list")).toContainText("Project Atlas");
  await page.locator("#project-name").fill("Project Beacon");
  await page.locator("#project-aliases").fill("beacon");
  await page.locator("#project-brief").fill("Constraints: use PostgreSQL");
  await page
    .getByRole("button", { name: "Guardar proyecto", exact: true })
    .click();
  await expect(page.locator("#project-list .record")).toHaveCount(2);
  const projects = await (await page.request.get("/api/projects")).json();
  const atlas = projects.find((project) => project.name === "Project Atlas");
  const beacon = projects.find((project) => project.name === "Project Beacon");
  await page
    .locator("#project-list .record")
    .filter({ hasText: "Project Atlas" })
    .getByRole("button", { name: "Editar", exact: true })
    .click();
  await expect(page.locator("#project-brief")).toHaveValue(
    "Objective: launch the office\nConstraints: use SQLite",
  );
  await page.locator("#project-description").fill("Updated office launch");
  await page.waitForResponse((response) =>
    response.url().endsWith("/api/status"),
  );
  await expect(page.locator("#project-description")).toHaveValue(
    "Updated office launch",
  );
  await page.screenshot({
    path: testInfo.outputPath("projects-desktop-light.png"),
    fullPage: true,
  });
  await page
    .getByRole("button", { name: "Guardar proyecto", exact: true })
    .click();
  await expect(page.locator("#project-list")).toContainText(
    "Updated office launch",
  );
  await page
    .locator("#project-list .record")
    .filter({ hasText: "Project Atlas" })
    .getByRole("button", { name: "Ver progreso" })
    .click();
  await expect(page.locator("#detail-content")).toContainText(
    "Constraints: use SQLite",
  );
  await page.locator("#close-dialog").click();

  await page.locator('[data-page="memory"]').click();
  for (const [scope, text] of [
    ["", "General: concise answers"],
    [atlas.id, "Atlas: SQLite only"],
    [beacon.id, "Beacon: PostgreSQL only"],
  ]) {
    await page.locator("#memory-filter").selectOption(scope);
    await page.locator("#memory-title").fill("Project preference");
    await page.locator("#memory-content").fill(text);
    await page
      .getByRole("button", { name: "Guardar recuerdo", exact: true })
      .click();
    await expect(page.locator("#memory-list")).toContainText(text);
    await expect(page.locator("#memory-list .record")).toHaveCount(1);
  }
  await page.locator("#memory-filter").selectOption(atlas.id);
  await expect(page.locator("#memory-list")).toContainText(
    "Atlas: SQLite only",
  );
  await expect(page.locator("#memory-list")).not.toContainText("PostgreSQL");
  await page
    .locator("#memory-list")
    .getByRole("button", { name: "Editar", exact: true })
    .click();
  await expect(page.locator("#memory-project")).toBeDisabled();
  await page.locator('[data-action="clear-memory"]').click();
  await expect(page.locator("#memory-project")).toBeEnabled();
  await page.locator("#memory-filter").selectOption("");
  await expect(page.locator("#memory-list")).toContainText(
    "General: concise answers",
  );
  await expect(page.locator("#memory-list")).not.toContainText("SQLite");

  await page.locator('[data-page="chat"]').click();
  await expect(page.locator("#chat-form select")).toHaveCount(0);
  await expect(page.locator("#message-list")).toContainText(
    "Archivo creado y verificado.",
  );
  await page.locator("#chat-input").fill("Dime cómo va Project Atlas");
  const submitted = page.waitForResponse(
    (response) =>
      response.url().endsWith("/api/messages") &&
      response.request().method() === "POST",
  );
  await page.locator("#chat-input").press("Enter");
  const { task_id: taskId } = await (await submitted).json();
  await expect
    .poll(
      async () =>
        (await (await page.request.get(`/api/tasks/${taskId}`)).json()).binding
          .project_id,
    )
    .toBe(atlas.id);
  const reply = page.locator(`[data-reply-task="${taskId}"]`).first();
  await expect(reply).toBeVisible();
  await expect(reply.locator("..")).toContainText("Project Atlas");
  await reply.click();
  await expect(page.locator("#reply-reference")).toContainText(
    taskId.slice(0, 8),
  );
  await page.locator("#chat-input").fill("Dame un resumen breve");
  const referenced = page.waitForRequest(
    (request) =>
      request.url().endsWith("/api/messages") && request.method() === "POST",
  );
  await page.locator("#chat-input").press("Enter");
  expect((await referenced).postDataJSON()).toEqual({
    text: "Dame un resumen breve",
    reply_to_task_id: taskId,
  });
  await expect(page.locator("#reply-reference")).toBeHidden();
  await expect(page.locator("#message-list")).toContainText(
    "Create a file called browser-check.txt",
  );
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator('[data-page="projects"]').click();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  expect(errors).toEqual([]);
});
