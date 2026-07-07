// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::AppState;
use crate::templates;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Fallback handler for unmatched routes - returns 404 with themed error page.
pub async fn not_found(State(state): State<AppState>) -> impl IntoResponse {
    let r = state.render();
    (StatusCode::NOT_FOUND, templates::pages::not_found(&r))
}
