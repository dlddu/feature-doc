//! Application error type. Every variant maps to an HTTP response with a
//! user-safe message; the `Display`/`Debug` forms deliberately never embed a
//! credential value (AC4.3c). Internal causes are logged at the boundary, not
//! returned to the client.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("authentication required")]
    Unauthorized,

    #[error("not found")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),

    /// Surfaced to the user verbatim — used for the AC4.2 revocation message.
    #[error("{0}")]
    Blocked(String),

    #[error("upstream provider rejected the request")]
    ProviderRejected,

    /// Anything unexpected. The cause is logged, never sent to the client.
    #[error("internal error")]
    Internal,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Blocked(_) => StatusCode::FORBIDDEN,
            AppError::ProviderRejected => StatusCode::BAD_GATEWAY,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = %self, "request failed");
        }
        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}

// Convert common internal failures into a redacting `Internal`. We log the
// raw cause here (these crate errors carry no secret material) and hand the
// client only the opaque message.
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "database error");
        AppError::Internal
    }
}

impl From<crate::crypto::CryptoError> for AppError {
    fn from(e: crate::crypto::CryptoError) -> Self {
        tracing::error!(error = %e, "crypto error");
        AppError::Internal
    }
}
