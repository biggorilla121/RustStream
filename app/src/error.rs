use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("TMDB API error: {0}")]
    Tmdb(String),

    #[error("Vidking API error: {0}")]
    Vidking(String),

    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    Internal,

    #[error("Internal error: {0}")]
    InternalWithMessage(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalWithMessage(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message): (StatusCode, String) = match &self {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            AppError::Http(_) => (StatusCode::INTERNAL_SERVER_ERROR, "HTTP error".to_string()),
            AppError::Tmdb(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::Vidking(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::InternalWithMessage(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", msg),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
