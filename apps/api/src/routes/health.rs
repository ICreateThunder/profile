// SPDX-License-Identifier: AGPL-3.0-or-later
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;

/// Liveness probe - if this fails, K8s restarts the pod.
/// Returns 200 as long as the process is running.
///
/// For the kubelet, but publicly reachable: the platform routes `/` as a
/// catch-all, so this is served through Cloudflare like any other path - it
/// intentionally returns no body, so public exposure discloses nothing.
pub async fn liveness() -> impl IntoResponse {
    StatusCode::OK
}

/// Readiness probe - if this fails, K8s removes the pod from the Service.
/// Returns 200 when the application is ready to serve traffic.
///
/// For the kubelet, but publicly reachable: the platform routes `/` as a
/// catch-all, so this is served through Cloudflare like any other path - it
/// intentionally returns no body, so public exposure discloses nothing.
pub async fn readiness() -> impl IntoResponse {
    StatusCode::OK
}

/// Public uptime probe - for external canary/status monitors, deliberately
/// distinct from the cluster-only `liveness`/`readiness`. Explicitly **public**
/// (route it through the edge) and **uncacheable** (`no-store`) so a prober sees
/// the true origin state rather than a cached `200` that would mask an outage.
pub async fn up() -> impl IntoResponse {
    (
        [
            (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            ),
        ],
        "{\"status\":\"ok\"}",
    )
}
