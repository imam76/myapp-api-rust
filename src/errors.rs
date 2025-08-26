//! This module defines the application's error handling infrastructure.
//!
//! It includes the main `AppError` enum, which consolidates all possible errors
//! that can occur within the application. It also provides `From` implementations
//! to seamlessly convert errors from external crates (like `sqlx`, `validator`, `axum`)
//! into the `AppError` type. This allows for the convenient use of the `?` operator.

use argon2::password_hash;
use axum::{
  Json,
  extract::rejection::{JsonRejection, QueryRejection},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use tracing::error;
use uuid::Uuid;
use validator::ValidationErrors;

/// The main application error type.
///
/// This enum consolidates all possible error types that can occur within the application.
/// It is designed to be the single source of truth for error handling, providing a consistent
/// way to represent and manage errors, from database issues to validation failures.
#[derive(Debug, Clone)]
pub enum AppError {
  /// For authentication-related failures.
  Authentication(AuthError),
  /// For authorization-related failures (e.g., insufficient permissions).
  Authorization(String),
  /// For failures in data validation, typically from user input.
  Validation(serde_json::Value),
  /// For errors originating from the database.
  Database(DatabaseError),
  /// For cases where a requested resource could not be found.
  NotFound(NotFoundError),
  /// For when a resource already exists.
  Conflict(String),
  /// For malformed requests that cannot be parsed or processed.
  BadRequest(String),
  /// For errors related to handling HTTP cookies.
  Cookie(CookieError),
  /// For errors during data serialization or deserialization.
  Serialization(String),
  /// For any other internal server errors that are not covered by other variants.
  Internal(String),
  /// For requests using an unsupported HTTP method.
  NotAllowed(String),
  /// A catch-all for unhandled or unexpected errors.
  Unhandled(String),
}

/// Represents authentication-specific errors.
#[derive(Debug, Clone)]
pub enum AuthError {
  /// The provided login credentials are invalid.
  InvalidCredentials,
  /// An authentication token is missing from the request.
  MissingToken,
  /// The provided token is invalid, malformed, or cannot be parsed.
  InvalidToken,
  InvalidWorkspace,
  /// The provided token has expired.
  ExpiredToken,
}

/// Represents database-specific errors.
#[derive(Debug, Clone)]
pub enum DatabaseError {
  /// Failed to establish a connection to the database.
  ConnectionFailed(String),
  /// A database query failed to execute.
  QueryFailed(String),
  /// A database transaction failed.
  TransactionFailed(String),
  /// A database migration failed.
  MigrationFailed(String),
  /// Trial user has exceeded their database size limit.
  SizeExceeded(String),
  /// Database schema doesn't match expected structure.
  SchemaMismatch(String),
  /// Column not found in database table.
  ColumnNotFound(String),
}

/// Represents errors related to HTTP cookies.
#[derive(Debug, Clone)]
pub enum CookieError {
  /// The cookie format is invalid.
  InvalidFormat,
  /// A required cookie is missing from the request.
  Missing,
  /// The cookie has expired.
  Expired,
}

/// Represents a single validation error for a specific field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
  /// The name of the field that failed validation.
  pub field: String,
  /// The error message describing the validation failure.
  pub message: String,
  /// An optional error code associated with the validation rule.
  pub code: Option<String>,
}

/// Represents an error for a resource that could not be found.
#[derive(Debug, Clone)]
pub struct NotFoundError {
  /// The type of the resource that was not found (e.g., "Contact", "User").
  pub resource: String,
  /// The unique identifier of the resource that was not found, if applicable.
  pub id: Option<Uuid>,
}

/// A standardized structure for JSON error responses.
///
/// This struct defines the shape of the JSON body that is sent to the client
/// in the event of an error. It provides a consistent and predictable format.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
  /// A high-level classification of the error (e.g., "AuthenticationFailure", "ValidationError").
  pub error: String,
  /// A human-readable message describing the error.
  pub message: String,
  /// Optional, machine-readable details about the error, such as validation messages.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
  /// An optional, application-specific error code.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub code: Option<String>,
  /// The timestamp when the error occurred, in ISO 8601 format.
  pub timestamp: String,
}

/// Converts an `AppError` into an HTTP `Response`.
///
/// This implementation is the cornerstone of the application's error handling. It takes any
/// `AppError` variant, logs it appropriately, and transforms it into a user-friendly
/// HTTP response with the correct status code and a JSON body defined by `ErrorResponse`.
impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, error_type, message, details, code) = match self {
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
        AuthError::InvalidWorkspace => (
          StatusCode::UNAUTHORIZED,
          "WORKSPACE_INVALID",
          "Invalid workspace access or workspace not found".to_string(),
          None,
          Some("AUTH_004".to_string()),
        ),
        AuthError::ExpiredToken => (
          StatusCode::UNAUTHORIZED,
          "TOKEN_EXPIRED",
          "Authentication token has expired".to_string(),
          None,
          Some("AUTH_005".to_string()),
        ),
      },
      AppError::Authorization(msg) => (
        StatusCode::FORBIDDEN,
        "AUTHORIZATION_FAILED",
        "Insufficient permissions".to_string(),
        Some(json!({ "details": msg })),
        Some("AUTHZ_001".to_string()),
      ),
      AppError::Validation(validation_err_json) => (
        StatusCode::UNPROCESSABLE_ENTITY,
        "VALIDATION_FAILED",
        "Request validation failed".to_string(),
        Some(validation_err_json),
        Some("VAL_001".to_string()),
      ),
      AppError::Database(db_err) => {
        match &db_err {
          DatabaseError::SchemaMismatch(msg) => {
            error!("Database schema mismatch: {}", msg);
            (
              StatusCode::INTERNAL_SERVER_ERROR,
              "DATABASE_SCHEMA_ERROR",
              "Database configuration error. Please contact system administrator.".to_string(),
              Some(json!({ "technical_details": msg })),
              Some("DB_SCHEMA_001".to_string()),
            )
          }
          DatabaseError::ColumnNotFound(msg) => {
            error!("Column not found: {}", msg);
            (
              StatusCode::INTERNAL_SERVER_ERROR,
              "DATABASE_COLUMN_ERROR",
              "Database structure error. Please contact system administrator.".to_string(),
              Some(json!({ "technical_details": msg })),
              Some("DB_COL_001".to_string()),
            )
          }
          _ => {
            error!("Database error: {:?}", db_err);
            (
              StatusCode::INTERNAL_SERVER_ERROR,
              "DATABASE_ERROR",
              "A database error occurred".to_string(),
              None, // Avoid leaking detailed db errors
              Some("DB_001".to_string()),
            )
          }
        }
      }
      AppError::NotFound(not_found_err) => (
        StatusCode::NOT_FOUND,
        "RESOURCE_NOT_FOUND",
        not_found_err.to_string(),
        not_found_err.id.map(|id| json!({ "resource": not_found_err.resource.clone(), "id": id })),
        Some("NF_001".to_string()),
      ),
      AppError::Conflict(msg) => (StatusCode::CONFLICT, "RESOURCE_CONFLICT", msg, None, Some("CONFLICT_001".to_string())),
      AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg, None, Some("BR_001".to_string())),
      AppError::Cookie(cookie_err) => (
        StatusCode::BAD_REQUEST,
        "COOKIE_ERROR",
        cookie_err.to_string(),
        None,
        Some("CK_001".to_string()),
      ),
      AppError::Serialization(msg) => {
        error!("Serialization error: {}", msg);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "SERIALIZATION_ERROR",
          "Data serialization failed".to_string(),
          None,
          Some("SER_001".to_string()),
        )
      }
      AppError::Internal(msg) => {
        error!("Internal server error: {}", msg);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "INTERNAL_ERROR",
          "An internal server error occurred".to_string(),
          None,
          Some("INT_001".to_string()),
        )
      }
      AppError::NotAllowed(msg) => (
        StatusCode::METHOD_NOT_ALLOWED,
        "METHOD_NOT_ALLOWED",
        msg,
        None,
        Some("NOT_ALLOWED_001".to_string()),
      ),
      AppError::Unhandled(msg) => {
        error!("Unhandled error: {}", msg);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "UNHANDLED_ERROR",
          "An unexpected error occurred".to_string(),
          None,
          Some("UNH_001".to_string()),
        )
      }
    };

    let error_response = ErrorResponse {
      error: error_type.to_string(),
      message: message.to_string(),
      details,
      code: code.map(String::from),
      timestamp: chrono::Utc::now().to_rfc3339(),
    };

    (status, Json(error_response)).into_response()
  }
}

impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      AppError::Authentication(err) => write!(f, "Authentication error: {}", err),
      AppError::Authorization(msg) => write!(f, "Authorization error: {}", msg),
      AppError::Validation(err) => write!(f, "Validation error: {}", err),
      AppError::Database(err) => write!(f, "Database error: {}", err),
      AppError::NotFound(err) => write!(f, "Not found: {}", err),
      AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
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
      AuthError::InvalidWorkspace => write!(f, "Invalid workspace access or workspace not found"),
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
      DatabaseError::SizeExceeded(msg) => write!(f, "Database size limit exceeded: {}", msg),
      DatabaseError::SchemaMismatch(msg) => write!(f, "Database schema mismatch: {}", msg),
      DatabaseError::ColumnNotFound(msg) => write!(f, "Column not found: {}", msg),
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

/// Converts `sqlx::Error` into `AppError`.
///
/// This allows for the use of the `?` operator on `sqlx::Result`, automatically
/// converting database errors into the application's custom error type.
/// It also handles specific database errors, like unique constraint violations.
impl From<sqlx::Error> for AppError {
  fn from(err: sqlx::Error) -> Self {
    match &err {
      sqlx::Error::RowNotFound => AppError::NotFound(NotFoundError {
        resource: "resource".to_string(), // Can be made more specific in the calling code
        id: None,
      }),
      sqlx::Error::ColumnNotFound(col_name) => {
        AppError::Database(DatabaseError::ColumnNotFound(format!("Column '{}' not found in query result", col_name)))
      }
      sqlx::Error::Database(db_err) => {
        if let Some(code) = db_err.code() {
          if code == "23505" {
            // Unique violation
            return AppError::Validation(json!({
                "code": "duplicate_entry",
                "message": "An entry with this value already exists."
            }));
          }
        }

        // Check for schema-related errors
        if db_err.message().contains("column") && (db_err.message().contains("does not exist") || db_err.message().contains("not found")) {
          return AppError::Database(DatabaseError::SchemaMismatch(format!("Database schema error: {}", db_err.message())));
        }

        AppError::Database(DatabaseError::QueryFailed(err.to_string()))
      }
      _ => AppError::Database(DatabaseError::QueryFailed(err.to_string())),
    }
  }
}

/// Converts `validator::ValidationErrors` into `AppError::Validation`.
///
/// This implementation enables the use of the `?` operator on the result of `validate()`.
/// It transforms the detailed error structure from the `validator` crate into a
/// `serde_json::Value`, which can then be included in the HTTP response body.
impl From<ValidationErrors> for AppError {
  fn from(errors: ValidationErrors) -> Self {
    let details = serde_json::to_value(errors.field_errors()).unwrap_or_else(|_| json!({"error": "Failed to serialize validation errors"}));
    AppError::Validation(details)
  }
}

/// Converts `axum::extract::rejection::JsonRejection` into `AppError::BadRequest`.
///
/// This handles errors that occur during the JSON deserialization of a request body.
/// If Axum fails to parse the JSON, this converts the rejection into a clear `BadRequest` error.
impl From<JsonRejection> for AppError {
  fn from(rejection: JsonRejection) -> Self {
    AppError::BadRequest(rejection.to_string())
  }
}

/// Converts `axum::extract::rejection::QueryRejection` into `AppError::BadRequest`.
///
/// This handles errors that occur during query parameter deserialization,
/// including unknown fields, invalid values, and other parsing issues.
impl From<QueryRejection> for AppError {
  fn from(rejection: QueryRejection) -> Self {
    let error_msg = rejection.to_string();

    // Make the error message more user-friendly
    if error_msg.contains("unknown field") {
      // Extract field name from error message
      if let Some(field_start) = error_msg.find("`") {
        if let Some(field_end) = error_msg[field_start + 1..].find("`") {
          let field_name = &error_msg[field_start + 1..field_start + 1 + field_end];
          return AppError::BadRequest(format!("Unknown query parameter: '{}'", field_name));
        }
      }
      AppError::BadRequest("Invalid query parameter provided".to_string())
    } else if error_msg.contains("Failed to deserialize query string") {
      AppError::BadRequest("Invalid query parameters format".to_string())
    } else {
      AppError::BadRequest(format!("Query parameter error: {}", error_msg))
    }
  }
}

/// Converts `serde_json::Error` into `AppError::Serialization`.
///
/// This handles errors from the `serde_json` crate, which can occur during either
/// serialization or deserialization.
impl From<serde_json::Error> for AppError {
  fn from(err: serde_json::Error) -> Self {
    AppError::Serialization(err.to_string())
  }
}

/// Converts `uuid::Error` into `AppError::NotFound`.
///
/// This is useful for handlers that parse a `Uuid` from a path or query parameter.
/// If the string is not a valid UUID, it results in a `NotFound` error indicating the resource doesn't exist.
impl From<uuid::Error> for AppError {
  fn from(_err: uuid::Error) -> Self {
    AppError::NotFound(NotFoundError {
      resource: "Resource".to_string(),
      id: None,
    })
  }
}

impl From<jsonwebtoken::errors::Error> for AppError {
  fn from(err: jsonwebtoken::errors::Error) -> Self {
    match err.kind() {
      jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::Authentication(AuthError::ExpiredToken),
      _ => AppError::Authentication(AuthError::InvalidToken),
    }
  }
}

impl From<password_hash::Error> for AppError {
  fn from(err: password_hash::Error) -> Self {
    error!("Password hashing error: {:?}", err);
    AppError::Internal("Error processing password".to_string())
  }
}

impl AppError {
  /// Create a validation error with a code.
  pub fn validation_with_code(field: &str, message: &str, code: &str) -> Self {
    let validation_error = ValidationError {
      field: field.to_string(),
      message: message.to_string(),
      code: Some(code.to_string()),
    };
    AppError::Validation(json!({ field: [validation_error] }))
  }

  /// Create a database size exceeded error for trial users.
  pub fn database_size_exceeded(message: &str) -> Self {
    AppError::Database(DatabaseError::SizeExceeded(message.to_string()))
  }

  /// Create a database schema mismatch error.
  pub fn schema_mismatch(message: &str) -> Self {
    AppError::Database(DatabaseError::SchemaMismatch(message.to_string()))
  }

  /// Create a column not found error.
  pub fn column_not_found(column_name: &str) -> Self {
    AppError::Database(DatabaseError::ColumnNotFound(format!("Column '{}' not found", column_name)))
  }

  /// Enhanced error handling for SQLx errors with context
  pub fn from_sqlx_error(error: sqlx::Error, query_context: &str) -> Self {
    match error {
      sqlx::Error::ColumnNotFound(column) => {
        tracing::error!("Column not found: {} in query: {}", column, query_context);
        Self::column_not_found(&column)
      }
      sqlx::Error::Database(db_err) => {
        let message = db_err.message();

        // Check for schema-related errors
        if message.contains("column") && message.contains("does not exist") {
          tracing::error!("Schema mismatch in query: {}, error: {}", query_context, message);
          Self::schema_mismatch(message)
        } else if message.contains("relation") && message.contains("does not exist") {
          tracing::error!("Schema mismatch (table) in query: {}, error: {}", query_context, message);
          Self::schema_mismatch(message)
        } else {
          tracing::error!("Database error in query: {}, error: {}", query_context, message);
          Self::Database(DatabaseError::QueryFailed(message.to_string()))
        }
      }
      sqlx::Error::RowNotFound => Self::NotFound(NotFoundError {
        resource: "Resource".to_string(),
        id: None,
      }),
      _ => {
        let error_msg = error.to_string();
        tracing::error!("SQLx error in query: {}, error: {}", query_context, error_msg);
        Self::Database(DatabaseError::QueryFailed(error_msg))
      }
    }
  }
}
