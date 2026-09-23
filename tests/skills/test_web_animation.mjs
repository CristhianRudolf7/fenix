/** Browser checks against a separately installed GSAP package; no production dependency changes. */
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(path.join(root, "apps/web/package.json"));
const { chromium } = require("@playwright/test");
const packagePath = process.argv[2];
assert(packagePath, "Pass the path to an isolated node_modules/gsap directory");
const source = await readFile(path.join(root, "resources/skills/web-animation/assets/scoped-reveal.mjs"), "utf8");
const gsapPackage = JSON.parse(await readFile(path.join(packagePath, "package.json"), "utf8"));
const checks = [];
const browser = await chromium.launch({ executablePath: "/usr/bin/chromium", headless: true, args: ["--no-sandbox"] });
const markup = `<main><section id="one"><p data-reveal style="color: rgb(1, 2, 3)">First readable text</p></section>
<section id="two"><p data-reveal>Second readable text</p></section><p id="outside" data-reveal>Outside</p></main>`;

async function pageWithMotion(reducedMotion) {
  const page = await browser.newPage({ reducedMotion });
  await page.setContent(markup);
  await page.addScriptTag({ path: path.join(packagePath, "dist/gsap.min.js") });
  await page.addScriptTag({ content: source.replace("export function", "function") + "\nwindow.mountReveal = mountReveal;" });
  return page;
}

try {
  const page = await pageWithMotion("no-preference");
  await page.evaluate(() => { window.dispose = mountReveal(gsap, document.querySelector("#one"), { duration: 0.1 }); });
  assert.equal(await page.locator("#outside").getAttribute("style"), null);
  assert.equal(await page.locator("#two p").getAttribute("style"), null);
  checks.push("selectors remain inside the supplied root");
  await page.waitForFunction(() => Number(getComputedStyle(document.querySelector("#one p")).opacity) === 1);
  assert.equal(await page.locator("#one p").textContent(), "First readable text");
  checks.push("normal animation reaches its readable end state");
  await page.evaluate(() => { dispose(); dispose(); });
  const styles = await page.locator("#one p").evaluate(el => [el.style.opacity, el.style.transform, el.style.color]);
  assert.deepEqual(styles, ["", "", "rgb(1, 2, 3)"]);
  checks.push("idempotent cleanup restores preexisting inline styles");
  assert.equal(await page.evaluate(() => gsap.globalTimeline.getChildren().length), 0);
  checks.push("cleanup leaves no owned tweens running");
  await page.evaluate(() => {
    for (let i = 0; i < 5; i++) mountReveal(gsap, document.querySelector("#one"), { duration: 10 })();
  });
  assert.equal(await page.evaluate(() => gsap.globalTimeline.getChildren().length), 0);
  checks.push("repeated mount and unmount do not accumulate tweens");

  await page.evaluate(() => {
    window.first = mountReveal(gsap, document.querySelector("#one"), { duration: 10 });
    window.second = mountReveal(gsap, document.querySelector("#two"), { duration: 10 });
    first();
  });
  assert.equal(await page.evaluate(() => gsap.getTweensOf(document.querySelector("#one p")).length), 0);
  assert.equal(await page.evaluate(() => gsap.getTweensOf(document.querySelector("#two p")).length), 1);
  checks.push("disposing one component preserves another component's animation");
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.waitForFunction(() => document.querySelector("#two p").style.transform === "");
  assert.equal(await page.locator("#two p").evaluate(el => getComputedStyle(el).opacity), "1");
  checks.push("a live reduced-motion change reverts movement and keeps content visible");
  await page.evaluate(() => second());
  await page.close();

  const reduced = await pageWithMotion("reduce");
  await reduced.evaluate(() => { window.dispose = mountReveal(gsap, document.querySelector("#one")); });
  assert.equal(await reduced.evaluate(() => gsap.globalTimeline.getChildren().length), 0);
  assert.equal(await reduced.locator("#one p").evaluate(el => getComputedStyle(el).opacity), "1");
  checks.push("initial reduced-motion preference creates no entrance tween");
  assert.equal(await reduced.evaluate(() => {
    try { mountReveal(gsap, document.querySelector("#one"), { duration: -1 }); return false; }
    catch (error) { return error instanceof TypeError; }
  }), true);
  checks.push("invalid animation parameters fail explicitly");
  await reduced.close();

  const priority = await pageWithMotion("no-preference");
  await priority.addScriptTag({ path: path.join(packagePath, "dist/ScrollTrigger.min.js") });
  const order = await priority.evaluate(() => {
    gsap.registerPlugin(ScrollTrigger);
    ScrollTrigger.create({ id: "low", trigger: "#one", start: 0, end: 100, refreshPriority: -1 });
    ScrollTrigger.create({ id: "high", trigger: "#two", start: 0, end: 100, refreshPriority: 1 });
    return ScrollTrigger.sort().map(trigger => trigger.vars.id);
  });
  assert.deepEqual(order, ["high", "low"]);
  checks.push("higher ScrollTrigger refreshPriority sorts before lower priority");
  await priority.close();

  const noScript = await browser.newPage({ javaScriptEnabled: false });
  await noScript.setContent(markup);
  assert.equal(await noScript.locator("#one p").isVisible(), true);
  checks.push("static content remains visible without JavaScript");
  await noScript.close();

  console.log(JSON.stringify({ gsap_version: gsapPackage.version, browser: browser.version(), passed: checks.length, checks }, null, 2));
} finally {
  await browser.close();
}
