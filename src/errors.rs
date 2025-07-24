use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use tracing::error;
use uuid::Uuid;

/// Main application error type that represents all possible errors in the application
#[derive(Debug, Clone)]
pub enum AppError {
  /// Authentication errors
  Authentication(AuthError),
  /// Authorization errors  
  Authorization(String),
  /// Validation errors
  Validation(ValidationError),
  /// Database related errors
  Database(DatabaseError),
  /// Resource not found errors
  NotFound(NotFoundError),
  /// Request format/parsing errors
  BadRequest(String),
  /// Cookie related errors
  Cookie(CookieError),
  /// Data serialization/deserialization errors
  Serialization(String),
  /// Internal server errors
  Internal(String),
  NotAllowed(String),
  /// Unhandled/unexpected errors
  Unhandled(String),
}

/// Authentication specific errors
#[derive(Debug, Clone)]
pub enum AuthError {
  /// Login credentials are invalid
  InvalidCredentials,
  /// Token is missing from request
  MissingToken,
  /// Token is invalid or malformed
  InvalidToken,
  /// Token has expired
  ExpiredToken,
}

/// Database specific errors
#[derive(Debug, Clone)]
pub enum DatabaseError {
  /// Connection to database failed
  ConnectionFailed(String),
  /// Query execution failed
  QueryFailed(String),
  /// Transaction failed
  TransactionFailed(String),
  /// Migration failed
  MigrationFailed(String),
}

/// Cookie related errors
#[derive(Debug, Clone)]
pub enum CookieError {
  /// Cookie format is invalid
  InvalidFormat,
  /// Cookie is missing
  Missing,
  /// Cookie has expired
  Expired,
}

/// Validation error with field information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
  pub field: String,
  pub message: String,
  pub code: Option<String>,
}

/// Not found error with resource information
#[derive(Debug, Clone)]
pub struct NotFoundError {
  pub resource: String,
  pub id: Option<Uuid>,
}

/// Standardized error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
  pub error: String,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub code: Option<String>,
  pub timestamp: String,
}

/// Convenient Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

// Display implementations
impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      AppError::Authentication(err) => write!(f, "Authentication error: {}", err),
      AppError::Authorization(msg) => write!(f, "Authorization error: {}", msg),
      AppError::Validation(err) => {
        write!(f, "Validation error: {} - {}", err.field, err.message)
      }
      AppError::Database(err) => write!(f, "Database error: {}", err),
      AppError::NotFound(err) => write!(f, "Not found: {}", err),
      AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
      AppError::Cookie(err) => write!(f, "Cookie error: {}", err),
      AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
      AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
      AppError::NotAllowed(msg) => write!(f, "Not allowed: {}", msg),
      AppError::Unhandled(msg) => write!(f, "Unhandled error: {}", msg),
    }
  }
}

impl fmt::Display for AuthError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      AuthError::InvalidCredentials => write!(f, "Invalid email or password"),
      AuthError::MissingToken => write!(f, "Authentication token is missing"),
      AuthError::InvalidToken => write!(f, "Authentication token is invalid"),
      AuthError::ExpiredToken => write!(f, "Authentication token has expired"),
    }
  }
}

impl fmt::Display for DatabaseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DatabaseError::ConnectionFailed(msg) => {
        write!(f, "Database connection failed: {}", msg)
      }
      DatabaseError::QueryFailed(msg) => write!(f, "Database query failed: {}", msg),
      DatabaseError::TransactionFailed(msg) => {
        write!(f, "Database transaction failed: {}", msg)
      }
      DatabaseError::MigrationFailed(msg) => write!(f, "Database migration failed: {}", msg),
    }
  }
}

impl fmt::Display for CookieError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CookieError::InvalidFormat => write!(f, "Cookie format is invalid"),
      CookieError::Missing => write!(f, "Required cookie is missing"),
      CookieError::Expired => write!(f, "Cookie has expired"),
    }
  }
}

impl fmt::Display for NotFoundError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.id {
      Some(id) => write!(f, "{} with id {} not found", self.resource, id),
      None => write!(f, "{} not found", self.resource),
    }
  }
}

// Standard Error trait implementations
impl std::error::Error for AppError {}
impl std::error::Error for AuthError {}
impl std::error::Error for DatabaseError {}
impl std::error::Error for CookieError {}

// Conversion implementations for easier error creation
impl From<AuthError> for AppError {
  fn from(err: AuthError) -> Self {
    AppError::Authentication(err)
  }
}

impl From<DatabaseError> for AppError {
  fn from(err: DatabaseError) -> Self {
    AppError::Database(err)
  }
}

impl From<CookieError> for AppError {
  fn from(err: CookieError) -> Self {
    AppError::Cookie(err)
  }
}

impl From<ValidationError> for AppError {
  fn from(err: ValidationError) -> Self {
    AppError::Validation(err)
  }
}

impl From<NotFoundError> for AppError {
  fn from(err: NotFoundError) -> Self {
    AppError::NotFound(err)
  }
}

// Helper constructors
impl AppError {
  /// Create a validation error
  pub fn validation(field: &str, message: &str) -> Self {
    AppError::Validation(ValidationError {
      field: field.to_string(),
      message: message.to_string(),
      code: None,
    })
  }

  /// Create a validation error with code
  pub fn validation_with_code(field: &str, message: &str, code: &str) -> Self {
    AppError::Validation(ValidationError {
      field: field.to_string(),
      message: message.to_string(),
      code: Some(code.to_string()),
    })
  }

  /// Create a not found error
  pub fn not_found(resource: &str) -> Self {
    AppError::NotFound(NotFoundError {
      resource: resource.to_string(),
      id: None,
    })
  }

  /// Create a not found error with ID
  pub fn not_found_with_id(resource: &str, id: Uuid) -> Self {
    AppError::NotFound(NotFoundError {
      resource: resource.to_string(),
      id: Some(id),
    })
  }

  /// Create a database connection error
  pub fn db_connection(message: &str) -> Self {
    AppError::Database(DatabaseError::ConnectionFailed(message.to_string()))
  }

  /// Create a database query error
  pub fn db_query(message: &str) -> Self {
    AppError::Database(DatabaseError::QueryFailed(message.to_string()))
  }

  /// Create an invalid credentials error
  pub fn invalid_credentials() -> Self {
    AppError::Authentication(AuthError::InvalidCredentials)
  }

  /// Create a missing token error
  pub fn missing_token() -> Self {
    AppError::Authentication(AuthError::MissingToken)
  }

  /// Create an invalid token error
  pub fn invalid_token() -> Self {
    AppError::Authentication(AuthError::InvalidToken)
  }

  pub fn not_allowed(message: &str) -> Self {
    AppError::NotAllowed(message.to_string())
  }
}

// Axum IntoResponse implementation
impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    // Log the error with appropriate level
    match &self {
      AppError::Internal(_) | AppError::Unhandled(_) | AppError::Database(_) => {
        error!("Application Error: {:?}", self);
      }
      _ => {
        tracing::warn!("Application Error: {:?}", self);
      }
    }

    let (status_code, error_type, message, details, code) = match &self {
      AppError::Authentication(auth_err) => match auth_err {
        AuthError::InvalidCredentials => (
          StatusCode::UNAUTHORIZED,
          "AUTHENTICATION_FAILED",
          "Invalid email or password".to_string(),
          None,
          Some("AUTH_001".to_string()),
        ),
        AuthError::MissingToken => (
          StatusCode::UNAUTHORIZED,
          "TOKEN_MISSING",
          "Authentication token is required".to_string(),
          None,
          Some("AUTH_002".to_string()),
        ),
        AuthError::InvalidToken => (
          StatusCode::UNAUTHORIZED,
          "TOKEN_INVALID",
          "Authentication token is invalid".to_string(),
          None,
          Some("AUTH_003".to_string()),
        ),
        AuthError::ExpiredToken => (
          StatusCode::UNAUTHORIZED,
          "TOKEN_EXPIRED",
          "Authentication token has expired".to_string(),
          None,
          Some("AUTH_004".to_string()),
        ),
      },
      AppError::Authorization(msg) => (
        StatusCode::FORBIDDEN,
        "AUTHORIZATION_FAILED",
        "Insufficient permissions".to_string(),
        Some(json!({ "details": msg })),
        Some("AUTHZ_001".to_string()),
      ),
      AppError::Validation(validation_err) => (
        StatusCode::BAD_REQUEST,
        "VALIDATION_FAILED",
        "Request validation failed".to_string(),
        Some(json!({
            "field": validation_err.field,
            "message": validation_err.message,
            "code": validation_err.code
        })),
        Some("VAL_001".to_string()),
      ),
      AppError::Database(db_err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "DATABASE_ERROR",
        "A database error occurred".to_string(),
        Some(json!({ "type": format!("{:?}", db_err) })),
        Some("DB_001".to_string()),
      ),
      AppError::NotFound(not_found_err) => (
        StatusCode::NOT_FOUND,
        "RESOURCE_NOT_FOUND",
        format!("{}", not_found_err),
        not_found_err.id.map(|id| json!({ "resource": not_found_err.resource, "id": id })),
        Some("NF_001".to_string()),
      ),
      AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone(), None, Some("BR_001".to_string())),
      AppError::Cookie(cookie_err) => (
        StatusCode::BAD_REQUEST,
        "COOKIE_ERROR",
        format!("{}", cookie_err),
        None,
        Some("CK_001".to_string()),
      ),
      AppError::Serialization(msg) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "SERIALIZATION_ERROR",
        "Data serialization failed".to_string(),
        Some(json!({ "details": msg })),
        Some("SER_001".to_string()),
      ),
      AppError::Internal(msg) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "An internal server error occurred".to_string(),
        Some(json!({ "details": msg })),
        Some("INT_001".to_string()),
      ),
      AppError::NotAllowed(msg) => (
        StatusCode::METHOD_NOT_ALLOWED,
        "METHOD_NOT_ALLOWED",
        msg.clone(),
        None,
        Some("NOT_ALLOWED_001".to_string()),
      ),
      AppError::Unhandled(msg) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "UNHANDLED_ERROR",
        "An unexpected error occurred".to_string(),
        Some(json!({ "details": msg })),
        Some("UNH_001".to_string()),
      ),
    };

    let error_response = ErrorResponse {
      error: error_type.to_string(),
      message: message.to_string(),
      details,
      code,
      timestamp: chrono::Utc::now().to_rfc3339(),
    };

    (status_code, Json(error_response)).into_response()
  }
}

// Helper macros for easier error creation
#[macro_export]
macro_rules! validation_error {
  ($field:expr, $message:expr) => {
    $crate::errors::AppError::validation($field, $message)
  };
  ($field:expr, $message:expr, $code:expr) => {
    $crate::errors::AppError::validation_with_code($field, $message, $code)
  };
}

#[macro_export]
macro_rules! not_found {
  ($resource:expr) => {
    $crate::errors::AppError::not_found($resource)
  };
  ($resource:expr, $id:expr) => {
    $crate::errors::AppError::not_found_with_id($resource, $id)
  };
}

#[macro_export]
macro_rules! bad_request {
  ($message:expr) => {
    $crate::errors::AppError::BadRequest($message.to_string())
  };
}

#[macro_export]
macro_rules! internal_error {
  ($message:expr) => {
    $crate::errors::AppError::Internal($message.to_string())
  };
}
