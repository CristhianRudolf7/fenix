import { test, expect } from "@playwright/test";
import { preview } from "vite";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { creationKind } from "../src/design/creation-intent.ts";

let server, url, catalog;
test.beforeAll(async () => {
  catalog = JSON.parse(
    await readFile(resolve("../../resources/designs/catalog.json"), "utf8"),
  );
  server = await preview({ preview: { host: "127.0.0.1", port: 0 } });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  await new Promise((done) => {
    server.httpServer.close(done);
    server.httpServer.closeAllConnections();
  });
});

test("new visual requests are distinguished from edits, explanations and explicit custom work", () => {
  for (const [text, kind] of [
    ["Oye, genérame una diapositiva sobre energía", "presentation"],
    ["Quiero una interfaz para una cafetería", "web"],
    ["Crea un PDF para el proyecto Atlas", "document"],
    ["Build a landing page for our studio", "web"],
    ["Generate a slide deck about water", "presentation"],
    ["Crea una presentación como un diario cultural", "presentation"],
    ["Crea una interfaz para mejorar la atención al cliente", "web"],
    ["Crea un gráfico de los datos de esta presentación", "chart"],
    ["Modifica la diapositiva 4", null],
    ["Quiero que edites esta presentación", null],
    ["Necesito cambiar el título del PDF", null],
    ["Necesito resumir este documento", null],
    ["¿Cómo crear una presentación?", null],
    ["No quiero generar una interfaz todavía", null],
    ["Crea una presentación sin plantilla", null],
    ["Build a website from scratch", null],
    ["Genera otra diapositiva (design_id: existing)", null],
    ["Me gustó la presentación de ayer", null],
    [
      "solamente puedes crear presentaciones o qué otras funcionalidades tienes",
      null,
    ],
    ["¿Qué puedes crear?", null],
    ["¿Puedes crear presentaciones?", null],
    ["Can you create presentations?", null],
    ["Quiero saber si puedes crear una presentación", null],
    ["Necesito aprender cómo crear un PDF", null],
    ["¿Qué otras capacidades tienes para generar imágenes?", null],
    ["¿Puedes crear una presentación sobre energía solar?", "presentation"],
    ["Genera un gráfico con las ventas de enero", "chart"],
  ])
    expect(creationKind(text), text).toBe(kind);
});

async function fixture(
  page,
  { catalogFailure = false, messageFailure = false } = {},
) {
  const sent = [],
    offsets = [];
  let failMessage = messageFailure;
  const generated = structuredClone(
    catalog.find((v) => v.id === "research-deck"),
  );
  generated.id = "generated-slides";
  generated.title = "Energía limpia para Atlas";
  generated.project_id = "atlas";
  generated.revision = 1;
  generated.content.slides[0].title = "Energía limpia para Atlas";
  await page.addInitScript(() => {
    window.EventSource = class extends EventTarget {
      constructor(url) {
        super();
        if (url.includes("/api/events")) window.creationEvents = this;
      }
      close() {}
    };
  });
  await page.route("**/api/**", async (route) => {
    const request = route.request(),
      path = new URL(request.url());
    let body = [],
      status = 200;
    if (path.pathname === "/api/auth/session")
      body = { email: "demo@example.test", user_id: "owner" };
    else if (path.pathname === "/api/status")
      body = {
        ready: true,
        session_id: "creation-test",
        event_cursor: 40,
        focus_project_id: "atlas",
        settings: { model: "test" },
        projects: [{ id: "atlas", name: "Atlas" }],
        pending_requests: [],
        active_tasks: [],
        subagents: [],
      };
    else if (path.pathname === "/api/projects")
      body = [{ id: "atlas", name: "Atlas" }];
    else if (path.pathname === "/api/live/settings")
      body = { configured: false, settings: {} };
    else if (path.pathname === "/api/design/artifacts")
      body = { designs: [], next_offset: null };
    else if (path.pathname === "/api/design/artifacts/generated-slides")
      body = generated;
    else if (path.pathname === "/api/design/templates") {
      if (catalogFailure) {
        status = 503;
        body = { error: "Catalog temporarily unavailable" };
      } else {
        const offset = Number(path.searchParams.get("offset") || 0);
        offsets.push(offset);
        const kind = path.searchParams.get("kind");
        const entries = catalog.filter((v) => !kind || v.kind === kind);
        body = {
          templates: entries
            .slice(offset, offset + 24)
            .map(({ content, ...v }) => v),
          total: entries.length,
          next_offset: offset + 24 < entries.length ? offset + 24 : null,
        };
      }
    } else if (path.pathname.startsWith("/api/design/templates/"))
      body = catalog.find((v) => v.id === path.pathname.split("/").at(-1));
    else if (path.pathname === "/api/documents")
      body = {
        documents: [
          {
            id: "source-1",
            title: "Notas de energía",
            filename: "energy.md",
            collection: "",
            tags: [],
            warnings: [],
            embedding_status: "ready",
            bytes: 120,
            project_id: "atlas",
            enabled: true,
            revision: 1,
            format: "md",
            status: "indexed",
          },
        ],
        next_offset: null,
      };
    else if (path.pathname === "/api/messages") {
      if (request.method() === "POST") {
        sent.push(request.postDataJSON());
        if (failMessage) {
          failMessage = false;
          status = 503;
          body = { error: "No se pudo enviar. Inténtalo otra vez." };
        } else body = { task_id: "creation-task" };
      } else
        body = [
          {
            seq: 1,
            id: "message-1",
            role: "assistant",
            text: "Podemos continuar con Atlas.",
            task_id: "parent-task",
            created_at: "2026-09-21T10:00:00Z",
          },
        ];
    }
    await route.fulfill({
      status,
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(`${url}chat`);
  await expect(page.locator("#chat-input")).toBeVisible();
  return { sent, offsets };
}

test("sending design requests goes directly to the agent without opening the picker", async ({
  page,
}) => {
  const { sent } = await fixture(page);
  for (const text of [
    "Genera una presentación sobre energía limpia",
    "Crea una landing page y verifica el diseño responsive",
    "Genera un PDF",
    "Hola, qué puedes hacer",
  ]) {
    await page.locator("#chat-input").fill(text);
    await page
      .locator("#chat-form")
      .getByRole("button", { name: "Enviar mensaje" })
      .click();
    await expect.poll(() => sent.at(-1)?.text).toBe(text);
    expect(sent.at(-1).text).toBe(text);
    await expect(page.locator("#chat-input")).toHaveValue("");
    await expect(page.locator(".design-creation")).toHaveCount(0);
    await expect(page.getByText("Elegir diseño", { exact: true })).toHaveCount(
      0,
    );
  }
});

async function requestDesign(page, text) {
  await page.locator("#chat-input").fill(text);
  await page.evaluate(() =>
    window.dispatchEvent(new Event("fenix:choose-design")),
  );
  return page.getByRole("dialog", { name: "Elegir diseño", exact: true });
}

test("explicit template selection preserves sources and reply context before showing the generated design", async ({
  page,
}, info) => {
  const { sent } = await fixture(page);
  await page.locator('a[data-page="documents"]').click();
  await page
    .getByRole("button", { name: "Preguntar a Fénix", exact: true })
    .click();
  await page.getByRole("button", { name: "Responder", exact: true }).click();
  const panel = await requestDesign(
    page,
    "Genera una presentación sobre energía limpia para Atlas",
  );
  await expect(panel).toBeVisible();
  await expect(panel.locator(".design-creation-card")).toHaveCount(10);
  await expect(page.locator("#chat-form .send svg")).toBeVisible();
  expect(sent).toEqual([]);
  const chat = await page.locator("#chat-form").boundingBox();
  const picker = await panel.boundingBox();
  expect(chat.x + chat.width).toBeLessThanOrEqual(picker.x + 2);
  await panel
    .getByRole("button", {
      name: "Seleccionar Resultados de investigación",
      exact: true,
    })
    .click();
  await expect(
    panel.getByRole("button", {
      name: "Seleccionar Resultados de investigación",
      exact: true,
    }),
  ).toHaveAttribute("aria-pressed", "true");
  await page.screenshot({ path: info.outputPath("design-choice-desktop.png") });
  await panel
    .getByRole("button", { name: "Generar con esta plantilla", exact: true })
    .evaluate((button) => {
      button.click();
      button.click();
    });
  await expect(panel).toBeHidden();
  expect(sent).toEqual([
    {
      text: "Genera una presentación sobre energía limpia para Atlas",
      document_ids: ["source-1"],
      reply_to_task_id: "parent-task",
      design_choice: { mode: "template", template_id: "research-deck" },
    },
  ]);
  await expect(page.locator("#chat-input")).toHaveValue("");
  await page.evaluate(() =>
    window.creationEvents.dispatchEvent(
      new MessageEvent("activity", {
        data: JSON.stringify({
          seq: 41,
          topic: "workspace.open",
          data: {
            kind: "design",
            id: "generated-slides",
            title: "Energía limpia para Atlas",
            project_id: "atlas",
          },
        }),
        lastEventId: "41",
      }),
    ),
  );
  await expect(page.locator(".slides-view")).toContainText(
    "Energía limpia para Atlas",
  );
  await page.screenshot({ path: info.outputPath("design-choice-result.png") });
});

test("all catalog pages, search, custom description and a phone-sized chooser", async ({
  page,
}, info) => {
  const { sent, offsets } = await fixture(page);
  const panel = await requestDesign(
    page,
    "Genera una interfaz para una cafetería",
  );
  await expect(panel.locator(".design-creation-card")).toHaveCount(20);
  await panel.getByLabel("Categoría de plantilla").selectOption("");
  await expect(panel.locator(".design-creation-card")).toHaveCount(72);
  expect(offsets).toContain(48);
  await panel.getByLabel("Buscar plantillas").fill("topográfico");
  await expect(panel.locator(".design-creation-card")).toHaveCount(1);
  await panel.getByLabel("Buscar plantillas").fill("");
  await panel.getByLabel("Categoría de plantilla").selectOption("web");
  const brief =
    "Crea una interfaz para una cafetería, con tipografía editorial y fotos grandes";
  await panel.getByLabel("Tu descripción").fill(brief);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(panel).toBeVisible();
  expect(await panel.evaluate((n) => n.scrollWidth <= n.clientWidth + 1)).toBe(
    true,
  );
  await page.screenshot({ path: info.outputPath("design-choice-mobile.png") });
  await page.keyboard.press("Escape");
  await expect(panel).toBeHidden();
  await expect(page.locator("#chat-input")).toHaveValue(brief);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth + 1,
    ),
  ).toBe(true);
  await expect(page.locator("#chat-form .send svg")).toBeVisible();
  expect(sent).toHaveLength(0);
  await page.evaluate(() =>
    window.dispatchEvent(new Event("fenix:choose-design")),
  );
  await expect(panel).toBeVisible();
  await panel
    .getByRole("button", { name: "Generar sin plantilla", exact: true })
    .click();
  await expect(panel).toBeHidden();
  expect(sent).toEqual([
    {
      text: brief,
      document_ids: [],
      reply_to_task_id: null,
      design_choice: { mode: "custom" },
    },
  ]);
});

test("catalog failures allow custom work and failed submission retains the draft for retry", async ({
  page,
}) => {
  const { sent } = await fixture(page, {
    catalogFailure: true,
    messageFailure: true,
  });
  const panel = await requestDesign(
    page,
    "Crea una diapositiva sobre reciclaje",
  );
  await expect(panel.getByRole("alert")).toContainText("No se pudieron cargar");
  await panel
    .getByRole("button", { name: "Generar sin plantilla", exact: true })
    .click();
  await expect(panel.locator("footer").getByRole("alert")).toContainText(
    "No se pudo enviar",
  );
  await expect(panel.getByLabel("Tu descripción")).toHaveValue(
    "Crea una diapositiva sobre reciclaje",
  );
  await panel
    .getByRole("button", { name: "Generar sin plantilla", exact: true })
    .click();
  await expect(panel).toBeHidden();
  expect(sent).toHaveLength(2);
  expect(sent[0]).toEqual(sent[1]);
});

test("editing and explicit no-template requests go straight to the agent", async ({
  page,
}) => {
  const { sent } = await fixture(page);
  for (const text of [
    "Modifica la diapositiva 4 para mostrar las cifras",
    "Genera una interfaz desde cero con fondo azul",
  ]) {
    await page.locator("#chat-input").fill(text);
    await page.locator("#chat-form .send").click();
    await expect(page.locator("#chat-input")).toHaveValue("");
    await expect(
      page.getByRole("dialog", { name: "Elegir diseño", exact: true }),
    ).toBeHidden();
  }
  expect(sent.map((v) => v.text)).toEqual([
    "Modifica la diapositiva 4 para mostrar las cifras",
    "Genera una interfaz desde cero con fondo azul",
  ]);
  expect(sent.every((v) => !v.design_choice)).toBe(true);
});

test("a saved selection link reopens the exact template", async ({ page }) => {
  await fixture(page);
  await page.goto(`${url}design?template=research-deck`);
  const panel = page.getByRole("dialog", {
    name: "Vista del diseño",
    exact: true,
  });
  await expect(panel).toBeVisible();
  await expect(
    panel.getByRole("heading", {
      name: "Better questions. Clearer evidence.",
      exact: true,
    }),
  ).toBeVisible();
  await expect(panel.locator(".slides-view")).toContainText(
    "Better questions.",
  );
});
