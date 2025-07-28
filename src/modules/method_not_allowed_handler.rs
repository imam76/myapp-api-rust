use axum::{
  extract::Request,
  response::{IntoResponse, Response},
};
use tracing::warn;

use crate::errors::AppError;

pub async fn fallback(req: Request) -> Response {
  let method = req.method().clone();
  let uri = req.uri().clone();

  warn!("Method not allowed: {} {}", method, uri);

  let error_message = format!(
    "Method {} is not allowed for endpoint {}. Please check the API documentation for supported methods.",
    method, uri
  );

  let app_error = AppError::not_allowed(&error_message);
  app_error.into_response()
}
