use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Represents a contact record in the database.
/// This struct is derived from `sqlx::FromRow` to allow direct mapping from database query results.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Contact {
  pub id: Uuid,
  pub code: String,
  pub name: String,
  pub email: String,
  pub position: String,
  #[sqlx(rename = "type")]
  pub contact_type: String, // Maps to database column "type" to avoid Rust keyword conflict
  pub address: Option<String>,
  pub is_active: bool,

  // Metadata
  pub workspace_id: Option<Uuid>,
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Represents the payload for creating a new contact.
/// This struct uses `validator` to enforce declarative validation rules on the incoming data.
/// The `created_by` field is automatically set from the authenticated user.
/// The `workspace_id` is now extracted from request headers via WorkspaceContext, not from the body.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateContactRequest {
  #[validate(length(min = 1, message = "Code is required"))]
  pub code: String,
  #[validate(length(min = 1, message = "Name is required"))]
  pub name: String,
  #[validate(email(message = "Invalid email format"))]
  pub email: String,
  #[validate(length(min = 1, message = "Position is required"))]
  pub position: String,
  #[validate(length(min = 1, message = "Contact type is required"))]
  pub contact_type: String,
  pub address: Option<String>,
}

/// Represents the payload for updating an existing contact.
/// All fields are optional, allowing for partial updates.
/// The `updated_by` field is automatically set from the authenticated user.
/// The `workspace_id` cannot be changed via update - it's workspace-scoped.
#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
  pub code: Option<String>,
  pub name: Option<String>,
  pub email: Option<String>,
  pub position: Option<String>,
  pub contact_type: Option<String>,
  pub address: Option<String>,
  pub is_active: Option<bool>,
}

/// Represents the data structure for a contact response.
/// This struct defines the public-facing representation of a contact,
/// including ownership and audit information.
#[derive(Debug, Serialize)]
pub struct ContactResponse {
  pub id: Uuid,
  pub code: String,
  pub name: String,
  pub email: String,
  pub position: String,
  pub contact_type: String,
  pub address: Option<String>,
  pub is_active: bool,

  // Metadata
  pub workspace_id: Option<Uuid>,
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Converts a `Contact` model into a `ContactResponse`.
/// This `From` implementation facilitates the transformation of the internal
/// database model into the public API response structure.
impl From<Contact> for ContactResponse {
  fn from(contact: Contact) -> Self {
    Self {
      id: contact.id,
      code: contact.code,
      name: contact.name,
      email: contact.email,
      position: contact.position,
      contact_type: contact.contact_type,
      address: contact.address,
      is_active: contact.is_active,

      // Metadata
      workspace_id: contact.workspace_id,
      created_by: contact.created_by,
      updated_by: contact.updated_by,
      created_at: contact.created_at,
      updated_at: contact.updated_at,
    }
  }
}

/// Query parameters for paginated requests with advanced filtering
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetContactsQuery {
  // Pagination
  pub page: Option<u32>,
  pub limit: Option<u32>,
  
  // Basic filtering
  pub search: Option<String>,
  pub contact_type: Option<String>,
  pub is_active: Option<bool>,
  
  // Advanced filtering
  pub code: Option<String>,
  pub email: Option<String>,
  pub include_types: Option<String>, // comma-separated: "customer,supplier"
  pub exclude_types: Option<String>, // comma-separated: "employee"
  pub include_ids: Option<String>,   // comma-separated UUIDs
  pub exclude_ids: Option<String>,   // comma-separated UUIDs
  
  // Sorting
  pub sort_by: Option<String>,       // "name", "email", "created_at", "updated_at", "code"
  pub sort_order: Option<String>,    // "asc" or "desc"
}

// Constants untuk consistency dengan handler
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;

#[derive(Debug, Clone)]
pub struct ContactFilters {
  pub search: Option<String>,
  pub contact_type: Option<String>,
  pub is_active: Option<bool>,
  pub code: Option<String>,
  pub email: Option<String>,
  pub include_types: Vec<String>,
  pub exclude_types: Vec<String>,
  pub include_ids: Vec<Uuid>,
  pub exclude_ids: Vec<Uuid>,
  pub sort_by: String,
  pub sort_order: String,
}

impl From<GetContactsQuery> for ContactFilters {
  fn from(query: GetContactsQuery) -> Self {
    // Parse include/exclude types
    let include_types = query.include_types
      .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
      .unwrap_or_default();
      
    let exclude_types = query.exclude_types
      .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
      .unwrap_or_default();
    
    // Parse include/exclude IDs
    let include_ids = query.include_ids
      .map(|s| s.split(',')
        .filter_map(|id| Uuid::parse_str(id.trim()).ok())
        .collect())
      .unwrap_or_default();
      
    let exclude_ids = query.exclude_ids
      .map(|s| s.split(',')
        .filter_map(|id| Uuid::parse_str(id.trim()).ok())
        .collect())
      .unwrap_or_default();
    
    // Validate and set sort parameters
    let sort_by = match query.sort_by.as_deref() {
      Some("name") => "name",
      Some("email") => "email", 
      Some("code") => "code",
      Some("contact_type") => "type",
      Some("created_at") => "created_at",
      Some("updated_at") => "updated_at",
      _ => "created_at" // default
    }.to_string();
    
    let sort_order = match query.sort_order.as_deref() {
      Some("asc") | Some("ASC") => "ASC",
      Some("desc") | Some("DESC") => "DESC", 
      _ => "DESC" // default
    }.to_string();

    Self {
      search: query.search,
      contact_type: query.contact_type,
      is_active: query.is_active,
      code: query.code,
      email: query.email,
      include_types,
      exclude_types,
      include_ids,
      exclude_ids,
      sort_by,
      sort_order,
    }
  }
}

impl Default for GetContactsQuery {
  fn default() -> Self {
    Self {
      page: Some(DEFAULT_PAGE),
      limit: Some(DEFAULT_LIMIT),
      search: None,
      contact_type: None,
      is_active: None,
      code: None,
      email: None,
      include_types: None,
      exclude_types: None,
      include_ids: None,
      exclude_ids: None,
      sort_by: None,
      sort_order: None,
    }
  }
}
