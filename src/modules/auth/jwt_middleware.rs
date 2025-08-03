use axum::{
  extract::{Request, State},
  http::{HeaderName, HeaderValue, header::AUTHORIZATION},
  middleware::Next,
  response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::sync::Arc;

use crate::{
  errors::{AppError, AuthError},
  modules::auth::auth_service::Claims,
  state::AppState,
};

pub async fn jwt_middleware(State(state): State<Arc<AppState>>, mut request: Request, next: Next) -> Result<Response, AppError> {
  // Get token from Authorization header
  let auth_header = request
    .headers()
    .get(AUTHORIZATION)
    .and_then(|header| header.to_str().ok())
    .ok_or(AppError::Authentication(AuthError::MissingToken))?;

  // Get workspace_id from header
  let workspace_id = request
    .headers()
    .get("X-Workspace-ID")
    .and_then(|header| header.to_str().ok())
    .map(|s| s.to_string());

  if !auth_header.starts_with("Bearer ") {
    return Err(AppError::Authentication(AuthError::InvalidToken));
  }

  let token = auth_header[7..].to_string();

  // Validate JWT token
  let claims = decode::<Claims>(&token, &DecodingKey::from_secret(state.jwt_secret.as_ref()), &Validation::default())
    .map_err(|_| AppError::Authentication(AuthError::InvalidToken))?
    .claims;

  // Add user to request
  request.extensions_mut().insert(claims.sub);

  // Process request
  let mut response = next.run(request).await;

  // Add response headers
  response
    .headers_mut()
    .insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap());

  if let Some(ws_id) = workspace_id {
    response
      .headers_mut()
      .insert(HeaderName::from_static("x-workspace-id"), HeaderValue::from_str(&ws_id).unwrap());
  }

  Ok(response)
}
