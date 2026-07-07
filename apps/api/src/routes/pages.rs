// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::AppState;
use crate::content::COLLECTIONS;
use crate::templates;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// GET / - landing page
pub async fn home(State(state): State<AppState>) -> impl IntoResponse {
    let r = state.render();
    templates::pages::home(&r)
}

/// GET /profile - professional profile
pub async fn profile(State(state): State<AppState>) -> impl IntoResponse {
    let r = state.render();
    templates::pages::profile(&r)
}

/// GET /articles - all articles across collections (round-robin interleaved)
pub async fn articles(State(state): State<AppState>) -> impl IntoResponse {
    let all = state.content.round_robin_articles(5);
    let r = state.render();
    templates::pages::articles(&r, &all)
}

/// GET /:collection - list articles in a collection
pub async fn collection(
    State(state): State<AppState>,
    Path(collection): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let r = state.render();
    if !COLLECTIONS.contains(&collection.as_str()) {
        return Err((StatusCode::NOT_FOUND, templates::pages::not_found(&r)));
    }

    let articles = state
        .content
        .by_collection
        .get(&collection)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    Ok(templates::pages::collection_page(&r, &collection, articles))
}

/// GET /:collection/:slug - individual article
pub async fn article(
    State(state): State<AppState>,
    Path((collection, slug)): Path<(String, String)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let r = state.render();
    let key = (collection, slug);
    match state.content.by_slug.get(&key) {
        Some(article) => Ok(templates::pages::article_page(&r, article)),
        None => Err((StatusCode::NOT_FOUND, templates::pages::not_found(&r))),
    }
}
