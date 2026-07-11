# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Release tooling extracts the notes for a tag `vX.Y.Z` from its `## [X.Y.Z]` section.

## [Unreleased]

## [0.1.0] - 2026-07-11

First public release: the site rewritten on the Rust MASH stack, published as a
signed and attested container image.

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
- Release pipeline: a `v*` tag builds the image to GitHub Container Registry with
  a keyless Cosign signature and an attested SLSA build provenance, gated on a
  Trivy scan. Verify with `cosign verify` and
  `cosign verify-attestation --type slsaprovenance1`.

### Fixed

- Profile page on small viewports: the identity header and the releases block
  were centred inside a fixed-height scroll box and clipped out of reach. The
  column now scrolls from the top on mobile and stays centred from `lg` up.
- Reduced background-animation cost on touch devices: the noise-shift transform,
  the multiply-blend CRT overlay, and the canvas backdrops are dropped under
  `(hover: none) and (pointer: coarse)`, keeping the static scanlines and noise.

### Removed

- The Astro toolchain and the privacy-respecting analytics beacon.

[0.1.0]: https://github.com/ICreateThunder/profile/releases/tag/v0.1.0
