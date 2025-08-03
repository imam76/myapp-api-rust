use argon2::{
  Argon2,
  password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
  errors::{AppError, AuthError},
  modules::{
    auth::{
      user_dto::{LoginUserDto, RegisterUserDto},
      user_model::User,
    },
    datastores::workspaces::{Workspace, workspace_models::CreateWorkspaceRequest},
  },
  state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub sub: Uuid,
  pub exp: usize,
  pub iat: usize,
}

pub async fn register_user(state: Arc<AppState>, user_data: RegisterUserDto) -> Result<(User, Workspace), AppError> {
  user_data.validate()?;

  if state.auth_repository.find_by_email(&user_data.email).await?.is_some() {
    return Err(AppError::Conflict("User with this email already exists".to_string()));
  }

  let salt = SaltString::generate(&mut OsRng);
  let argon2 = Argon2::default();
  let password_hash = argon2.hash_password(user_data.password.as_bytes(), &salt)?.to_string();

  let user = state.auth_repository.create_user(&user_data, &password_hash).await?;

  // Automatically create a personal workspace for the new user
  let workspace_payload = CreateWorkspaceRequest {
    name: format!("{}'s Personal Workspace", user.username),
    description: Some("Default personal workspace.".to_string()),
  };

  let workspace = state.workspace_repository.create_and_assign_owner(workspace_payload, user.id).await?;

  Ok((user, workspace))
}

pub async fn login_user(state: Arc<AppState>, login_data: LoginUserDto) -> Result<(String, User), AppError> {
  login_data.validate()?;

  let user = state
    .auth_repository
    .find_by_email(&login_data.email)
    .await?
    .ok_or_else(|| AppError::Authentication(AuthError::InvalidCredentials))?;

  let is_password_valid = argon2::PasswordHash::new(&user.password_hash)?
    .verify_password(&[&Argon2::default()], login_data.password.as_bytes())
    .is_ok();

  let dbsize = state.auth_repository.get_db_size().await?;
  // Limit for trial users is set to 100MB
  let max_allowed = 100 * 1024 * 1024;

  if dbsize > max_allowed {
    return Err(AppError::database_size_exceeded(
      "Trial users are limited to 100MB records. Upgrade to add more data.",
    ));
  }

  if !is_password_valid {
    return Err(AppError::Authentication(AuthError::InvalidCredentials));
  }

  let now = chrono::Utc::now();
  let iat = now.timestamp() as usize;
  let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;

  let claims = Claims { sub: user.id, exp, iat };

  let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(state.jwt_secret.as_ref()))?;

  Ok((token, user))
}
