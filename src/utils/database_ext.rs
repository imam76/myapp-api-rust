use sqlx::{Error as SqlxError, PgPool};
use tracing::debug;
use uuid::Uuid;

/// Extension trait for PostgreSQL session management
#[async_trait::async_trait]
pub trait PostgresSessionExt {
  /// Set session variables for Row Level Security
  async fn set_session_settings(&self, user_id: &Uuid, workspace_id: Option<&Uuid>) -> Result<(), SqlxError>;

  /// Clear session variables
  async fn clear_session_settings(&self) -> Result<(), SqlxError>;
}

#[async_trait::async_trait]
impl PostgresSessionExt for PgPool {
  async fn set_session_settings(&self, user_id: &Uuid, workspace_id: Option<&Uuid>) -> Result<(), SqlxError> {
    debug!("Setting session variables: user_id={}, workspace_id={:?}", user_id, workspace_id);

    // Start a transaction to ensure all settings are applied atomically
    let mut tx = self.begin().await?;

    // Set current user ID for RLS
    sqlx::query("SELECT set_config('app.current_user_id', $1, false)")
      .bind(user_id.to_string())
      .execute(&mut *tx)
      .await?;

    if let Some(ws_id) = workspace_id {
      let role_opt: Option<String> = sqlx::query_scalar("SELECT role::text FROM workspace_users WHERE user_id = $1 AND workspace_id = $2")
        .bind(user_id)
        .bind(ws_id)
        .fetch_optional(&mut *tx)
        .await?;

      let (ws_id_str, role_str) = match role_opt {
        Some(role) => (Some(ws_id.to_string()), Some(role)),
        None => (None, None),
      };

      // Set both workspace and role in a single query
      sqlx::query("SELECT set_config('app.current_workspace_id', $1, false), set_config('app.current_user_role', $2, false)")
        .bind(ws_id_str)
        .bind(role_str)
        .execute(&mut *tx)
        .await?;
    } else {
      // Clear both workspace and role in a single query
      sqlx::query("SELECT set_config('app.current_workspace_id', NULL, false), set_config('app.current_user_role', NULL, false)")
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    debug!("Session variables set successfully");
    Ok(())
  }

  async fn clear_session_settings(&self) -> Result<(), SqlxError> {
    debug!("Clearing all session variables");
    // Clear all variables in a single query for efficiency
    sqlx::query(
      "SELECT 
        set_config('app.current_user_id', NULL, true), 
        set_config('app.current_workspace_id', NULL, true),
        set_config('app.current_user_role', NULL, true)",
    )
    .execute(self)
    .await?;
    debug!("Session variables cleared");
    Ok(())
  }
}
