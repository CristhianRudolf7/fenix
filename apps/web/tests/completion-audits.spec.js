import { test, expect } from "@playwright/test";
import { preview } from "vite";

let server, url;
test.beforeAll(async () => {
  server = await preview({ preview: { host: "127.0.0.1", port: 0 } });
  url = server.resolvedUrls.local[0];
});
test.afterAll(async () => {
  if (server)
    await new Promise((resolve) => {
      server.httpServer.close(resolve);
      server.httpServer.closeAllConnections();
    });
});

test("tasks distinguish execution from verification and show evidence provenance", async ({
  page,
}) => {
  const audit = {
    id: "audit-1",
    node_id: "sources",
    status: "running",
    report: null,
  };
  const plan = {
    id: "plan-1",
    version: 1,
    status: "running",
    objective: "Monthly report",
    nodes: [
      {
        id: "sources",
        objective: "Read sources",
        expected_output: "Checked source data",
        status: "verifying",
        depends_on: [],
      },
    ],
    reviews: [],
    completion_audits: [audit],
    evidence_summary: { total: 3, verified: 1, pending: 1, doubtful: 1 },
  };
  await page.route("**/api/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    let body = [];
    if (path === "/api/auth/session")
      body = { email: "fixture@example.test", user_id: "fixture" };
    else if (path === "/api/status")
      body = {
        ready: true,
        settings: { model: "fixture" },
        projects: [],
        pending_requests: [],
        active_tasks: [],
        subagents: [],
        plans: [plan],
      };
    else if (path === "/api/plans/plan-1")
      body = {
        plan,
        revisions: [],
        reviews: [],
        completion_audits: [audit],
        evidence: [
          {
            id: "note:format",
            kind: "requirement",
            state: "doubtful",
            statement: "Export format",
            source: "Original owner request",
            question: "CSV or PDF?",
          },
        ],
      };
    else if (path === "/api/events" || path === "/api/progress") {
      await route.fulfill({
        contentType: "text/event-stream",
        body: ": connected\n\n",
      });
      return;
    }
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
  await page.goto(`${url}activity`, { waitUntil: "domcontentloaded" });
  await expect(page.locator("#plans-panel")).toContainText(
    "Pendiente de verificación",
  );
  await expect(page.locator("#plans-panel")).toContainText(
    "Verificación del resultado",
  );
  await expect(page.locator("#plans-panel")).toContainText("1 dudas");
  await page.getByRole("button", { name: "Ver revisiones" }).click();
  const dialog = page.locator("#detail-dialog");
  await dialog
    .getByText("Requisitos, artefactos y evidencia", { exact: true })
    .click();
  await expect(dialog).toContainText("Original owner request");
  await expect(dialog).toContainText("CSV or PDF?");
  await expect(dialog).toContainText("Duda pendiente");
});
