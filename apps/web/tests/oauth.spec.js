import { test, expect } from "@playwright/test";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";

test("OAuth returns across sites without weakening the Strict session cookie", async ({
  page,
}) => {
  const html = (
    await readFile(
      new URL("../../server/resources/oauth-callback.html", import.meta.url),
      "utf8",
    )
  ).replaceAll("{{nonce}}", "fixture-nonce");
  const requests = [];
  const server = createServer(async (req, res) => {
    requests.push({
      path: req.url,
      method: req.method,
      cookie: req.headers.cookie,
      site: req.headers["sec-fetch-site"],
    });
    res.setHeader("Content-Type", "text/html");
    if (req.url === "/login-fixture") {
      res.setHeader(
        "Set-Cookie",
        "fenix_session=fixture-owner; HttpOnly; SameSite=Strict; Path=/",
      );
      res.end("Signed in");
    } else if (req.method === "POST") {
      let body = "";
      for await (const part of req) body += part;
      requests.at(-1).body = JSON.parse(body);
      res.setHeader("Content-Type", "application/json");
      res.end(JSON.stringify({ redirect: "/integrations?connected=fixture" }));
    } else if (req.url.startsWith("/api/connectors/oauth/callback")) {
      res.setHeader(
        "Content-Security-Policy",
        "default-src 'none'; script-src 'nonce-fixture-nonce'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
      );
      res.end(html);
    } else res.end("Account connected");
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  try {
    await page.goto(`${origin}/login-fixture`);
    await page.route("http://provider.example.test/authorize", (route) =>
      route.fulfill({
        contentType: "text/html; charset=utf-8",
        body: `<a href="${origin}/api/connectors/oauth/callback?state=fixture-state&code=fixture-code">Return to Fénix</a>`,
      }),
    );
    await page.goto("http://provider.example.test/authorize");
    await page.getByRole("link", { name: "Return to Fénix" }).click();
    await expect(page).toHaveURL(`${origin}/integrations?connected=fixture`);
    const landing = requests.find(
      (r) => r.path.startsWith("/api/connectors/") && r.method === "GET",
    );
    const exchange = requests.find((r) => r.method === "POST");
    expect(landing.cookie).toBeUndefined();
    expect(landing.site).toBe("cross-site");
    expect(exchange.cookie).toContain("fenix_session=fixture-owner");
    expect(exchange.site).toBe("same-origin");
    expect(exchange.body).toEqual({
      state: "fixture-state",
      code: "fixture-code",
      error: null,
    });
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});
