// SPDX-License-Identifier: AGPL-3.0-or-later
//! Library crate root for `portfolio-api`.
//!
//! `main.rs` is a thin binary over this: it initialises telemetry, builds the
//! state and router via [`AppState::from_disk`] + [`build_app`], and serves.
//! Integration tests in `tests/` drive [`build_app`] directly, in-process (no
//! network), so the full middleware stack - security headers, CSP, Repr-Digest,
//! RFC 9421 signing, Fetch-Metadata, metric-label grouping - is under test.
#![forbid(unsafe_code)]

pub mod content;
pub mod error;
pub mod httpsec;
mod prelude;
pub mod routes;
pub mod security;
pub mod telemetry;
pub mod templates;

use axum::Router;
use axum::routing::get;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::{EndpointLabel, PrometheusMetricLayerBuilder};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Compiled Tailwind CSS, loaded once at startup and inlined into every HTML response.
    /// Eliminates the render-blocking stylesheet request - no FOUC on cold cache.
    pub inline_css: Arc<str>,
    /// Base64 SHA-256 of `inline_css` - the CSP `style-src` hash, precomputed once.
    pub style_hash: Arc<str>,
    /// All articles loaded from markdown at startup - no filesystem I/O at request time.
    pub content: Arc<content::ContentStore>,
    /// Ed25519 signer for RFC 9421 HTTP Message Signatures on responses.
    pub signer: Arc<httpsec::Signer>,
}

impl AppState {
    /// Render context (inlined CSS + its CSP hash) threaded into the templates.
    pub fn render(&self) -> templates::layout::Render<'_> {
        templates::layout::Render {
            inline_css: &self.inline_css,
            style_hash: &self.style_hash,
            content: &self.content,
        }
    }

    /// Build state from disk: compiled CSS (`static/styles.css`), markdown
    /// content (`src/content`), and the response signer. Used by `main` and by
    /// the integration tests (which run with the crate dir as the CWD, so the
    /// relative paths resolve).
    pub fn from_disk() -> Self {
        // Compiled CSS - inlined into every HTML response.
        let css = std::fs::read_to_string("static/styles.css")
            .expect("static/styles.css not found - run `cargo build` to compile Tailwind");
        // Markdown content, loaded once.
        let content = content::ContentStore::load(Path::new("src/content"));
        // CSP style-src hash - precomputed once (inlined CSS is constant per process).
        let style_hash = security::sha256_b64(&css);
        // Response signing key (RFC 9421): stable if RESPONSE_SIG_SEED is set,
        // otherwise ephemeral per process (warns - see httpsec::Signer::from_env).
        let signer = Arc::new(httpsec::Signer::from_env());

        AppState {
            inline_css: Arc::from(css),
            style_hash: Arc::from(style_hash),
            content: Arc::new(content),
            signer,
        }
    }
}

/// Assemble the public application router with the full response-path middleware
/// stack, and return the Prometheus handle for the (separately served) metrics
/// endpoint.
///
/// Endpoint label for requests that carry no `MatchedPath` - the nested
/// `ServeDir` under `/static`, and the 404 fallback. Each source collapses to
/// one fixed label so a raw URI never widens the metric label set. See
/// [`build_app`].
fn group_unmatched_endpoint(path: &str) -> String {
    if path == "/static" || path.starts_with("/static/") {
        "/static".to_owned()
    } else {
        "/other".to_owned()
    }
}

/// The metric layer keeps the `endpoint` label cardinality bounded. Requests
/// that carry no `MatchedPath` - the nested `ServeDir` under `/static`, and the
/// 404 fallback - would otherwise be labelled by their raw URI, so an unbounded
/// set of paths could produce an unbounded set of time-series. A fallback
/// function ([`group_unmatched_endpoint`]) collapses those two sources to the
/// fixed labels `/static` and `/other`. Every real route carries a `MatchedPath`
/// (its pattern, e.g. `/{collection}/{slug}`), so it is labelled by that pattern
/// and path params never widen cardinality.
pub fn build_app(state: AppState) -> (Router, PrometheusHandle) {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_endpoint_label_type(EndpointLabel::MatchedPathWithFallbackFn(
            group_unmatched_endpoint,
        ))
        .with_default_metrics()
        .build_pair();

    // Page routes - cached at Cloudflare edge (s-maxage) and browser (max-age)
    let pages = Router::new()
        .route("/", get(routes::pages::home))
        .route("/profile", get(routes::pages::profile))
        .route("/articles", get(routes::pages::articles))
        .route("/{collection}", get(routes::pages::collection))
        .route("/{collection}/{slug}", get(routes::pages::article))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("public, max-age=60, s-maxage=300"),
        ));

    let app = Router::new()
        .merge(pages)
        // Feeds - cached at edge, XML content type
        .route("/rss.xml", get(routes::feeds::rss))
        .route("/sitemap.xml", get(routes::feeds::sitemap))
        // Text endpoints - robots, humans, RFC 9116 security.txt
        .route("/robots.txt", get(routes::meta::robots))
        .route("/humans.txt", get(routes::meta::humans))
        .route("/.well-known/security.txt", get(routes::meta::security_txt))
        // Web Key Directory (WKD) - OpenPGP key lookup by email. Direct method,
        // plus the advanced-method path for when served at openpgpkey.<domain>.
        .route(
            "/.well-known/openpgpkey/policy",
            get(routes::meta::wkd_policy),
        )
        .route(
            "/.well-known/openpgpkey/hu/{hash}",
            get(routes::meta::wkd_key),
        )
        .route(
            "/.well-known/openpgpkey/{domain}/policy",
            get(routes::meta::wkd_policy),
        )
        .route(
            "/.well-known/openpgpkey/{domain}/hu/{hash}",
            get(routes::meta::wkd_key_advanced),
        )
        // WebFinger (RFC 7033) + Global Privacy Control + response-signing JWK
        .route("/.well-known/webfinger", get(routes::meta::webfinger))
        .route("/.well-known/gpc.json", get(routes::meta::gpc))
        .route("/.well-known/http-msg-sig.jwk", get(routes::meta::sig_jwk))
        // Hidden easter egg - RFC 2324 (HTCPCP)
        .route("/teapot", get(routes::meta::teapot))
        // Health probes - no caching (K8s needs real-time liveness).
        // `/healthz` + `/readyz` are kubelet-facing but publicly reachable (the
        // platform routes `/` as a catch-all); they return a bare 200 with no
        // body, so that exposure discloses nothing. `/up` is the public probe
        // for external uptime monitors.
        .route("/healthz", get(routes::health::liveness))
        .route("/readyz", get(routes::health::readiness))
        .route("/up", get(routes::health::up))
        // Static assets (Tailwind CSS, fonts, images) - immutable caching set in static_files
        .nest_service("/static", routes::static_files::serve())
        // 404 fallback - themed error page for unmatched routes
        .fallback(routes::fallback::not_found)
        // --- Response-path layers, listed inner→outer ---
        // 1) Repr-Digest over the uncompressed representation (innermost).
        .layer(axum::middleware::from_fn(httpsec::repr_digest))
        // 2) RFC 9421 signature, covering the digest just added.
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            httpsec::sign_response,
        ))
        // 3) Cross-origin isolation.
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("cross-origin-opener-policy"),
            axum::http::HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("cross-origin-resource-policy"),
            axum::http::HeaderValue::from_static("same-origin"),
        ))
        // Global security headers. HSTS is owned by Cloudflare at the edge.
        //
        // HTML pages set their own hash-based CSP in the layout (script-src hash
        // varies per page). This `if_not_present` layer backstops every OTHER
        // response - static assets, feeds, ServeDir 404s, and crucially any
        // self-hosted SVG opened as a top-level document - with a locked-down
        // policy, so active content can never execute unsandboxed.
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::from_static(
                "default-src 'none'; frame-ancestors 'none'; base-uri 'none'",
            ),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("permissions-policy"),
            axum::http::HeaderValue::from_static(
                "camera=(), microphone=(), geolocation=(), browsing-topics=()",
            ),
        ))
        // 4) Fetch-Metadata isolation - reject cross-site non-navigation requests.
        .layer(axum::middleware::from_fn(httpsec::fetch_metadata))
        .layer(CompressionLayer::new())
        .layer(prometheus_layer)
        // Origin-side request guards (defence-in-depth; mTLS-to-Cloudflare is the
        // primary control). TimeoutLayer bounds total handling time; the body
        // limit caps any request body a handler reads - every current route is
        // GET, so it forward-guards future body-handling endpoints.
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(15),
        ))
        .with_state(state);

    (app, metric_handle)
}
