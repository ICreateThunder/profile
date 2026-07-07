<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# Vendored client-side assets

These files are served verbatim under `/static/vendor/` and are covered by the
strict `script-src 'self'` CSP (no inline scripts, no CDN). Because they are
hand-vendored, they are **not** tracked by Dependabot / `npm audit` - so this
file is the manual record and bump procedure.

## Inventory

| File | Upstream | Pinned version | Notes |
|---|---|---|---|
| `htmx.min.js` | <https://github.com/bigskysoftware/htmx> | **2.0.4** | `selfRequestsOnly` defaults on (not overridden); do not enable `allowEval`. |
| `htmx-preload.js` | htmx `preload` extension | matches htmx 2.x | hover/`mouseover` preloading. |
| `app.js` | first-party (this repo) | - | retro/CRT progressive enhancement; `textContent`-only, no `eval`. |

## Bump procedure (do this, don't let it drift)

1. Watch upstream htmx releases / advisories (GitHub "Watch → Releases", or the
   htmx security page). htmx is the only third-party JS here.
2. Download the new `htmx.min.js` (and the matching `preload` extension) from the
   tagged release - **not** `latest` - over HTTPS.
3. Verify integrity before committing: compare against the release's published
   checksum, e.g.
   ```sh
   sha384sum htmx.min.js   # compare to the value in the release notes
   ```
4. Update the **Pinned version** column above and note the bump in `CHANGELOG.md`.
5. Re-run the app and confirm zero CSP violations in the browser console (the
   vendored file must load under `script-src 'self'`).

> Future option: manage htmx via `package.json` + a `build.rs` copy step so
> Dependabot's npm ecosystem watches it automatically. Until then, this manual
> record is the control.
