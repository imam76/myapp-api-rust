use crate::errors::AppError;
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
