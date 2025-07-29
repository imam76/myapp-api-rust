use async_trait::async_trait;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::modules::auth::user_model::User;

use super::user_dto::RegisterUserDto;

#[async_trait]
pub trait AuthRepository: Send + Sync {
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
  async fn find_by_id(&self, user_id: uuid::Uuid) -> Result<Option<User>, AppError>;
  async fn create_user(&self, user_data: &RegisterUserDto, hashed_password: &str) -> Result<User, AppError>;
}

pub struct AuthRepositoryImpl {
  pool: PgPool,
}

impl AuthRepositoryImpl {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl AuthRepository for AuthRepositoryImpl {
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as!(
      User,
      "SELECT id, username, email, password_hash, is_active, created_at, updated_at FROM users WHERE email = $1",
      email
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(user)
  }

  async fn find_by_id(&self, user_id: uuid::Uuid) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as!(
      User,
      "SELECT id, username, email, password_hash, is_active, created_at, updated_at FROM users WHERE id = $1",
      user_id
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(user)
  }

  async fn create_user(&self, user_data: &RegisterUserDto, hashed_password: &str) -> Result<User, AppError> {
    let user = sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, password_hash, is_active, created_at, updated_at",
            user_data.username,
            user_data.email,
            hashed_password
        )
        .fetch_one(&self.pool)
        .await?;

    Ok(user)
  }
}
