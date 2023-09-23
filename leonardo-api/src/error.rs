use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub struct AppError(anyhow::Error);

#[derive(Serialize)]
pub struct ErrorInfo {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        self.print_error();
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorInfo {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

impl AppError {
    pub fn print_error(&self) {
        error!("There was an error!\nInfo: {}", self.0.to_string());
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
