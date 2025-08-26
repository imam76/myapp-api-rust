use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::errors::{AppError, AuthError};

/// Wrapper for User ID to distinguish from Workspace ID in request extensions
#[derive(Debug, Clone, Copy)]
pub struct UserId(pub Uuid);

/// Wrapper for Workspace ID to distinguish from User ID in request extensions  
#[derive(Debug, Clone, Copy)]
pub struct WorkspaceId(pub Uuid);

/// Extractor for getting the current authenticated user's ID from the request.
///
/// This extractor retrieves the user ID that was added to the request extensions
/// by the JWT middleware. It can be used in protected route handlers to access
/// the authenticated user's information.
///
/// # Example
///
/// ```rust
/// pub async fn protected_handler(
///     current_user: CurrentUser,
/// ) -> Result<Json<String>, AppError> {
///     let user_id = current_user.user_id;
///     Ok(Json(format!("Hello user {}", user_id)))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CurrentUser {
  pub user_id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    // Extract the user ID from the request extensions using the typed wrapper
    let user_id = parts
      .extensions
      .get::<UserId>()
      .map(|uid| uid.0)
      .ok_or_else(|| AppError::Authentication(AuthError::MissingToken))?;

    Ok(CurrentUser { user_id })
  }
}
