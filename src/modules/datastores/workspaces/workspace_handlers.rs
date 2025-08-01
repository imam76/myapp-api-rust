use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    modules::auth::current_user::CurrentUser,
    state::AppState,
};

use super::{
    workspace_models::{
        CreateWorkspaceRequest, UpdateWorkspaceRequest, AddUserToWorkspaceRequest,
        UpdateUserRoleRequest, Workspace, WorkspaceWithRole, WorkspaceUserInfo
    },
};

pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(request): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>, AppError> {
    let workspace = state
        .workspace_repository
        .create_workspace(&request, current_user.user_id)
        .await?;
    
    Ok(Json(workspace))
}

pub async fn get_workspace(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Workspace>, AppError> {
    // Check if user has access to this workspace
    let role = state
        .workspace_repository
        .check_user_workspace_access(current_user.user_id, workspace_id)
        .await?;
    
    if role.is_none() {
        return Err(AppError::Authorization("Access denied to workspace".to_string()));
    }
    
    let workspace = state
        .workspace_repository
        .get_workspace_by_id(workspace_id)
        .await?
        .ok_or_else(|| AppError::NotFound(crate::errors::NotFoundError {
            resource: "Workspace".to_string(),
            id: Some(workspace_id),
        }))?;
    
    Ok(Json(workspace))
}

pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(workspace_id): Path<Uuid>,
    Json(request): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>, AppError> {
    // Check if user is workspace owner
    let is_owner = state
        .workspace_repository
        .is_workspace_owner(current_user.user_id, workspace_id)
        .await?;
    
    if !is_owner {
        return Err(AppError::Authorization("Only workspace owner can update workspace".to_string()));
    }
    
    let workspace = state
        .workspace_repository
        .update_workspace(workspace_id, &request)
        .await?;
    
    Ok(Json(workspace))
}

pub async fn delete_workspace(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(workspace_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check if user is workspace owner
    let is_owner = state
        .workspace_repository
        .is_workspace_owner(current_user.user_id, workspace_id)
        .await?;
    
    if !is_owner {
        return Err(AppError::Authorization("Only workspace owner can delete workspace".to_string()));
    }
    
    state
        .workspace_repository
        .delete_workspace(workspace_id)
        .await?;
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_user_workspaces(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
) -> Result<Json<Vec<WorkspaceWithRole>>, AppError> {
    let workspaces = state
        .workspace_repository
        .get_user_workspaces(current_user.user_id)
        .await?;
    
    Ok(Json(workspaces))
}

pub async fn get_workspace_users(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(workspace_id): Path<Uuid>,
) -> Result<Json<Vec<WorkspaceUserInfo>>, AppError> {
    // Check if user has access to this workspace
    let role = state
        .workspace_repository
        .check_user_workspace_access(current_user.user_id, workspace_id)
        .await?;
    
    if role.is_none() {
        return Err(AppError::Authorization("Access denied to workspace".to_string()));
    }
    
    let users = state
        .workspace_repository
        .get_workspace_users(workspace_id)
        .await?;
    
    Ok(Json(users))
}

pub async fn add_user_to_workspace(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(workspace_id): Path<Uuid>,
    Json(request): Json<AddUserToWorkspaceRequest>,
) -> Result<Json<()>, AppError> {
    // Check if user is workspace owner
    let is_owner = state
        .workspace_repository
        .is_workspace_owner(current_user.user_id, workspace_id)
        .await?;
    
    if !is_owner {
        return Err(AppError::Authorization("Only workspace owner can add users".to_string()));
    }
    
    state
        .workspace_repository
        .add_user_to_workspace(workspace_id, request.user_id, request.role)
        .await?;
    
    Ok(Json(()))
}

pub async fn remove_user_from_workspace(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Check if user is workspace owner
    let is_owner = state
        .workspace_repository
        .is_workspace_owner(current_user.user_id, workspace_id)
        .await?;
    
    if !is_owner {
        return Err(AppError::Authorization("Only workspace owner can remove users".to_string()));
    }
    
    // Prevent owner from removing themselves
    if user_id == current_user.user_id {
        return Err(AppError::BadRequest("Cannot remove workspace owner".to_string()));
    }
    
    state
        .workspace_repository
        .remove_user_from_workspace(workspace_id, user_id)
        .await?;
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> Result<Json<()>, AppError> {
    // Check if user is workspace owner
    let is_owner = state
        .workspace_repository
        .is_workspace_owner(current_user.user_id, workspace_id)
        .await?;
    
    if !is_owner {
        return Err(AppError::Authorization("Only workspace owner can update user roles".to_string()));
    }
    
    state
        .workspace_repository
        .update_user_role(workspace_id, user_id, request.role)
        .await?;
    
    Ok(Json(()))
}
