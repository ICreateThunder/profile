// SPDX-License-Identifier: AGPL-3.0-or-later
// Accessibility (WCAG 2 A/AA) scan of each main page via axe-core.
import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

// Scan in a stable, settled state: reduced-motion skips the one-time boot
// animation (whose transient CRT dim otherwise races axe and reports darkened
// colours). Contrast/labels/structure are identical at rest.
test.use({ reducedMotion: "reduce" });

const pages = ["/", "/profile", "/articles", "/projects"];

for (const path of pages) {
  test(`a11y: ${path} has no WCAG 2 A/AA violations`, async ({ page }) => {
    // Deterministically skip the one-time boot animation (its transient CRT dim
    // darkens every colour to ~black and races the scan). Seed the same
    // sessionStorage flag the boot uses, and wait for any boot class to clear.
    await page.addInitScript(() => {
      try {
        sessionStorage.setItem("booted", "1");
      } catch (e) {
        /* ignore */
      }
    });
    await page.goto(path);
    await page
      .waitForFunction(
        () => !document.documentElement.classList.contains("booting"),
        null,
        { timeout: 3000 },
      )
      .catch(() => {});
    // Settle entrance + idle effects to their resting state *instantly* so axe
    // measures true colours: the one-shot `.animate-fade-in` container, the
    // scroll-reveal cards (which axe never scrolls into view), and the looping
    // CRT `.flicker-*` text (e.g. the empty-collection state) would otherwise be
    // captured mid-animation at a dimmed opacity. CSSOM styling is allowed under
    // our CSP (unlike a <style>).
    await page.evaluate(() => {
      document
        .querySelectorAll(
          ".reveal, .animate-fade-in, .flicker-a, .flicker-b, .flicker-c",
        )
        .forEach((el) => {
          const e = el as HTMLElement;
          e.classList.add("in");
          e.style.animation = "none";
          e.style.transition = "none";
          e.style.opacity = "1";
          e.style.transform = "none";
        });
    });
    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa"])
      .analyze();
    expect(results.violations).toEqual([]);
  });
}
