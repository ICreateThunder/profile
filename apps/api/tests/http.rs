// SPDX-License-Identifier: AGPL-3.0-or-later
//! In-process HTTP integration tests (Tier 1).
//!
//! Drives the real router from `build_app` via `tower::ServiceExt::oneshot` - no
//! network, no spawned server. Covers happy paths, unhappy paths, and the
//! **security invariants** that turn manual security probes into permanent
//! regression tests (header suite, strict CSP, Fetch-Metadata isolation,
//! Repr-Digest + RFC 9421 signature, and the metric-label collapse).

use std::sync::OnceLock;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use http_body_util::BodyExt;
use tower::ServiceExt; // oneshot

use portfolio_api::{AppState, build_app};

/// Build the app once per test binary. `build_pair` installs a *global* metrics
/// recorder and panics if called twice, so all tests share one instance and
/// clone the (cheap, `Clone`) router per request.
fn shared() -> &'static (Router, PrometheusHandle) {
    static APP: OnceLock<(Router, PrometheusHandle)> = OnceLock::new();
    APP.get_or_init(|| build_app(AppState::from_disk()))
}

fn app() -> Router {
    shared().0.clone()
}

async fn get(uri: &str) -> Response {
    app()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn send(req: Request<Body>) -> Response {
    app().oneshot(req).await.unwrap()
}

async fn body_string(resp: Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn ct(resp: &Response) -> &str {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
}

// ---------------------------------------------------------------------------
// Happy paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn happy_html_pages_return_200() {
    for uri in [
        "/",
        "/profile",
        "/articles",
        "/projects",
        "/newsletters",
        "/resources",
        "/tricks",
    ] {
        let r = get(uri).await;
        assert_eq!(r.status(), StatusCode::OK, "GET {uri}");
        assert!(
            ct(&r).starts_with("text/html"),
            "GET {uri} content-type = {}",
            ct(&r)
        );
    }
}

#[tokio::test]
async fn happy_article_page_from_sitemap() {
    // Derive a real (collection, slug) from the sitemap rather than hard-coding,
    // so the test survives content changes - including a content-less deploy,
    // where the sitemap lists no articles and there is nothing to fetch.
    let xml = body_string(get("/sitemap.xml").await).await;
    let Some(path) = first_article_path(&xml) else {
        return; // no articles published - nothing to assert
    };
    let r = get(&path).await;
    assert_eq!(r.status(), StatusCode::OK, "GET {path}");
    assert!(ct(&r).starts_with("text/html"));
}

#[tokio::test]
async fn happy_feeds_and_meta_content_types() {
    for (uri, want_ct) in [
        ("/rss.xml", "application/rss+xml"),
        ("/sitemap.xml", "application/xml"),
        ("/robots.txt", "text/plain"),
        ("/humans.txt", "text/plain"),
        ("/.well-known/security.txt", "text/plain"),
        ("/.well-known/gpc.json", "application/json"),
        ("/.well-known/http-msg-sig.jwk", "application/jwk+json"),
    ] {
        let r = get(uri).await;
        assert_eq!(r.status(), StatusCode::OK, "GET {uri}");
        assert!(
            ct(&r).starts_with(want_ct),
            "GET {uri} ct={} !~ {want_ct}",
            ct(&r)
        );
    }
}

#[tokio::test]
async fn happy_webfinger_known_resource() {
    let r = get("/.well-known/webfinger?resource=acct:robert@robertshalders.com").await;
    assert_eq!(r.status(), StatusCode::OK);
    assert!(ct(&r).starts_with("application/jrd+json"));
    assert_eq!(
        r.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        "*"
    );
}

#[tokio::test]
async fn happy_wkd_known_hash_returns_key() {
    let r = get("/.well-known/openpgpkey/hu/nmw11xsgscg89kfy1jixeky87rwhn4nx").await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(
        r.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/octet-stream"
    );
}

#[tokio::test]
async fn happy_health_and_teapot() {
    assert_eq!(get("/healthz").await.status(), StatusCode::OK);
    assert_eq!(get("/readyz").await.status(), StatusCode::OK);
    assert_eq!(get("/teapot").await.status(), StatusCode::IM_A_TEAPOT);
}

/// The public uptime probe: 200 + uncacheable, so an external canary reads the
/// real origin state rather than an edge-cached 200.
#[tokio::test]
async fn public_uptime_probe_is_uncacheable() {
    let r = get("/up").await;
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.headers().get(header::CACHE_CONTROL).unwrap(), "no-store");
}

// ---------------------------------------------------------------------------
// Unhappy paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unhappy_unknown_routes_404() {
    for uri in [
        "/nope",
        "/not-a-collection",
        "/projects/no-such-slug",
        "/.well-known/webfinger",
        "/.well-known/webfinger?resource=acct:nobody@example.com",
        "/.well-known/openpgpkey/hu/deadbeefdeadbeefdeadbeefdeadbeef",
    ] {
        assert_eq!(get(uri).await.status(), StatusCode::NOT_FOUND, "GET {uri}");
    }
}

#[tokio::test]
async fn unhappy_disallowed_methods_405_with_allow() {
    for method in ["POST", "PUT", "DELETE", "PATCH"] {
        let req = Request::builder()
            .method(method)
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let r = send(req).await;
        assert_eq!(r.status(), StatusCode::METHOD_NOT_ALLOWED, "{method} /");
        let allow = r.headers().get(header::ALLOW).unwrap().to_str().unwrap();
        assert!(allow.contains("GET"), "{method} / Allow={allow}");
    }
}

#[tokio::test]
async fn unhappy_404_body_does_not_reflect_path() {
    let resp = get("/qq-XSSMARKER-qq").await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = body_string(resp).await;
    assert!(
        !body.contains("XSSMARKER"),
        "404 body must not echo the request path"
    );
}

// ---------------------------------------------------------------------------
// Security invariants (regression guards)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn security_header_suite_present() {
    let r = get("/").await;
    let h = r.headers();
    assert_eq!(h.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(), "nosniff");
    assert_eq!(h.get(header::X_FRAME_OPTIONS).unwrap(), "DENY");
    assert_eq!(
        h.get(header::REFERRER_POLICY).unwrap(),
        "strict-origin-when-cross-origin"
    );
    assert_eq!(h.get("cross-origin-opener-policy").unwrap(), "same-origin");
    assert_eq!(
        h.get("cross-origin-resource-policy").unwrap(),
        "same-origin"
    );
    assert!(h.get("permissions-policy").is_some());
}

#[tokio::test]
async fn csp_is_strict_no_unsafe() {
    let r = get("/").await;
    let csp = r
        .headers()
        .get(header::CONTENT_SECURITY_POLICY)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(csp.contains("default-src 'none'"), "CSP={csp}");
    assert!(
        !csp.contains("unsafe-inline"),
        "CSP must not allow unsafe-inline: {csp}"
    );
    assert!(
        !csp.contains("unsafe-eval"),
        "CSP must not allow unsafe-eval: {csp}"
    );
    assert!(csp.contains("frame-ancestors 'none'"), "CSP={csp}");
}

/// Every non-HTML response (here a static asset + a 404) must still carry
/// a locked-down fallback CSP, so a self-hosted SVG opened as a document can't
/// run script. The HTML pages keep their own hash-based CSP (asserted above).
#[tokio::test]
async fn fallback_csp_on_non_html_responses() {
    for uri in ["/static/favicon.svg", "/definitely-not-a-real-route"] {
        let r = get(uri).await;
        let csp = r
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .unwrap_or_else(|| panic!("no CSP on {uri}"))
            .to_str()
            .unwrap();
        assert!(csp.contains("default-src 'none'"), "{uri} CSP={csp}");
        assert!(!csp.contains("unsafe"), "{uri} CSP={csp}");
    }
}

#[tokio::test]
async fn fetch_metadata_blocks_cross_site_non_navigation() {
    // cross-site, non-navigation (e.g. hot-linked image) → 403
    let blocked = Request::builder()
        .uri("/")
        .header("sec-fetch-site", "cross-site")
        .header("sec-fetch-mode", "no-cors")
        .header("sec-fetch-dest", "image")
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(blocked).await.status(), StatusCode::FORBIDDEN);

    // cross-site top-level navigation (clicking a link to the site) → allowed
    let allowed = Request::builder()
        .uri("/")
        .header("sec-fetch-site", "cross-site")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-dest", "document")
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(allowed).await.status(), StatusCode::OK);
}

#[tokio::test]
async fn repr_digest_and_rfc9421_signature_verify() {
    use base64::Engine;
    use ed25519_dalek::Verifier;

    // Public verification key from the published JWK.
    let jwk = body_string(get("/.well-known/http-msg-sig.jwk").await).await;
    let v: serde_json::Value = serde_json::from_str(&jwk).unwrap();
    let x = v["x"].as_str().expect("jwk.x");
    let pk: [u8; 32] = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(x)
        .unwrap()
        .try_into()
        .unwrap();
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&pk).unwrap();

    // A signed response. `sign_response` skips an empty authority, so set Host.
    let req = Request::builder()
        .uri("/")
        .header("host", "robertshalders.com")
        .body(Body::empty())
        .unwrap();
    let resp = send(req).await;
    let h = resp.headers();
    let rd = h.get("repr-digest").unwrap().to_str().unwrap().to_owned();
    let content_type = h
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let sig_input = h
        .get("signature-input")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let sig_hdr = h.get("signature").unwrap().to_str().unwrap().to_owned();

    assert!(rd.starts_with("sha-256=:"), "repr-digest = {rd}");
    let params = sig_input
        .strip_prefix("sig1=")
        .expect("signature-input prefix");

    // Reconstruct the RFC 9421 signature base and verify.
    let base = format!(
        "\"@authority\": robertshalders.com\n\"@path\": /\n\"content-type\": {content_type}\n\"repr-digest\": {rd}\n\"@signature-params\": {params}"
    );
    let sig_b64 = sig_hdr
        .strip_prefix("sig1=:")
        .and_then(|s| s.strip_suffix(':'))
        .expect("signature value format");
    let sig = ed25519_dalek::Signature::from_slice(
        &base64::engine::general_purpose::STANDARD
            .decode(sig_b64)
            .unwrap(),
    )
    .unwrap();
    vk.verify(base.as_bytes(), &sig)
        .expect("RFC 9421 response signature must verify against the published JWK");
}

/// `/static/*` assets are immutable + edge-cached, so they are NOT
/// buffered/digested/signed (a dynamic page still is - asserted above).
#[tokio::test]
async fn static_assets_are_not_digested_or_signed() {
    let h = get("/static/favicon.svg").await;
    assert_eq!(h.status(), StatusCode::OK);
    assert!(h.headers().get("repr-digest").is_none(), "static digested");
    assert!(h.headers().get("signature").is_none(), "static signed");
}

#[tokio::test]
async fn metrics_collapse_static_paths_f08() {
    // Two distinct /static paths (they 404, but are still recorded). Each must
    // collapse to a single `endpoint="/static"` label, not a raw per-path label.
    for p in ["/static/aaa-unique-1.css", "/static/bbb-unique-2.css"] {
        let resp = get(p).await;
        // Drain the body so the metric layer records the request.
        let _ = resp.into_body().collect().await.unwrap();
    }
    let text = shared().1.render();
    assert!(
        !text.contains("aaa-unique-1") && !text.contains("bbb-unique-2"),
        "raw /static paths must not appear as metric labels"
    );
    assert!(
        text.contains("endpoint=\"/static\""),
        "/static requests should collapse to a single endpoint label"
    );
}

#[tokio::test]
async fn metrics_collapse_unmatched_paths() {
    // Deep unmatched paths hit the 404 fallback, which carries no `MatchedPath`.
    // Each must collapse to a single `endpoint="/other"` label rather than a raw
    // per-URI label, keeping the metric label set bounded.
    for p in ["/zzz/unmatched-metric-1/x", "/zzz/unmatched-metric-2/y"] {
        let resp = get(p).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "GET {p} should 404");
        // Drain the body so the metric layer records the request.
        let _ = resp.into_body().collect().await.unwrap();
    }
    let text = shared().1.render();
    assert!(
        !text.contains("unmatched-metric-1") && !text.contains("unmatched-metric-2"),
        "raw unmatched paths must not appear as metric labels"
    );
    assert!(
        text.contains("endpoint=\"/other\""),
        "unmatched requests should collapse to a single endpoint label"
    );
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// First `<loc>` in a sitemap whose URL is a two-segment `/<collection>/<slug>`
/// article path; returns the path (without the site origin).
fn first_article_path(sitemap: &str) -> Option<String> {
    const COLLECTIONS: [&str; 4] = ["newsletters", "projects", "resources", "tricks"];
    for chunk in sitemap.split("<loc>").skip(1) {
        let url = chunk.split("</loc>").next()?;
        let Some(path) = url.strip_prefix("https://robertshalders.com") else {
            continue;
        };
        let segs: Vec<&str> = path.trim_matches('/').split('/').collect();
        if segs.len() == 2 && COLLECTIONS.contains(&segs[0]) {
            return Some(path.to_string());
        }
    }
    None
}
