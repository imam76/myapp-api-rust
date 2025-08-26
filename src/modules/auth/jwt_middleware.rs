use axum::{
  extract::{Request, State},
  http::{HeaderName, HeaderValue, header::AUTHORIZATION},
  middleware::Next,
  response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

use crate::modules::datastores::workspaces::workspace_models::WorkspaceRole;

use crate::{
  errors::{AppError, AuthError},
  modules::auth::{auth_service::Claims, current_user::{UserId, WorkspaceId}},
  state::AppState,
  utils::PostgresSessionExt,
};

pub async fn jwt_middleware(State(state): State<Arc<AppState>>, mut request: Request, next: Next) -> Result<Response, AppError> {
  // Get token from Authorization header
  let auth_header = request
    .headers()
    .get(AUTHORIZATION)
    .and_then(|header| header.to_str().ok())
    .ok_or(AppError::Authentication(AuthError::MissingToken))?;

  // Get workspace_id from header and parse as UUID
  let workspace_id = request
    .headers()
    .get("X-Workspace-ID")
    .and_then(|header| header.to_str().ok())
    .and_then(|s| Uuid::parse_str(s).ok());

  if !auth_header.starts_with("Bearer ") {
    return Err(AppError::Authentication(AuthError::InvalidToken));
  }

  let token = auth_header[7..].to_string();

  // Validate JWT token
  let claims = decode::<Claims>(&token, &DecodingKey::from_secret(state.jwt_secret.as_ref()), &Validation::default())
    .map_err(|e| {
      error!("JWT validation failed: {}", e);
      AppError::Authentication(AuthError::InvalidToken)
    })?
    .claims;

  // Get user_id from claims
  let user_id = claims.sub;

  if let Some(ws_id) = workspace_id {
    // Check access and get role
    let role_access = sqlx::query!(
      "SELECT role as \"role!: WorkspaceRole\" 
       FROM workspace_users 
       WHERE user_id = $1 AND workspace_id = $2",
      user_id,
      ws_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
      error!("Failed to verify workspace access: {}", e);
      AppError::Internal("Database error while verifying workspace access".to_string())
    })?;

    match role_access {
      Some(row) => {
        // Add role to request extensions for route-level authorization
        request.extensions_mut().insert(row.role);
      }
      None => return Err(AppError::Authentication(AuthError::InvalidWorkspace)),
    }
  }

  // Set database session settings for RLS
  if let Err(e) = state.db.set_session_settings(&user_id, workspace_id.as_ref()).await {
    error!("Failed to set session settings: {}", e);
    // Convert SQLx error to AppError properly
    return Err(AppError::Internal(format!("Failed to set database session: {}", e)));
  }

  debug!("Session settings configured for user: {}, workspace: {:?}", user_id, workspace_id);

  // Add user to request using typed wrapper
  request.extensions_mut().insert(UserId(user_id));

  // Add workspace_id to request if present using typed wrapper
  if let Some(ws_id) = workspace_id {
    request.extensions_mut().insert(WorkspaceId(ws_id));
  }

  // Process request
  let mut response = next.run(request).await;

  if let Err(e) = state.db.clear_session_settings().await {
    error!("Failed to clear session settings: {}", e);
    // Do not fail the request, just log the error.
  }

  // Add response headers
  response
    .headers_mut()
    .insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap());

  if let Some(ws_id) = workspace_id {
    response.headers_mut().insert(
      HeaderName::from_static("x-workspace-id"),
      HeaderValue::from_str(&ws_id.to_string()).unwrap(),
    );
  }

  Ok(response)
}
