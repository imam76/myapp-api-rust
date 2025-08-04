use std::sync::Arc;

use crate::{
  AppResult, AppState,
  errors::{AppError, NotFoundError},
  helper::{WorkspaceContext, workspace::check_workspace_permission},
  modules::{
    auth::current_user::CurrentUser,
    datastores::{
      contacts::contact_models::{ContactResponse, CreateContactRequest, GetContactsQuery, UpdateContactRequest},
      workspaces::workspace_models::WorkspaceRole,
    },
  },
  responses::{ApiResponse, PaginatedResponse, PaginationMeta},
};
use axum::{
  Json,
  extract::{Path, Query, State, rejection::JsonRejection},
  http::StatusCode,
};
use uuid::Uuid;
use validator::Validate;

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

/// Handles the request to retrieve a paginated list of contacts for the authenticated user.
/// This handler will get contacts from the user's default workspace or all accessible workspaces.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Query(params)`: The query parameters for pagination (`page`, `limit`).
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response containing a paginated list of `ContactResponse` objects that belong to the user.
#[axum::debug_handler]
pub async fn get_list(
  State(state): State<Arc<AppState>>,
  Query(params): Query<GetContactsQuery>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
) -> AppResult<Json<ApiResponse<PaginatedResponse<ContactResponse>>>> {
  let repository = &state.contact_repository;

  let page = params.page.unwrap_or(DEFAULT_PAGE);
  let mut limit = params.limit.unwrap_or(DEFAULT_LIMIT);

  if limit > MAX_LIMIT {
    limit = MAX_LIMIT;
  }

  tracing::debug!("Fetching contacts for workspace_id {}: page={}, limit={}", workspace_id, page, limit);

  // Check workspace permissions
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization("You don't have permission to access this workspace".to_string()));
  }

  let (contacts, total) = repository
    .find_all_by_workspace_paginated(workspace_id, current_user.user_id, page, limit)
    .await?;
  let pagination = PaginationMeta::new(page, limit, total);

  tracing::debug!("Retrieved {} contacts for workspace {}", contacts.len(), workspace_id);

  let response = ApiResponse::success(
    PaginatedResponse {
      list: contacts.into_iter().map(ContactResponse::from).collect(),
      pagination,
    },
    "Contacts retrieved successfully",
  );
  Ok(Json(response))
}
/// Handles the request to create a new contact for the authenticated user.
/// The contact will be created in the specified workspace or user's default workspace.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `current_user`: The authenticated user extracted from the JWT token.
/// * `payload`: The JSON payload containing the new contact's data.
///
/// # Returns
///
/// A `Json` response containing the newly created `ContactResponse`.
#[axum::debug_handler]
pub async fn create(
  State(state): State<Arc<AppState>>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
  payload: Result<Json<CreateContactRequest>, JsonRejection>,
) -> AppResult<(StatusCode, Json<ApiResponse<ContactResponse>>)> {
  let repository = &state.contact_repository;

  // Extract and validate the payload
  let Json(payload) = payload?;
  payload.validate()?;

  tracing::debug!(
    "Creating contact with code: {} for user: {} in workspace: {}",
    payload.code,
    current_user.user_id,
    workspace_id
  );

  // Validate workspace access
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(&workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization(
      "You don't have permission to create contacts in this workspace".to_string(),
    ));
  }

  // Check if code already exists in this workspace
  if let Some(_) = repository.find_by_code_and_workspace(&payload.code, workspace_id).await? {
    return Err(AppError::validation_with_code(
      "code",
      "Contact code already exists in this workspace",
      "DUPLICATE_CODE",
    ));
  }

  let contact = repository.create_by_workspace(payload, workspace_id, current_user.user_id).await?;

  tracing::info!("Contact created successfully with ID: {} for user: {}", contact.id, current_user.user_id);

  let response = ApiResponse::success(ContactResponse::from(contact), "Contact created successfully");

  Ok((StatusCode::CREATED, Json(response)))
}

/// Handles the request to retrieve a single contact by its ID for the authenticated user.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the contact to retrieve, extracted from the URL path.
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response containing the `ContactResponse` if found and accessible by the user, otherwise a 404 Not Found error.
#[axum::debug_handler]
pub async fn get_by_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<Uuid>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
) -> AppResult<Json<ApiResponse<ContactResponse>>> {
  let repository = &state.contact_repository;

  tracing::debug!("Fetching contact with ID: {} for user: {}", id, current_user.user_id);

  let contact = repository
    .find_by_id_and_workspace(id, workspace_id, current_user.user_id)
    .await?
    .ok_or_else(|| {
      AppError::NotFound(NotFoundError {
        resource: "Contact".to_string(),
        id: Some(id),
      })
    })?;

  tracing::debug!("Contact with ID {} found for user {}", id, current_user.user_id);

  let response = ApiResponse::success(ContactResponse::from(contact), "Contact retrieved successfully");
  Ok(Json(response))
}

/// Handles the request to update an existing contact for the authenticated user.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the contact to update.
/// * `current_user`: The authenticated user extracted from the JWT token.
/// * `payload`: The JSON payload with the fields to update.
///
/// # Returns
///
/// A `Json` response containing the updated `ContactResponse` if successful, otherwise a 404 error.
#[axum::debug_handler]
pub async fn update(
  State(state): State<Arc<AppState>>,
  Path(id): Path<Uuid>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
  payload: Result<Json<UpdateContactRequest>, JsonRejection>,
) -> AppResult<Json<ApiResponse<ContactResponse>>> {
  let repository = &state.contact_repository;

  // Extract the payload
  let Json(payload) = payload?;

  tracing::debug!(
    "Updating contact with ID: {} for user: {} in workspace: {}",
    id,
    current_user.user_id,
    workspace_id
  );

  // Validate workspace access
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(&workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization(
      "You don't have permission to update contacts in this workspace".to_string(),
    ));
  }

  let updated_contact = repository
    .update_by_workspace(id, workspace_id, payload, current_user.user_id)
    .await?
    .ok_or_else(|| {
      AppError::NotFound(NotFoundError {
        resource: "Contact".to_string(),
        id: Some(id),
      })
    })?;

  tracing::info!("Contact with ID {} updated successfully for workspace {}", id, workspace_id);
  let response = ApiResponse::success(ContactResponse::from(updated_contact), "Contact updated successfully");
  Ok(Json(response))
}
/// Handles the request to delete a contact by its ID for the authenticated user.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the contact to delete.
/// * `current_user`: The authenticated user extracted from the JWT token.
///
/// # Returns
///
/// A `Json` response with a success message if the deletion was successful, otherwise a 404 error.
#[axum::debug_handler]
pub async fn delete(
  State(state): State<Arc<AppState>>,
  Path(id): Path<Uuid>,
  current_user: CurrentUser,
  WorkspaceContext(workspace_id): WorkspaceContext, // Extracted from request headers
) -> AppResult<Json<ApiResponse<()>>> {
  let repository = &state.contact_repository;

  tracing::debug!(
    "Deleting contact with ID: {} for user: {} in workspace: {}",
    id,
    current_user.user_id,
    workspace_id
  );

  // Validate workspace access
  let workspace_repository = &state.workspace_repository;
  if !check_workspace_permission(&workspace_repository, workspace_id, current_user.user_id, WorkspaceRole::Member).await? {
    return Err(AppError::Authorization(
      "You don't have permission to delete contacts in this workspace".to_string(),
    ));
  }

  // Delete contact by workspace and user
  let deleted = repository.delete_by_workspace_and_user(id, workspace_id, current_user.user_id).await?;

  if !deleted {
    return Err(AppError::NotFound(NotFoundError {
      resource: "Contact".to_string(),
      id: Some(id),
    }));
  }

  tracing::info!("Contact with ID {} deleted successfully for user {}", id, current_user.user_id);

  let response = ApiResponse::success((), "Contact deleted successfully");
  Ok(Json(response))
}
