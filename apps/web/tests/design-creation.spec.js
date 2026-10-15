import { test, expect } from "@playwright/test";
import { preview } from "vite";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

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

const metadata = (id) => {
  const { content, ...entry } = catalog.find((v) => v.id === id);
  return { ...entry, project_id: "atlas" };
};
const proposals = [
  "saas-launch",
  "portfolio",
  "restaurant",
  "conference",
  "pricing-page",
];

async function fixture(
  page,
  {
    offers = [
      {
        id: "offer-web",
        task_id: "design-task",
        project_id: "atlas",
        templates: proposals.map(metadata),
        created_at: "2026-09-21T10:02:00Z",
      },
    ],
    messages = [
      {
        seq: 1,
        id: "message-1",
        role: "user",
        text: "Quiero una página web para mi estudio",
        task_id: "design-task",
        created_at: "2026-09-21T10:00:00Z",
      },
      {
        seq: 2,
        id: "message-2",
        role: "assistant",
        text: "Estas propuestas encajan con un estilo sobrio en verde. ¿Cuál prefieres?",
        task_id: "design-task",
        created_at: "2026-09-21T10:02:05Z",
      },
    ],
    messageFailures = 0,
    favoriteFailures = 0,
  } = {},
) {
  const sent = [],
    favorites = new Set(["portfolio"]);
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
        model_health: {
          can_send: true,
          roles: {
            intelligent: { status: "available" },
            fast: { status: "available" },
          },
        },
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
    else if (path.pathname === "/api/model-roles/fast")
      body = { configured: false, settings: {} };
    else if (path.pathname === "/api/design/artifacts")
      body = { designs: [], next_offset: null };
    else if (path.pathname === "/api/design/offers") body = offers;
    else if (path.pathname === "/api/design/favorites")
      body = [...favorites].map((id) => ({ id }));
    else if (path.pathname.startsWith("/api/design/favorites/")) {
      if (favoriteFailures > 0) {
        favoriteFailures -= 1;
        status = 503;
        body = { error: "Favorites unavailable" };
      } else {
        const id = path.pathname.split("/").at(-1);
        if (request.postDataJSON().enabled) favorites.add(id);
        else favorites.delete(id);
        body = { id };
      }
    } else if (path.pathname.startsWith("/api/design/templates/"))
      body = catalog.find((v) => v.id === path.pathname.split("/").at(-1));
    else if (path.pathname === "/api/documents")
      body = { documents: [], next_offset: null };
    else if (path.pathname === "/api/messages") {
      if (request.method() === "POST") {
        sent.push(request.postDataJSON());
        if (messageFailures > 0) {
          messageFailures -= 1;
          status = 503;
          body = { error: "No se pudo enviar. Inténtalo otra vez." };
        } else body = { task_id: "design-task" };
      } else body = messages;
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
  return { sent, favorites };
}

test("Design is no longer a page: navigation has no entry and old URLs land in chat", async ({
  page,
}) => {
  await fixture(page, { offers: [] });
  await expect(page.locator('a[data-page="skills"]')).toBeVisible();
  await expect(page.locator('a[data-page="design"]')).toHaveCount(0);
  await page.goto(`${url}design`);
  await expect(page).toHaveURL(/\/chat$/);
  await expect(page.locator("#chat-input")).toBeVisible();
  await expect(page.locator(".design-modal")).toHaveCount(0);
});

test("an old template link opens its preview over the chat", async ({
  page,
}) => {
  await fixture(page, { offers: [] });
  await page.goto(`${url}design?template=research-deck`);
  await expect(page).toHaveURL(/\/chat$/);
  const modal = page.getByRole("dialog", {
    name: "Resultados de investigación",
  });
  await expect(modal).toBeVisible();
  await expect(modal).toHaveAttribute("aria-modal", "true");
  await expect(modal.locator(".slides-view")).toContainText(
    "Better questions.",
  );
  await expect(page.locator("#chat-input")).toBeAttached();
  await expect(page.locator("html")).not.toHaveClass(/workspace-open/);
});

test("agent proposals appear as an in-chat carousel and a choice continues the conversation", async ({
  page,
}, info) => {
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const { sent } = await fixture(page);
  const reply = page.locator("article.message.assistant", {
    hasText: "¿Cuál prefieres?",
  });
  const carousel = reply.getByRole("region", {
    name: "Propuestas de diseño",
  });
  await expect(carousel).toBeVisible();
  const cards = carousel.locator(".design-offer-card");
  await expect(cards).toHaveCount(5);
  // Cards show real template covers, not a text-only list.
  for (const cover of await carousel.locator("img.design-thumbnail").all()) {
    await cover.scrollIntoViewIfNeeded();
    await expect(cover).toHaveJSProperty("naturalWidth", 960);
  }
  await expect(cards.first()).toContainText("Lanzamiento SaaS");
  await expect(
    carousel.getByRole("button", { name: "Favorito: Portafolio creativo" }),
  ).toHaveAttribute("aria-pressed", "true");
  // Choosing never uses the right-hand workspace panel.
  await expect(page.locator(".workspace-chrome")).toHaveCount(0);
  await expect(page.locator("html")).not.toHaveClass(/workspace-open/);
  await page.screenshot({ path: info.outputPath("proposals-desktop.png") });

  await carousel
    .getByRole("button", { name: "1. Lanzamiento SaaS · Abrir vista previa" })
    .click();
  const modal = page.getByRole("dialog", { name: "Lanzamiento SaaS" });
  await expect(modal).toBeVisible();
  await expect(
    modal.frameLocator("iframe").getByRole("heading").first(),
  ).toBeVisible({ timeout: 20_000 });
  const box = await modal.boundingBox();
  expect(Math.abs(box.x - (1440 - box.x - box.width))).toBeLessThan(2);
  await expect(page.locator(".workspace-chrome")).toHaveCount(0);
  await expect(
    modal.getByRole("button", { name: "Cerrar vista" }),
  ).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(modal).toHaveCount(0);
  await expect(
    carousel.getByRole("button", {
      name: "1. Lanzamiento SaaS · Abrir vista previa",
    }),
  ).toBeFocused();

  await carousel
    .getByRole("button", {
      name: "3. Restaurante · Abrir vista previa",
    })
    .click();
  await page
    .getByRole("dialog", { name: "Restaurante" })
    .getByRole("button", { name: "Elegir este diseño" })
    .click();
  await expect(page.locator(".design-modal")).toHaveCount(0);
  await expect.poll(() => sent.length).toBe(1);
  expect(sent[0]).toEqual({
    text: "Elijo el diseño «Restaurante».",
    document_ids: [],
    reply_to_task_id: null,
    design_choice: { mode: "template", template_id: "restaurant" },
  });
  await expect(page.locator("#chat-template-context")).toBeHidden();
  await expect(page.locator("html")).not.toHaveClass(/workspace-open/);
  expect(errors).toEqual([]);
});

test("the carousel scrolls within the chat on a phone without widening the page", async ({
  page,
}, info) => {
  await fixture(page);
  await page.setViewportSize({ width: 390, height: 844 });
  const carousel = page.getByRole("region", { name: "Propuestas de diseño" });
  const track = carousel.getByRole("list", { name: "Propuestas de diseño" });
  await expect(carousel.locator(".design-offer-card")).toHaveCount(5);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth + 1,
    ),
  ).toBe(true);
  expect(await track.evaluate((n) => n.scrollWidth > n.clientWidth)).toBe(true);
  const previous = carousel.getByRole("button", {
    name: "Propuestas anteriores",
  });
  await expect(previous).toBeDisabled();
  await carousel.getByRole("button", { name: "Más propuestas" }).click();
  await expect
    .poll(() => track.evaluate((n) => n.scrollLeft))
    .toBeGreaterThan(0);
  await expect(previous).toBeEnabled();
  await page.screenshot({ path: info.outputPath("proposals-mobile.png") });
  await carousel
    .getByRole("button", {
      name: "2. Portafolio creativo · Abrir vista previa",
    })
    .click();
  await expect(
    page.getByRole("dialog", { name: "Portafolio creativo" }),
  ).toBeVisible();
  await expect
    .poll(() => page.locator(".design-modal").boundingBox())
    .toEqual({ x: 0, y: 0, width: 390, height: 844 });
});

test("a failed choice stays attached for the next message and favorites roll back on errors", async ({
  page,
}) => {
  const { sent, favorites } = await fixture(page, {
    messageFailures: 1,
    favoriteFailures: 1,
  });
  const carousel = page.getByRole("region", { name: "Propuestas de diseño" });
  const star = carousel.getByRole("button", {
    name: "Favorito: Conferencia",
  });
  await star.click();
  await expect(carousel.getByRole("status")).toContainText(
    "No se pudo actualizar el favorito",
  );
  await expect(star).toHaveAttribute("aria-pressed", "false");
  await star.click();
  await expect(star).toHaveAttribute("aria-pressed", "true");
  expect(favorites.has("conference")).toBe(true);

  await carousel
    .getByRole("button", { name: "4. Conferencia · Abrir vista previa" })
    .click();
  await page.getByRole("button", { name: "Elegir este diseño" }).click();
  await expect(page.locator("#chat-template-context")).toContainText(
    "Conferencia",
  );
  await expect(page.locator("#toast")).toContainText("No se pudo enviar");
  await expect(page.locator("#chat-input")).toBeFocused();
  await page.locator("#chat-input").fill("Sí, usa esa con fondo claro");
  await page.locator("#chat-form .send").click();
  await expect.poll(() => sent.length).toBe(2);
  expect(sent[1]).toEqual({
    text: "Sí, usa esa con fondo claro",
    document_ids: [],
    reply_to_task_id: null,
    design_choice: { mode: "template", template_id: "conference" },
  });
  await expect(page.locator("#chat-template-context")).toBeHidden();
});

test("a chosen design link in history reopens its preview in place", async ({
  page,
}) => {
  await fixture(page, {
    offers: [],
    messages: [
      {
        seq: 1,
        id: "message-1",
        role: "user",
        text: "Elijo el diseño «Presentación de proyecto».\n\nPlantilla elegida / Selected template: [Presentación de proyecto / Project pitch](/design?template=pitch-deck).",
        task_id: "design-task",
        created_at: "2026-09-21T10:00:00Z",
      },
    ],
  });
  const pages = [];
  page.context().on("page", (p) => pages.push(p));
  await page
    .locator(".message.user a", { hasText: "Presentación de proyecto" })
    .click();
  const modal = page.getByRole("dialog", { name: "Presentación de proyecto" });
  await expect(modal.locator(".designed-slide")).toBeVisible();
  await expect(page).toHaveURL(/\/chat$/);
  expect(pages).toEqual([]);
});

test("design requests go directly to the agent without a side-panel picker", async ({
  page,
}) => {
  const { sent } = await fixture(page, { offers: [] });
  for (const text of [
    "Genera una presentación sobre energía limpia",
    "Crea una landing page y verifica el diseño responsive",
  ]) {
    await page.locator("#chat-input").fill(text);
    await page
      .locator("#chat-form")
      .getByRole("button", { name: "Enviar mensaje" })
      .click();
    await expect.poll(() => sent.at(-1)?.text).toBe(text);
    expect(sent.at(-1).design_choice).toBeUndefined();
    await expect(page.locator("#chat-input")).toHaveValue("");
    await expect(page.locator(".design-panel")).toHaveCount(0);
    await expect(page.locator("html")).not.toHaveClass(/workspace-open/);
  }
});
