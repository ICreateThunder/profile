<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# Browser e2e + accessibility tests (Tier 3)

Isolated Playwright suite - its own `package.json`/toolchain, **separate from the
Rust build**. Runs against a *running* instance (a container or `cargo run`) via
`BASE_URL`.

## Run locally

```sh
# 1. start the app (e.g. the smoke image, or cargo run from apps/api)
podman run -d --name profile -p 8080:8080 localhost/profile:local

# 2. install + run
cd tests/e2e
npm install
npx playwright install --with-deps chromium
BASE_URL=http://127.0.0.1:8080 npm test
```

## What it covers

- **a11y.spec.ts** - axe-core WCAG 2 A/AA scan of each main page.
- **smoke.spec.ts** - every page loads with no console errors and **no CSP
  violations** (proves the strict CSP doesn't block our own scripts).
- **interactions.spec.ts** - ⌘/Ctrl-K command palette, HTMX nav + focus-to-`<main>`,
  skip link, and reduced-motion gating.

CI runs this in a dedicated `e2e` job (see `.github/workflows/ci.yml`).
