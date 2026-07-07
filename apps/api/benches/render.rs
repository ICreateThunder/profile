// SPDX-License-Identifier: AGPL-3.0-or-later
//! Benchmarks for the hot paths. Run with `cargo bench -p portfolio-api`.
//! Criterion persists baselines under `target/criterion/`, so re-running after a
//! change reports a regression/improvement delta - this is the perf-regression
//! guard that complements the functional regression tests in `tests/http.rs`.

use std::hint::black_box;
use std::path::Path;

use axum::body::Body;
use axum::http::Request;
use criterion::{Criterion, criterion_group, criterion_main};
use http_body_util::BodyExt;
use tower::ServiceExt;

use portfolio_api::content::ContentStore;
use portfolio_api::{AppState, build_app, security};

/// Markdown parse + graph build at startup.
fn bench_content_load(c: &mut Criterion) {
    c.bench_function("content_load", |b| {
        b.iter(|| black_box(ContentStore::load(black_box(Path::new("src/content")))));
    });
}

/// CSP/Repr-Digest hashing over a representative (CSS-sized) input.
fn bench_sha256(c: &mut Criterion) {
    let css = std::fs::read_to_string("static/styles.css").unwrap_or_default();
    c.bench_function("sha256_b64_css", |b| {
        b.iter(|| black_box(security::sha256_b64(black_box(&css))));
    });
}

/// Full request path for the landing page: render + Repr-Digest + Ed25519 sign +
/// the header/metric layers. This is the real per-request cost on a cache miss.
fn bench_request(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // Build the app once, inside the runtime (build_pair installs a global
    // recorder + spawns an upkeep task, so it needs a Tokio reactor).
    let (app, _handle) = rt.block_on(async { build_app(AppState::from_disk()) });

    c.bench_function("get_home_full_stack", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let resp = app
                    .oneshot(
                        Request::builder()
                            .uri("/")
                            .header("host", "robertshalders.com")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                black_box(resp.into_body().collect().await.unwrap().to_bytes());
            }
        });
    });
}

criterion_group!(benches, bench_content_load, bench_sha256, bench_request);
criterion_main!(benches);
