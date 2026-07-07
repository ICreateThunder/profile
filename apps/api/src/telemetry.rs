// SPDX-License-Identifier: AGPL-3.0-or-later
//! Observability init. Logs are structured JSON to stdout (collected by the
//! platform's log pipeline). OTLP span export is added behind the `otlp`
//! feature and only activates when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.

use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialise the global tracing subscriber. Returns a guard that, when dropped
/// at shutdown, flushes any pending OTLP spans (a no-op without the `otlp`
/// feature).
pub fn init() -> Guard {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,portfolio_api=debug"));

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json().with_current_span(true));

    #[cfg(feature = "otlp")]
    let registry = registry.with(otlp::layer());

    registry.init();
    Guard
}

/// Dropped at shutdown to flush exporters.
pub struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        #[cfg(feature = "otlp")]
        otlp::shutdown();
    }
}

#[cfg(feature = "otlp")]
mod otlp {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::Resource;
    use std::sync::OnceLock;
    use tracing_subscriber::Layer;

    static PROVIDER: OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> = OnceLock::new();

    /// Build the OpenTelemetry tracing layer. No-op (returns an inert layer) when
    /// `OTEL_EXPORTER_OTLP_ENDPOINT` is unset, so traces simply don't flow until
    /// the collector lands.
    pub fn layer<S>() -> Option<Box<dyn Layer<S> + Send + Sync>>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync,
    {
        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok()?;
        let service = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "portfolio-api".into());

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .ok()?;

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(
                Resource::builder()
                    .with_service_name(service.clone())
                    .build(),
            )
            .build();

        let tracer = provider.tracer(service);
        let _ = PROVIDER.set(provider);
        Some(tracing_opentelemetry::layer().with_tracer(tracer).boxed())
    }

    pub fn shutdown() {
        if let Some(p) = PROVIDER.get() {
            let _ = p.shutdown();
        }
    }
}
