//! Uniform HTTP error type.
//!
//! Client-facing messages are intentionally terse and never echo credential
//! material. `Internal` carries an operator-only detail that is logged (never
//! returned) — and even that is constructed by us, so secrets never flow in.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    Forbidden,
    NotFound,
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl AppError {
    pub fn internal(detail: impl Into<String>) -> Self {
        AppError::Internal(detail.into())
    }

    fn parts(&self) -> (StatusCode, String) {
        match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".into()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let AppError::Internal(detail) = &self {
            tracing::error!(detail = %detail, "request failed");
        }
        let (status, message) = self.parts();
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(format!("db: {e}"))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
