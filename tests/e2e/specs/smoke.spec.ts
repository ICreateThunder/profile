// SPDX-License-Identifier: AGPL-3.0-or-later
// Each page loads with no console errors and no CSP violations - proves the
// strict CSP doesn't block the site's own (vendored, self-hosted) scripts.
import { test, expect } from "@playwright/test";

// Capture console errors + client-side CSP violations for a navigation.
async function collect(page: import("@playwright/test").Page, path: string) {
  const errors: string[] = [];
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
  });
  page.on("pageerror", (e) => errors.push(String(e)));
  await page.addInitScript(() => {
    (window as unknown as { __csp: string[] }).__csp = [];
    document.addEventListener("securitypolicyviolation", (e) => {
      (window as unknown as { __csp: string[] }).__csp.push(
        (e as SecurityPolicyViolationEvent).violatedDirective,
      );
    });
  });
  const resp = await page.goto(path);
  const csp = await page.evaluate(
    () => (window as unknown as { __csp: string[] }).__csp || [],
  );
  return { resp, errors, csp };
}

// 200 pages: zero console errors AND zero CSP violations.
for (const path of ["/", "/profile", "/articles", "/projects"]) {
  test(`no console errors or CSP violations: ${path}`, async ({ page }) => {
    const { resp, errors, csp } = await collect(page, path);
    expect(resp?.status(), `status ${path}`).toBe(200);
    expect(csp, `CSP violations on ${path}`).toEqual([]);
    expect(errors, `console errors on ${path}`).toEqual([]);
  });
}

// The themed 404 page must render with a real 404 status and no CSP violations.
// Chromium logs the document's own non-2xx as a benign "Failed to load
// resource … 404" console error - that one is expected; any *other* error is not.
test("404 page: real 404 status, CSP-clean, no unexpected console errors", async ({
  page,
}) => {
  const { resp, errors, csp } = await collect(page, "/nope-404");
  expect(resp?.status()).toBe(404);
  expect(csp, "CSP violations on 404").toEqual([]);
  const unexpected = errors.filter((e) => !/Failed to load resource.*404/.test(e));
  expect(unexpected, "unexpected console errors on 404").toEqual([]);
});
