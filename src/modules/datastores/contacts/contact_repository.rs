use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::contact_models::{Contact, CreateContactRequest, UpdateContactRequest};
use crate::AppResult;

#[async_trait]
pub trait ContactRepository {
  async fn find_all(&self) -> AppResult<Vec<Contact>>;
  async fn find_all_paginated(&self, page: u32, limit: u32) -> AppResult<(Vec<Contact>, u64)>;
  async fn find_all_by_user(&self, user_id: Uuid) -> AppResult<Vec<Contact>>;
  async fn find_all_by_user_paginated(&self, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Contact>, u64)>;
  async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Contact>>;
  async fn find_by_id_and_user(&self, id: Uuid, user_id: Uuid) -> AppResult<Option<Contact>>;
  async fn find_by_code(&self, code: &str) -> AppResult<Option<Contact>>;
  async fn create(&self, contact: CreateContactRequest, user_id: Uuid) -> AppResult<Contact>;
  async fn update(&self, id: Uuid, contact: UpdateContactRequest, user_id: Uuid) -> AppResult<Option<Contact>>;
  async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<bool>;
  async fn find_by_type(&self, contact_type: &str) -> AppResult<Vec<Contact>>;
  async fn find_by_type_and_user(&self, contact_type: &str, user_id: Uuid) -> AppResult<Vec<Contact>>;
  async fn find_active(&self) -> AppResult<Vec<Contact>>;
  async fn find_active_by_user(&self, user_id: Uuid) -> AppResult<Vec<Contact>>;
}

pub struct SqlxContactRepository {
  db: PgPool,
}

impl SqlxContactRepository {
  pub fn new(db: PgPool) -> Self {
    Self { db }
  }
}

#[async_trait]
impl ContactRepository for SqlxContactRepository {
  async fn find_all(&self) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        ORDER BY created_at DESC
      "#
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }

  async fn find_all_paginated(&self, page: u32, limit: u32) -> AppResult<(Vec<Contact>, u64)> {
    let offset = (page - 1) * limit;
    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM contacts")
      .fetch_one(&self.db)
      .await?
      .unwrap_or(0);

    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
      "#,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await?;

    Ok((contacts, total_count as u64))
  }

  async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Contact>> {
    let contact = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE id = $1
      "#,
      id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(contact)
  }

  async fn find_by_code(&self, code: &str) -> AppResult<Option<Contact>> {
    let contact = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE code = $1
      "#,
      code
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(contact)
  }

  async fn create(&self, contact: CreateContactRequest, user_id: Uuid) -> AppResult<Contact> {
    let new_contact = sqlx::query_as!(
      Contact,
      r#"
        INSERT INTO contacts (code, name, email, position, type, address, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
      "#,
      contact.code,
      contact.name,
      contact.email,
      contact.position,
      contact.contact_type,
      contact.address,
      user_id
    )
    .fetch_one(&self.db)
    .await?;

    Ok(new_contact)
  }

  async fn update(&self, id: Uuid, contact: UpdateContactRequest, user_id: Uuid) -> AppResult<Option<Contact>> {
    // First check if contact exists and belongs to user
    let existing = self.find_by_id_and_user(id, user_id).await?;
    if existing.is_none() {
      return Ok(None);
    }

    let updated_contact = sqlx::query_as!(
      Contact,
      r#"
        UPDATE contacts 
        SET 
          code = COALESCE($1, code),
          name = COALESCE($2, name),
          email = COALESCE($3, email),
          position = COALESCE($4, position),
          type = COALESCE($5, type),
          address = COALESCE($6, address),
          is_active = COALESCE($7, is_active),
          updated_by = $8,
          updated_at = NOW()
        WHERE id = $9 AND created_by = $10
        RETURNING 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
      "#,
      contact.code,
      contact.name,
      contact.email,
      contact.position,
      contact.contact_type,
      contact.address,
      contact.is_active,
      user_id,
      id,
      user_id
    )
    .fetch_one(&self.db)
    .await?;

    Ok(Some(updated_contact))
  }

  async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!(
      "DELETE FROM contacts WHERE id = $1 AND created_by = $2", 
      id, 
      user_id
    ).execute(&self.db).await?;

    Ok(result.rows_affected() > 0)
  }

  async fn find_by_type(&self, contact_type: &str) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE type = $1 AND is_active = true
        ORDER BY created_at DESC
      "#,
      contact_type
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }

  async fn find_active(&self) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE is_active = true
        ORDER BY created_at DESC
      "#
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }

  // User-specific methods for data isolation
  async fn find_all_by_user(&self, user_id: Uuid) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE created_by = $1
        ORDER BY created_at DESC
      "#,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }

  async fn find_all_by_user_paginated(&self, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Contact>, u64)> {
    let offset = (page - 1) * limit;

    // Get total count for pagination
    let total_result = sqlx::query!(
      "SELECT COUNT(*) FROM contacts WHERE created_by = $1",
      user_id
    )
    .fetch_one(&self.db)
    .await?;

    let total = total_result.count.unwrap_or(0) as u64;

    // Get paginated results
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE created_by = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
      "#,
      user_id,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await?;

    Ok((contacts, total))
  }

  async fn find_by_id_and_user(&self, id: Uuid, user_id: Uuid) -> AppResult<Option<Contact>> {
    let contact = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE id = $1 AND created_by = $2
      "#,
      id,
      user_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(contact)
  }

  async fn find_by_type_and_user(&self, contact_type: &str, user_id: Uuid) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE type = $1 AND created_by = $2 AND is_active = true
        ORDER BY created_at DESC
      "#,
      contact_type,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }

  async fn find_active_by_user(&self, user_id: Uuid) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE created_by = $1 AND is_active = true
        ORDER BY created_at DESC
      "#,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }
}
