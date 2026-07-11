# Robert Shalders - Portfolio

[![CI](https://github.com/ICreateThunder/profile/actions/workflows/ci.yml/badge.svg)](https://github.com/ICreateThunder/profile/actions/workflows/ci.yml)
[![CodeQL](https://github.com/ICreateThunder/profile/actions/workflows/codeql.yml/badge.svg)](https://github.com/ICreateThunder/profile/actions/workflows/codeql.yml)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/ICreateThunder/profile/badge)](https://securityscorecards.dev/viewer/?uri=github.com/ICreateThunder/profile)
[![dependencies](https://deps.rs/repo/github/ICreateThunder/profile/status.svg)](https://deps.rs/repo/github/ICreateThunder/profile)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/License-AGPL_3.0--or--later-blue.svg)](LICENSE)

Personal portfolio and technical blog at [robertshalders.com](https://robertshalders.com).
Server-rendered Rust, no client framework, built to paint correctly on the first
frame and navigate instantly.

## Stack

The **MASH** stack - server-rendered by a single Rust binary:

- [**M**aud](https://maud.lambda.xyz/) - compile-time-checked HTML templates
- [**A**xum](https://github.com/tokio-rs/axum) - async web framework (Tokio)
- [**S**QLx](https://github.com/launchbadge/sqlx) - compile-time-checked SQL (arriving with the status page)
- [**H**TMX](https://htmx.org/) - hover-prefetched partial navigation (vendored, no CDN)

with [Tailwind CSS](https://tailwindcss.com/) compiled at build time, Rust 2024
edition, and a distroless `linux/amd64` runtime image.

## Design notes

- **No FOUC.** Critical CSS is inlined into every response, so there is no
  render-blocking stylesheet request.
- **Instant nav.** HTMX prefetches on hover and swaps the body - McMaster-Carr style.
- **Edge-cached.** Responses carry explicit `Cache-Control`; Cloudflare is the
  only CDN (the app owns no cache server).
- **Self-hosted fonts** (Bebas Neue, JetBrains Mono, Roboto, Trivial) - no
  third-party CDN at runtime.

## Workspace layout

```
apps/api/          the Axum binary (server-renders MASH)
  src/             main, routes/, templates/, content/
  styles/          Tailwind input
  static/          compiled CSS, vendored htmx, fonts, images
  build.rs         compiles Tailwind via Bun at build time
crates/lib-core/   shared domain types / IDs / audit (grows with the roadmap)
Dockerfile         multi-stage, distroless, amd64
```

## Development

Prerequisites: Rust 1.85+ (2024 edition), Bun 1.3+, Docker.

```bash
cd apps/api && bun install --frozen-lockfile && cd -   # one-time Tailwind setup
cargo build                                            # build.rs compiles Tailwind
cargo test --workspace
cd apps/api && cargo run                               # serve (reads static/ + content/ from cwd)
```

The full local gate (lint, test, audit, deny, secret/typo scan) is
`scripts/check-all.sh`, which CI also runs.

## Content

Articles are Markdown with frontmatter under `apps/api/src/content/`, across four
collections: `projects`, `newsletters`, `resources`, `tricks`. Loaded into memory
at startup - no filesystem I/O per request.

## Deployment

Built in CI as a distroless container, published to GitHub Container Registry
with an **SLSA build-provenance attestation** and a **keyless Cosign signature**,
and pinned **by digest**. The image is public and self-contained - it holds no
secrets and can be rebuilt from a fork. Deployed via GitOps (Flux) with
progressive delivery (Flagger) into an isolated namespace on a hosted Kubernetes
cluster, behind Cloudflare. No cloud credentials are stored in CI - image
publishing uses the ephemeral `GITHUB_TOKEN`.

## Security

See [SECURITY.md](SECURITY.md) for the disclosure policy, PGP key, and threat
model. The site ships **no client-side analytics, no third-party trackers, and
no phone-home** - no cookies, no fingerprinting, and nothing about a visitor sold
or sent to an analytics vendor. First-party operational metrics and PII-free logs
(about the machine, not the visitor), and the Cloudflare edge that fronts the
origin, are covered in [CODE_OF_ETHICS.md](CODE_OF_ETHICS.md).

## Contributing & governance

[CONTRIBUTING.md](CONTRIBUTING.md) (DCO + signed commits + Conventional Commits),
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [GOVERNANCE.md](GOVERNANCE.md),
[SUPPORT.md](SUPPORT.md).

## Tools used

Built with assistance from [Claude](https://claude.ai) and self-hosted
[GPT-OSS](https://github.com/openai/gpt-oss) for scaffolding, iteration, and
content drafting. Design decisions, architecture, and final content are my own.

## License

[AGPL-3.0-or-later](LICENSE). If you run a modified version as a network service,
AGPL §13 requires you to offer its source to users - the source is this repository.
