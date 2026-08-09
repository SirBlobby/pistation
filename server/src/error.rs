use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("token error")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            AppError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, "bad_request", message.clone())
            }
            AppError::Unauthorized(message) => {
                (StatusCode::UNAUTHORIZED, "unauthorized", message.clone())
            }
            AppError::Forbidden(message) => (StatusCode::FORBIDDEN, "forbidden", message.clone()),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message.clone()),
            AppError::Conflict(message) => (StatusCode::CONFLICT, "conflict", message.clone()),
            AppError::Database(error) => {
                tracing::error!(%error, "database failure");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal server error".to_string(),
                )
            }
            AppError::Token(error) => {
                tracing::error!(%error, "token failure");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal server error".to_string(),
                )
            }
            AppError::Internal(message) => {
                tracing::error!(%message, "internal failure");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal server error".to_string(),
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
