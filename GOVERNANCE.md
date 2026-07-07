# Governance

This document describes how decisions are made in this project. It is
deliberately small, matching the project's size.

## Model

The project is run by a single maintainer (BDFL - benevolent dictator for life):

- **Maintainer / BDFL:** [@ICreateThunder](https://github.com/ICreateThunder) (Robert Shalders)

This is appropriate for a personal portfolio. The structure below exists so the
project can scale if it ever needs to, not because it needs it today.

## Roles

- **Contributors** - anyone with a merged pull request, filed issue, or
  discussion. No commitment expected.
- **Maintainers** - review, merge, triage, and release. Bound by the response
  targets in [SECURITY.md](SECURITY.md) for security reports.
- **BDFL** - final authority on direction, design, and disputes. Used rarely and,
  for anything architectural, justified in writing (an ADR - see below).

## How decisions are made

- **Day-to-day** (bug fixes, docs, small features): a maintainer reviews, CI is
  green, and it merges. See [CONTRIBUTING.md](CONTRIBUTING.md).
- **Architectural** (new dependency, data model, a structural change): record an
  **Architecture Decision Record** and open it as a PR; leave a short comment
  period for feedback before merging it as `accepted`. ADRs
  are immutable history - supersede, don't rewrite.
- **Disputes:** the BDFL decides and writes down why.

## Becoming a maintainer

Not currently needed. If sustained, high-quality contribution ever warrants it,
the bar is: a track record of merged PRs across multiple areas, active and
constructive review participation, and alignment with the project's values
(privacy, no visitor tracking, secure-by-default, AGPL).

## Transition plan

If three or more active maintainers beyond the BDFL ever exist, governance moves
from BDFL to consensus among maintainers (lazy consensus over a fixed comment
period, falling back to a simple majority vote), and this document is updated to
reflect it.

## Licensing of contributions

Contributions are licensed under [AGPL-3.0-or-later](LICENSE), asserted per
commit via DCO sign-off (see [CONTRIBUTING.md](CONTRIBUTING.md)). There is no
copyright assignment - you keep copyright to your contributions.
