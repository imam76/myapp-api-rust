use axum::{
  extract::{Path, State},
  response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
  AppResult,
  errors::AppError, 
  modules::auth::current_user::CurrentUser, 
  responses::ApiResponse,
  state::AppState
};

use super::workspace_models::{
  AddUserToWorkspaceRequest, CreateWorkspaceRequest, UpdateUserRoleRequest, UpdateWorkspaceRequest, Workspace, WorkspaceUserInfo, WorkspaceWithRole,
};

pub async fn create_workspace(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Json(request): Json<CreateWorkspaceRequest>,
) -> AppResult<Json<ApiResponse<Workspace>>> {
  let workspace = state.workspace_repository.create_and_assign_owner(request, current_user.user_id).await?;

  let response = ApiResponse::success(workspace, "Workspace created successfully");
  Ok(Json(response))
}

pub async fn get_workspace(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Path(workspace_id): Path<String>,
) -> AppResult<Json<ApiResponse<Workspace>>> {
  // Parse UUID with global error handling
  let workspace_id = workspace_id.parse::<Uuid>()?;

  // Check if user has access to this workspace
  let role = state
    .workspace_repository
    .check_user_workspace_access(current_user.user_id, workspace_id)
    .await?;

  if role.is_none() {
    return Err(AppError::Authorization("Access denied to workspace".to_string()));
  }

  let workspace = state.workspace_repository.get_workspace_by_id(workspace_id).await?.ok_or_else(|| {
    AppError::NotFound(crate::errors::NotFoundError {
      resource: "Workspace".to_string(),
      id: Some(workspace_id),
    })
  })?;

  let response = ApiResponse::success(workspace, "Workspace retrieved successfully");
  Ok(Json(response))
}

pub async fn update_workspace(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Path(workspace_id): Path<String>,
  Json(request): Json<UpdateWorkspaceRequest>,
) -> AppResult<Json<ApiResponse<Workspace>>> {
  // Parse UUID with global error handling
  let workspace_id = workspace_id.parse::<Uuid>()?;

  // Check if user is workspace owner
  let is_owner = state.workspace_repository.is_workspace_owner(current_user.user_id, workspace_id).await?;

  if !is_owner {
    return Err(AppError::Authorization("Only workspace owner can update workspace".to_string()));
  }

  let workspace = state.workspace_repository.update_workspace(workspace_id, &request).await?;

  let response = ApiResponse::success(workspace, "Workspace updated successfully");
  Ok(Json(response))
}

pub async fn delete_workspace(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Path(workspace_id): Path<String>,
) -> AppResult<Json<ApiResponse<()>>> {
  // Parse UUID with global error handling
  let workspace_id = workspace_id.parse::<Uuid>()?;

  // Check if user is workspace owner
  let is_owner = state.workspace_repository.is_workspace_owner(current_user.user_id, workspace_id).await?;

  if !is_owner {
    return Err(AppError::Authorization("Only workspace owner can delete workspace".to_string()));
  }

  state.workspace_repository.delete_workspace(workspace_id).await?;

  let response = ApiResponse::success((), "Workspace deleted successfully");
  Ok(Json(response))
}

pub async fn get_user_workspaces(State(state): State<Arc<AppState>>, current_user: CurrentUser) -> AppResult<Json<ApiResponse<Vec<WorkspaceWithRole>>>> {
  let workspaces = state.workspace_repository.get_user_workspaces(current_user.user_id).await?;

  let response = ApiResponse::success(workspaces, "User workspaces retrieved successfully");
  Ok(Json(response))
}

pub async fn get_workspace_users(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Path(workspace_id): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<WorkspaceUserInfo>>>> {
  // Parse UUID with global error handling
  let workspace_id = workspace_id.parse::<Uuid>()?;

  // Check if user has access to this workspace
  let role = state
    .workspace_repository
    .check_user_workspace_access(current_user.user_id, workspace_id)
    .await?;

  if role.is_none() {
    return Err(AppError::Authorization("Access denied to workspace".to_string()));
  }

  let users = state.workspace_repository.get_workspace_users(workspace_id).await?;

  let response = ApiResponse::success(users, "Workspace users retrieved successfully");
  Ok(Json(response))
}

pub async fn add_user_to_workspace(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Path(workspace_id): Path<String>,
  Json(request): Json<AddUserToWorkspaceRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
  // Parse UUID with global error handling
  let workspace_id = workspace_id.parse::<Uuid>()?;

  // Check if user is workspace owner
  let is_owner = state.workspace_repository.is_workspace_owner(current_user.user_id, workspace_id).await?;

  if !is_owner {
    return Err(AppError::Authorization("Only workspace owner can add users".to_string()));
  }

  state
    .workspace_repository
    .add_user_to_workspace(workspace_id, request.user_id, request.role)
    .await?;

  let response = ApiResponse::success((), "User added to workspace successfully");
  Ok(Json(response))
}

pub async fn remove_user_from_workspace(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Path((workspace_id, user_id)): Path<(String, String)>,
) -> AppResult<Json<ApiResponse<()>>> {
  // Parse UUIDs with global error handling
  let workspace_id = workspace_id.parse::<Uuid>()?;
  let user_id = user_id.parse::<Uuid>()?;

  // Check if user is workspace owner
  let is_owner = state.workspace_repository.is_workspace_owner(current_user.user_id, workspace_id).await?;

  if !is_owner {
    return Err(AppError::Authorization("Only workspace owner can remove users".to_string()));
  }

  // Prevent owner from removing themselves
  if user_id == current_user.user_id {
    return Err(AppError::BadRequest("Cannot remove workspace owner".to_string()));
  }

  state.workspace_repository.remove_user_from_workspace(workspace_id, user_id).await?;

  let response = ApiResponse::success((), "User removed from workspace successfully");
  Ok(Json(response))
}

pub async fn update_user_role(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  Path((workspace_id, user_id)): Path<(String, String)>,
  Json(request): Json<UpdateUserRoleRequest>,
) -> AppResult<Json<ApiResponse<()>>> {
  // Parse UUIDs with global error handling
  let workspace_id = workspace_id.parse::<Uuid>()?;
  let user_id = user_id.parse::<Uuid>()?;

  // Check if user is workspace owner
  let is_owner = state.workspace_repository.is_workspace_owner(current_user.user_id, workspace_id).await?;

  if !is_owner {
    return Err(AppError::Authorization("Only workspace owner can update user roles".to_string()));
  }

  state.workspace_repository.update_user_role(workspace_id, user_id, request.role).await?;

  let response = ApiResponse::success((), "User role updated successfully");
  Ok(Json(response))
}
