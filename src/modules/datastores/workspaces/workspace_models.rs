use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workspace {
  pub id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub owner_id: Uuid,
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkspaceUser {
  pub workspace_id: Uuid,
  pub user_id: Uuid,
  pub role: WorkspaceRole,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "workspace_role", rename_all = "lowercase")]
pub enum WorkspaceRole {
  Admin,
  Member,
  Viewer,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
  pub name: String,
  pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
  pub name: Option<String>,
  pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddUserToWorkspaceRequest {
  pub user_id: Uuid,
  pub role: WorkspaceRole,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
  pub role: WorkspaceRole,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceWithRole {
  #[serde(flatten)]
  pub workspace: Workspace,
  pub user_role: WorkspaceRole,
  pub owner_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceUserInfo {
  pub user_id: Uuid,
  pub role: WorkspaceRole,
  pub created_at: DateTime<Utc>,
}
