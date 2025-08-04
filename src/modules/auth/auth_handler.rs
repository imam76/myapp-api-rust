use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};

use crate::{
  errors::{AppError, NotFoundError},
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
  let user_response =
    json!({"status": "success", "user": serde_json::to_value(user).unwrap(), "workspace": serde_json::to_value(workspace).unwrap()});
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
pub async fn get_current_user_handler(State(state): State<Arc<AppState>>, current_user: CurrentUser) -> Result<Json<Value>, AppError> {
  // Find the user in the database using the ID from the JWT token
  let user = state.auth_repository.find_by_id(current_user.user_id).await?;

  if let Some(user) = user {
    let workspace = state.workspace_repository.get_user_workspaces(user.id).await?;
    let response = json!({"status": "success", "user": user, "workspace": workspace});
    Ok((StatusCode::OK, Json(response)))
  } else {
    Err(AppError::NotFound(NotFoundError {
      resource: "User".to_string(),
      id: Some(current_user.user_id),
    }))
  }
}
