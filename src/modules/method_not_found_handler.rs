use axum::{
  http::Uri,
  response::{IntoResponse, Response},
};
use tracing::warn;

use crate::errors::{AppError, NotFoundError};

pub async fn fallback(uri: Uri) -> Response {
  warn!("Route not found: {}", uri);

  let error_message = format!(
    "The requested endpoint '{}' does not exist. Please check the API documentation for available endpoints.",
    uri
  );

  let app_error = AppError::NotFound(NotFoundError {
    resource: error_message,
    id: None,
  });
  app_error.into_response()
}
