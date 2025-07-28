use axum::{
  extract::Request,
  response::{IntoResponse, Response},
};
use tracing::warn;

use crate::errors::AppError;

pub async fn fallback(req: Request) -> Response {
  let method = req.method().clone();
  let uri = req.uri().clone();

  warn!("Method not found: {} {}", method, uri);

  let error_message = format!(
    "Enpoint {} - {} is not found. please check the API documentation for available endpoints.",
    method, uri
  );

  let app_error = AppError::not_found(&error_message);
  app_error.into_response()
}
