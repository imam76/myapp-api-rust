use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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
  pub created_by: Option<Uuid>,
  pub updated_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactRequest {
  pub code: String,
  pub name: String,
  pub email: String,
  pub position: String,
  pub contact_type: String,
  pub address: Option<String>,
  pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
  pub code: Option<String>,
  pub name: Option<String>,
  pub email: Option<String>,
  pub position: Option<String>,
  pub contact_type: Option<String>,
  pub address: Option<String>,
  pub is_active: Option<bool>,
  pub updated_by: Option<Uuid>,
}

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
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

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
      created_at: contact.created_at,
      updated_at: contact.updated_at,
    }
  }
}
