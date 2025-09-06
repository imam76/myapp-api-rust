use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};

use crate::{
  errors::{AppError, AuthError},
  modules::auth::{
    auth_service::{login_user, register_user},
    current_user::CurrentUser,
    user_dto::{LoginUserDto, RegisterUserDto},
  },
  state::AppState,
};

pub async fn register_user_handler(
  State(state): State<Arc<AppState>>,
  Json(body): Json<RegisterUserDto>,
) -> Result<(StatusCode, Json<Value>), AppError> {
  let (user, workspace) = register_user(state, body).await?;
  let user_response = json!({"status": "success", "user": user, "workspace": workspace});
  Ok((StatusCode::CREATED, Json(user_response)))
}

pub async fn login_user_handler(State(state): State<Arc<AppState>>, Json(body): Json<LoginUserDto>) -> Result<(StatusCode, Json<Value>), AppError> {
  let (token, user) = login_user(state.clone(), body).await?;
  let workspace = state.clone().workspace_repository.get_user_workspaces(user.id).await?;
  let token_response = json!({"status": "success", "token": token, "user": user, "workspace": workspace});
  Ok((StatusCode::OK, Json(token_response)))
}

/// Protected endpoint that returns information about the current authenticated user.
///
/// This handler demonstrates how to use the `CurrentUser` extractor to access
/// the authenticated user's information in protected routes.
/// 
/// Note: With RLS enabled, the workspace query will automatically be filtered
/// based on the current session variables set by the JWT middleware.
pub async fn get_current_user_handler(State(state): State<Arc<AppState>>, current_user: CurrentUser) -> Result<(StatusCode, Json<Value>), AppError> {
  // Find the user in the database using the ID from the JWT token
  let user = state.auth_repository.find_by_id(current_user.user_id).await?;

  if let Some(user) = user {
    // Get user default workspace - this query is now protected by RLS and will only return
    // the default workspace accessible to the current user based on session variables
    let workspace = state.workspace_repository.get_user_default_workspace(user.id).await?;
    let response = json!({"status": "success", "user": user, "workspace": workspace});
    Ok((StatusCode::OK, Json(response)))
  } else {
    // If JWT token is valid but user doesn't exist in database,
    // this indicates an invalid/expired token or data inconsistency
    // Return authentication error instead of not found error
    Err(AppError::Authentication(AuthError::InvalidToken))
  }
}
