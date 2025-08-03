use std::sync::Arc;

use crate::{
  AppResult,
  errors::AppError,
  modules::datastores::workspaces::{WorkspaceRepository, WorkspaceRole},
};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

pub struct WorkspaceContext(pub Option<Uuid>);

#[async_trait]
impl<S> FromRequestParts<S> for WorkspaceContext
where
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    if let Some(workspace_header) = parts.headers.get("X-Workspace-ID") {
      let workspace_str = workspace_header
        .to_str()
        .map_err(|_| AppError::BadRequest("Invalid workspace header".to_string()))?;

      let workspace_id = Uuid::parse_str(workspace_str).map_err(|_| AppError::BadRequest("Invalid workspace ID format".to_string()))?;

      Ok(WorkspaceContext(Some(workspace_id)))
    } else {
      Ok(WorkspaceContext(None))
    }
  }
}

/// Helper function to check if user has required role or higher in workspace
pub async fn check_workspace_permission(
  workspace_repository: &Arc<dyn WorkspaceRepository + Send + Sync>,
  workspace_id: Uuid,
  user_id: Uuid,
  required_role: WorkspaceRole,
) -> AppResult<bool> {
  let user_role = workspace_repository.check_user_workspace_access(user_id, workspace_id).await?;

  match user_role {
    Some(role) => {
      let has_permission = match required_role {
        WorkspaceRole::Viewer => matches!(role, WorkspaceRole::Viewer | WorkspaceRole::Member | WorkspaceRole::Admin),
        WorkspaceRole::Member => matches!(role, WorkspaceRole::Member | WorkspaceRole::Admin),
        WorkspaceRole::Admin => matches!(role, WorkspaceRole::Admin),
      };
      Ok(has_permission)
    }
    None => Ok(false),
  }
}
