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
  pub contact_type: String, // Using contact_type to avoid Rust keyword conflict
  pub address: Option<String>,
  pub is_active: bool,
  pub workspace_id: Option<Uuid>,
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Represents the payload for creating a new contact.
/// This struct uses `validator` to enforce declarative validation rules on the incoming data.
/// The `created_by` field is automatically set from the authenticated user.
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
  pub workspace_id: Option<Uuid>,
}

/// Represents the payload for updating an existing contact.
/// All fields are optional, allowing for partial updates.
/// The `updated_by` field is automatically set from the authenticated user.
#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
  pub code: Option<String>,
  pub name: Option<String>,
  pub email: Option<String>,
  pub position: Option<String>,
  pub contact_type: Option<String>,
  pub address: Option<String>,
  pub is_active: Option<bool>,
  pub workspace_id: Option<Uuid>,
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
      workspace_id: contact.workspace_id,
      created_by: contact.created_by,
      updated_by: contact.updated_by,
      created_at: contact.created_at,
      updated_at: contact.updated_at,
    }
  }
}
