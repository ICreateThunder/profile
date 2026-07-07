# Security policy

## Supported versions

| Version | Supported |
|---------|-----------|
| latest on `main` | Yes |
| Older commits | No |

Only the latest deployment from `main` receives security fixes.

## Reporting a vulnerability

**Do not** open public issues for security vulnerabilities.

**Preferred:** [GitHub Private Security Advisories](https://github.com/ICreateThunder/profile/security/advisories/new). Encrypted in transit, scoped to maintainers, audit-logged.

**Fallback:** email `robert@shalders.co.uk`.

### What to include

- Affected version (commit SHA or release tag)
- Reproducer or proof of concept
- CVSS 3.1 vector if you have assessed it
- Whether you want public credit

### Response SLAs

| Stage | Target |
|---|---|
| Acknowledgement of receipt | 48 hours |
| Initial assessment | 5 working days |
| Coordinated disclosure (typical) | 90 days from initial assessment |

Maintainers will provide status updates at least every 7 days during active investigation. If you do not hear back within the acknowledgement window, please follow up - the message may not have reached the maintainers.

## PGP key

A PGP key is available for signed and encrypted security correspondence.

| | |
|---|---|
| **Primary UID** | Robert Shalders &lt;robert@shalders.co.uk&gt; |
| **Fingerprint** | `1A44 8CE4 18BD 8D37 1D12  B697 418D 45B7 1F57 D61F` |
| **Algorithms** | Ed25519 (sign) / Curve25519 (encrypt) |
| **Hardware** | Hardware-token-backed; private key material never leaves the device |

Fetch the public key from any of these sources:

- **Keyserver** (verified): <https://keys.openpgp.org/search?q=robert@shalders.co.uk>

## Scope

### In scope

- Source code in this repository
- Container images built from this repository
- The Dockerfile and CI/CD pipeline configuration

### Out of scope

- Misconfiguration of self-hosted deployments where documentation correctly describes secure defaults
- Issues in third-party dependencies (report upstream; we track via Dependabot and `cargo-audit`)
- Vulnerabilities requiring physical access to infrastructure
- Social-engineering attacks against maintainers
- Denial-of-service via resource exhaustion requiring resources not normally available

## Bounty

This project does not operate a bug bounty programme. Reporters acting in good faith receive acknowledgement in the security advisory (unless anonymity is preferred).

## Threat model

Summary of the trust boundaries defended:

- **Credential compromise** - assume a maintainer's GitHub credentials are stolen. Defence: org-wide 2FA, signed commits (GPG/SSH) + DCO, branch-protection and signed-tag rulesets, and **no long-lived cloud secrets in CI** - image publishing authenticates via short-lived **OIDC federation** (no stored registry or cloud credentials).
- **Supply chain** - assume an upstream dependency is malicious or compromised. Defence: Dependabot version updates, `cargo-deny` (RUSTSEC advisories, licence allow-list, and registry/source bans) enforced in CI on every PR, CodeQL semantic analysis, gitleaks secret scanning, and a distroless base image.
- **Container image tampering** - assume the container registry is hostile. Defence: images are pinned **by digest** in the deploy manifests (never a mutable tag) and ship content-hashed static assets. The publish workflow attaches an SLSA build-provenance attestation and a keyless Cosign signature (Sigstore OIDC, no stored keys), and a Trivy scan gates a fixable High/Critical vulnerability before the image is signed.

## No visitor tracking

The site ships no phone-home, no usage analytics, no error aggregation, and no third-party trackers. First-party operational metrics and structured logs carry no visitor data, and the origin is fronted by Cloudflare, which processes requests as a data processor for the operator. No visitor data reaches any other third party. See [CODE_OF_ETHICS.md](CODE_OF_ETHICS.md).

## Licence

This policy is licensed under [AGPL-3.0-or-later](LICENSE) along with the rest of the project.
