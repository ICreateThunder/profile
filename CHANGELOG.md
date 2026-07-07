# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Release tooling extracts the notes for a tag `vX.Y.Z` from its `## [X.Y.Z]` section.

## [Unreleased]

### Changed

- Rewrote the site from Astro (static hosting) to the Rust **MASH** stack
  (Maud + Axum + HTMX), server-rendered, with Tailwind compiled at build time.
- Restructured into a Cargo workspace (`apps/api` + `crates/lib-core`).
- **Relicensed from MIT to AGPL-3.0-or-later.** All code in this repository is
  AGPL-3.0-or-later.

### Added

- Governance and supply-chain hardening: `GOVERNANCE.md`, `CODE_OF_ETHICS.md`,
  `SUPPORT.md`, and `cargo-deny`/`typos`/`gitleaks` configuration with a
  `scripts/check-all.sh` local gate.

### Removed

- The Astro toolchain and the privacy-respecting analytics beacon.
