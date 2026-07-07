// SPDX-License-Identifier: AGPL-3.0-or-later
// Client-side behaviour: command palette, HTMX nav + focus management, skip
// link, and reduced-motion gating - the parts in-process Rust tests can't cover.
import { test, expect } from "@playwright/test";

test("command palette (Ctrl/Cmd+K) opens, filters, and navigates", async ({
  page,
}) => {
  await page.goto("/");
  await page.keyboard.press("Control+k");
  await expect(page.locator("[data-cmdk]")).toBeVisible();
  await page.locator(".cmdk-input").fill("profile");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/profile$/);
});

test("HTMX nav swaps content and moves focus to <main>", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Articles" }).first().click();
  await expect(page).toHaveURL(/\/articles$/);
  // app.js focuses #main after htmx:afterSettle.
  await expect.poll(() => page.evaluate(() => document.activeElement?.id)).toBe(
    "main",
  );
});

test("skip link is the first focusable element", async ({ page }) => {
  await page.goto("/");
  await page.keyboard.press("Tab");
  const text = await page.evaluate(() => document.activeElement?.textContent);
  expect(text).toContain("Skip to content");
});

test("reduced motion disables the boot animation", async ({ browser }) => {
  const ctx = await browser.newContext({ reducedMotion: "reduce" });
  const page = await ctx.newPage();
  await page.goto("/");
  const booting = await page.evaluate(() =>
    document.documentElement.classList.contains("booting"),
  );
  expect(booting).toBe(false);
  await ctx.close();
});
