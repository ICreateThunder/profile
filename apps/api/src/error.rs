// SPDX-License-Identifier: AGPL-3.0-or-later
//! The application error type and `Result` alias. Today the request paths are
//! infallible (content is in memory), so the variants are foundational - they
//! are populated as fallible request paths land. The HTTP mapping logs full
//! detail server-side and returns an opaque status to the client.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[allow(dead_code)] // used as fallible request paths land
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[allow(dead_code)] // variants are constructed as fallible request paths land
pub enum Error {
    /// A requested entity does not exist.
    NotFound { entity: &'static str },
    /// The caller is not permitted to perform the action.
    Unauthorized,
    /// An unexpected internal failure (DB, IO, …) - never surfaced to clients.
    Internal(String),
}

// Display = Debug keeps the enum the single source of truth.
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Full detail to logs/traces; opaque body to the client.
        tracing::error!(error = ?self, "request error");
        let status = match &self {
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, "").into_response()
    }
}
