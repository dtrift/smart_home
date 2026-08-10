//! REST API errors and their mapping to HTTP responses.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use smart_home::SmartHomeError;

/// API error.
#[derive(Debug)]
pub enum ApiError {
    /// Room or device not found.
    NotFound(String),
    /// Bad request (e.g. negative power).
    BadRequest(String),
    /// Conflict (a room/device with this key already exists).
    Conflict(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}

impl From<SmartHomeError> for ApiError {
    fn from(err: SmartHomeError) -> Self {
        ApiError::NotFound(err.to_string())
    }
}
