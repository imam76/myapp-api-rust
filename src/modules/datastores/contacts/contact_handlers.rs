use std::sync::Arc;

use crate::{
  AppResult, AppState,
  errors::{AppError, NotFoundError},
  modules::datastores::contacts::contact_models::{ContactResponse, CreateContactRequest, UpdateContactRequest},
  responses::{ApiResponse, GetContactsQuery, PaginatedResponse, PaginationMeta},
};
use axum::{
  Json,
  extract::{Path, Query, State, rejection::JsonRejection},
};
use uuid::Uuid;
use validator::Validate;

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

/// Handles the request to retrieve a paginated list of contacts.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Query(params)`: The query parameters for pagination (`page`, `limit`).
///
/// # Returns
///
/// A `Json` response containing a paginated list of `ContactResponse` objects.
#[axum::debug_handler]
pub async fn get_list(
  State(state): State<Arc<AppState>>,
  Query(params): Query<GetContactsQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<ContactResponse>>>> {
  let repository = &state.contact_repository;

  let page = params.page.unwrap_or(DEFAULT_PAGE);
  let mut limit = params.limit.unwrap_or(DEFAULT_LIMIT);

  if limit > MAX_LIMIT {
    limit = MAX_LIMIT;
  }

  tracing::debug!("Fetching contacts: page={}, limit={}", page, limit);

  let (contacts, total) = repository.find_all_paginated(page, limit).await?;
  let pagination = PaginationMeta::new(page, limit, total);

  tracing::debug!("Retrieved {} contacts", contacts.len());

  let response = ApiResponse::success(
    PaginatedResponse {
      list: contacts.into_iter().map(ContactResponse::from).collect(),
      pagination,
    },
    "Contacts retrieved successfully",
  );

  Ok(Json(response))
}

/// Handles the request to create a new contact.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `payload`: The JSON payload containing the new contact's data.
///
/// # Returns
///
/// A `Json` response containing the newly created `ContactResponse`.
#[axum::debug_handler]
pub async fn create(
  State(state): State<Arc<AppState>>,
  payload: Result<Json<CreateContactRequest>, JsonRejection>,
) -> AppResult<Json<ApiResponse<ContactResponse>>> {
  let repository = &state.contact_repository;

  // Extract and validate the payload
  let Json(payload) = payload?;
  payload.validate()?;

  tracing::debug!("Creating contact with code: {}", payload.code);

  // Check if code already exists
  if let Some(_) = repository.find_by_code(&payload.code).await? {
    return Err(AppError::validation_with_code("code", "Contact code already exists", "DUPLICATE_CODE"));
  }

  let contact = repository.create(payload).await?;

  tracing::info!("Contact created successfully with ID: {}", contact.id);

  let response = ApiResponse::success(ContactResponse::from(contact), "Contact created successfully");

  Ok(Json(response))
}

/// Handles the request to retrieve a single contact by its ID.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the contact to retrieve, extracted from the URL path.
///
/// # Returns
///
/// A `Json` response containing the `ContactResponse` if found, otherwise a 404 Not Found error.
#[axum::debug_handler]
pub async fn get_by_id(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<ContactResponse>>> {
  let repository = &state.contact_repository;

  tracing::debug!("Fetching contact with ID: {}", id);

  let contact = repository.find_by_id(id).await?.ok_or_else(|| {
    AppError::NotFound(NotFoundError {
      resource: "Contact".to_string(),
      id: Some(id),
    })
  })?;

  tracing::debug!("Contact with ID {} found", id);

  let response = ApiResponse::success(ContactResponse::from(contact), "Contact retrieved successfully");
  Ok(Json(response))
}

/// Handles the request to update an existing contact.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the contact to update.
/// * `payload`: The JSON payload with the fields to update.
///
/// # Returns
///
/// A `Json` response containing the updated `ContactResponse` if successful, otherwise a 404 error.
#[axum::debug_handler]
pub async fn update(
  State(state): State<Arc<AppState>>,
  Path(id): Path<Uuid>,
  payload: Result<Json<UpdateContactRequest>, JsonRejection>,
) -> AppResult<Json<ApiResponse<ContactResponse>>> {
  let repository = &state.contact_repository;
  let Json(payload) = payload?;

  tracing::debug!("Updating contact with ID: {}", id);

  let updated_contact = repository.update(id, payload).await?.ok_or_else(|| {
    AppError::NotFound(NotFoundError {
      resource: "Contact".to_string(),
      id: Some(id),
    })
  })?;

  tracing::info!("Contact with ID {} updated successfully", id);

  let response = ApiResponse::success(ContactResponse::from(updated_contact), "Contact updated successfully");
  Ok(Json(response))
}

/// Handles the request to delete a contact by its ID.
///
/// # Arguments
///
/// * `State(state)`: The shared application state.
/// * `Path(id)`: The ID of the contact to delete.
///
/// # Returns
///
/// A `Json` response with a success message if the deletion was successful, otherwise a 404 error.
#[axum::debug_handler]
pub async fn delete(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<()>>> {
  let repository = &state.contact_repository;

  tracing::debug!("Deleting contact with ID: {}", id);

  let deleted = repository.delete(id).await?;

  if !deleted {
    return Err(AppError::NotFound(NotFoundError {
      resource: "Contact".to_string(),
      id: Some(id),
    }));
  }

  tracing::info!("Contact with ID {} deleted successfully", id);

  let response = ApiResponse::success((), "Contact deleted successfully");
  Ok(Json(response))
}
