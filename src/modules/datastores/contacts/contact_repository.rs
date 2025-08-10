use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::contact_models::{Contact, ContactFilters, CreateContactRequest, UpdateContactRequest};
use crate::{
  AppResult,
  utils::code_generator::{CodeGenerator, CodeGeneratorConfig},
};

#[async_trait]
pub trait ContactRepository {
  // Core workspace-scoped methods - these are the only ones we need
  async fn create_by_workspace(&self, contact: CreateContactRequest, workspace_id: Uuid, user_id: Uuid) -> AppResult<Contact>;
  async fn find_all_by_workspace_paginated(&self, workspace_id: Uuid, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Contact>, u64)>;
  async fn find_by_id_and_workspace(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Option<Contact>>;
  async fn find_by_code_and_workspace(&self, code: &str, workspace_id: Uuid) -> AppResult<Option<Contact>>;
  async fn update_by_workspace(
    &self,
    id: Uuid,
    workspace_id: Uuid,
    contact_data: UpdateContactRequest,
    updated_by: Uuid,
  ) -> AppResult<Option<Contact>>;
  async fn delete_by_workspace_and_user(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<bool>;

  // Code generation methods
  async fn get_next_available_code(&self, workspace_id: Uuid, contact_name: &str) -> AppResult<String>;
  async fn code_exists(&self, code: &str, workspace_id: Uuid) -> AppResult<bool>;

  // Optional methods for specific use cases
  async fn find_by_type_and_workspace(&self, contact_type: &str, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Contact>>;
  async fn find_active_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Contact>>;
  
  // Advanced filtering method
  async fn find_by_filters_paginated(
    &self,
    workspace_id: Uuid,
    user_id: Uuid,
    page: u32,
    limit: u32,
    filters: ContactFilters,
  ) -> AppResult<(Vec<Contact>, u64)>;
}

pub struct SqlxContactRepository {
  db: PgPool,
}

impl SqlxContactRepository {
  pub fn new(db: PgPool) -> Self {
    Self { db }
  }

  /// Get access to the underlying database pool
  pub fn get_pool(&self) -> PgPool {
    self.db.clone()
  }
}

#[async_trait]
impl ContactRepository for SqlxContactRepository {
  // Workspace-scoped methods

  async fn create_by_workspace(&self, contact: CreateContactRequest, workspace_id: Uuid, user_id: Uuid) -> AppResult<Contact> {
    let new_contact = sqlx::query_as!(
      Contact,
      r#"
        INSERT INTO contacts (code, name, email, position, type, address, workspace_id, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING 
          id, code, name, email, position, type as contact_type, 
          address, is_active, workspace_id, created_by, updated_by, created_at, updated_at
      "#,
      contact.code,
      contact.name,
      contact.email,
      contact.position,
      contact.contact_type,
      contact.address,
      workspace_id,
      user_id
    )
    .fetch_one(&self.db)
    .await?;

    Ok(new_contact)
  }

  async fn find_all_by_workspace_paginated(&self, workspace_id: Uuid, user_id: Uuid, page: u32, limit: u32) -> AppResult<(Vec<Contact>, u64)> {
    let offset = (page - 1) * limit;

    let total_count = sqlx::query_scalar!(
      r#"
        SELECT COUNT(*) 
        FROM contacts 
        WHERE workspace_id = $1 
          AND id IN (
            SELECT c.id FROM contacts c
            JOIN workspaces w ON c.workspace_id = w.id
            JOIN workspace_users wu ON w.id = wu.workspace_id
            WHERE wu.user_id = $2
          )
      "#,
      workspace_id,
      user_id
    )
    .fetch_one(&self.db)
    .await?
    .unwrap_or(0);

    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, workspace_id, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE workspace_id = $1 
          AND id IN (
            SELECT c.id FROM contacts c
            JOIN workspaces w ON c.workspace_id = w.id
            JOIN workspace_users wu ON w.id = wu.workspace_id
            WHERE wu.user_id = $2
          )
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
      "#,
      workspace_id,
      user_id,
      limit as i64,
      offset as i64
    )
    .fetch_all(&self.db)
    .await?;

    Ok((contacts, total_count as u64))
  }

  async fn find_by_id_and_workspace(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<Option<Contact>> {
    let contact = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, workspace_id, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE id = $1 AND workspace_id = $2
          AND id IN (
            SELECT c.id FROM contacts c
            JOIN workspaces w ON c.workspace_id = w.id
            JOIN workspace_users wu ON w.id = wu.workspace_id
            WHERE wu.user_id = $3
          )
      "#,
      id,
      workspace_id,
      user_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(contact)
  }

  async fn find_by_type_and_workspace(&self, contact_type: &str, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, workspace_id, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE type = $1 AND workspace_id = $2
          AND id IN (
            SELECT c.id FROM contacts c
            JOIN workspaces w ON c.workspace_id = w.id
            JOIN workspace_users wu ON w.id = wu.workspace_id
            WHERE wu.user_id = $3
          )
        ORDER BY created_at DESC
      "#,
      contact_type,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }

  async fn find_active_by_workspace(&self, workspace_id: Uuid, user_id: Uuid) -> AppResult<Vec<Contact>> {
    let contacts = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, workspace_id, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE workspace_id = $1 AND is_active = true
          AND id IN (
            SELECT c.id FROM contacts c
            JOIN workspaces w ON c.workspace_id = w.id
            JOIN workspace_users wu ON w.id = wu.workspace_id
            WHERE wu.user_id = $2
          )
        ORDER BY created_at DESC
      "#,
      workspace_id,
      user_id
    )
    .fetch_all(&self.db)
    .await?;

    Ok(contacts)
  }

  async fn find_by_code_and_workspace(&self, code: &str, workspace_id: Uuid) -> AppResult<Option<Contact>> {
    let contact = sqlx::query_as!(
      Contact,
      r#"
        SELECT 
          id, code, name, email, position, type as contact_type, 
          address, is_active, workspace_id, created_by, updated_by, created_at, updated_at
        FROM contacts 
        WHERE code = $1 AND workspace_id = $2
      "#,
      code,
      workspace_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(contact)
  }

  async fn update_by_workspace(
    &self,
    id: Uuid,
    workspace_id: Uuid,
    contact_data: UpdateContactRequest,
    updated_by: Uuid,
  ) -> AppResult<Option<Contact>> {
    let contact = sqlx::query_as!(
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
        WHERE id = $9 AND workspace_id = $10
        RETURNING 
          id, code, name, email, position, type as contact_type, 
          address, is_active, workspace_id, created_by, updated_by, created_at, updated_at
      "#,
      contact_data.code,
      contact_data.name,
      contact_data.email,
      contact_data.position,
      contact_data.contact_type,
      contact_data.address,
      contact_data.is_active,
      updated_by,
      id,
      workspace_id
    )
    .fetch_optional(&self.db)
    .await?;

    Ok(contact)
  }

  async fn delete_by_workspace_and_user(&self, id: Uuid, workspace_id: Uuid, user_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!(
      "DELETE FROM contacts WHERE id = $1 AND workspace_id = $2 AND created_by = $3",
      id,
      workspace_id,
      user_id
    )
    .execute(&self.db)
    .await?;

    Ok(result.rows_affected() > 0)
  }

  async fn get_next_available_code(&self, workspace_id: Uuid, contact_name: &str) -> AppResult<String> {
    let code_generator = CodeGenerator::new(self.db.clone());
    let config = CodeGeneratorConfig {
      table_name: "contacts".to_string(),
      code_column: "code".to_string(),
      workspace_column: Some("workspace_id".to_string()),
      prefix_length: 2,
      number_length: 5,
      separator: "-".to_string(),
    };

    code_generator.get_next_available_code(&config, contact_name, Some(workspace_id)).await
  }

  async fn code_exists(&self, code: &str, workspace_id: Uuid) -> AppResult<bool> {
    let code_generator = CodeGenerator::new(self.db.clone());
    let config = CodeGeneratorConfig {
      table_name: "contacts".to_string(),
      code_column: "code".to_string(),
      workspace_column: Some("workspace_id".to_string()),
      ..Default::default()
    };

    code_generator.code_exists(&config, code, Some(workspace_id)).await
  }

  async fn find_by_filters_paginated(
    &self,
    workspace_id: Uuid,
    user_id: Uuid,
    page: u32,
    limit: u32,
    filters: ContactFilters,
  ) -> AppResult<(Vec<Contact>, u64)> {
    use super::contact_query_builder::ContactQueryBuilder;
    
    let offset = (page - 1) * limit;
    
    // Build queries using Sea Query
    let (mut select_sql, count_sql) = ContactQueryBuilder::build_filtered_query(
      workspace_id, 
      user_id, 
      &filters
    );
    
    // Add pagination to select query
    select_sql = format!("{} LIMIT {} OFFSET {}", select_sql, limit, offset);

    tracing::debug!("Executing count query: {}", count_sql);
    tracing::debug!("Executing select query: {}", select_sql);

    // Execute count query first
    let total_count: i64 = sqlx::query_scalar::<_, Option<i64>>(&count_sql)
      .fetch_one(&self.db)
      .await?
      .unwrap_or(0);

    // If count is 0, return empty result
    if total_count == 0 {
      return Ok((vec![], 0));
    }

    // Execute data query
    let contacts = sqlx::query_as::<_, Contact>(&select_sql)
      .fetch_all(&self.db)
      .await
      .map_err(|e| {
        tracing::error!("Failed to execute filtered query: {}", e);
        tracing::error!("Query: {}", select_sql);
        e
      })?;

    tracing::debug!("Found {} contacts with total count {}", contacts.len(), total_count);

    Ok((contacts, total_count as u64))
  }
}
