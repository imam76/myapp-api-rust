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

    // Set current user ID for RLS
    sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
      .bind(user_id.to_string())
      .execute(self)
      .await?;

    // Set workspace ID if provided
    if let Some(ws_id) = workspace_id {
      sqlx::query("SELECT set_config('app.current_workspace_id', $1, true)")
        .bind(ws_id.to_string())
        .execute(self)
        .await?;
    } else {
      // Clear workspace setting if not provided
      sqlx::query("SELECT set_config('app.current_workspace_id', NULL, true)")
        .execute(self)
        .await?;
    }

    debug!("Session variables set successfully");
    Ok(())
  }

  async fn clear_session_settings(&self) -> Result<(), SqlxError> {
    debug!("Clearing session variables");

    sqlx::query("SELECT set_config('app.current_user_id', NULL, true)").execute(self).await?;

    sqlx::query("SELECT set_config('app.current_workspace_id', NULL, true)")
      .execute(self)
      .await?;

    debug!("Session variables cleared");
    Ok(())
  }
}
