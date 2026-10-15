import { test, expect } from "@playwright/test";
import { createServer } from "node:http";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawn } from "node:child_process";

const appPort = process.env.FENIX_TEST_APP_PORT || "4189";
const appURL = `http://127.0.0.1:${appPort}`;
test.use({ baseURL: appURL });
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
    if (
      body.model === "chat-test" &&
      JSON.stringify(body.input).includes("Write a brief progress message")
    ) {
      response.end(JSON.stringify(completion("Estoy revisando tu documento.")));
      return;
    }
    if (body.model === "chat-test") {
      response.end(
        JSON.stringify(
          completion(
            body.input.length > 1
              ? JSON.stringify({
                  reply: "Request received.",
                  action: "delegate",
                  task_id: null,
                })
              : "OK",
          ),
        ),
      );
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
      send({ type: "response.output_text.delta", delta: "de trabajo" });
      await waiting;
      if (response.destroyed) return;
      send({ type: "response.output_text.delta", delta: " final." });
      send({
        type: "response.completed",
        response: completion("Respuesta parcial de trabajo final."),
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
  const executable =
    process.env.FENIX_TEST_BINARY || resolve("../../target/debug/fenix");
  const provision = spawn(
    executable,
    [
      "owner",
      "test@example.test",
      "--password-stdin",
      "--data-dir",
      join(directory, "data"),
    ],
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  provision.stdin.end("fixture-browser-password\n");
  await new Promise((resolve, reject) => {
    provision.once("error", reject);
    provision.once("exit", (code) =>
      code === 0 ? resolve() : reject(new Error("Owner provisioning failed")),
    );
  });
  service = spawn(
    process.env.FENIX_TEST_BINARY || resolve("../../target/debug/fenix"),
    [
      "--port",
      appPort,
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
          return (await fetch(`${appURL}/api/auth/session`)).status;
        } catch {
          return 0;
        }
      },
      { timeout: 15_000 },
    )
    .toBe(401);
});
test.beforeEach(async ({ page }) => {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "test@example.test", password: "fixture-browser-password" },
  });
  expect(response.ok()).toBe(true);
});
test.afterAll(async () => {
  if (service?.exitCode === null) {
    service.kill("SIGTERM");
    await new Promise((resolve) => service.once("exit", resolve));
  }
  if (modelServer) await new Promise((resolve) => modelServer.close(resolve));
  if (directory) await rm(directory, { recursive: true, force: true });
});

test("personal welcome configures the model after owner login and defaults to warm light", async ({
  page,
}, testInfo) => {
  await page.emulateMedia({ colorScheme: "dark" });
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Conecta tu modelo" }),
  ).toBeVisible();
  await expect(page.locator("#api_key")).toBeVisible();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await expect(page.locator("body")).toHaveCSS(
    "background-color",
    "rgb(247, 246, 242)",
  );
  await page.screenshot({
    path: testInfo.outputPath("welcome-light.png"),
    fullPage: true,
  });
  // Service credentials are provisioned by the operator, never through the UI.
  const { default_settings: settings } = await (
    await page.request.get("/api/status")
  ).json();
  const configured = await page.request.post("/api/settings", {
    data: {
      ...settings,
      base_url: modelURL,
      workspace: directory,
      api_key: "fake-browser-key",
    },
  });
  expect(configured.ok()).toBe(true);
  await page.reload();
  await expect(page.locator("#chat-input")).toBeVisible();
  // Chat uses its own interlocutor; the master executes delegated tasks.
  const interlocutor = await page.request.post("/api/model-roles/fast", {
    data: {
      provider: "openai",
      model: "chat-test",
      base_url: modelURL,
      api_key: "chat-test-key",
    },
  });
  expect(interlocutor.ok()).toBe(true);
  expect(
    JSON.stringify(await (await page.request.get("/api/status")).json()),
  ).not.toContain("fake-browser-key");
  expect(errors).toEqual([]);
});

test("navigation can collapse without losing routes or preferences", async ({
  page,
}, testInfo) => {
  await page.goto("/");
  await expect(page.locator("#chat-input")).toBeVisible();
  await page.screenshot({
    path: testInfo.outputPath("chat-welcome-light.png"),
    fullPage: true,
  });
  await page
    .getByRole("button", { name: "Colapsar menú", exact: true })
    .click();
  await expect(page.locator(".shell")).toHaveClass(/is-collapsed/);
  await page.locator('[data-page="skills"]').click();
  await expect(page.locator(".personal-skills")).toBeVisible();
  await page.reload();
  await expect(page.locator(".shell")).toHaveClass(/is-collapsed/);
  await page
    .getByRole("button", { name: "Expandir menú", exact: true })
    .click();
  await expect(page.locator(".shell")).not.toHaveClass(/is-collapsed/);
  await expect(page.locator(".personal-skills")).toBeVisible();
  await expect(page).toHaveURL(/\/skills$/);
});

test("compact composer and left drawer work across screen sizes", async ({
  page,
}, testInfo) => {
  for (const width of [320, 390, 620, 768, 1440]) {
    await page.setViewportSize({ width, height: 844 });
    await page.goto("/chat");
    await expect(page.locator("#chat-input")).toBeVisible();
    const input = await page.locator("#chat-input").boundingBox();
    expect(input.height).toBeLessThanOrEqual(40);
    if (width === 390 || width === 1440)
      await page.screenshot({ path: testInfo.outputPath(`chat-${width}.png`) });
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    if (width <= 620) {
      await expect(page.locator("#main-sidebar")).toBeHidden();
      await page.locator(".sidebar-toggle").click();
      await expect(page.locator("#main-sidebar")).toBeVisible();
      await expect
        .poll(async () =>
          Math.round((await page.locator("#main-sidebar").boundingBox()).x),
        )
        .toBe(0);
      const sidebar = await page.locator("#main-sidebar").boundingBox();
      expect(sidebar.x).toBeCloseTo(0, 0);
      expect(sidebar.height).toBe(844);
      if (width === 390)
        await page.screenshot({
          path: testInfo.outputPath("left-drawer-390.png"),
        });
      await page.keyboard.press("Escape");
      await expect(page.locator("#main-sidebar")).toBeHidden();
      await expect(page.locator(".sidebar-toggle")).toBeFocused();
      await page.locator(".sidebar-toggle").click();
      await page.locator('[data-page="skills"]').click();
      await expect(page).toHaveURL(/skills$/);
      await expect(page.locator("#main-sidebar")).toBeHidden();
    }
  }
});

test("section URLs survive reload and history with accessible loading", async ({
  page,
}) => {
  await page.goto("/chat");
  await expect(page.locator("#chat-input")).toBeVisible();
  await expect(page.locator('[data-page="browser"]')).toHaveCount(1);
  await page.locator('[data-page="projects"]').click();
  await expect(page).toHaveURL(/\/projects$/);
  await expect(page.locator("#project-list")).toBeVisible();
  await page.locator('[data-page="memory"]').click();
  await expect(page).toHaveURL(/\/memory$/);
  await page.reload();
  await expect(page.locator('[data-page="memory"]')).toHaveAttribute(
    "aria-current",
    "page",
  );
  await page.goBack();
  await expect(page).toHaveURL(/\/projects$/);
  await expect(page.locator("#project-list")).toBeVisible();
  await page.goForward();
  await expect(page).toHaveURL(/\/memory$/);
  let release;
  const delayed = new Promise((resolve) => {
    release = resolve;
  });
  await page.route("**/api/skills", async (route) => {
    await delayed;
    await route.continue();
  });
  await page.locator('[data-page="skills"]').click();
  await expect(page.locator(".loading-state")).toBeVisible();
  await expect(page.locator(".loading-state")).toHaveAttribute(
    "role",
    "status",
  );
  await page.emulateMedia({ reducedMotion: "reduce" });
  expect(
    await page
      .locator(".loading-emblem")
      .evaluate((e) => getComputedStyle(e).animationName),
  ).toBe("none");
  release();
  await expect(page.locator(".loading-state")).toHaveCount(0);
  for (const section of ["schedules", "activity", "settings", "chat"]) {
    await page.goto("/" + section);
    await expect(page.locator(`[data-page="${section}"]`)).toHaveAttribute(
      "aria-current",
      "page",
    );
  }
  expect((await page.request.get("/api/browser/status")).ok()).toBe(true);
  expect((await page.request.get("/browser")).status()).toBe(200);
  expect((await page.request.get("/api/does-not-exist")).status()).toBe(404);
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
  const skills = await (await page.request.get("/api/skills")).json();
  await expect(page.locator("[data-skill]")).toHaveCount(skills.length);
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
  await page.locator(".sidebar-toggle").click();
  await page.locator('.mobile-brand [data-action="theme"]').click();
  if (await page.locator(".mobile-menu-close").isVisible())
    await page.locator(".mobile-menu-close").click();
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
  if ((page.viewportSize()?.width || 1000) <= 620)
    await page.locator(".sidebar-toggle").click();
  await page.locator('[data-page="settings"]').click();
  await expect(page.locator("#telegram-form")).toBeVisible();
  await expect(page.locator("#api_key")).toHaveCount(0);
  await page.locator(".sidebar-toggle").click();
  await page.locator('.mobile-brand [data-action="language"]').click();
  if (await page.locator(".mobile-menu-close").isVisible())
    await page.locator(".mobile-menu-close").click();
  await expect(page.locator("#telegram-form")).toContainText("Optional");
});

test("master drafts stay private across reload and final replies remain in history", async ({
  page,
}) => {
  await page.goto("/");
  const before = streamRequests;
  await page.locator("#chat-input").fill("stream-preview reload");
  await page.locator("#chat-input").press("Enter");
  try {
    await expect.poll(() => streamRequests - before).toBe(1);
    await expect(page.locator("#live-message")).not.toContainText(
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
    await expect.poll(() => streamRequests - before).toBe(1);
    await expect(page.locator("#live-message")).not.toContainText(
      "Respuesta parcial",
    );
    expect(streamRequests - before).toBe(1);
  } finally {
    releasePreview?.();
  }
  await expect(page.locator("#live-message")).toBeEmpty();
  await expect(page.locator("#message-list")).toContainText(
    "Respuesta parcial de trabajo final.",
  );
  await expect(
    page
      .locator("#message-list .message-text")
      .filter({ hasText: "Respuesta parcial de trabajo final." }),
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
  await expect.poll(() => streamRequests - before).toBe(1);
  await expect(page.locator("#live-message")).not.toContainText(
    "Respuesta parcial",
  );
  const status = await (await page.request.get("/api/status")).json();
  await page.locator("#chat-input").fill("Deserta la tarea");
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
  await expect(
    panel.locator("code").filter({ hasText: "parent-task-1234567890" }),
  ).toBeHidden();
  await expect(page.locator("#task-pending")).toContainText(
    "Write the report?",
  );
  await expect(page.locator("#task-pending")).toContainText("Which file?");
  await expect(page.locator("#task-pending")).not.toContainText(
    "approval-child-123",
  );
  await expect(page.locator("#task-pending")).not.toContainText(
    "question-child-456",
  );
  await expect(
    page.getByRole("button", { name: /aprobar|rechazar/i }),
  ).toHaveCount(0);
  await page.screenshot({
    path: testInfo.outputPath("background-tasks-desktop-light.png"),
    fullPage: true,
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator(".sidebar-toggle").click();
  await page.locator('.mobile-brand [data-action="theme"]').click();
  if (await page.locator(".mobile-menu-close").isVisible())
    await page.locator(".mobile-menu-close").click();
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
  await panel.locator(".subagent-result summary").click();
  await expect(panel.locator("pre")).toContainText("file.txt:2");
  await expect(panel.locator("report")).toHaveCount(0);
  await page.locator(".sidebar-toggle").click();
  await page.locator('.mobile-brand [data-action="language"]').click();
  if (await page.locator(".mobile-menu-close").isVisible())
    await page.locator(".mobile-menu-close").click();
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

async function mockSpeech(page) {
  await page.addInitScript(() => {
    class Recognition {
      start() {
        window.__recognition = this;
      }
      abort() {
        setTimeout(() => this.onend?.(), 0);
      }
    }
    window.SpeechRecognition = Recognition;
  });
}

test("dictation fills the draft and never submits it automatically", async ({
  page,
}) => {
  await mockSpeech(page);
  await page.goto("/");
  const before = (await (await page.request.get("/api/tasks")).json()).length;
  await page
    .getByRole("button", { name: "Dictar mensaje", exact: true })
    .click();
  await page.evaluate(() =>
    window.__recognition.onresult({
      resultIndex: 0,
      results: [{ isFinal: true, 0: { transcript: "Una nota dictada" } }],
    }),
  );
  await expect(page.locator("#chat-input")).toHaveValue("Una nota dictada");
  expect((await (await page.request.get("/api/tasks")).json()).length).toBe(
    before,
  );
  await page
    .getByRole("button", { name: "Dictar mensaje", exact: true })
    .click();
});

test("missing browser recognition leaves text chat usable", async ({
  page,
}) => {
  await page.addInitScript(() => {
    window.SpeechRecognition = undefined;
    window.webkitSpeechRecognition = undefined;
  });
  await page.goto("/");
  await expect(
    page.getByRole("button", { name: "Dictar mensaje" }),
  ).toBeDisabled();
  await expect(page.locator("#chat-input")).toBeEnabled();
  await expect(
    page.getByText(
      "El dictado no está disponible en este navegador. Puedes escribir tu mensaje.",
    ),
  ).toBeVisible();
});

test("denied dictation permission leaves text chat available", async ({
  page,
}) => {
  await mockSpeech(page);
  await page.addInitScript(() => {
    window.SpeechRecognition.prototype.start = function () {
      this.onerror?.({ error: "not-allowed" });
    };
  });
  await page.goto("/");
  await page
    .getByRole("button", { name: "Dictar mensaje", exact: true })
    .click();
  await expect(page.getByRole("alert")).toContainText(
    "Permite el acceso al micrófono",
  );
  await expect(page.locator("#chat-input")).toBeEnabled();
  await page
    .getByRole("alert")
    .getByRole("button", { name: "Cerrar", exact: true })
    .click();
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("agent-authored diagrams support exploration and preserve the last valid view", async ({
  page,
}, testInfo) => {
  test.setTimeout(90_000);
  const response = await page.request.post("/api/design/artifacts", {
    data: {
      title: "Purchasing approval",
      kind: "diagram",
      content: {
        graph: {
          version: 1,
          direction: "LR",
          groups: [{ id: "office", label: "Office" }],
          nodes: [
            {
              id: "request",
              label: 'Request "A" <B>',
              group: "office",
              fill: "#e8f2ec",
              stroke: "#2f7250",
              detail: "Employee provides the purchase amount.",
              references: ["Purchasing policy, section 2"],
            },
            { id: "threshold", label: "Over S/5,000?", shape: "diamond" },
            { id: "manager", label: "Manager approval", shape: "rectangle" },
            { id: "archive", label: "Records", shape: "database" },
            { id: "done", label: "Approved", shape: "circle" },
          ],
          edges: [
            { id: "submit", from: "request", to: "threshold" },
            { id: "escalate", from: "threshold", to: "manager", label: "Yes" },
            {
              id: "normal",
              from: "threshold",
              to: "archive",
              label: "No",
              style: "dashed",
            },
            { id: "record", from: "manager", to: "archive" },
            { id: "finish", from: "archive", to: "done" },
          ],
        },
      },
    },
  });
  expect(response.ok()).toBe(true);
  const diagram = await response.json();
  const path = `/api/design/artifacts/${diagram.id}`;
  const archive = join(
    directory,
    "data",
    "diagrams",
    diagram.id,
    "v000001.json",
  );
  expect(
    JSON.parse(await readFile(archive, "utf8")).content.graph.nodes,
  ).toHaveLength(5);
  // Finished results open from the chat in the right-hand panel.
  await page.goto("/chat");
  await page
    .locator(".chat-design-results button")
    .filter({ hasText: "Purchasing approval" })
    .first()
    .click();
  const panel = page.getByRole("dialog", { name: "Vista del diseño" });
  const image = panel.locator(".diagram-viewport img");
  await expect(image).toBeVisible({ timeout: 20_000 });
  await expect(image).toHaveJSProperty("complete", true);
  await expect(panel.getByRole("alert")).toHaveCount(0);
  const labels = await image.evaluate((img) => {
    const svg = decodeURIComponent(img.src.slice(img.src.indexOf(",") + 1));
    return [
      ...new DOMParser()
        .parseFromString(svg, "image/svg+xml")
        .querySelectorAll("text"),
    ]
      .map((n) => n.textContent)
      .join(" ");
  });
  expect(labels).toContain('"A"');
  expect(labels).toContain("<B>");
  expect(labels).not.toContain("&#34;");
  const zoom = panel.getByLabel("Zoom", { exact: true });
  const previous = await zoom.textContent();
  await panel.getByRole("button", { name: "Acercar", exact: true }).click();
  await expect(zoom).not.toHaveText(previous);
  const viewport = panel.locator(".diagram-viewport");
  const box = await viewport.boundingBox();
  const before = await image.getAttribute("style");
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(
    box.x + box.width / 2 + 50,
    box.y + box.height / 2 + 25,
  );
  await page.mouse.up();
  await expect(image).not.toHaveAttribute("style", before);
  expect((await (await page.request.get(path)).json()).content).toEqual(
    diagram.content,
  );
  await panel.getByRole("button", { name: "Ajustar", exact: true }).click();
  await panel
    .getByRole("button", { name: "Pantalla completa", exact: true })
    .click();
  await expect(
    panel.getByRole("button", {
      name: "Salir de pantalla completa",
      exact: true,
    }),
  ).toBeVisible();
  await panel
    .getByRole("button", { name: "Salir de pantalla completa", exact: true })
    .click();
  await panel.getByLabel("Buscar elementos del diagrama").fill("Employee");
  await expect(panel.locator(".diagram-node-list button")).toHaveCount(1);
  await panel.locator(".diagram-node-list button").click();
  await expect(panel.locator(".diagram-node-detail")).toContainText(
    "Purchasing policy, section 2",
  );
  const downloaded = page.waitForEvent("download");
  await panel.getByRole("button", { name: "SVG", exact: true }).click();
  expect((await downloaded).suggestedFilename()).toMatch(/\.svg$/);
  await panel.getByRole("button", { name: "Contenido", exact: true }).click();
  await expect(panel.locator("textarea")).toHaveAttribute("readonly", "");
  await expect(
    panel.getByRole("button", { name: "Guardar una nueva versión" }),
  ).toHaveCount(0);
  expect(
    (
      await page.request.put(path, {
        data: { ...diagram, expected_revision: 1 },
      })
    ).status(),
  ).toBe(400);
  await panel
    .getByRole("button", { name: "Vista previa", exact: true })
    .click();
  await expect(image).toBeVisible();
  const original = await image.getAttribute("src");
  // Simulate an incoming agent revision. An invalid render must retain the prior image.
  const broken = {
    ...diagram,
    revision: 2,
    task_id: "agent-revision",
    content: { source: "flowchart LR\n A[broken" },
  };
  await page.route(`**${path}`, (route) => route.fulfill({ json: broken }));
  await page.route("**/api/design/artifacts", (route) =>
    route.fulfill({ json: { designs: [broken], next_offset: null } }),
  );
  await expect(panel.getByRole("alert")).toContainText(
    "último diagrama válido",
    { timeout: 25_000 },
  );
  await expect(image).toHaveAttribute("src", original);
  await page.screenshot({
    path: testInfo.outputPath("diagram-read-only-desktop.png"),
  });
  await page.setViewportSize({ width: 390, height: 844 });
  const mobile = await panel.boundingBox();
  expect(mobile.x).toBe(0);
  expect(mobile.width).toBe(390);
  await expect(
    panel.getByRole("button", { name: "Pedir cambios en el chat" }),
  ).toBeVisible();
  await page.screenshot({
    path: testInfo.outputPath("diagram-read-only-mobile.png"),
  });
  await panel.getByRole("button", { name: "Pedir cambios en el chat" }).click();
  await expect(page.locator("#chat-input")).toHaveValue(new RegExp(diagram.id));
});

// Template previews open over the chat, the same way a saved choice link does.
async function previewTemplate(page, id) {
  await page.evaluate((id) => {
    const link = document.createElement("a");
    link.href = `/design?template=${id}`;
    document.body.append(link);
    link.click();
    link.remove();
  }, id);
  return page.locator(".design-modal");
}

test("template previews cover all eight formats in a centered modal over the chat", async ({
  page,
}, testInfo) => {
  test.setTimeout(120_000);
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/");
  await expect(page.locator('[data-page="design"]')).toHaveCount(0);
  const cases = [
    ["decision-flow", "Flujo de decisiones", ".diagram-canvas img"],
    ["saas-launch", "Lanzamiento SaaS", "iframe"],
    ["research-deck", "Resultados de investigación", ".slide"],
    ["weekly-sales", "Ventas semanales", ".chart-view svg"],
    ["executive-report", "Informe ejecutivo", ".template-document"],
    ["budget-sheet", "Presupuesto de proyecto", ".data-grid table"],
    ["event-poster", "Cartel de evento", ".image-preview"],
    ["motion-intro", "Introducción animada", "video"],
  ];
  for (const [id, title, selector] of cases) {
    const modal = await previewTemplate(page, id);
    await expect(
      modal.getByRole("heading", { name: title, exact: true }),
    ).toBeVisible();
    await expect(modal.locator(selector)).toBeVisible({ timeout: 20_000 });
    await expect(modal).toHaveAttribute("aria-modal", "true");
    await expect(page.locator(".workspace-chrome")).toHaveCount(0);
    if (id === "saas-launch") {
      const box = await modal.boundingBox();
      expect(Math.abs(box.x - (1280 - box.x - box.width))).toBeLessThan(2);
      await page.screenshot({ path: testInfo.outputPath("design-modal.png") });
    }
    if (id === "motion-intro")
      await expect
        .poll(() => modal.locator("video").evaluate((v) => v.readyState))
        .toBeGreaterThan(0);
    await page.keyboard.press("Escape");
    await expect(modal).toHaveCount(0);
  }
  await page.setViewportSize({ width: 390, height: 844 });
  const modal = await previewTemplate(page, "research-deck");
  await expect
    .poll(() => modal.boundingBox())
    .toEqual({ x: 0, y: 0, width: 390, height: 844 });
  await page.screenshot({ path: testInfo.outputPath("design-mobile.png") });
  await page.keyboard.press("Escape");
  expect(errors).toEqual([]);
});

test("choosing a template preview sends the choice through chat and results use the side panel", async ({
  page,
}) => {
  test.setTimeout(60_000);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/design?template=saas-launch");
  await expect(page).toHaveURL(/\/chat$/);
  const modal = page.getByRole("dialog", { name: "Lanzamiento SaaS" });
  await expect(
    modal.frameLocator("iframe").getByRole("heading").first(),
  ).toBeVisible({ timeout: 20_000 });
  await expect(modal.getByRole("button", { name: "Descargar" })).toHaveCount(0);
  const requests = [];
  await page.route("**/api/messages", async (route) => {
    if (route.request().method() !== "POST") return route.fallback();
    requests.push(route.request().postDataJSON());
    return route.fulfill({ json: {} });
  });
  await modal.getByRole("button", { name: "Elegir este diseño" }).click();
  await expect.poll(() => requests.length).toBe(1);
  expect(requests[0].design_choice).toEqual({
    mode: "template",
    template_id: "saas-launch",
  });
  await expect(page.locator(".workspace-chrome")).toHaveCount(0);
  await page.unroute("**/api/messages");
  // A finished result, unlike a proposal, opens in the chat's right-hand panel.
  const created = await (
    await page.request.post("/api/design/templates/saas-launch/use", {
      data: { project_id: null },
    })
  ).json();
  await page.reload();
  await page
    .locator(".chat-design-results button")
    .filter({ hasText: created.title_es || created.title })
    .first()
    .click();
  const panel = page.getByRole("dialog", { name: "Vista del diseño" });
  await expect(panel.locator("iframe")).toBeVisible({ timeout: 20_000 });
  await expect(page.locator(".workspace-chrome")).toBeVisible();
  await expect(page.locator(".design-modal-backdrop")).toHaveCount(0);
});

test("untrusted HTML results cannot access the app", async ({ page }) => {
  const response = await page.request.post("/api/design/artifacts", {
    data: {
      title: "Isolation check",
      kind: "web",
      content: {
        html: '<h1>Safe preview</h1><script>parent.document.body.dataset.compromised="yes"</script><img src="https://external.invalid/pixel"><a href="https://external.invalid">External</a>',
      },
    },
  });
  expect(response.ok()).toBeTruthy();
  await page.goto("/chat");
  await page
    .locator(".chat-design-results button")
    .filter({ hasText: "Isolation check" })
    .first()
    .click();
  await expect(
    page
      .frameLocator(".design-panel iframe")
      .getByRole("heading", { name: "Safe preview" }),
  ).toBeVisible();
  expect(
    await page.locator("body").getAttribute("data-compromised"),
  ).toBeNull();
  await expect(
    page.frameLocator(".design-panel iframe").locator("a"),
  ).not.toHaveAttribute("href", /external/);
});

test("Design catalog renders every template and can regenerate real thumbnails", async ({
  page,
}, testInfo) => {
  test.setTimeout(720_000);
  await page.goto("/");
  let entries = [];
  let offset = 0;
  do {
    const p = await (
      await page.request.get(`/api/design/templates?offset=${offset}`)
    ).json();
    entries.push(...p.templates);
    offset = p.next_offset;
  } while (offset !== null);
  expect(entries).toHaveLength(72);
  const output = process.env.UPDATE_DESIGN_THUMBNAILS
    ? resolve("public/assets/design-thumbnails")
    : testInfo.outputPath("thumbnails");
  await mkdir(output, { recursive: true });
  const selectedKinds = new Set(
    (process.env.DESIGN_TEMPLATE_KINDS || "").split(",").filter(Boolean),
  );
  for (const entry of entries.filter(
    (entry) => !selectedKinds.size || selectedKinds.has(entry.kind),
  )) {
    const panel = await previewTemplate(page, entry.id);
    await expect(
      panel.getByRole("heading", { name: entry.title_es, exact: true }),
    ).toBeVisible();
    if (entry.kind === "diagram") {
      await expect(panel.locator(".diagram-canvas img")).toBeVisible({
        timeout: 20000,
      });
      await expect
        .poll(() =>
          panel.locator(".diagram-canvas img").evaluate((n) => n.naturalWidth),
        )
        .toBeGreaterThan(0);
      const diagramText = await panel
        .locator(".diagram-canvas img")
        .evaluate((n) => {
          const svg = decodeURIComponent(n.src.split(",").slice(1).join(","));
          return [
            ...new DOMParser()
              .parseFromString(svg, "image/svg+xml")
              .querySelectorAll("text"),
          ]
            .map((node) => node.textContent)
            .join(" ");
        });
      expect(diagramText.trim().length).toBeGreaterThan(10);
      if (entry.id === "decision-flow")
        expect(diagramText).toContain("New request");
    }
    if (entry.kind === "web")
      await expect(panel.frameLocator("iframe").locator("h1")).toBeVisible();
    if (entry.kind === "video") {
      await expect
        .poll(() => panel.locator("video").evaluate((n) => n.readyState))
        .toBeGreaterThan(0);
      await panel.locator("video").evaluate((n) => {
        n.currentTime = 1.5;
      });
    }
    if (entry.kind === "image")
      await expect
        .poll(() =>
          panel.locator(".image-preview").evaluate((n) => n.naturalWidth),
        )
        .toBeGreaterThan(0);
    await expect(panel.locator('[role="alert"]')).toHaveCount(0);
    if (entry.kind === "chart" || entry.kind === "diagram") {
      await expect(panel.locator(".template-frame")).toHaveCSS(
        "background-color",
        "rgb(255, 255, 255)",
      );
    }
    if (entry.id === "pitch-deck") {
      await expect(panel.locator(".designed-slide")).toHaveCSS(
        "background-color",
        "rgb(24, 27, 48)",
      );
      await expect(panel.locator(".designed-slide")).toHaveCSS(
        "color",
        "rgb(246, 243, 255)",
      );
    }
    if (entry.kind === "presentation") {
      const template = await (
        await page.request.get(`/api/design/templates/${entry.id}`)
      ).json();
      for (const slide of template.content.slides) {
        await panel
          .getByRole("button", { name: slide.title, exact: true })
          .click();
        await expect(panel.locator(".designed-slide h2")).toHaveText(
          slide.title,
        );
        expect(
          await panel
            .locator(".designed-slide")
            .evaluate((node) => node.scrollWidth <= node.clientWidth + 2),
          `${entry.id}: ${slide.title}`,
        ).toBe(true);
      }
      await panel
        .getByRole("button", {
          name: template.content.slides[0].title,
          exact: true,
        })
        .click();
      await page.setViewportSize({ width: 390, height: 844 });
      await expect(panel.locator(".designed-slide h2")).toBeVisible();
      expect(
        await panel
          .locator(".designed-slide")
          .evaluate((node) => node.scrollWidth <= node.clientWidth + 2),
      ).toBe(true);
      await page.screenshot({
        path: testInfo.outputPath(`${entry.id}-mobile.png`),
      });
      await page.setViewportSize({ width: 1280, height: 720 });
    }
    // Render the real template at a consistent canvas size. Only screenshot framing
    // changes; template markup, content and styles remain the production renderer.
    // Chart covers show the data marks without the viewer's table controls.
    await page.setViewportSize({ width: 1440, height: 1000 });
    const framing = await page.addStyleTag({
      content: `
      .design-panel { width: 1000px !important; }
      .design-preview-content { width: 960px !important; height: 600px !important; min-height: 0 !important; overflow: hidden; }
      .design-frame { height: 600px !important; min-height: 0 !important; }
      .designed-slide { min-height: 600px !important; }
      .template-chart { min-height: 600px !important; }
      .template-chart .data-grid { display: none !important; }
    `,
    });
    // Fit tall documents and data canvases as complete compositions rather than
    // cutting off diagram nodes, table rows or document sections in the cover.
    await panel.locator(".design-preview-content").evaluate((canvas) => {
      canvas.scrollTop = 0;
      const paper = canvas.querySelector(
        ".template-frame, .template-document, .composed-slide",
      );
      if (!paper) return;
      const height = paper.getBoundingClientRect().height;
      if (height <= 600) return;
      const scale = 600 / height;
      canvas.style.position = "relative";
      canvas.style.background = getComputedStyle(paper).backgroundColor;
      Object.assign(paper.style, {
        width: "960px",
        position: "absolute",
        top: "0",
        left: `${(960 - 960 * scale) / 2}px`,
        transform: `scale(${scale})`,
        transformOrigin: "top left",
      });
    });
    await page.evaluate(
      () =>
        new Promise((resolve) =>
          requestAnimationFrame(() => requestAnimationFrame(resolve)),
        ),
    );
    const canvas = await panel.locator(".design-preview-content").boundingBox();
    await page.screenshot({
      path: join(output, `${entry.id}.png`),
      clip: {
        x: Math.floor(canvas.x),
        y: Math.floor(canvas.y),
        width: 960,
        height: 600,
      },
    });
    await framing.evaluate((node) => node.remove());
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.keyboard.press("Escape");
    await expect(panel).toHaveCount(0);
  }
});
