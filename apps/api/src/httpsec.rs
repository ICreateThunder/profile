// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP-layer security middleware:
//!   * `repr_digest`      - RFC 9530 `Repr-Digest` (encoding-agnostic, CDN-safe).
//!   * `fetch_metadata`   - Fetch-Metadata resource-isolation policy.
//!   * `Signer`/`sign_response` - RFC 9421 HTTP Message Signatures (Ed25519),
//!     proving origin authenticity *past* a TLS-terminating CDN like Cloudflare.
//!
//! Repr-Digest is chosen over Content-Digest so Cloudflare's (de)compression
//! doesn't invalidate it, and the signature covers Repr-Digest (not the encoded
//! bytes) so it survives the edge too. Signatures are deterministic (no nonce /
//! no `created`) so they stay valid in the edge cache.

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use ed25519_dalek::{Signer as _, SigningKey};

use crate::AppState;

const SITE_URL: &str = "https://robertshalders.com";
const REPR_DIGEST: HeaderName = HeaderName::from_static("repr-digest");
const SIGNATURE_INPUT: HeaderName = HeaderName::from_static("signature-input");
const SIGNATURE: HeaderName = HeaderName::from_static("signature");

/// Maximum body we'll buffer to digest/sign (this is a small static site).
const MAX_BODY: usize = 8 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Repr-Digest (RFC 9530)
// ---------------------------------------------------------------------------

/// Buffer the response body, attach `Repr-Digest: sha-256=:…:` over the
/// (unencoded) representation. Must run *inside* the compression layer.
///
/// Skips `/static/*`: those assets are immutable, edge-cached, and don't benefit
/// from origin-authenticity signatures, so buffering them into memory to digest
/// is pure cost, and the only place a body could approach `MAX_BODY` and trip
/// the 500 branch below. `sign_response` then also skips them
/// automatically, since it no-ops when no `Repr-Digest` header is present.
pub async fn repr_digest(req: Request, next: Next) -> Response {
    if req.uri().path().starts_with("/static/") {
        return next.run(req).await;
    }
    let resp = next.run(req).await;
    let (mut parts, body) = resp.into_parts();
    let bytes = match axum::body::to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let b64 = base64::engine::general_purpose::STANDARD
        .encode(<sha2::Sha256 as sha2::Digest>::digest(&bytes));
    if let Ok(v) = HeaderValue::from_str(&format!("sha-256=:{b64}:")) {
        parts.headers.insert(REPR_DIGEST, v);
    }
    Response::from_parts(parts, Body::from(bytes))
}

// ---------------------------------------------------------------------------
// Fetch-Metadata resource isolation
// ---------------------------------------------------------------------------

/// Reject cross-site requests that aren't top-level GET navigations. Blocks
/// hot-linking / cross-origin embedding while allowing people to click through
/// to the site. Best-effort behind an edge cache (cached hits skip the origin).
pub async fn fetch_metadata(req: Request, next: Next) -> Response {
    let h = req.headers();
    let site = h.get("sec-fetch-site").and_then(|v| v.to_str().ok());
    let allowed = match site {
        // No Fetch-Metadata (old browser / non-browser client) → allow.
        None => true,
        Some("same-origin") | Some("same-site") | Some("none") => true,
        Some("cross-site") => {
            let mode = h.get("sec-fetch-mode").and_then(|v| v.to_str().ok());
            let dest = h.get("sec-fetch-dest").and_then(|v| v.to_str().ok());
            req.method() == Method::GET
                && mode == Some("navigate")
                && !matches!(dest, Some("object") | Some("embed"))
        }
        // Unknown future value → fail open.
        Some(_) => true,
    };
    if allowed {
        next.run(req).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}

// ---------------------------------------------------------------------------
// HTTP Message Signatures (RFC 9421)
// ---------------------------------------------------------------------------

/// Ed25519 response signer. DEV: an ephemeral key is generated per process if
/// none is provided - the matching public key is published as a JWK so the
/// current run is verifiable. PROD: inject a stable seed (see `from_env`).
pub struct Signer {
    key: SigningKey,
    jwk: String,
    /// RFC 9421 `@signature-params` value - constant per process (only the fixed
    /// keyid varies it), so it is built once here instead of per response.
    sig_params: String,
    /// Precomputed `Signature-Input` header value (`sig1=<sig_params>`).
    sig_input: HeaderValue,
}

impl Signer {
    /// Build from `RESPONSE_SIG_SEED` (base64 of a 32-byte Ed25519 seed) if set,
    /// otherwise generate an ephemeral key for this process (dev).
    pub fn from_env() -> Self {
        let seed = std::env::var("RESPONSE_SIG_SEED")
            .ok()
            .and_then(|s| {
                base64::engine::general_purpose::STANDARD
                    .decode(s.trim())
                    .ok()
            })
            .and_then(|b| <[u8; 32]>::try_from(b).ok());
        let key = match seed {
            Some(seed) => SigningKey::from_bytes(&seed),
            None => {
                // Loud, not silent: an ephemeral key is fine for dev but breaks
                // verification across replicas/restarts in production (the JWK at
                // the fixed keyid would not match other pods' signatures).
                tracing::warn!(
                    "RESPONSE_SIG_SEED is not set - using an EPHEMERAL response-signing \
                     key. Response signatures will not verify across replicas or after a \
                     restart. Set a shared 32-byte seed (base64, from a Secret) in production."
                );
                SigningKey::generate(&mut rand_core::OsRng)
            }
        };
        Self::new(key)
    }

    fn new(key: SigningKey) -> Self {
        let keyid = format!("{SITE_URL}/.well-known/http-msg-sig.jwk");
        let x =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(key.verifying_key().to_bytes());
        let jwk = format!(
            "{{\"kty\":\"OKP\",\"crv\":\"Ed25519\",\"x\":\"{x}\",\"use\":\"sig\",\"alg\":\"EdDSA\",\"kid\":\"{keyid}\"}}"
        );
        let sig_params = format!(
            "(\"@authority\" \"@path\" \"content-type\" \"repr-digest\");keyid=\"{keyid}\";alg=\"ed25519\""
        );
        let sig_input =
            HeaderValue::from_str(&format!("sig1={sig_params}")).expect("signature-input is ASCII");
        Self {
            key,
            jwk,
            sig_params,
            sig_input,
        }
    }

    /// The public verification key as a JWK (served at the `keyid` URL).
    pub fn jwk(&self) -> &str {
        &self.jwk
    }

    /// Short fingerprint (first bytes of the public key) for log lines.
    pub fn fingerprint(&self) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&self.key.verifying_key().to_bytes()[..8])
    }
}

/// Sign each response over `@authority @path content-type repr-digest` with
/// Ed25519 (deterministic - edge-cache safe). `repr_digest` must run inside
/// (before) this on the response path so the digest header exists to cover.
pub async fn sign_response(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let authority = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let path = req.uri().path().to_string();

    let mut resp = next.run(req).await;

    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let rd = resp
        .headers()
        .get(&REPR_DIGEST)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let (Some(ct), Some(rd)) = (ct, rd) else {
        return resp; // nothing to cover (e.g. empty body / no digest)
    };
    if authority.is_empty() {
        return resp;
    }

    let signer = &state.signer;
    // RFC 9421 signature base: one `"name": value` line per component, ending
    // with the `@signature-params` line. Joined by LF, no trailing newline.
    let base = format!(
        "\"@authority\": {authority}\n\"@path\": {path}\n\"content-type\": {ct}\n\"repr-digest\": {rd}\n\"@signature-params\": {}",
        signer.sig_params
    );
    let sig = signer.key.sign(base.as_bytes());
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.to_bytes());

    if let Ok(sv) = HeaderValue::from_str(&format!("sig1=:{sig_b64}:")) {
        resp.headers_mut()
            .insert(SIGNATURE_INPUT, signer.sig_input.clone());
        resp.headers_mut().insert(SIGNATURE, sv);
    }
    resp
}
