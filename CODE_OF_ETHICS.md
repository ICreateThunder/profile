# Code of Ethics

The technical choices in this project are constrained by a few non-negotiable
commitments. They are recorded here so they are not quietly eroded over time.

## No visitor tracking, ever

This project ships **no phone-home, no usage analytics, no error aggregation, and
no third-party trackers**. No cookies, no fingerprinting, and nothing about a
visitor is sold or handed to an analytics vendor. A change that adds visitor
tracking will not be merged, regardless of how it is framed.

Two caveats keep this honest. The site runs first-party operational
observability - aggregate request metrics and structured logs about the running
system, not about the visitor (see Privacy below). And the origin is fronted by
Cloudflare, which as a reverse proxy processes each request, including its IP
address, as a data processor for the operator; no other third party sits in the
request path.

## Privacy and data minimisation

- Collect only what a feature genuinely requires.
- No PII in logs, traces, or metric labels.
- Where data is stored (e.g. the status page's own metrics), it is operational
  data about the *machine*, not about *people*.
- EU data residency is the default posture for any hosted data.

## No dark patterns

No tricks to inflate engagement, no manipulative UI, no consent theatre. The site
should be honest about what it is and what it does.

## Self-hosted and dependency-honest

- Runtime assets (fonts, scripts) are self-hosted - no third-party CDN that could
  observe visitors or inject code.
- Dependencies are kept minimal, audited (`cargo-audit`/`cargo-deny`), and
  preferred when they reduce unsafe surface (see the choice of TOML over an
  unsafe YAML parser, and `serde_json`-class parsers over C bindings).

## Transparency

The site is open source under [AGPL-3.0-or-later](LICENSE); AGPL §13 keeps that
true for anyone who runs a modified version as a network service.

These commitments bind maintainers and contributors alike. If a future feature
appears to require breaking one of them, that is a signal to redesign the feature,
not to relax the commitment.
