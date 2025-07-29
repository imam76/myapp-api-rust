use super::workspace_models::{
  CreateWorkspaceRequest, UpdateWorkspaceRequest, Workspace, WorkspaceRole, WorkspaceUser, WorkspaceUserInfo, WorkspaceWithRole,
};
use crate::errors::AppError;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
  // Workspace CRUD operations
  async fn create_workspace(&self, request: &CreateWorkspaceRequest, owner_id: Uuid) -> Result<Workspace, AppError>;
  async fn create_and_assign_owner(&self, payload: CreateWorkspaceRequest, owner_id: Uuid) -> Result<Workspace, AppError>;
  async fn get_workspace_by_id(&self, workspace_id: Uuid) -> Result<Option<Workspace>, AppError>;
  async fn update_workspace(&self, workspace_id: Uuid, request: &UpdateWorkspaceRequest) -> Result<Workspace, AppError>;
  async fn delete_workspace(&self, workspace_id: Uuid) -> Result<(), AppError>;

  // User workspace access
  async fn get_user_workspaces(&self, user_id: Uuid) -> Result<Vec<WorkspaceWithRole>, AppError>;
  async fn get_workspace_users(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceUserInfo>, AppError>;

  // User management in workspace
  async fn add_user_to_workspace(&self, workspace_id: Uuid, user_id: Uuid, role: WorkspaceRole) -> Result<WorkspaceUser, AppError>;
  async fn remove_user_from_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), AppError>;
  async fn update_user_role(&self, workspace_id: Uuid, user_id: Uuid, role: WorkspaceRole) -> Result<WorkspaceUser, AppError>;

  // Permission checks
  async fn check_user_workspace_access(&self, user_id: Uuid, workspace_id: Uuid) -> Result<Option<WorkspaceRole>, AppError>;
  async fn is_workspace_owner(&self, user_id: Uuid, workspace_id: Uuid) -> Result<bool, AppError>;
}

pub struct PostgresWorkspaceRepository {
  pool: PgPool,
}

impl PostgresWorkspaceRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl WorkspaceRepository for PostgresWorkspaceRepository {
  async fn create_and_assign_owner(&self, payload: CreateWorkspaceRequest, owner_id: Uuid) -> Result<Workspace, AppError> {
    let mut tx = self.pool.begin().await?;

    // Step 1: Create the workspace
    let workspace = sqlx::query_as!(
      Workspace,
      r#"
        INSERT INTO workspaces (name, description, owner_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
      payload.name,
      payload.description,
      owner_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // Step 2: Add the owner to workspace_users table with 'Admin' role
    sqlx::query!(
      r#"
        INSERT INTO workspace_users (workspace_id, user_id, role)
        VALUES ($1, $2, $3)
        "#,
      workspace.id,
      owner_id,
      WorkspaceRole::Admin as WorkspaceRole
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(workspace)
  }

  async fn create_workspace(&self, request: &CreateWorkspaceRequest, owner_id: Uuid) -> Result<Workspace, AppError> {
    let workspace_id = Uuid::new_v4();

    let mut tx = self.pool.begin().await?;

    // Create workspace
    let workspace = sqlx::query_as!(
      Workspace,
      r#"
            INSERT INTO workspaces (id, name, description, owner_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, owner_id, created_at, updated_at
            "#,
      workspace_id,
      request.name,
      request.description,
      owner_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // Add owner as admin
    sqlx::query!(
      r#"
            INSERT INTO workspace_users (workspace_id, user_id, role)
            VALUES ($1, $2, $3)
            "#,
      workspace_id,
      owner_id,
      WorkspaceRole::Admin as WorkspaceRole
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(workspace)
  }

  async fn get_workspace_by_id(&self, workspace_id: Uuid) -> Result<Option<Workspace>, AppError> {
    let workspace = sqlx::query_as!(
      Workspace,
      r#"
            SELECT id, name, description, owner_id, created_at, updated_at
            FROM workspaces
            WHERE id = $1
            "#,
      workspace_id
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(workspace)
  }

  async fn update_workspace(&self, workspace_id: Uuid, request: &UpdateWorkspaceRequest) -> Result<Workspace, AppError> {
    let workspace = sqlx::query_as!(
      Workspace,
      r#"
            UPDATE workspaces
            SET 
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, name, description, owner_id, created_at, updated_at
            "#,
      workspace_id,
      request.name,
      request.description
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(workspace)
  }

  async fn delete_workspace(&self, workspace_id: Uuid) -> Result<(), AppError> {
    let mut tx = self.pool.begin().await?;

    // Remove all users from workspace
    sqlx::query!("DELETE FROM workspace_users WHERE workspace_id = $1", workspace_id)
      .execute(&mut *tx)
      .await?;

    // Delete workspace
    sqlx::query!("DELETE FROM workspaces WHERE id = $1", workspace_id)
      .execute(&mut *tx)
      .await?;

    tx.commit().await?;

    Ok(())
  }

  async fn get_user_workspaces(&self, user_id: Uuid) -> Result<Vec<WorkspaceWithRole>, AppError> {
    let workspaces = sqlx::query!(
      r#"
            SELECT w.id, w.name, w.description, w.owner_id, w.created_at, w.updated_at,
                   wu.role as "role!: WorkspaceRole"
            FROM workspaces w
            JOIN workspace_users wu ON w.id = wu.workspace_id
            WHERE wu.user_id = $1
            ORDER BY w.name
            "#,
      user_id
    )
    .fetch_all(&self.pool)
    .await?
    .into_iter()
    .map(|row| WorkspaceWithRole {
      workspace: Workspace {
        id: row.id,
        name: row.name,
        description: row.description,
        owner_id: row.owner_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
      },
      user_role: row.role,
    })
    .collect();

    Ok(workspaces)
  }

  async fn get_workspace_users(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceUserInfo>, AppError> {
    let users = sqlx::query_as!(
      WorkspaceUserInfo,
      r#"
            SELECT user_id, role as "role!: WorkspaceRole", created_at
            FROM workspace_users
            WHERE workspace_id = $1
            ORDER BY created_at
            "#,
      workspace_id
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(users)
  }

  async fn add_user_to_workspace(&self, workspace_id: Uuid, user_id: Uuid, role: WorkspaceRole) -> Result<WorkspaceUser, AppError> {
    let workspace_user = sqlx::query_as!(
      WorkspaceUser,
      r#"
            INSERT INTO workspace_users (workspace_id, user_id, role)
            VALUES ($1, $2, $3)
            RETURNING workspace_id, user_id, role as "role!: WorkspaceRole", created_at
            "#,
      workspace_id,
      user_id,
      role as WorkspaceRole
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(workspace_user)
  }

  async fn remove_user_from_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
      "DELETE FROM workspace_users WHERE workspace_id = $1 AND user_id = $2",
      workspace_id,
      user_id
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn update_user_role(&self, workspace_id: Uuid, user_id: Uuid, role: WorkspaceRole) -> Result<WorkspaceUser, AppError> {
    let workspace_user = sqlx::query_as!(
      WorkspaceUser,
      r#"
            UPDATE workspace_users
            SET role = $3
            WHERE workspace_id = $1 AND user_id = $2
            RETURNING workspace_id, user_id, role as "role!: WorkspaceRole", created_at
            "#,
      workspace_id,
      user_id,
      role as WorkspaceRole
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(workspace_user)
  }

  async fn check_user_workspace_access(&self, user_id: Uuid, workspace_id: Uuid) -> Result<Option<WorkspaceRole>, AppError> {
    let role = sqlx::query!(
      r#"
            SELECT role as "role!: WorkspaceRole"
            FROM workspace_users
            WHERE user_id = $1 AND workspace_id = $2
            "#,
      user_id,
      workspace_id
    )
    .fetch_optional(&self.pool)
    .await?
    .map(|row| row.role);

    Ok(role)
  }

  async fn is_workspace_owner(&self, user_id: Uuid, workspace_id: Uuid) -> Result<bool, AppError> {
    let count = sqlx::query!(
      "SELECT COUNT(*) as count FROM workspaces WHERE id = $1 AND owner_id = $2",
      workspace_id,
      user_id
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(count.count.unwrap_or(0) > 0)
  }
}
