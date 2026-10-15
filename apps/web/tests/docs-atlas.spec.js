import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
const base = process.env.FENIX_DOCS_TEST_URL;
test.skip(!base, "Serve the built documentation and set FENIX_DOCS_TEST_URL.");
const inventory = base
  ? JSON.parse(
      await readFile(
        new URL("../../docs/src/data/atlas-inventory.json", import.meta.url),
        "utf8",
      ),
    )
  : { files: [] };
test("atlas covers backend files, opens processes and searches symbols", async ({
  page,
}) => {
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto(`${base}/atlas/`);
  await expect(page.locator(".map-node")).toHaveCount(16);
  await expect(page.locator("[data-file]")).toHaveCount(inventory.files.length);
  await page.locator('.map-node[data-area="browser"]').click();
  await expect(page.locator('[data-panel="browser"]')).toBeVisible();
  await expect(
    page.locator(
      '[data-panel="browser"] [data-file="crates/fenix-browser/src/tools.rs"]',
    ),
  ).toHaveCount(1);
  await page
    .locator('[data-panel="browser"] .source-file summary')
    .first()
    .click();
  await expect(
    page.locator('[data-panel="browser"] .file-body').first(),
  ).toBeVisible();
  await page.reload();
  await expect(page.locator('[data-panel="browser"]')).toBeVisible();
  await page.locator("#atlas-search").fill("build_context");
  await expect(page.locator(".search-result")).not.toHaveCount(0);
  await page.locator(".search-result").first().click();
  await expect(page.locator('[data-panel="context"]')).toBeVisible();
  await expect(
    page.locator('[data-panel="context"] details[open]'),
  ).toHaveCount(1);
  await page.locator("#atlas-reset").click();
  await page.locator("#diagram-in").click();
  await expect(page.locator("#route-diagram")).toHaveAttribute("style", /125%/);
  await page.locator("#diagram-fit").click();
  await page.screenshot({
    path: "/tmp/fenix-atlas-desktop.png",
    fullPage: true,
  });
  expect(errors).toEqual([]);
});
test("atlas works on mobile without overflowing the page", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${base}/atlas/`);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.locator('.map-node[data-area="planning"]').click();
  await expect(page.locator('[data-panel="planning"]')).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: "/tmp/fenix-atlas-mobile.png",
    fullPage: true,
  });
});
test("landing links to the atlas and keeps mobile navigation inside the viewport", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(base);
  await expect(page.locator('nav a[href="/atlas/"]')).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.locator('nav a[href="/atlas/"]').click();
  await expect(page.locator("#atlas-overview")).toBeVisible();
});
