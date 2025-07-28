use chrono::{DateTime, Utc};
use serde::Serialize;

/// Standard API Response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
  pub status: String,
  pub message: String,
  pub data: Option<T>,
  pub timestamp: DateTime<Utc>,
}

/// Paginated response structure
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
  pub list: Vec<T>,
  pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Serialize)]
pub struct PaginationMeta {
  pub page: u32,
  pub limit: u32,
  pub total: u64,
  pub total_pages: u32,
  pub has_next: bool,
  pub has_prev: bool,
}

/// Query parameters for paginated requests
#[derive(Debug, serde::Deserialize)]
pub struct GetContactsQuery {
  pub page: Option<u32>,
  pub limit: Option<u32>,
  pub search: Option<String>,
  pub contact_type: Option<String>,
  pub is_active: Option<bool>,
}

// Constants untuk consistency dengan handler
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;

impl Default for GetContactsQuery {
  fn default() -> Self {
    Self {
      page: Some(DEFAULT_PAGE),
      limit: Some(DEFAULT_LIMIT),
      search: None,
      contact_type: None,
      is_active: None,
    }
  }
}

/// Helper functions for creating responses
impl<T> ApiResponse<T> {
  pub fn success(data: T, message: &str) -> Self {
    Self {
      status: "success".to_string(),
      message: message.to_string(),
      data: Some(data),
      timestamp: Utc::now(),
    }
  }

  pub fn error(message: &str) -> Self {
    Self {
      status: "error".to_string(),
      message: message.to_string(),
      data: None,
      timestamp: Utc::now(),
    }
  }

  /// Create success response with default message
  pub fn success_default(data: T) -> Self {
    Self::success(data, "Operation completed successfully")
  }

  /// Create error response for validation failures
  pub fn validation_error(message: &str) -> Self {
    Self {
      status: "validation_error".to_string(),
      message: message.to_string(),
      data: None,
      timestamp: Utc::now(),
    }
  }
}

impl PaginationMeta {
  pub fn new(page: u32, limit: u32, total: u64) -> Self {
    let total_pages = (total as f64 / limit as f64).ceil() as u32;
    let has_next = page < total_pages;
    let has_prev = page > 1;

    Self {
      page,
      limit,
      total,
      total_pages,
      has_next,
      has_prev,
    }
  }
}
