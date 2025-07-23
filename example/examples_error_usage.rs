// This file contains examples of how to use the refactored error handling system
// You can remove this file after understanding the usage patterns

use crate::errors::{AppError, AppResult, AuthError, DatabaseError};
use axum::{Json, response::Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct User {
  id: Uuid,
  email: String,
  name: String,
}

#[derive(Deserialize)]
struct LoginRequest {
  email: String,
  password: String,
}

// Example: Authentication handler using the new error system
async fn login_handler(Json(payload): Json<LoginRequest>) -> AppResult<Json<User>> {
  // Validation example
  if payload.email.is_empty() {
    return Err(AppError::validation("email", "Email is required"));
  }

  if payload.password.len() < 8 {
    return Err(AppError::validation_with_code(
      "password",
      "Password must be at least 8 characters",
      "PWD_TOO_SHORT",
    ));
  }

  // Database lookup example (simulated)
  match find_user_by_email(&payload.email).await {
    Ok(Some(user)) => {
      // Verify password (simulated)
      if verify_password(&payload.password, &user.email) {
        Ok(Json(user))
      } else {
        Err(AppError::invalid_credentials())
      }
    }
    Ok(None) => Err(AppError::invalid_credentials()),
    Err(db_err) => Err(db_err),
  }
}

// Example: Get user handler with various error cases
async fn get_user_handler(user_id: Uuid) -> AppResult<Json<User>> {
  match find_user_by_id(user_id).await {
    Ok(Some(user)) => Ok(Json(user)),
    Ok(None) => Err(AppError::not_found_with_id("User", user_id)),
    Err(db_err) => Err(db_err),
  }
}

// Example: Update user with validation
async fn update_user_handler(user_id: Uuid, Json(payload): Json<UpdateUserRequest>) -> AppResult<Json<User>> {
  // Multiple validation checks
  let mut errors = Vec::new();

  if payload.email.as_ref().map_or(false, |email| email.is_empty()) {
    errors.push(AppError::validation("email", "Email cannot be empty"));
  }

  if payload.name.as_ref().map_or(false, |name| name.len() < 2) {
    errors.push(AppError::validation("name", "Name must be at least 2 characters"));
  }

  // For simplicity, we'll just return the first error
  // In a real app, you might want to collect all validation errors
  if let Some(error) = errors.into_iter().next() {
    return Err(error);
  }

  // Database update
  match update_user_in_db(user_id, payload).await {
    Ok(Some(user)) => Ok(Json(user)),
    Ok(None) => Err(AppError::not_found_with_id("User", user_id)),
    Err(db_err) => Err(db_err),
  }
}

// Example: Using macros for easier error creation
async fn delete_user_handler(user_id: Uuid) -> AppResult<Json<serde_json::Value>> {
  match delete_user_from_db(user_id).await {
    Ok(true) => Ok(Json(serde_json::json!({ "message": "User deleted successfully" }))),
    Ok(false) => Err(not_found!("User", user_id)),
    Err(_) => Err(internal_error!("Failed to delete user")),
  }
}

// Example: Converting external errors to AppError
async fn external_api_call() -> AppResult<String> {
  // Simulating an external HTTP call that might fail
  match fetch_external_data().await {
    Ok(data) => Ok(data),
    Err(e) => Err(AppError::Internal(format!("External API failed: {}", e))),
  }
}

// Example: Database error conversion
impl From<sqlx::Error> for AppError {
  fn from(err: sqlx::Error) -> Self {
    match err {
      sqlx::Error::RowNotFound => AppError::not_found("Resource"),
      sqlx::Error::Database(db_err) => {
        // Handle specific database constraints
        if let Some(code) = db_err.code() {
          match code.as_ref() {
            "23505" => AppError::BadRequest("Duplicate entry".to_string()),
            "23503" => AppError::BadRequest("Foreign key constraint violation".to_string()),
            _ => AppError::Database(DatabaseError::QueryFailed(db_err.to_string())),
          }
        } else {
          AppError::Database(DatabaseError::QueryFailed(db_err.to_string()))
        }
      }
      _ => AppError::Database(DatabaseError::QueryFailed(err.to_string())),
    }
  }
}

// Mock functions for examples (you would replace these with real implementations)
#[derive(Deserialize)]
struct UpdateUserRequest {
  email: Option<String>,
  name: Option<String>,
}

async fn find_user_by_email(_email: &str) -> Result<Option<User>, AppError> {
  // Mock implementation
  Ok(Some(User {
    id: Uuid::new_v4(),
    email: "user@example.com".to_string(),
    name: "John Doe".to_string(),
  }))
}

async fn find_user_by_id(_user_id: Uuid) -> Result<Option<User>, AppError> {
  // Mock implementation
  Ok(None) // Simulating user not found
}

async fn update_user_in_db(_user_id: Uuid, _payload: UpdateUserRequest) -> Result<Option<User>, AppError> {
  // Mock implementation
  Ok(None)
}

async fn delete_user_from_db(_user_id: Uuid) -> Result<bool, AppError> {
  // Mock implementation
  Ok(false)
}

async fn fetch_external_data() -> Result<String, Box<dyn std::error::Error>> {
  // Mock external API call
  Err("Network timeout".into())
}

fn verify_password(_password: &str, _email: &str) -> bool {
  // Mock password verification
  true
}

// Example of how to use the error system in middleware
async fn auth_middleware() -> AppResult<()> {
  // Check for token in headers/cookies
  let token = extract_token().await?;

  // Validate token
  validate_token(&token)?;

  Ok(())
}

async fn extract_token() -> AppResult<String> {
  // Mock token extraction from request
  // In reality, this would extract from headers or cookies
  Err(AppError::missing_token())
}

fn validate_token(_token: &str) -> AppResult<()> {
  // Mock token validation
  // In reality, this would verify JWT signature, expiration, etc.
  Err(AppError::invalid_token())
}
