use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::models::ErrorBody;

pub enum AppError {
    NotFound,
    InvalidUrl,
    Db(sqlx::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Db(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "link not found".to_string()),
            AppError::InvalidUrl => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "url must start with http:// or https://".to_string(),
            ),
            AppError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
