// SPDX-License-Identifier: AGPL-3.0-or-later
use axum::http::HeaderValue;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeader;

/// Serve static files from the `static/` directory with long-lived immutable
/// caching. These assets (self-hosted fonts, vendored htmx, images) change
/// rarely; when they do, the deploy purges the Cloudflare edge cache.
/// Content-hashed filenames are a future refinement that would remove the need
/// to purge.
pub fn serve() -> SetResponseHeader<ServeDir, HeaderValue> {
    SetResponseHeader::overriding(
        ServeDir::new("static"),
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    )
}
