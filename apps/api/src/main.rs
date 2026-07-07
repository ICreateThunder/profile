// SPDX-License-Identifier: AGPL-3.0-or-later
//! Binary entry point. The app itself lives in the library crate
//! (`portfolio_api`) so it is testable in-process; this file only wires up
//! telemetry, builds the router via [`portfolio_api::build_app`], and serves the
//! public app and the Prometheus metrics endpoint on their separate ports.
#![forbid(unsafe_code)]

use axum::Router;
use axum::routing::get;
use portfolio_api::{AppState, build_app, telemetry};

#[tokio::main]
async fn main() {
    // Structured JSON logs to stdout; OTLP spans when built with `--features otlp`.
    let _telemetry = telemetry::init();

    // Load CSS + content + signer from disk.
    let state = AppState::from_disk();
    tracing::info!(key = %state.signer.fingerprint(), "response signing key ready");

    // Public app router + the metrics handle (served separately, below).
    let (app, metric_handle) = build_app(state);

    // Metrics app - Prometheus exposition on its own port (scraped by VMServiceScrape),
    // never exposed through the public gateway.
    let metrics_app = Router::new().route(
        "/metrics",
        get(move || {
            let handle = metric_handle.clone();
            async move { handle.render() }
        }),
    );

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let metrics_port = std::env::var("METRICS_PORT").unwrap_or_else(|_| "9090".to_string());
    let app_addr = format!("0.0.0.0:{port}");
    let metrics_addr = format!("0.0.0.0:{metrics_port}");
    tracing::info!(%app_addr, %metrics_addr, "portfolio listening");

    let app_listener = tokio::net::TcpListener::bind(&app_addr)
        .await
        .expect("failed to bind app TCP listener");
    let metrics_listener = tokio::net::TcpListener::bind(&metrics_addr)
        .await
        .expect("failed to bind metrics TCP listener");

    let app_server = axum::serve(app_listener, app).with_graceful_shutdown(shutdown_signal());
    let metrics_server =
        axum::serve(metrics_listener, metrics_app).with_graceful_shutdown(shutdown_signal());

    let (app_res, metrics_res) = tokio::join!(app_server, metrics_server);
    app_res.expect("app server error");
    metrics_res.expect("metrics server error");
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received ctrl+c, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
