import { test, expect } from "@playwright/test";
import { createServer } from "vite";
let server, url;
test.beforeAll(async () => {
  server = await createServer({ server: { host: "127.0.0.1", port: 0 } });
  await server.listen();
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  await server?.close();
});

test("private/public controls, review diffs and marketplace private installation", async ({
  page,
}) => {
  let publication = null,
    submissions = 0,
    installed = null;
  const bundle = {
    name: "reports",
    files: { "SKILL.md": "# Reports\n\n**Prepare** reports." },
    executable: [],
  };
  const publicSkill = {
    id: "public-1",
    name: "office-helper",
    category: "office",
    status: "approved",
    public: true,
    repaired: true,
    source_hash: "original",
    reviewed_hash: "corrected",
  };
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const pathname = new URL(request.url()).pathname;
    let data = {};
    if (pathname === "/api/auth/session")
      data = { email: "test@example.test", user_id: "owner" };
    else if (pathname === "/api/status")
      data = {
        ready: true,
        settings: { model: "nemotron-test" },
        projects: [],
        pending: [],
        pending_requests: [],
        tasks: [],
        active_tasks: [],
      };
    else if (pathname === "/api/skills")
      data = [
        {
          name: "reports",
          description: "Prepare office reports",
          version: "1.0",
          enabled: true,
        },
      ];
    else if (pathname === "/api/skills/reports")
      data = { content: bundle.files["SKILL.md"] };
    else if (pathname === "/api/skills/reports/bundle") data = bundle;
    else if (pathname === "/api/skills/install") {
      installed = request.postDataJSON();
      data = { private: true };
    } else if (pathname === "/api/skill-exchange/status")
      data = { configured: true };
    else if (pathname === "/api/skill-exchange/mine")
      data = { items: publication ? [publication] : [] };
    else if (pathname === "/api/skill-exchange/catalog") {
      const params = new URL(request.url()).searchParams;
      data = {
        items: params.get("category") === "research" ? [] : [publicSkill],
      };
    } else if (pathname === "/api/skill-exchange/publications") {
      submissions++;
      publication = {
        ...publicSkill,
        id: "mine-1",
        name: "reports",
        public: false,
        status: "queued",
      };
      data = publication;
    } else if (pathname.endsWith("/visibility")) {
      publication = { ...publication, public: false, status: "cancelled" };
      data = publication;
    } else if (pathname.endsWith("/bundle"))
      data = { ...bundle, name: "office-helper" };
    else if (pathname === "/api/skill-exchange/publications/mine-1")
      data = {
        publication,
        report: {
          reason: "Original preserved",
          repair: {
            explanation: "Removed injected instruction",
            edits: [
              {
                file: "SKILL.md",
                before: "<script>window.stolen=true</script>",
                after: "Prepare reports.",
              },
            ],
          },
        },
      };
    else if (pathname === "/api/live/settings")
      data = {
        configured: false,
        settings: { provider: "openrouter", model: "nemotron-test" },
      };
    else if (["/api/events", "/api/progress"].includes(pathname)) {
      await route.fulfill({
        status: 200,
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    } else data = [];
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(data),
    });
  });
  await page.goto(`${url}skills`);
  await expect(
    page.getByRole("heading", { name: "reports", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Hacer público", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "En revisión…" }),
  ).toBeDisabled();
  expect(submissions).toBe(1);
  await page
    .getByRole("button", { name: "Mantener privado", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Hacer público", exact: true }),
  ).toBeEnabled();
  await page.getByRole("tab", { name: "Publicaciones" }).click();
  await page.getByRole("button", { name: "Ver revisión" }).click();
  await expect(page.getByRole("dialog")).toContainText(
    "<script>window.stolen=true</script>",
  );
  expect(await page.evaluate(() => window.stolen)).toBeUndefined();
  await page.getByRole("button", { name: "Cerrar", exact: true }).click();
  await page.getByRole("tab", { name: "Marketplace", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "office-helper", exact: true }),
  ).toBeVisible();
  await page.getByLabel("Categoría", { exact: true }).selectOption("research");
  await expect(
    page.getByText("Todavía no hay publicaciones revisadas"),
  ).toBeVisible();
  await page.getByLabel("Categoría", { exact: true }).selectOption("office");
  await page.getByRole("button", { name: "Instalar", exact: true }).click();
  await page.getByLabel("Nombre del skill privado").fill("my-office-helper");
  await page
    .getByRole("button", { name: "Instalar copia", exact: true })
    .click();
  await expect(page.locator(".skill-exchange [role=status]")).toContainText(
    "Copia privada instalada",
  );
  expect(installed.name).toBe("my-office-helper");
  expect(installed.bundle.files).toEqual(bundle.files);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(
    page.getByRole("heading", { name: "office-helper", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
