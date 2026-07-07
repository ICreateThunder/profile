// SPDX-License-Identifier: AGPL-3.0-or-later
//! Small text endpoints: robots.txt, humans.txt, and RFC 9116 security.txt.
//! humans.txt and security.txt also serve as machine-discoverable AGPL §13
//! "corresponding source" pointers.

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;

use crate::AppState;

const SITE_URL: &str = "https://robertshalders.com";
const REPO_URL: &str = "https://github.com/ICreateThunder/profile";

/// OpenPGP public key (binary, dearmored) served via Web Key Directory (WKD).
/// Robert Shalders <robert@shalders.co.uk>, fpr 1A44 8CE4 18BD 8D37 1D12 B697
/// 418D 45B7 1F57 D61F. Baked into the binary - no runtime file dependency.
const WKD_KEY: &[u8] = include_bytes!("../wkd_key.pgp");
/// WKD hash of the local-part "robert" (z-base-32 SHA-1, via gpg-wks-client).
const WKD_HASH: &str = "nmw11xsgscg89kfy1jixeky87rwhn4nx";

fn plain(body: String, cache: &'static str) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::CACHE_CONTROL, cache),
        ],
        body,
    )
}

/// GET /robots.txt
pub async fn robots() -> impl IntoResponse {
    let body = format!("User-agent: *\nAllow: /\n\nSitemap: {SITE_URL}/sitemap.xml\n");
    plain(body, "public, max-age=86400")
}

/// GET /humans.txt
pub async fn humans() -> impl IntoResponse {
    let body = format!(
        "/* TEAM */\n  Engineer: Robert Shalders\n  Site: {SITE_URL}\n  GitHub: https://github.com/ICreateThunder\n\n\
         /* SITE */\n  Stack: Rust · Axum · Maud · HTMX · Tailwind (MASH)\n  Source: {REPO_URL}\n  License: AGPL-3.0-or-later\n"
    );
    plain(body, "public, max-age=86400")
}

/// GET /.well-known/security.txt (RFC 9116)
pub async fn security_txt() -> impl IntoResponse {
    let body = format!(
        "Contact: mailto:robert@shalders.co.uk\n\
         Contact: {REPO_URL}/security/advisories/new\n\
         Expires: 2027-06-21T00:00:00.000Z\n\
         Encryption: https://keys.openpgp.org/vks/v1/by-fingerprint/1A448CE418BD8D371D12B697418D45B71F57D61F\n\
         Preferred-Languages: en\n\
         Canonical: {SITE_URL}/.well-known/security.txt\n\
         Policy: {REPO_URL}/blob/main/SECURITY.md\n"
    );
    plain(body, "public, max-age=86400")
}

/// WKD policy file - its mere presence advertises Web Key Directory support.
/// Served for both the direct and advanced method paths.
pub async fn wkd_policy() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        "",
    )
}

/// WKD key lookup - direct method `/.well-known/openpgpkey/hu/{hash}`.
/// Returns the binary key when the hash matches the local-part "robert".
pub async fn wkd_key(Path(hash): Path<String>) -> Response {
    wkd_respond(&hash)
}

/// WKD key lookup - advanced method `/.well-known/openpgpkey/{domain}/hu/{hash}`
/// (served from openpgpkey.<domain>). Domain segment is ignored; hash is matched.
pub async fn wkd_key_advanced(Path((_domain, hash)): Path<(String, String)>) -> Response {
    wkd_respond(&hash)
}

fn wkd_respond(hash: &str) -> Response {
    if hash == WKD_HASH {
        (
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
                (header::CACHE_CONTROL, "public, max-age=86400"),
            ],
            Bytes::from_static(WKD_KEY),
        )
            .into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

/// GET /.well-known/webfinger?resource=acct:robert@robertshalders.com (RFC 7033)
/// Identity discovery: `rel="me"` links today; an ActivityPub `self` link can be
/// added here if a fediverse actor is ever stood up.
pub async fn webfinger(Query(q): Query<HashMap<String, String>>) -> Response {
    let resource = q.get("resource").map(String::as_str).unwrap_or_default();
    let known = matches!(
        resource,
        "acct:robert@robertshalders.com" | "acct:robert@shalders.co.uk" | SITE_URL
    );
    if !known {
        return StatusCode::NOT_FOUND.into_response();
    }
    let jrd = format!(
        "{{\"subject\":\"acct:robert@robertshalders.com\",\
          \"aliases\":[\"{SITE_URL}\",\"https://github.com/ICreateThunder\"],\
          \"links\":[\
            {{\"rel\":\"http://webfinger.net/rel/profile-page\",\"type\":\"text/html\",\"href\":\"{SITE_URL}/profile\"}},\
            {{\"rel\":\"me\",\"href\":\"https://github.com/ICreateThunder\"}},\
            {{\"rel\":\"me\",\"href\":\"https://www.linkedin.com/in/robertshalders/\"}}\
          ]}}"
    );
    (
        [
            (header::CONTENT_TYPE, "application/jrd+json; charset=utf-8"),
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        jrd,
    )
        .into_response()
}

/// GET /.well-known/gpc.json - Global Privacy Control. Honest: no trackers,
/// nothing sold; the signal is respected by construction.
pub async fn gpc() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        "{\"gpc\":true,\"lastUpdate\":\"2026-06-22\"}",
    )
}

/// GET /.well-known/http-msg-sig.jwk - public key verifying the RFC 9421
/// `Signature` header on responses.
pub async fn sig_jwk(State(state): State<AppState>) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/jwk+json"),
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        state.signer.jwk().to_owned(),
    )
}

/// GET /teapot - hidden easter egg, RFC 2324 (HTCPCP). 418, naturally.
pub async fn teapot() -> impl IntoResponse {
    let body = "\
             ___\n\
        _,--'   \"`-.\n\
     ,-'  _,-.    .`.\n\
    (_,--' `-'\\   |  |     418 I'M A TEAPOT\n\
     `.        `--'  /     RFC 2324 - HTCPCP/1.0\n\
       `--.________,'      The requested entity body is short and stout.\n";
    (
        StatusCode::IM_A_TEAPOT,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
}
