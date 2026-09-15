import { test, expect } from "@playwright/test";
import { createServer } from "node:http";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
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
  await mkdir(join(directory, "data"));
  await writeFile(
    join(directory, "data", "web-search.json"),
    JSON.stringify({ enabled: false }),
  );
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
    if (
      (last?.role === "user" && last.content.includes("browser-check.txt")) ||
      (last?.type === "function_call_output" &&
        last.call_id === "browser-file" &&
        JSON.parse(last.output).code === "tool_schema_loaded")
    ) {
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
  await page.locator("#advanced-settings > summary").click();
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

test("chat stays available while an exact textual approval is pending", async ({
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
  await expect(page.locator("#message-list")).toContainText(
    "Hello from Fénix.",
  );
  expect(
    (await (await page.request.get("/api/status")).json()).pending.id,
  ).toBe(status.pending.id);
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
  await expect(page.locator("#memory-auto")).toBeChecked();
  await page.locator("#memory-auto").uncheck();
  await page.locator("#memory-kind").selectOption("preference");
  await page.locator("#memory-state").selectOption("candidate");
  await page.locator("#memory-title").fill("Test preference");
  await page.locator("#memory-content").fill("Prefiero café peruano");
  await page.getByRole("button", { name: "Guardar recuerdo" }).click();
  await expect(page.locator("#memory-list")).toContainText(
    "Prefiero café peruano",
  );
  await expect(page.locator("#memory-list")).toContainText("Candidato");
  await page.locator("#memory-query").fill("cafe");
  await page.getByRole("button", { name: "Buscar", exact: true }).click();
  await expect(page.locator("#memory-list")).toContainText("Test preference");
  await page.getByRole("button", { name: "Editar", exact: true }).click();
  await page.locator("#memory-state").selectOption("confirmed");
  await page.locator("#memory-content").fill("Prefiero té verde");
  await page.getByRole("button", { name: "Guardar recuerdo" }).click();
  await expect(page.locator("#memory-list")).toContainText("Prefiero té verde");
  await expect(page.locator("#memory-list")).toContainText("Confirmado");
  await page.getByRole("button", { name: "Historial", exact: true }).click();
  await expect(page.locator("#detail-content")).toContainText(
    "Prefiero café peruano",
  );
  await page.locator("#close-dialog").click();
  await page.getByRole("button", { name: "Eliminar recuerdo" }).click();
  await expect(page.locator("#memory-list")).toContainText(
    "Aún no hay recuerdos",
  );
  await page.locator('[data-page="skills"]').click();
  await expect(page.locator("[data-skill]")).toHaveCount(13);
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
    const saved = page.waitForResponse(
      (response) =>
        response.url().endsWith("/api/memory") &&
        response.request().method() === "POST",
    );
    await page
      .getByRole("button", { name: "Guardar recuerdo", exact: true })
      .click();
    expect((await saved).ok()).toBe(true);
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

test("plan milestones, reviews and revisions stay readable without action approval buttons", async ({
  page,
}, testInfo) => {
  const plan = {
    id: "plan-browser-example",
    version: 3,
    status: "paused",
    objective: "Prepare the office launch",
    reviews: [
      {
        id: "review-browser-example",
        version: 3,
        status: "blocked",
        round: 1,
        reason: "Confirm the owner's report format <unsafe>",
        node_ids: ["report"],
      },
    ],
    nodes: [
      {
        id: "analysis",
        objective: "Analysis complete",
        expected_output: "Cited sources",
        status: "completed",
        depends_on: [],
      },
      {
        id: "report",
        objective: "Revise the report <unsafe>",
        expected_output: "Updated document",
        status: "pending",
        depends_on: ["analysis"],
      },
    ],
  };
  await page.route("**/api/status", async (route) => {
    const response = await route.fetch();
    const status = await response.json();
    status.plans = [plan];
    await route.fulfill({ response, json: status });
  });
  await page.route("**/api/plans/plan-browser-example", (route) =>
    route.fulfill({
      json: {
        plan,
        reviews: plan.reviews,
        review_metrics: [
          { review_id: "review-browser-example", reported_total_tokens: 321 },
        ],
        revisions: [
          {
            version: 3,
            reason: "Keep the finished analysis and change the report",
            created_at: "2026-09-14T12:00:00Z",
            plan,
          },
        ],
      },
    }),
  );
  await page.goto("/");
  await page.locator('[data-page="activity"]').click();
  const panel = page.locator("#plans-panel");
  await expect(panel).toContainText("Prepare the office launch");
  await expect(panel).toContainText("v3");
  await expect(panel).toContainText("analysis");
  await expect(panel).toContainText("Revisión del plan");
  await expect(panel).toContainText("Ronda 2/2");
  await expect(panel).toContainText("Confirm the owner's report format");
  await expect(panel).toContainText("replanificar plan-browser-example");
  await expect(panel.locator("unsafe")).toHaveCount(0);
  await panel.locator("[data-plan]").click();
  await expect(page.locator("#detail-content")).toContainText(
    "Keep the finished analysis",
  );
  await page
    .getByText("Consumo y tiempo de revisiones", { exact: true })
    .click();
  await expect(page.locator("#detail-content")).toContainText(
    '"reported_total_tokens": 321',
  );
  await expect(page.locator("#detail-content unsafe")).toHaveCount(0);
  await page.locator("#close-dialog").click();
  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: testInfo.outputPath("editable-plans-mobile.png"),
    fullPage: true,
  });
  await expect(
    page.getByRole("button", { name: /aprobar|rechazar|approve|reject/i }),
  ).toHaveCount(0);
});

test("durable web events reconnect with a saved cursor without creating work", async ({
  page,
}) => {
  await page.goto("/");
  await expect
    .poll(() =>
      page.evaluate(() =>
        Object.keys(sessionStorage).some(
          (key) =>
            key.startsWith("fenix-events:") &&
            Number(sessionStorage.getItem(key)) > 0,
        ),
      ),
    )
    .toBe(true);
  const before = (await (await page.request.get("/api/tasks")).json()).map(
    (task) => task.id,
  );
  const streams = [];
  page.on("request", (request) => {
    if (request.url().includes("/api/events?")) streams.push(request.url());
  });
  await page.reload();
  await expect
    .poll(() =>
      streams.some((url) => Number(new URL(url).searchParams.get("after")) > 0),
    )
    .toBe(true);
  await expect(page.locator("#chat-input")).toBeVisible();
  expect(
    (await (await page.request.get("/api/tasks")).json()).map(
      (task) => task.id,
    ),
  ).toEqual(before);
});

test("coding settings persist language servers and context budgets through the web", async ({
  page,
}) => {
  await page.goto("/");
  await page.locator('[data-page="settings"]').click();
  await page.locator("#advanced-settings > summary").click();
  await page.locator("#advanced-settings details > summary").first().click();
  const servers = [
    { name: "rust", command: ["rust-analyzer"], extensions: ["rs"] },
  ];
  await page.locator("#language-servers").fill(JSON.stringify(servers));
  await page.locator("#context_window_tokens").fill("65536");
  await page.locator("#compact_at_tokens").fill("32000");
  await page.locator("#keep_recent_tokens").fill("6000");
  await page
    .getByText(/Execution limits|Límites de ejecución/, { exact: true })
    .click();
  await page.locator("#max_model_steps").fill("256");
  await page.locator("#command_timeout_seconds").fill("3600");
  await page.locator('#model-form button[type="submit"]').click();
  await expect
    .poll(async () => {
      const result = await page.request.get("/api/status");
      return (await result.json()).settings.coding.context_window_tokens;
    })
    .toBe(65536);
  await expect(page.locator("#toast")).toContainText(
    /Connection verified|Conexión verificada/,
  );
  await page.reload();
  await page.locator('[data-page="settings"]').click();
  await page.locator("#advanced-settings > summary").click();
  await page.locator("#advanced-settings details > summary").first().click();
  await expect(page.locator("#context_window_tokens")).toHaveValue("65536");
  await page
    .getByText(/Execution limits|Límites de ejecución/, { exact: true })
    .click();
  await expect(page.locator("#max_model_steps")).toHaveValue("256");
  await expect(page.locator("#command_timeout_seconds")).toHaveValue("3600");
  expect(
    JSON.parse(await page.locator("#language-servers").inputValue()),
  ).toEqual([{ ...servers[0], initialization_options: null }]);
});

test("local browser controls preserve profiles and resize the rendered page", async ({
  page,
}) => {
  test.setTimeout(120_000);
  const status = await (await page.request.get("/api/browser/status")).json();
  test.skip(!status.available, "Requires installed sandbox-capable Chromium");
  const fixture = createServer((_request, response) => {
    response.setHeader("content-type", "text/html");
    response.end(
      '<!doctype html><meta name="viewport" content="width=device-width"><style>body{margin:0}</style><h1>Local browser fixture</h1><input aria-label="Name" id="name"><button onclick="document.querySelector(\'h1\').textContent=document.querySelector(\'input\').value">Apply</button>',
    );
  });
  await new Promise((resolve) => fixture.listen(0, "127.0.0.1", resolve));
  try {
    await page.goto("/");
    await page.locator('[data-page="browser"]').click();
    await page.locator("#browser-create input").fill("ui-personal");
    await page.locator("#browser-create button").click();
    await expect(page.locator("#browser-profile")).toHaveValue("ui-personal");
    await page
      .locator("#browser-open input")
      .fill(`http://127.0.0.1:${fixture.address().port}/`);
    // Chromium startup may use the runtime's 45-second cold-start budget.
    // Check its response before waiting for the first rendered screenshot.
    const opened = page.waitForResponse(
      (response) =>
        response.url().endsWith("/api/browser/command") &&
        response.request().method() === "POST" &&
        response.request().postDataJSON()?.tool === "browser_open",
      { timeout: 65_000 },
    );
    await page.locator("#browser-open button").last().click();
    const openResponse = await opened;
    expect(openResponse.ok(), await openResponse.text()).toBe(true);
    await expect
      .poll(
        () => page.locator("#browser-screen").evaluate((e) => e.naturalWidth),
        { timeout: 20_000 },
      )
      .toBe(1280);
    await page.locator("#browser-preset").selectOption("390,844");
    await page.locator("#browser-resize").click();
    await expect
      .poll(() =>
        page.locator("#browser-screen").evaluate((e) => e.naturalWidth),
      )
      .toBe(390);
    const observation = await (
      await page.request.post("/api/browser/command", {
        data: { tool: "browser_observe", args: { profile: "ui-personal" } },
      })
    ).json();
    const screen = page.locator("#browser-screen");
    const box = await screen.boundingBox();
    const clickElement = async (name) => {
      const e = observation.elements.find((e) => e.name === name);
      await screen.click({
        position: {
          x: ((e.rect.x + e.rect.width / 2) * box.width) / 390,
          y: ((e.rect.y + e.rect.height / 2) * box.width) / 390,
        },
      });
    };
    await clickElement("Name");
    await page.keyboard.type("Owner");
    await clickElement("Apply");
    await expect
      .poll(
        async () =>
          (
            await (
              await page.request.post("/api/browser/command", {
                data: {
                  tool: "browser_check",
                  args: {
                    profile: "ui-personal",
                    condition: "text",
                    text: "Owner",
                  },
                },
              })
            ).json()
          ).passed,
      )
      .toBe(true);
    await page.locator("#browser-agent").check();
    await expect
      .poll(
        async () =>
          (
            await (await page.request.get("/api/browser/status")).json()
          ).profiles.find((p) => p.name === "ui-personal").allow_agent,
      )
      .toBe(true);
    await page.locator("#browser-control").click();
    await expect(page.locator("#browser-agent")).not.toBeChecked();
    await page.locator("#browser-close").click();
    await expect(page.locator("#browser-screen")).toBeHidden();
    await page.reload();
    await page.locator('[data-page="browser"]').click();
    await expect(page.locator("#browser-profile")).toHaveValue("ui-personal");
  } finally {
    await new Promise((resolve) => fixture.close(resolve));
  }
});

test("a late skill update does not replace the page selected by the owner", async ({
  page,
}) => {
  await page.goto("/");
  await page.locator('[data-page="skills"]').click();
  let release;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  await page.route("**/api/skills/conversation-recap", async (route) => {
    const response = await route.fetch();
    await gate;
    await route.fulfill({ response });
  });
  const finished = page.waitForResponse((response) =>
    response.url().endsWith("/api/skills/conversation-recap"),
  );
  await page.locator('[data-skill="conversation-recap"]').uncheck();
  await page.locator('[data-page="activity"]').click();
  await expect(page.locator("#task-history")).toBeVisible();
  release();
  await finished;
  // Let the response handler and any follow-up rendering complete.
  await page.waitForTimeout(300);
  await expect(page.locator("#task-history")).toBeVisible();
  await expect(page.locator('[data-page="activity"]')).toHaveAttribute(
    "aria-current",
    "page",
  );
});
