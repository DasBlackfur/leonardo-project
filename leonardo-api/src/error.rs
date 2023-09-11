use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use tracing::error;

pub struct AppError(anyhow::Error);

#[derive(Serialize)]
pub struct ErrorInfo {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("There was an error!\nInfo: {}", self.0.to_string());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorInfo {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
