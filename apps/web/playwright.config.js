import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  workers: 1,
  fullyParallel: false,
  timeout: 30_000,
  expect: { timeout: Number(process.env.FENIX_TEST_EXPECT_TIMEOUT || 5000) },
  reporter: "list",
  use: { baseURL: "http://127.0.0.1:4189", trace: "retain-on-failure" },
});
