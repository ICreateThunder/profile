# Contributing

Thanks for your interest. This document describes how contributions land in the codebase.

## Code of Conduct

This project has a [Code of Conduct](CODE_OF_CONDUCT.md). By participating you agree to uphold it. Report unacceptable behaviour to `robert@shalders.co.uk`, or via [GitHub Private Security Advisories](https://github.com/ICreateThunder/profile/security/advisories/new) if you prefer a private channel.

## Developer Certificate of Origin (DCO)

Every commit must be signed off. This asserts you have the right to contribute it under the project's [AGPL-3.0-or-later](LICENSE) licence. Sign-off is done with the `-s` flag:

```bash
git commit -s -m "feat(web): add contact form"
```

This appends a trailer to the commit message:

```text
Signed-off-by: Your Name <your.email@example.com>
```

The full DCO text is at <https://developercertificate.org>. By signing off you affirm that statement for the contribution. Pull requests with any unsigned commit will be rejected by the DCO check.

**Bot exemption:** automated dependency-update bots (Dependabot) are not required to sign off - they cannot add the trailer, and a dependency bump is a mechanical metadata change rather than a copyrightable contribution. Their commits are still GPG-signed by GitHub, so the cryptographic signing requirement still applies.

**One-time git setup:**

```bash
git config --global user.name  "Your Name"
git config --global user.email "your.email@example.com"
```

## Signed commits (cryptographic, separate from DCO)

Commits to `main` must also be **GPG- or SSH-signed**. Sign-off (`-s`) is the DCO; signing (`-S`) is cryptographic proof of authorship. They are different things and both are required.

SSH signing is usually lowest-friction. Once configured:

```bash
git config --global commit.gpgsign true
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/your_signing_key.pub
```

Verify with `git log --show-signature`. GitHub's full setup guide: <https://docs.github.com/en/authentication/managing-commit-signature-verification>

## Conventional commits

PR titles must follow [Conventional Commits 1.0](https://www.conventionalcommits.org/en/v1.0.0/). The squash merge takes its commit message from the PR title, so the PR title is what lands in `main`'s history.

Format:

```text
<type>(<scope>): <short summary>
```

**Types:** `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`, `build`, `perf`, `security`, `style`, `revert`.

**Scopes** (suggested, not exhaustive): `web`, `api`, `content`, `styling`, `deploy`, `docs`.

Examples:

- `feat(web): add hover prefetch for instant navigation`
- `fix(styling): resolve CRT flicker on Safari`
- `security(deps): bump tokio to 1.42 (CVE-2024-XXXXX)`

Breaking changes: append `!` after type/scope and include `BREAKING CHANGE:` in the body.

## Pull request process

1. **Fork** (external contributors) or **branch** from `main` (maintainers).
2. **Branch naming** is a soft convention: `feat/<short>`, `fix/<short>`, `chore/<short>`. Squash-merge means branch names don't appear in `main`'s history, so this is for your benefit, not enforced.
3. **Commit early, often.** Branch commits are squashed at merge - they don't need to be polished. The PR title is what becomes the squash commit message.
4. **Open a draft PR** as soon as you have something to discuss; mark "Ready for review" when complete.
5. **PR checklist** (in template):
   - [ ] DCO sign-off on every commit
   - [ ] Commits are signed
   - [ ] Tests added or updated
   - [ ] Rust: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass
   - [ ] User-facing changes documented
6. **Review**: minimum 1 maintainer approval. CODEOWNERS paths may require specific reviewers.
7. **Merge**: squash-only. A maintainer merges once approved, CI green, conversations resolved.

## What we're looking for

- **Bug fixes** - always welcome
- **Tests** - always welcome
- **Documentation** - always welcome
- **Features** - discuss first in a GitHub Discussion or Issue before opening a PR. Saves both sides time if the design doesn't fit.

## What we're not looking for

- **Whitespace-only churn** or formatter reflows without functional change
- **Comments restating what code obviously does**
- **Features that add visitor tracking, third-party analytics, or phone-home telemetry**

## Development setup

Prerequisites:

- Rust 1.85+ (`cargo`, `rustc`, `rustfmt`, `clippy`) - 2024 edition
- Bun 1.3+ - `apps/api/build.rs` invokes Bun + `@tailwindcss/cli` to compile the stylesheet. Required at build time; not in production runtime.
- Docker (for building the container image)

```bash
cd apps/api && bun install --frozen-lockfile && cd -   # one-time Tailwind CLI setup
cargo build                                            # build.rs compiles Tailwind on first run
cargo test --workspace                                 # run tests
```

The binary reads `static/` and `src/content/` relative to the working
directory, so run it from `apps/api` (`cd apps/api && cargo run`).

## Reporting bugs

Use the **Bug report** issue template. Include:

- Version (commit SHA)
- Steps to reproduce
- Expected vs actual behaviour
- Logs with secrets redacted

## Reporting security issues

**Do not** open public issues for security vulnerabilities. Use [GitHub Private Security Advisories](https://github.com/ICreateThunder/profile/security/advisories/new). Full policy at [SECURITY.md](SECURITY.md).

## Questions

GitHub Discussions is for open-ended questions. Issues are for actionable bugs/features. Sensitive matters via the [SECURITY.md](SECURITY.md) channels.

Thanks for contributing.
