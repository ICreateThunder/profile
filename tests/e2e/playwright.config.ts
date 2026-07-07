// SPDX-License-Identifier: AGPL-3.0-or-later
import { defineConfig, devices } from "@playwright/test";

// The app is started outside Playwright (a running container or `cargo run`);
// point at it via BASE_URL. CI builds + runs the image, then sets BASE_URL.
const baseURL = process.env.BASE_URL || "http://127.0.0.1:8080";

export default defineConfig({
  testDir: "./specs",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL,
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
